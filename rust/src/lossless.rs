//! Lossless coding: median-edge-detector prediction plus coded residuals.
//!
//! This is the path the format uses for `MODE_GRAY_LOSSLESS`, `MODE_RGB_LOSSLESS`
//! and for the alpha channel. There is no DCT and no quantiser, so decode is
//! exact. Planes are coded at their real size: the 8x8 block padding that the
//! lossy path needs does not apply here.
//!
//! The predictor and the activity classes follow the reference encoder: `a` is
//! the pixel to the left, `b` the one above, `c` the one above-left, the first
//! row pretends `b = c = a`, and the first column uses `a = 0`.

use crate::arith::{Dec, Enc};
use crate::codec::{Clip, Plane};
use crate::entropy::ResidualModel;

/// Median edge detector: pick the smaller neighbour next to an edge, otherwise
/// extrapolate along the diagonal.
#[inline]
fn med(a: i16, b: i16, c: i16) -> i32 {
    let (a, b, c) = (a as i32, b as i32, c as i32);
    let (mx, mn) = if a > b { (a, b) } else { (b, a) };
    if c >= mx {
        mn
    } else if c <= mn {
        mx
    } else {
        a + b - c
    }
}

/// Activity class from the two neighbour differences: five buckets, coarser as
/// the neighbourhood gets busier.
#[inline]
pub fn ctx_of(activity: i16) -> usize {
    let activity = activity as i32;
    match activity {
        i32::MIN..=1 => 0,
        2..=3 => 1,
        4..=7 => 2,
        8..=15 => 3,
        _ => 4,
    }
}

/// Encode planes, each `w * h` samples, into one arithmetic-coded payload.
pub fn encode_planes(planes: &[Plane]) -> Vec<u8> {
    let mut model = ResidualModel::new(planes.len());
    let mut enc = Enc::new();
    for (pl, plane) in planes.iter().enumerate() {
        let w = plane.w;
        for y in 0..plane.h {
            for x in 0..w {
                let a = if x > 0 { plane.px[y * w + x - 1] } else { 0 };
                let (b, c) = if y == 0 {
                    (a, a)
                } else {
                    let b = plane.px[(y - 1) * w + x];
                    let c = if x > 0 {
                        plane.px[(y - 1) * w + x - 1]
                    } else {
                        a
                    };
                    (b, c)
                };
                let p = med(a, b, c);
                let act = ((a - c).abs() + (b - c).abs()) as i16;
                model.enc_res(&mut enc, pl, ctx_of(act), plane.px[y * w + x] as i32 - p);
            }
        }
    }
    enc.finish()
}

/// Inverse of [`encode_planes`]; `clips` carries the legal range per plane.
pub fn decode_planes(
    payload: &[u8],
    shapes: &[(usize, usize)],
    clips: &[Clip],
) -> Vec<Plane> {
    let mut model = ResidualModel::new(shapes.len());
    let mut dec = Dec::new(payload);
    let mut out = Vec::with_capacity(shapes.len());
    for (pl, &(w, h)) in shapes.iter().enumerate() {
        let clip = clips[pl];
        let mut px = vec![0i16; w * h];
        for y in 0..h {
            for x in 0..w {
                let a = if x > 0 { px[y * w + x - 1] } else { 0 };
                let (b, c) = if y == 0 {
                    (a, a)
                } else {
                    let b = px[(y - 1) * w + x];
                    let c = if x > 0 { px[(y - 1) * w + x - 1] } else { a };
                    (b, c)
                };
                let p = med(a, b, c);
                let act = ((a - c).abs() + (b - c).abs()) as i16;
                let r = model.dec_res(&mut dec, pl, ctx_of(act));
                px[y * w + x] = (p + r).clamp(clip.lo as i32, clip.hi as i32) as i16;
            }
        }
        out.push(Plane { w, h, px });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{clip_for_plane, Plane};

    fn plane(w: usize, h: usize, seed: u32) -> Plane {
        let mut px = vec![0i16; w * h];
        let mut s = seed.wrapping_mul(2654435761);
        for y in 0..h {
            for x in 0..w {
                s = s.wrapping_mul(1664525).wrapping_add(1013904223);
                px[y * w + x] = ((x * 3 + y * 5) as i32 + ((s >> 29) as i32) - 3).clamp(0, 255) as i16;
            }
        }
        Plane { w, h, px }
    }

    #[test]
    fn activity_classes_match_the_reference_buckets() {
        assert_eq!(ctx_of(0), 0);
        assert_eq!(ctx_of(1), 0);
        assert_eq!(ctx_of(2), 1);
        assert_eq!(ctx_of(3), 1);
        assert_eq!(ctx_of(4), 2);
        assert_eq!(ctx_of(7), 2);
        assert_eq!(ctx_of(8), 3);
        assert_eq!(ctx_of(15), 3);
        assert_eq!(ctx_of(16), 4);
        assert_eq!(ctx_of(300), 4);
    }

    #[test]
    fn gray_round_trip_is_exact() {
        let p = plane(37, 23, 1);
        let payload = encode_planes(std::slice::from_ref(&p));
        let clips = vec![clip_for_plane(0)];
        let back = decode_planes(&payload, &[(37, 23)], &clips);
        assert_eq!(back[0].px, p.px);
    }

    #[test]
    fn colour_planes_round_trip_with_chroma_range() {
        // Chroma planes can be negative; the clip range must not clamp them.
        let y = plane(32, 16, 7);
        let co = Plane {
            w: 32,
            h: 16,
            px: (0..32 * 16).map(|i| ((i as i32 % 511) - 255) as i16).collect(),
        };
        let cg = Plane {
            w: 32,
            h: 16,
            px: (0..32 * 16).map(|i| (120 - (i as i32 % 241)) as i16).collect(),
        };
        let payload = encode_planes(&[y.clone(), co.clone(), cg.clone()]);
        let clips: Vec<_> = (0..3).map(clip_for_plane).collect();
        let back = decode_planes(&payload, &[(32, 16); 3], &clips);
        assert_eq!(back[0].px, y.px);
        assert_eq!(back[1].px, co.px);
        assert_eq!(back[2].px, cg.px);
    }

    #[test]
    fn flat_plane_costs_almost_nothing() {
        let p = Plane {
            w: 64,
            h: 64,
            px: vec![77; 64 * 64],
        };
        let payload = encode_planes(std::slice::from_ref(&p));
        // 4096 samples go through the coder as 4024 zero residuals.
        assert!(payload.len() < 128, "{} bytes", payload.len());
    }
}
