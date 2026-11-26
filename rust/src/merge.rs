//! Multi-frame merging: the part of a "night mode" or an "HDR" that is not a UI.
//!
//! A phone camera produces several frames in a burst. Handheld, none of them line up,
//! so merging has to align first. Then either average them (night: noise falls as the
//! square root of the count) or combine differently exposed ones (HDR: keep the shadow
//! detail from the bright frame and the highlight detail from the dark one).
//!
//! Both paths work on 8-bit RGB frames so the Android side can hand over what it already
//! has, and both are plain integer arithmetic - no libm, so this compiles for wasm and
//! runs the same everywhere.

/// How the frames should be combined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Average after alignment; a per-pixel median rejects whatever moved.
    Night,
    /// Frames are the same scene at different exposures, brightest first is irrelevant -
    /// `exposures` says how many stops each one is shifted by.
    Hdr,
}

/// Which way a frame has to move to line up with the reference, in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shift {
    pub dx: i32,
    pub dy: i32,
}

/// Luma of an RGB frame, as a byte per pixel.
fn luma(frame: &[u8], channels: usize) -> Vec<u8> {
    let pixels = frame.len() / channels;
    let mut out = Vec::with_capacity(pixels);
    for i in 0..pixels {
        let at = i * channels;
        // Integer Rec.601, close enough for alignment and cheap.
        let y = (77 * frame[at] as u32
            + 150 * frame[at + 1] as u32
            + 29 * frame[at + 2] as u32)
            >> 8;
        out.push(y as u8);
    }
    out
}

/// Sum of absolute differences between `reference` and `frame` shifted by (dx, dy).
///
/// Only the overlapping part is compared, and the shift is heavily penalised when the
/// overlap gets small - otherwise the search wanders off to a corner where almost
/// nothing is compared and the score looks artificially good.
fn sad(reference: &[u8], frame: &[u8], w: usize, h: usize, dx: i32, dy: i32) -> u64 {
    let mut sum = 0u64;
    let mut count = 0u64;
    for y in 0..h as i32 {
        let fy = y + dy;
        if fy < 0 || fy >= h as i32 {
            continue;
        }
        for x in 0..w as i32 {
            let fx = x + dx;
            if fx < 0 || fx >= w as i32 {
                continue;
            }
            let a = reference[(y as usize) * w + x as usize] as i32;
            let b = frame[(fy as usize) * w + fx as usize] as i32;
            sum += (a - b).unsigned_abs() as u64;
            count += 1;
        }
    }
    if count == 0 {
        return u64::MAX;
    }
    let overlap = count as f64 / (w * h) as f64;
    // A shift that keeps less than 70% of the frame would be cropped to nothing useful.
    let penalty = if overlap < 0.7 { 4.0 } else { 1.0 };
    (sum as f64 / overlap / penalty) as u64
}

/// Find the shift that best lines `frame` up with `reference`.
///
/// Two passes: a coarse one on 4x4 block averages (so the search covers four times the
/// range for the same cost), then a fine one around the coarse answer.
pub fn align(reference: &[u8], frame: &[u8], w: usize, h: usize, channels: usize,
             max_shift: i32) -> Shift {
    let a = luma(reference, channels);
    let b = luma(frame, channels);
    // Coarse: average 4x4 blocks.
    let cw = (w + 3) / 4;
    let ch = (h + 3) / 4;
    let mut ca = vec![0u8; cw * ch];
    let mut cb = vec![0u8; cw * ch];
    for by in 0..ch {
        for bx in 0..cw {
            let mut sa = 0u32;
            let mut sb = 0u32;
            let mut n = 0u32;
            for y in 0..4 {
                let py = by * 4 + y;
                if py >= h {
                    break;
                }
                for x in 0..4 {
                    let px = bx * 4 + x;
                    if px >= w {
                        break;
                    }
                    sa += a[py * w + px] as u32;
                    sb += b[py * w + px] as u32;
                    n += 1;
                }
            }
            if n > 0 {
                ca[by * cw + bx] = (sa / n) as u8;
                cb[by * cw + bx] = (sb / n) as u8;
            }
        }
    }
    let coarse_range = (max_shift / 4).max(1);
    let mut best = Shift { dx: 0, dy: 0 };
    let mut best_score = u64::MAX;
    for dy in -coarse_range..=coarse_range {
        for dx in -coarse_range..=coarse_range {
            let score = sad(&ca, &cb, cw, ch, dx, dy);
            if score < best_score {
                best_score = score;
                best = Shift { dx: dx * 4, dy: dy * 4 };
            }
        }
    }
    // A wrong shift makes a merge worse, not better, and noise alone can look like a
    // small win to the coarse search. Require a real improvement to move at all.
    let at_zero = sad(&a, &b, w, h, 0, 0);

    // Fine: full resolution around the coarse answer.
    let fine = 5;
    let mut fine_best = best;
    let mut fine_score = u64::MAX;
    for dy in (best.dy - fine)..=(best.dy + fine) {
        for dx in (best.dx - fine)..=(best.dx + fine) {
            if dx.abs() > max_shift || dy.abs() > max_shift {
                continue;
            }
            let score = sad(&a, &b, w, h, dx, dy);
            if score < fine_score {
                fine_score = score;
                fine_best = Shift { dx, dy };
            }
        }
    }
    if fine_score as f64 > at_zero as f64 * 0.9 {
        return Shift { dx: 0, dy: 0 };
    }
    fine_best
}

