//! Coefficient block coder: DC difference, last position, significance,
//! magnitude class, sign and extra bits over eight zig-zag bands.
//!
//! Two model layouts exist:
//!
//! * `LAYOUT_FLAT` (container version 1): one probability per plane, band and
//!   magnitude-tree node. Every file written by the Python encoder uses it, so
//!   it stays exactly as it was.
//! * `LAYOUT_CTX` (version 2): the same symbols, except that the `last` symbol
//!   is conditioned on the previous block's `last`. Neighbouring blocks in a
//!   picture have similar sparsity, and the decoder has already decoded that
//!   block, so the information is free.
//!
//! Two further contexts were measured and dropped: conditioning significance on
//! the previous block's non-zero pattern, and conditioning the DC class on the
//! previous block's DC. Both diluted the per-band probabilities and made files
//! larger; see README "Adaptive quantisation and contexts".

use crate::arith::{dec_bits, dec_tree, enc_bits, enc_tree, Dec, Enc, PROB_INIT};

pub const LAYOUT_FLAT: u8 = 1;
pub const LAYOUT_CTX: u8 = 2;

/// What the previously coded block looked like.
#[derive(Clone, Copy, Default, Debug)]
pub struct Left {
    /// Highest non-zero zig-zag index of the previous block.
    pub last: usize,
    /// Quantised DC of the previous block.
    pub dc: i32,
    /// Bit `b` set when the previous block had a non-zero in band `b`.
    pub nz_mask: u8,
}

impl Left {
    pub fn of(qz: &[i32; 64]) -> Left {
        let mut last = 0usize;
        for i in (1..64).rev() {
            if qz[i] != 0 {
                last = i;
                break;
            }
        }
        let mut nz_mask = 0u8;
        for i in 1..=last {
            if qz[i] != 0 {
                nz_mask |= 1u8 << (i >> 3);
            }
        }
        Left {
            last,
            dc: qz[0],
            nz_mask,
        }
    }
}

pub struct Model {
    layout: u8,
    planes: usize,
    probs: Vec<i16>,
}

impl Model {
    pub fn new(layout: u8, planes: usize) -> Model {
        Model {
            layout,
            planes,
            probs: vec![PROB_INIT; Model::per_plane(layout) * planes],
        }
    }

    pub fn layout(&self) -> u8 {
        self.layout
    }

    /// Probability slots per plane.
    pub fn per_plane(layout: u8) -> usize {
        if layout == LAYOUT_CTX {
            817
        } else {
            369
        }
    }

    /// Group base offsets: dc_cls, dc_sign, dc_extra, last, sig, ac_cls,
    /// ac_sign, ac_extra.
    fn bases(&self, pl: usize) -> [usize; 8] {
        let p = self.planes;
        if self.layout == LAYOUT_CTX {
            [
                pl * 16,
                16 * p + pl,
                17 * p + pl * 16,
                33 * p + pl * 512,
                545 * p + pl * 8,
                553 * p + pl * 128,
                681 * p + pl * 8,
                689 * p + pl * 128,
            ]
        } else {
            [
                pl * 16,
                16 * p + pl,
                17 * p + pl * 16,
                33 * p + pl * 64,
                97 * p + pl * 8,
                105 * p + pl * 128,
                233 * p + pl * 8,
                241 * p + pl * 128,
            ]
        }
    }
}

/// Residual model for the lossless paths: per plane, five activity classes, each
/// with its own magnitude-class tree, sign probability and magnitude-bit
/// probabilities. The indices are private to this crate; only encoder and
/// decoder agreeing matters.
pub struct ResidualModel {
    probs: Vec<i16>,
}

/// Slots per plane: 5 class trees of 16 nodes, 5 signs, 5 sets of 16 bits.
const RES_PLANE: usize = 165;

impl ResidualModel {
    pub fn new(planes: usize) -> ResidualModel {
        ResidualModel {
            probs: vec![PROB_INIT; planes * RES_PLANE],
        }
    }

    #[inline]
    fn class_base(&self, pl: usize, ctx: usize) -> usize {
        pl * RES_PLANE + ctx.min(4) * 16
    }

