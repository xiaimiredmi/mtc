//! Plane coding: 8x8 DCT, quantisation (optionally adaptive per tile), the
//! entropy coder and the inverse path.
//!
//! A plane is a rectangle of integer samples whose sides are multiples of 8.
//! Chroma planes are quantised 1.35x harder, the same as the reference encoder.

use crate::aq;
use crate::arith::{Dec, Enc};
use crate::entropy::{dec_block, enc_block, Left, Model};
use crate::transform::{dct_matrix, fdct, idct_f32, round_i32, step_table, Basis, ZZ};

pub const CHROMA_GAIN: f64 = 1.35;

#[derive(Clone, Debug)]
pub struct Plane {
    pub w: usize,
    pub h: usize,
    pub px: Vec<i16>,
}

#[derive(Clone, Copy, Debug)]
pub struct Clip {
    pub lo: i16,
    pub hi: i16,
}

/// Luma is clipped to a byte range after reconstruction, chroma to the range the
/// reversible YCoCg transform can reach.
pub fn clip_for_plane(pl: usize) -> Clip {
    if pl == 0 {
        Clip { lo: 0, hi: 255 }
    } else {
        Clip {
            lo: -300,
            hi: 300,
        }
    }
}

fn plane_step(base: &[[f64; 8]; 8], pl: usize) -> [[f64; 8]; 8] {
    if pl == 0 {
        *base
    } else {
        let mut s = [[0.0f64; 8]; 8];
        for u in 0..8 {
            for v in 0..8 {
                s[u][v] = base[u][v] * CHROMA_GAIN;
            }
        }
        s
    }
}

/// Half-width of the zero bin, as a fraction of a step: 0.5 is plain
/// round-to-nearest. Wider bins trade a little peak accuracy for a real rate
/// win, because a coefficient that quantises to zero costs one cheap symbol and
/// nothing else. The DC coefficient keeps the neutral value: rounding block
/// means to zero shifts whole tiles, which costs more than it saves.
const DEAD_ZONE_AC: f64 = 0.65;
const DEAD_ZONE_DC: f64 = 0.5;

/// Quantise one coefficient, zeroing everything inside the dead zone.
#[inline]
fn quantise(c: f64, step: f64, off: f64) -> i32 {
    let v = c / step;
    let a = v.abs();
    let q = (a + (1.0 - off)).floor();
    let q = if q < 0.0 { 0.0 } else { q } as i32;
    if v < 0.0 {
        -q
    } else {
        q
    }
}

/// DC predictor for the context layout: the left coefficient on the first row,
/// a median edge detector afterwards. Both sides see the same already-decoded
/// DC values, so the predictor costs nothing in the stream.
#[inline]
fn dc_predict(up: &[i32], cur: &[i32], bx: usize, by: usize) -> i32 {
    let left = if bx > 0 { cur[bx - 1] } else { 0 };
    if by == 0 {
        return left;
    }
    let top = up[bx];
    let tl = if bx > 0 { up[bx - 1] } else { left };
    let (mx, mn) = if left > top { (left, top) } else { (top, left) };
    if tl >= mx {
        mn
    } else if tl <= mn {
        mx
    } else {
        left + top - tl
    }
}

/// `(dc, ac)` dead-zone widths for this stream.
fn dead_zones(layout: u8) -> (f64, f64) {
    if layout != crate::entropy::LAYOUT_CTX {
        // Version 1 stays a faithful reproduction of the reference encoder.
        return (0.5, 0.5);
    }
    (DEAD_ZONE_DC, DEAD_ZONE_AC)
}

