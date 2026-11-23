//! 8x8 DCT, zig-zag scan and the quantiser table.
//!
//! Blocks are row-major: `blk[y][x]` holds the sample at row `y`, column `x`.
//! The coefficient at `[u][v]` therefore has `u` as the vertical frequency and
//! `v` as the horizontal one, which is the convention the Python encoder
//! established and every existing `.mtc` file is written in.

pub const ZZ: [(usize, usize); 64] = {
    let mut out = [(0usize, 0usize); 64];
    let mut k = 0usize;
    let mut s = 0usize;
    while s < 15 {
        let mut v = 0usize;
        // Even diagonals run down-left, odd diagonals up-right.
        if s % 2 == 0 {
            while v <= s {
                if s - v < 8 && v < 8 {
                    out[k] = (s - v, v);
                    k += 1;
                }
                v += 1;
            }
        } else {
            while v <= s {
                if s - v < 8 && v < 8 {
                    out[k] = (v, s - v);
                    k += 1;
                }
                v += 1;
            }
        }
        s += 1;
    }
    out
};

pub fn dct_matrix() -> [[f64; 8]; 8] {
    let mut m = [[0.0f64; 8]; 8];
    for u in 0..8 {
        for x in 0..8 {
            m[u][x] = (((2 * x + 1) as f64) * (u as f64) * std::f64::consts::PI / 16.0).cos()
                * 0.5;
        }
    }
    for x in 0..8 {
        m[0][x] = (1.0f64 / 8.0).sqrt();
    }
    m
}

/// `out[u][v] = sum_y sum_x M[u][y] blk[y][x] M[v][x]`.
pub fn fdct(blk: &[[f64; 8]; 8], m: &[[f64; 8]; 8]) -> [[f64; 8]; 8] {
    let mut out = [[0.0f64; 8]; 8];
    let mut tmp = [[0.0f64; 8]; 8];
    for u in 0..8 {
        for x in 0..8 {
            let mut s = 0.0;
            for y in 0..8 {
                s += m[u][y] * blk[y][x];
            }
            tmp[u][x] = s;
        }
    }
    for u in 0..8 {
        for v in 0..8 {
            let mut s = 0.0;
            for x in 0..8 {
                s += tmp[u][x] * m[v][x];
            }
            out[u][v] = s;
        }
    }
    out
}

/// `out[y][x] = sum_u sum_v M[u][y] coef[u][v] M[v][x]`.
pub fn idct(coef: &[[f64; 8]; 8], m: &[[f64; 8]; 8]) -> [[f64; 8]; 8] {
    let mut out = [[0.0f64; 8]; 8];
    let mut tmp = [[0.0f64; 8]; 8];
    for u in 0..8 {
        for x in 0..8 {
            let mut s = 0.0;
            for v in 0..8 {
                s += coef[u][v] * m[v][x];
            }
            tmp[u][x] = s;
        }
    }
    for y in 0..8 {
        for x in 0..8 {
            let mut s = 0.0;
            for u in 0..8 {
                s += m[u][y] * tmp[u][x];
            }
            out[y][x] = s;
        }
    }
    out
}

/// The forward matrix as `f32`, built once per frame, both ways round.
///
/// `m[u][x]` is the forward basis in `f32`, built once per frame.
pub struct Basis {
    pub m: [[f32; 8]; 8],
}

impl Basis {
    pub fn new() -> Basis {
        let d = dct_matrix();
        let mut m = [[0.0f32; 8]; 8];
        for u in 0..8 {
            for x in 0..8 {
                m[u][x] = d[u][x] as f32;
            }
        }
        Basis { m }
    }
}

#[inline(always)]
pub fn round_i32(v: f32) -> i32 {
    let shifted = v + 0.5f32.copysign(v);
    shifted.clamp(-32768.0, 32767.0) as i32
}

/// Inverse DCT of one block, in `f32`.
///
/// The passes are written out term by term: a loop over the eight coefficients with
/// a `continue` for zeros reads better but the compiler does not unroll it, and that
/// form measured 70% slower. The first pass writes its transpose so the second also
/// reads straight lines, and both skip all-zero rows and columns.
pub fn idct_f32(coef: &[[f64; 8]; 8], basis: &Basis) -> [[f32; 8]; 8] {
    let m = &basis.m;
    let mut tmp = [[0.0f32; 8]; 8]; // tmp[x][u]
    for u in 0..8 {
        let c = &coef[u];
        if c.iter().all(|v| *v == 0.0) {
            continue;
        }
        let c0 = c[0] as f32;
        let c1 = c[1] as f32;
        let c2 = c[2] as f32;
        let c3 = c[3] as f32;
        let c4 = c[4] as f32;
        let c5 = c[5] as f32;
        let c6 = c[6] as f32;
        let c7 = c[7] as f32;
        for x in 0..8 {
            tmp[x][u] = c0 * m[0][x]
                + c1 * m[1][x]
                + c2 * m[2][x]
                + c3 * m[3][x]
                + c4 * m[4][x]
                + c5 * m[5][x]
                + c6 * m[6][x]
                + c7 * m[7][x];
        }
    }

    let mut out = [[0.0f32; 8]; 8];
    for x in 0..8 {
        let r = &tmp[x];
        if r.iter().all(|v| *v == 0.0) {
            continue;
        }
        let r0 = r[0];
        let r1 = r[1];
        let r2 = r[2];
        let r3 = r[3];
        let r4 = r[4];
        let r5 = r[5];
        let r6 = r[6];
        let r7 = r[7];
        for y in 0..8 {
            out[y][x] = r0 * m[0][y]
                + r1 * m[1][y]
                + r2 * m[2][y]
                + r3 * m[3][y]
                + r4 * m[4][y]
                + r5 * m[5][y]
                + r6 * m[6][y]
                + r7 * m[7][y];
        }
    }
    out
}