    #[inline]
    fn sign_base(&self, pl: usize, ctx: usize) -> usize {
        pl * RES_PLANE + 80 + ctx.min(4)
    }

    #[inline]
    fn extra_base(&self, pl: usize, ctx: usize) -> usize {
        pl * RES_PLANE + 85 + ctx.min(4) * 16
    }

    pub fn enc_res(&mut self, e: &mut Enc, pl: usize, ctx: usize, value: i32) {
        let class = self.class_base(pl, ctx);
        let sign = self.sign_base(pl, ctx);
        let extra = self.extra_base(pl, ctx);
        let probs = &mut self.probs[..];
        let mut v = value;
        let mut sg = 0u32;
        if v < 0 {
            v = -v;
            sg = 1;
        }
        let c = bit_len(v);
        enc_tree(e, probs, class, 4, c as i32);
        if c > 0 {
            e.bit(probs, sign, sg);
            if c > 1 {
                enc_bits(e, probs, extra, c - 1, v - (1 << (c - 1)));
            }
        }
    }

    pub fn dec_res(&mut self, d: &mut Dec, pl: usize, ctx: usize) -> i32 {
        let class = self.class_base(pl, ctx);
        let sign = self.sign_base(pl, ctx);
        let extra = self.extra_base(pl, ctx);
        let probs = &mut self.probs[..];
        let c = dec_tree(d, probs, class, 4);
        if c == 0 {
            return 0;
        }
        let sg = d.bit(probs, sign);
        let mut v = 1i32 << (c - 1);
        if c > 1 {
            v += dec_bits(d, probs, extra, c - 1) as i32;
        }
        if sg != 0 {
            -v
        } else {
            v
        }
    }
}

/// Context for the `last` symbol, from the previous block's `last`.
#[inline(always)]
fn last_ctx(layout: u8, left: &Left) -> usize {
    if layout == LAYOUT_CTX {
        (left.last >> 3).min(7)
    } else {
        0
    }
}

#[inline]
fn bit_len(v: i32) -> u32 {
    let mut n = 0;
    let mut x = v;
    while x > 0 {
        n += 1;
        x >>= 1;
    }
    n
}

pub fn enc_block(
    e: &mut Enc,
    m: &mut Model,
    pl: usize,
    qc: &[i32; 64],
    prev_dc: i32,
    left: &Left,
) {
    let b = m.bases(pl);
    let layout = m.layout;
    let lctx = last_ctx(layout, left);
    let probs = &mut m.probs[..];

    let mut d = qc[0] - prev_dc;
    let mut sign = 0u32;
    if d < 0 {
        d = -d;
        sign = 1;
    }
    let c = bit_len(d);
    enc_tree(e, probs, b[0], 4, c as i32);
    if c > 0 {
        e.bit(probs, b[1], sign);
        if c > 1 {
            enc_bits(e, probs, b[2], c - 1, d - (1 << (c - 1)));
        }
    }

    let mut last = 0usize;
    for i in (1..64).rev() {
        if qc[i] != 0 {
            last = i;
            break;
        }
    }
    enc_tree(e, probs, b[3] + lctx * 64, 6, last as i32);

    for i in 1..=last {
        let band = i >> 3;
        let mut v = qc[i];
        if v != 0 {
            e.bit(probs, b[4] + band, 1);
            let mut sg = 0u32;
            if v < 0 {
                v = -v;
                sg = 1;
            }
            let c2 = bit_len(v);
            enc_tree(e, probs, b[5] + band * 16, 4, c2 as i32);
            e.bit(probs, b[6] + band, sg);
            if c2 > 1 {
                enc_bits(e, probs, b[7] + band * 16, c2 - 1, v - (1 << (c2 - 1)));
            }
        } else {
            e.bit(probs, b[4] + band, 0);
        }
    }
}