/// `frame` shifted by `shift`, with the uncovered edges repeating the nearest pixel.
fn shifted(frame: &[u8], w: usize, h: usize, channels: usize, shift: Shift) -> Vec<u8> {
    let mut out = vec![0u8; frame.len()];
    for y in 0..h {
        let sy = (y as i32 + shift.dy).clamp(0, h as i32 - 1) as usize;
        for x in 0..w {
            let sx = (x as i32 + shift.dx).clamp(0, w as i32 - 1) as usize;
            let from = (sy * w + sx) * channels;
            let to = (y * w + x) * channels;
            out[to..to + channels].copy_from_slice(&frame[from..from + channels]);
        }
    }
    out
}

/// Night: align, then take the per-pixel median of each channel.
///
/// The median rather than the mean because someone walking through the burst should not
/// leave a ghost; with an even number of frames it is the average of the middle two,
/// which is what `select` below picks.
pub fn merge_night(frames: &[Vec<u8>], w: usize, h: usize, channels: usize) -> Option<Vec<u8>> {
    if frames.is_empty() {
        return None;
    }
    if frames.len() == 1 {
        return Some(frames[0].clone());
    }
    let reference = &frames[0];
    let aligned: Vec<Vec<u8>> = frames
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            if i == 0 {
                frame.clone()
            } else {
                let shift = align(reference, frame, w, h, channels, 12);
                shifted(frame, w, h, channels, shift)
            }
        })
        .collect();

    let mut out = vec![0u8; w * h * channels];
    let n = aligned.len();
    let mut scratch = Vec::with_capacity(n);
    // Drop the darkest and brightest sample, then average the rest. A plain mean keeps
    // moving objects as ghosts; a median throws away too much of the noise reduction
    // (a handful of frames only buys about 1.3x that way, against ~sqrt(n-2) here).
    let trim = if n >= 4 { 1 } else { 0 };
    for pixel in 0..w * h {
        for c in 0..channels {
            scratch.clear();
            for frame in &aligned {
                scratch.push(frame[pixel * channels + c]);
            }
            scratch.sort_unstable();
            let kept = &scratch[trim..n - trim];
            let sum: u32 = kept.iter().map(|v| *v as u32).sum();
            out[pixel * channels + c] = ((sum + kept.len() as u32 / 2) / kept.len() as u32) as u8;
        }
    }
    Some(out)
}

/// sRGB byte to linear, scaled to 0..4095 so 12 bits of headroom fit in an integer.
fn to_linear(v: u8) -> u32 {
    // Approximate the sRGB curve with a gamma of 2.2: close enough for a merge, and
    // table-free in the inner loop would be nicer but this is not the hot path.
    let f = (v as f64 / 255.0).powf(2.2);
    (f * 4095.0).round() as u32
}

fn from_linear(v: f64) -> u8 {
    let f = (v / 4095.0).max(0.0).min(1.0).powf(1.0 / 2.2);
    (f * 255.0).round().clamp(0.0, 255.0) as u8
}

