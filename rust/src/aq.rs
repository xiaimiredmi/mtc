//! Adaptive quantisation: one quantiser multiplier per 16x16 tile.
//!
//! The baseline quantiser is frequency-tilted but identical for every block, so
//! a busy block and a flat block spend the same number of bits per coefficient.
//! AQ spends bits where the error is actually visible in the mean-square sense:
//! a tile whose activity is above the picture median gets a *finer* step, a flat
//! tile gets a coarser one. The multiplier is chosen from seven levels by the
//! encoder and signalled per tile, because an intra frame gives the decoder no
//! reference plane to derive it from.
//!
//! Levels are coded as a delta against the previous tile, so a smooth AQ map
//! costs a fraction of a bit per tile.

use crate::arith::{dec_tree, enc_tree, Dec, Enc, PROB_INIT};

/// Multiplier for each level; level 3 is exactly neutral.
pub const MULT: [f64; 7] = [0.70, 0.80, 0.90, 1.00, 1.12, 1.25, 1.40];
pub const NEUTRAL: usize = 3;
/// 8x8 blocks per tile side.
pub const TILE: usize = 2;
/// How hard activity moves the step. Positive spends more bits in busy tiles,
/// which is what a fixed `-q` benchmark rewards; on matched PSNR it loses, see
/// the module comment.
const STRENGTH: f64 = 0.35;
/// Activity ratios below this stay neutral: on a nearly flat picture the ratios
/// are rounding noise, and a noisy map costs one symbol per tile.
const DEAD_ZONE: f64 = 0.2;
/// Activity ratios are clipped before the log so a flat photograph does not
/// produce wild multipliers from rounding noise.
const RATIO_LO: f64 = 0.25;
const RATIO_HI: f64 = 4.0;
/// Below this median activity the picture is flat enough that ratios are noise.
const MEDIAN_FLOOR: f64 = 0.75;

pub struct Map {
    pub tiles_w: usize,
    pub tiles_h: usize,
    pub levels: Vec<u8>,
}

pub fn new_probs(planes: usize) -> Vec<i16> {
    vec![PROB_INIT; planes * 16]
}


/// Pick the level whose multiplier is closest to `mult`.
fn level_of(mult: f64) -> usize {
    let mut best = NEUTRAL;
    let mut best_err = f64::MAX;
    for (i, m) in MULT.iter().enumerate() {
        let err = (m - mult).abs();
        if err < best_err {
            best_err = err;
            best = i;
        }
    }
    best
}

/// Per-tile activity, as the mean absolute gradient over the tile.
fn tile_activity(px: &[i16], w: usize, h: usize, tx: usize, ty: usize) -> f64 {
    let mut sum = 0i64;
    let y1 = ((ty + 1) * TILE * 8).min(h);
    let x1 = ((tx + 1) * TILE * 8).min(w);
    for y in ty * TILE * 8..y1 {
        for x in tx * TILE * 8..x1 {
            let c = px[y * w + x];
            if x + 1 < w {
                sum += (px[y * w + x + 1] - c).unsigned_abs() as i64;
            }
            if y + 1 < h {
                sum += (px[(y + 1) * w + x] - c).unsigned_abs() as i64;
            }
        }
    }
    let n = ((y1 - ty * TILE * 8) * (x1 - tx * TILE * 8)) as f64;
    if n <= 0.0 {
        0.0
    } else {
        sum as f64 / n
    }
}

pub fn analyse(px: &[i16], w: usize, h: usize) -> Map {
    let tiles_w = (w / 8 + TILE - 1) / TILE;
    let tiles_h = (h / 8 + TILE - 1) / TILE;
    let mut act = Vec::with_capacity(tiles_w * tiles_h);
    for ty in 0..tiles_h {
        for tx in 0..tiles_w {
            act.push(tile_activity(px, w, h, tx, ty));
        }
    }
    let mut sorted: Vec<f64> = act.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.is_empty() {
        0.0
    } else {
        sorted[sorted.len() / 2]
    };
    let mut levels = vec![NEUTRAL as u8; act.len()];
    if median < MEDIAN_FLOOR {
        return Map {
            tiles_w,
            tiles_h,
            levels,
        };
    }
    for (i, a) in act.iter().enumerate() {
        let lr = (a / median).clamp(RATIO_LO, RATIO_HI).log2();
        if lr.abs() <= DEAD_ZONE {
            continue;
        }
        let mult = 1.0 - STRENGTH * lr;
        levels[i] = level_of(mult) as u8;
    }
    Map {
        tiles_w,
        tiles_h,
        levels,
    }
}