/// Encode a set of planes into one arithmetic-coded payload.
pub fn encode_planes(planes: &[Plane], quality: u8, layout: u8, aq_on: bool) -> Vec<u8> {
    let m = dct_matrix();
    let base = step_table(quality);
    let mut model = Model::new(layout, planes.len());
    let mut aqprobs = aq::new_probs(planes.len());
    let mut enc = Enc::new();
    let (dc_off, ac_off) = dead_zones(layout);

    for (pl, plane) in planes.iter().enumerate() {
        let steps = aq::level_steps(&plane_step(&base, pl));
        let pw = plane.w;
        let blocks_w = plane.w / 8;
        let blocks_h = plane.h / 8;
        let tiles_w = (blocks_w + aq::TILE - 1) / aq::TILE;
        let map = if aq_on {
            Some(aq::analyse(&plane.px, plane.w, plane.h))
        } else {
            None
        };
        let mut carry = aq::NEUTRAL as i32;
        let mut prev_same = false;
        let mut row_levels = vec![aq::NEUTRAL; tiles_w.max(1)];
        let mut left = Left::default();
        let mut up = vec![0i32; blocks_w];
        let mut cur = vec![0i32; blocks_w];
        let med = layout == crate::entropy::LAYOUT_CTX;

        for by in 0..blocks_h {
            if let Some(map) = map.as_ref() {
                if by % aq::TILE == 0 {
                    let ty = by / aq::TILE;
                    for tx in 0..tiles_w {
                        let target = map.levels[ty * tiles_w + tx] as i32;
                        let delta = (target - carry).clamp(-3, 3);
                        let level = (carry + delta).clamp(0, 6) as usize;
                        row_levels[tx] = level;
                        aq::enc_level(&mut enc, &mut aqprobs, pl, prev_same, delta);
                        carry = level as i32;
                        prev_same = delta == 0;
                    }
                }
            }
            let mut prev = 0i32;
            for bx in 0..blocks_w {
                let level = if map.is_some() {
                    row_levels[bx / aq::TILE]
                } else {
                    aq::NEUTRAL
                };
                let step = &steps[level];
                let mut blk = [[0.0f64; 8]; 8];
                for y in 0..8 {
                    for x in 0..8 {
                        blk[y][x] = plane.px[(by * 8 + y) * pw + bx * 8 + x] as f64;
                    }
                }
                let coef = fdct(&blk, &m);
                let mut qz = [0i32; 64];
                for k in 0..64 {
                    let (u, v) = ZZ[k];
                    let off = if k == 0 { dc_off } else { ac_off };
                    qz[k] = quantise(coef[u][v], step[u][v], off);
                }
                if med {
                    prev = dc_predict(&up, &cur, bx, by);
                }
                enc_block(&mut enc, &mut model, pl, &qz, prev, &left);
                left = Left::of(&qz);
                cur[bx] = qz[0];
                if !med {
                    prev = qz[0];
                }
            }
            if med {
                std::mem::swap(&mut up, &mut cur);
            }
        }
    }
    enc.finish()
}