/// Inverse DCT straight to 8-bit pixels, four lanes at a time.
///
/// Same arithmetic as `idct_f32` - the two passes are linear combinations of basis
/// rows with scalar coefficients - but two rows' halves ride in one register, and the
/// rounding, clamping and narrowing to `i16` all happen in vectors. 512 scalar
/// multiplies become 128 vector ones.
///
/// SSE2 is part of the x86-64 ABI, so there is nothing to detect at run time; on
/// everything else the scalar path in `idct_block` is used instead.
#[cfg(target_arch = "x86_64")]
pub fn idct_block_i16(coef: &[[f64; 8]; 8], lo: i16, hi: i16, basis: &Basis) -> [[i16; 8]; 8] {
    use std::arch::x86_64::*;
    unsafe {
        let m = &basis.m;
        let mut blo = [_mm_setzero_ps(); 8];
        let mut bhi = [_mm_setzero_ps(); 8];
        for v in 0..8 {
            blo[v] = _mm_setr_ps(m[v][0], m[v][1], m[v][2], m[v][3]);
            bhi[v] = _mm_setr_ps(m[v][4], m[v][5], m[v][6], m[v][7]);
        }

        // Pass one: t[u][x] = sum_v coef[u][v] * m[v][x].
        let mut tlo = [_mm_setzero_ps(); 8];
        let mut thi = [_mm_setzero_ps(); 8];
        for u in 0..8 {
            let mut alo = _mm_setzero_ps();
            let mut ahi = _mm_setzero_ps();
            for v in 0..8 {
                let c = coef[u][v] as f32;
                if c != 0.0 {
                    let cv = _mm_set1_ps(c);
                    alo = _mm_add_ps(alo, _mm_mul_ps(cv, blo[v]));
                    ahi = _mm_add_ps(ahi, _mm_mul_ps(cv, bhi[v]));
                }
            }
            tlo[u] = alo;
            thi[u] = ahi;
        }

        // Pass two: out[y][x] = sum_u m[u][y] * t[u][x].
        let lo_f = _mm_set1_ps(lo as f32);
        let hi_f = _mm_set1_ps(hi as f32);
        let half = _mm_set1_ps(0.5);
        let sign = _mm_set1_ps(-0.0);
        let mut out = [[0i16; 8]; 8];
        for y in 0..8 {
            let mut alo = _mm_setzero_ps();
            let mut ahi = _mm_setzero_ps();
            for u in 0..8 {
                let c = m[u][y];
                if c != 0.0 {
                    let cv = _mm_set1_ps(c);
                    alo = _mm_add_ps(alo, _mm_mul_ps(cv, tlo[u]));
                    ahi = _mm_add_ps(ahi, _mm_mul_ps(cv, thi[u]));
                }
            }
            // Clamp while still in float, then round half away from zero (the same
            // rounding the scalar path does) and narrow.
            let alo = _mm_max_ps(_mm_min_ps(alo, hi_f), lo_f);
            let ahi = _mm_max_ps(_mm_min_ps(ahi, hi_f), lo_f);
            let alo = _mm_add_ps(alo, _mm_or_ps(_mm_and_ps(alo, sign), half));
            let ahi = _mm_add_ps(ahi, _mm_or_ps(_mm_and_ps(ahi, sign), half));
            let ilo = _mm_cvttps_epi32(alo);
            let ihi = _mm_cvttps_epi32(ahi);
            let packed = _mm_packs_epi32(ilo, ihi);
            _mm_storeu_si128(out[y].as_mut_ptr() as *mut __m128i, packed);
        }
        out
    }
}

/// Scalar fallback for targets without SSE2.
#[cfg(not(target_arch = "x86_64"))]
pub fn idct_block_i16(coef: &[[f64; 8]; 8], lo: i16, hi: i16, basis: &Basis) -> [[i16; 8]; 8] {
    let blk = idct_f32(coef, basis);
    let mut out = [[0i16; 8]; 8];
    for y in 0..8 {
        for x in 0..8 {
            out[y][x] = round_i32(blk[y][x]).clamp(lo as i32, hi as i32) as i16;
        }
    }
    out
}

pub fn base_weight(u: usize, v: usize) -> f64 {
    let f = (((u * u + v * v) as f64)).sqrt();
    1.0 + 0.30 * f.powf(1.15)
}