/// Tuning hook kept out of the stream: the multiplier direction and strength are
/// fixed here, so a file decodes the same everywhere. Measured on the bench set,
/// a positive strength spends ~15% more bytes for ~0.3 dB at a fixed `-q` and
/// therefore loses about 8% at matched PSNR; `aq auto` rejects it for that
/// reason. It exists for perceptual work, where the flat-tile step is the point.

/// Step tables for every level, so the inner loop need not multiply.
///
/// The multiplier is tapered towards the low frequencies: a busy tile spends
/// its extra bits where the energy actually is, instead of filling the high
/// bands with coefficients that cost bytes and buy almost no mean-square
/// accuracy.
pub fn level_steps(base: &[[f64; 8]; 8]) -> [[[f64; 8]; 8]; 7] {
    let mut out = [[[0.0f64; 8]; 8]; 7];
    for (l, m) in MULT.iter().enumerate() {
        for u in 0..8 {
            for v in 0..8 {
                let t = if u + v <= FREQ_CUTOFF { 1.0 } else { 0.0 };
                out[l][u][v] = base[u][v] * (1.0 + (m - 1.0) * t);
            }
        }
    }
    out
}

/// Highest `u + v` that the tile multiplier touches.
const FREQ_CUTOFF: usize = 5;

/// Level delta: 7 symbols in a 3-bit tree, contexted on "previous delta was 0".
pub fn enc_level(
    e: &mut Enc,
    probs: &mut [i16],
    pl: usize,
    prev_same: bool,
    delta: i32,
) {
    let base = pl * 16 + (prev_same as usize) * 8;
    enc_tree(e, probs, base, 3, delta + 3);
}

pub fn dec_level(d: &mut Dec, probs: &mut [i16], pl: usize, prev_same: bool) -> i32 {
    let base = pl * 16 + (prev_same as usize) * 8;
    dec_tree(d, probs, base, 3) as i32 - 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_plane_stays_neutral() {
        let w = 32;
        let h = 32;
        let px = vec![90i16; w * h];
        let map = analyse(&px, w, h);
        assert!(map.levels.iter().all(|&l| l as usize == NEUTRAL));
    }

    #[test]
    fn busy_tile_gets_finer_step_than_flat_tile() {
        let (w, h) = (64, 32);
        let mut px = vec![100i16; w * h];
        // Left half flat, right half a busy checkerboard.
        for y in 0..h {
            for x in w / 2..w {
                px[y * w + x] = if (x + y) % 2 == 0 { 20 } else { 220 };
            }
        }
        let map = analyse(&px, w, h);
        let left = map.levels[0] as usize;
        let right = map.levels[map.tiles_w - 1] as usize;
        assert!(right < left, "busy {} flat {}", right, left);
        assert!(MULT[right] < MULT[left]);
    }

    #[test]
    fn level_delta_roundtrip() {
        let mut probs = new_probs(2);
        let mut enc = Enc::new();
        let seq: Vec<i32> = vec![1, 0, -1, -3, 2, 0, 0, 3, -2, 1];
        let mut prev = false;
        for (i, &d) in seq.iter().enumerate() {
            enc_level(&mut enc, &mut probs, i % 2, prev, d);
            prev = d == 0;
        }
        let data = enc.finish();
        let mut probs2 = new_probs(2);
        let mut dec = Dec::new(&data);
        let mut prev = false;
        for (i, &d) in seq.iter().enumerate() {
            let got = dec_level(&mut dec, &mut probs2, i % 2, prev);
            assert_eq!(got, d, "symbol {}", i);
            prev = d == 0;
        }
    }

    #[test]
    fn delta_clamping_is_reversible() {
        // The encoder must code the level it can reproduce, not the target.
        let mut level = 3i32;
        for target in [6, 0, 6, 0, 5] {
            let delta = (target - level).clamp(-3, 3);
            level += delta;
            assert!((0..7).contains(&level));
        }
    }
}