#[inline]
pub fn dec_block(
    d: &mut Dec,
    m: &mut Model,
    pl: usize,
    prev_dc: i32,
    left: &Left,
) -> [i32; 64] {
    let b = m.bases(pl);
    let layout = m.layout;
    let lctx = last_ctx(layout, left);
    let probs = &mut m.probs[..];

    let c = dec_tree(d, probs, b[0], 4);
    let mut dc = 0i32;
    if c > 0 {
        let sign = d.bit(probs, b[1]);
        let mut mag = 1i32 << (c - 1);
        if c > 1 {
            mag += dec_bits(d, probs, b[2], c - 1) as i32;
        }
        dc = if sign != 0 { -mag } else { mag };
    }
    dc += prev_dc;

    let last = dec_tree(d, probs, b[3] + lctx * 64, 6) as usize;
    let mut qc = [0i32; 64];
    qc[0] = dc;
    for i in 1..=last {
        let band = i >> 3;
        if d.bit(probs, b[4] + band) != 0 {
            let cls = dec_tree(d, probs, b[5] + band * 16, 4);
            let sign = d.bit(probs, b[6] + band);
            let mut mag = 1i32 << (cls - 1);
            if cls > 1 {
                mag += dec_bits(d, probs, b[7] + band * 16, cls - 1) as i32;
            }
            qc[i] = if sign != 0 { -mag } else { mag };
        }
    }
    qc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(planes: usize, n: usize) -> Vec<([i32; 64], usize, i32)> {
        let mut out = Vec::with_capacity(n);
        let mut seed = 7u32;
        let mut prev = vec![0i32; planes];
        for k in 0..n {
            let pl = k % planes;
            let mut qc = [0i32; 64];
            for i in 0..64 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let r = (seed >> 24) as i32 % 17 - 8;
                qc[i] = if (seed >> 8) & 3 == 0 { r } else { 0 };
            }
            // Occasional all-zero blocks, so the contexts see that case too.
            if k % 5 == 0 {
                qc = [0i32; 64];
            }
            out.push((qc, pl, prev[pl]));
            prev[pl] = qc[0];
        }
        out
    }

    fn roundtrip(layout: u8, planes: usize) {
        let blocks = blocks(planes, 240);
        let mut enc = Enc::new();
        let mut m = Model::new(layout, planes);
        let mut left = vec![Left::default(); planes];
        for (qc, pl, pdc) in blocks.iter() {
            enc_block(&mut enc, &mut m, *pl, qc, *pdc, &left[*pl]);
            left[*pl] = Left::of(qc);
        }
        let data = enc.finish();

        let mut dec = Dec::new(&data);
        let mut m2 = Model::new(layout, planes);
        let mut left2 = vec![Left::default(); planes];
        for (qc, pl, pdc) in blocks.iter() {
            let got = dec_block(&mut dec, &mut m2, *pl, *pdc, &left2[*pl]);
            assert_eq!(got, *qc, "layout {} plane {}", layout, pl);
            left2[*pl] = Left::of(&got);
        }
    }

    #[test]
    fn flat_layout_roundtrip() {
        roundtrip(LAYOUT_FLAT, 3);
    }

    #[test]
    fn ctx_layout_roundtrip() {
        roundtrip(LAYOUT_CTX, 1);
        roundtrip(LAYOUT_CTX, 3);
        roundtrip(LAYOUT_CTX, 4);
    }

    #[test]
    fn left_context_is_derived_from_the_block() {
        let mut qc = [0i32; 64];
        qc[0] = 12;
        qc[5] = -3;
        qc[19] = 1;
        let l = Left::of(&qc);
        assert_eq!(l.last, 19);
        assert_eq!(l.dc, 12);
        assert_eq!(l.nz_mask & 0b1, 1); // band 0 covers indices 1..7
        assert_eq!((l.nz_mask >> 2) & 1, 1); // band 2 covers 16..23
        assert_eq!((l.nz_mask >> 1) & 1, 0);
    }

    #[test]
    fn group_offsets_fit_the_table() {
        for layout in [LAYOUT_FLAT, LAYOUT_CTX] {
            let m = Model::new(layout, 4);
            assert_eq!(m.probs.len(), Model::per_plane(layout) * 4);
            let b = m.bases(3);
            assert!(b[7] + 128 <= m.probs.len(), "layout {}", layout);
            // Groups must not overlap for the last plane.
            for i in 0..7 {
                assert!(b[i] < b[i + 1], "layout {} group {}", layout, i);
            }
        }
    }
}