pub fn quality_scale(quality: u8) -> f64 {
    let q = quality.clamp(1, 100) as f64;
    0.85 + (100.0 - q) * 0.40
}

pub fn step_table(quality: u8) -> [[f64; 8]; 8] {
    let scale = quality_scale(quality);
    let mut t = [[0.0f64; 8]; 8];
    for u in 0..8 {
        for v in 0..8 {
            t[u][v] = base_weight(u, v) * scale;
        }
    }
    t
}

/// Round half to even, like numpy's rint.
pub fn rint(x: f64) -> f64 {
    let f = x.floor();
    let frac = x - f;
    if frac > 0.5 {
        f + 1.0
    } else if frac < 0.5 {
        f
    } else if (f as i64) & 1 == 0 {
        f
    } else {
        f + 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zigzag_is_a_permutation() {
        let mut seen = [false; 64];
        for &(u, v) in ZZ.iter() {
            assert!(u < 8 && v < 8);
            assert!(!seen[u * 8 + v]);
            seen[u * 8 + v] = true;
        }
        assert!(seen.iter().all(|&b| b));
    }

    #[test]
    fn zigzag_matches_python_order() {
        // The order the Python encoder wrote: even diagonals descend, odd ones
        // ascend.
        let expect = [
            (0usize, 0usize),
            (0, 1),
            (1, 0),
            (2, 0),
            (1, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (2, 1),
            (3, 0),
        ];
        for (k, e) in expect.iter().enumerate() {
            assert_eq!(ZZ[k], *e, "coefficient {}", k);
        }
    }

    #[test]
    fn fast_idct_agrees_with_the_matrix_form() {
        // Same maths, narrower arithmetic and a different memory order: the pixels
        // have to come out the same to well under half a step.
        let m = dct_matrix();
        let basis = Basis::new();
        let mut worst = 0.0f64;
        let mut seed = 0x0bad_c0de_dead_beefu64;
        let steps = step_table(85);
        for _ in 0..500 {
            let mut coef = [[0.0f64; 8]; 8];
            for k in 0..64 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let r = ((seed >> 33) as i32 % 200) - 100;
                let (u, v) = ZZ[k];
                coef[u][v] = r as f64 * steps[u][v];
            }
            let slow = idct(&coef, &m);
            let fast = idct_f32(&coef, &basis);
            for y in 0..8 {
                for x in 0..8 {
                    let d = (slow[y][x] - fast[y][x] as f64).abs();
                    if d > worst {
                        worst = d;
                    }
                }
            }
        }
        assert!(worst < 0.05, "f32 idct differs by {}", worst);
    }
    #[test]
    fn simd_idct_matches_the_scalar_path_exactly() {
        // Not "close": the same pixels. The vector path only changes how the same
        // arithmetic is packed.
        let basis = Basis::new();
        let steps = step_table(85);
        let mut seed = 0xfeed_face_cafe_beefu64;
        for _ in 0..300 {
            let mut coef = [[0.0f64; 8]; 8];
            for k in 0..64 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let r = ((seed >> 33) as i32 % 260) - 130;
                let (u, v) = ZZ[k];
                coef[u][v] = r as f64 * steps[u][v];
            }
            for clip in [(0i16, 255i16), (-300, 300)] {
                let scalar = idct_f32(&coef, &basis);
                let fast = idct_block_i16(&coef, clip.0, clip.1, &basis);
                for y in 0..8 {
                    for x in 0..8 {
                        let a = round_i32(scalar[y][x]).clamp(clip.0 as i32, clip.1 as i32) as i16;
                        assert_eq!(a, fast[y][x], "pixel {y},{x} clip {clip:?}");
                    }
                }
            }
        }
    }
    #[test]
    fn dct_roundtrip() {
        let m = dct_matrix();
        let mut blk = [[0.0f64; 8]; 8];
        for y in 0..8 {
            for x in 0..8 {
                blk[y][x] = ((x * 17 + y * 31) % 256) as f64 - 128.0;
            }
        }
        let coef = fdct(&blk, &m);
        let back = idct(&coef, &m);
        for y in 0..8 {
            for x in 0..8 {
                assert!((back[y][x] - blk[y][x]).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn dct_axes_are_not_interchangeable() {
        // A vertical ramp must concentrate energy in u, not in v.
        let m = dct_matrix();
        let mut blk = [[0.0f64; 8]; 8];
        for y in 0..8 {
            for x in 0..8 {
                blk[y][x] = (y * 24) as f64;
            }
        }
        let coef = fdct(&blk, &m);
        let vertical: f64 = (1..8).map(|u| coef[u][0].abs()).sum();
        let horizontal: f64 = (1..8).map(|v| coef[0][v].abs()).sum();
        assert!(vertical > 100.0 * horizontal, "{} vs {}", vertical, horizontal);
    }

    #[test]
    fn step_table_matches_python() {
        let t = step_table(85);
        assert!((t[0][0] - 6.85).abs() < 1e-12);
        let f = (2.0f64).sqrt();
        let w11 = 1.0 + 0.30 * f.powf(1.15);
        assert!((t[1][1] - w11 * 6.85).abs() < 1e-12);
    }
}