/// Inverse of [`encode_planes`]. Shapes are `(width, height)` per plane and must
/// be the padded sizes, i.e. multiples of 8.
pub fn decode_planes(
    payload: &[u8],
    shapes: &[(usize, usize)],
    clips: &[Clip],
    quality: u8,
    layout: u8,
    aq_on: bool,
) -> Vec<Plane> {
    let basis = Basis::new();
    let base = step_table(quality);
    let mut model = Model::new(layout, shapes.len());
    let mut aqprobs = aq::new_probs(shapes.len());
    let mut dec = Dec::new(payload);
    // Time the two halves of a block separately: the entropy decoder and the
    // transform. Enabled by an environment variable so it costs nothing in a game.
    let profiling = std::env::var_os("MTCODEC_PROFILE").is_some();
    let mut t_entropy = 0f64;
    let mut t_idct = 0f64;
    let mut out = Vec::with_capacity(shapes.len());

    for (pl, &(pw, ph)) in shapes.iter().enumerate() {
        let steps = aq::level_steps(&plane_step(&base, pl));
        let blocks_w = pw / 8;
        let blocks_h = ph / 8;
        let tiles_w = (blocks_w + aq::TILE - 1) / aq::TILE;
        let mut carry = aq::NEUTRAL as i32;
        let mut prev_same = false;
        let mut row_levels = vec![aq::NEUTRAL; tiles_w.max(1)];
        let mut left = Left::default();
        let mut up = vec![0i32; blocks_w];
        let mut cur = vec![0i32; blocks_w];
        let med = layout == crate::entropy::LAYOUT_CTX;
        let mut plane = vec![0i16; pw * ph];
        let clip = clips[pl];

        for by in 0..blocks_h {
            if aq_on && by % aq::TILE == 0 {
                for tx in 0..tiles_w {
                    let delta = aq::dec_level(&mut dec, &mut aqprobs, pl, prev_same);
                    let level = (carry + delta).clamp(0, 6) as usize;
                    row_levels[tx] = level;
                    carry = level as i32;
                    prev_same = delta == 0;
                }
            }
            let mut prev = 0i32;
            for bx in 0..blocks_w {
                let level = if aq_on {
                    row_levels[bx / aq::TILE]
                } else {
                    aq::NEUTRAL
                };
                let step = &steps[level];
                if med {
                    prev = dc_predict(&up, &cur, bx, by);
                }
                let clock = std::time::Instant::now();
                let qz = dec_block(&mut dec, &mut model, pl, prev, &left);
                if profiling {
                    t_entropy += clock.elapsed().as_secs_f64();
                }
                left = Left::of(&qz);
                cur[bx] = qz[0];
                if !med {
                    prev = qz[0];
                }
                let mut coef = [[0.0f64; 8]; 8];
                for k in 0..64 {
                    let (u, v) = ZZ[k];
                    coef[u][v] = qz[k] as f64 * step[u][v];
                }
                let clock = std::time::Instant::now();
                let blk = idct_f32(&coef, &basis);
                for y in 0..8 {
                    for x in 0..8 {
                        let v = round_i32(blk[y][x]).clamp(clip.lo as i32, clip.hi as i32) as i16;
                        plane[(by * 8 + y) * pw + bx * 8 + x] = v;
                    }
                }
                if profiling {
                    t_idct += clock.elapsed().as_secs_f64();
                }
            }
            if med {
                std::mem::swap(&mut up, &mut cur);
            }
        }
        out.push(Plane { w: pw, h: ph, px: plane });
    }
    if profiling {
        eprintln!(
            "profile: entropy {:.1} ms, idct {:.1} ms",
            t_entropy * 1000.0,
            t_idct * 1000.0
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entropy::{LAYOUT_CTX, LAYOUT_FLAT};

    fn ramp_plane(w: usize, h: usize, seed: u32) -> Plane {
        let mut px = vec![0i16; w * h];
        let mut s = seed.wrapping_mul(2654435761);
        for y in 0..h {
            for x in 0..w {
                s = s.wrapping_mul(1664525).wrapping_add(1013904223);
                let noise = ((s >> 28) as i32) - 8;
                px[y * w + x] = (40 + ((x * 2 + y * 3) % 96) as i32 + noise).clamp(0, 255) as i16;
            }
        }
        Plane { w, h, px }
    }

    fn psnr(a: &Plane, b: &Plane) -> f64 {
        let mut se = 0f64;
        for (p, q) in a.px.iter().zip(b.px.iter()) {
            let d = (*p - *q) as f64;
            se += d * d;
        }
        let mse = se / a.px.len() as f64;
        10.0 * (255.0 * 255.0 / mse.max(1e-9)).log10()
    }

    fn roundtrip(layout: u8, aq_on: bool, quality: u8) -> (Vec<u8>, f64) {
        let planes = vec![ramp_plane(64, 48, 3), ramp_plane(32, 24, 9)];
        let payload = encode_planes(&planes, quality, layout, aq_on);
        let shapes: Vec<(usize, usize)> = planes.iter().map(|p| (p.w, p.h)).collect();
        let clips: Vec<Clip> = (0..planes.len()).map(clip_for_plane).collect();
        let back = decode_planes(&payload, &shapes, &clips, quality, layout, aq_on);
        assert_eq!(back.len(), planes.len());
        (payload, psnr(&planes[0], &back[0]))
    }

    #[test]
    fn flat_layout_roundtrip() {
        let (payload, p) = roundtrip(LAYOUT_FLAT, false, 85);
        assert!(p > 30.0, "psnr {:.2} bytes {}", p, payload.len());
    }

    #[test]
    fn ctx_layout_roundtrip() {
        let (payload, p) = roundtrip(LAYOUT_CTX, false, 85);
        assert!(p > 30.0, "psnr {:.2} bytes {}", p, payload.len());
    }

    #[test]
    fn ctx_layout_with_aq_roundtrip() {
        for q in [40u8, 70, 85, 95] {
            let (payload, p) = roundtrip(LAYOUT_CTX, true, q);
            assert!(p > 25.0, "q{} psnr {:.2} bytes {}", q, p, payload.len());
        }
    }

    #[test]
    fn quality_controls_size() {
        let (small, _) = roundtrip(LAYOUT_CTX, false, 40);
        let (big, _) = roundtrip(LAYOUT_CTX, false, 95);
        assert!(big.len() > small.len());
    }

    #[test]
    fn flat_plane_is_unchanged_by_aq() {
        let planes = vec![Plane {
            w: 32,
            h: 32,
            px: vec![120i16; 32 * 32],
        }];
        let shapes = vec![(32usize, 32usize)];
        let clips = vec![clip_for_plane(0)];
        let off = encode_planes(&planes, 80, LAYOUT_CTX, false);
        let on = encode_planes(&planes, 80, LAYOUT_CTX, true);
        let a = decode_planes(&off, &shapes, &clips, 80, LAYOUT_CTX, false);
        let b = decode_planes(&on, &shapes, &clips, 80, LAYOUT_CTX, true);
        assert_eq!(a[0].px, b[0].px);
        // A neutral tile row still costs a few level symbols.
        assert!(on.len() >= off.len());
    }
}