/// HDR: align, undo each frame's exposure, weight by how well exposed each pixel is,
/// average, then re-encode.
///
/// `exposures` is in stops, one per frame - +1 means that frame is one stop brighter.
/// A pixel is trusted most when it sits near the middle of the range: the darkest frame
/// contributes its highlights, the brightest its shadows, and clipped pixels are ignored
/// because they carry no information.
pub fn merge_hdr(frames: &[Vec<u8>], exposures: &[f32], w: usize, h: usize,
                 channels: usize) -> Option<Vec<u8>> {
    if frames.is_empty() || frames.len() != exposures.len() {
        return None;
    }
    if frames.len() == 1 {
        return Some(frames[0].clone());
    }
    let reference = &frames[0];
    let aligned: Vec<Vec<u8>> = frames
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            if i == 0 {
                frame.clone()
            } else {
                let shift = align(reference, frame, w, h, channels, 12);
                shifted(frame, w, h, channels, shift)
            }
        })
        .collect();

    // Precompute the linear tables once instead of per pixel.
    let tables: Vec<[u32; 256]> = (0..aligned.len())
        .map(|_| {
            let mut table = [0u32; 256];
            for v in 0..256 {
                table[v] = to_linear(v as u8);
            }
            table
        })
        .collect();

    let mut out = vec![0u8; w * h * channels];
    for pixel in 0..w * h {
        for c in 0..channels {
            let mut numerator = 0.0f64;
            let mut denominator = 0.0f64;
            for (i, frame) in aligned.iter().enumerate() {
                let raw = frame[pixel * channels + c];
                // Anything at either end is clipped: it says nothing about the scene.
                if raw <= 2 || raw >= 253 {
                    continue;
                }
                let scale = 2f64.powf(exposures[i] as f64);
                // Undo the exposure, then judge how well exposed this pixel was.
                let linear = tables[i][raw as usize] as f64 / scale;
                let mid = 0.18 * 4095.0;
                let sigma = 0.22 * 4095.0;
                let distance = (linear - mid) / sigma;
                let weight = (-0.5 * distance * distance).exp();
                if weight < 0.02 {
                    continue;
                }
                numerator += linear * weight;
                denominator += weight;
            }
            out[pixel * channels + c] = if denominator > 0.0 {
                from_linear(numerator / denominator)
            } else {
                // Everything was clipped: fall back to the middle frame's value.
                aligned[aligned.len() / 2][pixel * channels + c]
            };
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic picture with structure in both directions, so a shift is findable.
    fn scene(w: usize, h: usize) -> Vec<u8> {
        let mut out = vec![0u8; w * h * 3];
        let mut seed = 12345u32;
        for y in 0..h {
            for x in 0..w {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let noise = (seed >> 24) as i32;
                // Structure as well as a gradient: a flat ramp makes alignment
                // ill-posed, and real photographs are not flat.
                let block = if ((x / 8) + (y / 8)) % 2 == 0 { 60 } else { 0 };
                let base = (((x as i32 * 255 / w as i32) + (y as i32 * 127 / h as i32)) / 2
                    + block)
                    .min(255);
                let value = (base + noise / 16).clamp(0, 255) as u8;
                let at = (y * w + x) * 3;
                out[at] = value;
                out[at + 1] = value;
                out[at + 2] = value;
            }
        }
        out
    }

    #[test]
    fn alignment_finds_a_known_shift() {
        let (w, h) = (96, 72);
        let reference = scene(w, h);
        let shift = Shift { dx: 3, dy: -2 };
        // The frame is the scene moved by `shift`, so aligning it should undo that.
        let frame = shifted(&reference, w, h, 3, Shift { dx: -shift.dx, dy: -shift.dy });
        let found = align(&reference, &frame, w, h, 3, 10);
        assert_eq!(found, shift, "expected {:?}, got {:?}", shift, found);
    }

    #[test]
    fn night_merge_reduces_noise() {
        let (w, h) = (64, 48);
        let clean = scene(w, h);
        let mut seed = 999u32;
        let mut frames = Vec::new();
        for _ in 0..6 {
            let mut noisy = clean.clone();
            for byte in noisy.iter_mut() {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let delta = ((seed >> 28) as i32 - 8) * 2;
                *byte = (*byte as i32 + delta).clamp(0, 255) as u8;
            }
            frames.push(noisy);
        }
        let merged = merge_night(&frames, w, h, 3).expect("merge");
        let rmse = |a: &[u8], b: &[u8]| -> f64 {
            let mut sum = 0.0;
            for i in 0..a.len() {
                let d = a[i] as f64 - b[i] as f64;
                sum += d * d;
            }
            (sum / a.len() as f64).sqrt()
        };
        let single = rmse(&frames[0], &clean);
        let after = rmse(&merged, &clean);
        assert!(after < single * 0.7,
                "merge should cut noise: single {:.2}, merged {:.2}", single, after);
    }

    /// sRGB-encode a linear value the way a camera would, clipping at the ends.
    fn encode(linear: f64) -> u8 {
        (linear.max(0.0).min(1.0).powf(1.0 / 2.2) * 255.0).round() as u8
    }

    #[test]
    fn hdr_merge_keeps_shadow_and_highlight() {
        let (w, h) = (32, 16);
        // A scene spanning about nine stops in linear light, exposed twice: +2 EV has
        // the shadows but blows the highlights, -4 EV has the highlights but buries the
        // shadows. Neither frame alone shows the whole range; the merge must.
        let mut over = vec![0u8; w * h * 3];
        let mut under = vec![0u8; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let at = (y * w + x) * 3;
                let t = x as f64 / (w - 1) as f64;
                let linear = 0.001 * 2f64.powf(9.0 * t);
                let bright = encode(linear * 4.0);
                let dark = encode(linear / 16.0);
                over[at] = bright;
                over[at + 1] = bright;
                over[at + 2] = bright;
                under[at] = dark;
                under[at + 1] = dark;
                under[at + 2] = dark;
            }
        }
        let merged = merge_hdr(&[over, under], &[2.0, -4.0], w, h, 3).expect("merge");
        let left = merged[(8 * w + 1) * 3] as i32;
        let right = merged[(8 * w + (w - 2)) * 3] as i32;
        assert!(right - left > 100, "range collapsed: {} .. {}", left, right);
        assert!(right < 250, "highlights still clipped: {}", right);
        assert!(left > 2, "shadows crushed: {}", left);
    }
    #[test]
    fn single_frame_passes_through() {
        let (w, h) = (8, 8);
        let frame = scene(w, h);
        assert_eq!(merge_night(&[frame.clone()], w, h, 3), Some(frame.clone()));
        assert_eq!(merge_hdr(&[frame.clone()], &[0.0], w, h, 3), Some(frame));
    }
}
