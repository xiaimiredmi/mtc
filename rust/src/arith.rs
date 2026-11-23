//! Binary arithmetic coder, matching the Python implementation bit for bit.
//!
//! 11-bit probabilities, 5-bit adaptation, 24-bit range renormalisation. The
//! decoder is the exact inverse of the encoder, so a stream written here is
//! readable by the old Python decoder while the migration is in progress.

pub const PROB_BITS: u32 = 11;
pub const PROB_TOTAL: u64 = 1 << PROB_BITS;
pub const PROB_INIT: i16 = (PROB_TOTAL / 2) as i16;
pub const MOVE_BITS: u32 = 5;
pub const TOP: u32 = 1 << 24;
const MASK32: u64 = 0xFFFF_FFFF;

pub struct Enc {
    low: u64,
    range: u32,
    cache: u64,
    cache_size: u32,
    out: Vec<u8>,
}

impl Enc {
    pub fn new() -> Enc {
        Enc {
            low: 0,
            range: MASK32 as u32,
            cache: 0,
            cache_size: 1,
            out: Vec::with_capacity(4096),
        }
    }

    #[inline]
    fn shift_low(&mut self) {
        let low = self.low;
        if low < 0xFF00_0000 || low > MASK32 {
            let carry = (low >> 32) as u8;
            self.out.push(self.cache.wrapping_add(carry as u64) as u8);
            let n = self.cache_size - 1;
            if n > 0 {
                let byte = 0xFFu8.wrapping_add(carry);
                for _ in 0..n {
                    self.out.push(byte);
                }
            }
            self.cache = (low >> 24) & 0xFF;
            self.cache_size = 0;
        }
        self.cache_size += 1;
        self.low = ((low & MASK32) << 8) & MASK32;
    }

    #[inline]
    pub fn bit(&mut self, probs: &mut [i16], idx: usize, bit: u32) {
        let p = probs[idx] as u64;
        let bound = ((self.range >> PROB_BITS) as u64) * p;
        if bit != 0 {
            self.low += bound;
            self.range -= bound as u32;
            probs[idx] = (p - (p >> MOVE_BITS)) as i16;
        } else {
            self.range = bound as u32;
            probs[idx] = (p + ((PROB_TOTAL - p) >> MOVE_BITS)) as i16;
        }
        while self.range < TOP {
            self.range <<= 8;
            self.shift_low();
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        for _ in 0..5 {
            self.shift_low();
        }
        self.out
    }
}

pub struct Dec<'a> {
    data: &'a [u8],
    pos: usize,
    range: u32,
    code: u32,
}

impl<'a> Dec<'a> {
    pub fn new(data: &'a [u8]) -> Dec<'a> {
        let mut d = Dec {
            data,
            pos: 0,
            range: MASK32 as u32,
            code: 0,
        };
        for _ in 0..5 {
            d.code = (d.code << 8) | d.next_byte();
        }
        d
    }

    #[inline(always)]
    fn next_byte(&mut self) -> u32 {
        let p = self.pos;
        self.pos = p + 1;
        if p < self.data.len() {
            self.data[p] as u32
        } else {
            0
        }
    }
    /// One binary decision.
    ///
    /// The probability slot is read and written through a raw pointer: with a slice
    /// index the same `idx` is checked twice per bit, and a bit is the inner loop of
    /// the whole decoder. Callers only ever pass an index inside `probs` - the model
    /// layout is fixed and the bases are computed from it - so the check is not
    /// earning anything.
    #[inline(always)]
    pub fn bit(&mut self, probs: &mut [i16], idx: usize) -> u32 {
        debug_assert!(idx < probs.len());
        let slot = unsafe { probs.get_unchecked_mut(idx) };
        let p = *slot as u32;
        let bound = (self.range >> PROB_BITS) * p;
        let bit;
        if self.code < bound {
            self.range = bound;
            *slot = (p + ((PROB_TOTAL as u32 - p) >> MOVE_BITS)) as i16;
            bit = 0;
        } else {
            self.code -= bound;
            self.range -= bound;
            *slot = (p - (p >> MOVE_BITS)) as i16;
            bit = 1;
        }
        while self.range < TOP {
            self.range <<= 8;
            self.code = ((self.code << 8) | self.next_byte()) & (MASK32 as u32);
        }
        bit
    }
}

#[inline(always)]
pub fn enc_tree(e: &mut Enc, probs: &mut [i16], base: usize, bits: u32, value: i32) {
    let mut node = 1usize;
    let mut i = bits as i32 - 1;
    while i >= 0 {
        let bit = ((value >> i) & 1) as u32;
        e.bit(probs, base + node, bit);
        node = (node << 1) | bit as usize;
        i -= 1;
    }
}

#[inline(always)]
pub fn dec_tree(d: &mut Dec, probs: &mut [i16], base: usize, bits: u32) -> u32 {
    let mut node = 1usize;
    let mut i = bits;
    while i > 0 {
        i -= 1;
        let bit = d.bit(probs, base + node) as usize;
        node = (node << 1) | bit;
    }
    (node as u32) - (1u32 << bits)
}

#[inline(always)]
pub fn enc_bits(e: &mut Enc, probs: &mut [i16], base: usize, count: u32, value: i32) {
    let mut i = count as i32 - 1;
    while i >= 0 {
        let bit = ((value >> i) & 1) as u32;
        e.bit(probs, base + i as usize, bit);
        i -= 1;
    }
}

#[inline(always)]
pub fn dec_bits(d: &mut Dec, probs: &mut [i16], base: usize, count: u32) -> u32 {
    let mut v = 0u32;
    let mut i = count;
    while i > 0 {
        i -= 1;
        v |= d.bit(probs, base + i as usize) << i;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_random_bits() {
        let probs = vec![PROB_INIT; 64];
        let mut src = Enc::new();
        let mut bits = Vec::new();
        let mut seed = 12345u32;
        for i in 0..4000usize {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let b = if (seed >> 16) & 3 == 0 { 1 } else { 0 };
            bits.push(b);
            src.bit(&mut probs.clone(), i % 64, b);
        }
        // Re-encode with shared probabilities so the decoder mirrors it.
        let mut probs2 = vec![PROB_INIT; 64];
        let mut enc = Enc::new();
        for (i, &b) in bits.iter().enumerate() {
            enc.bit(&mut probs2, i % 64, b);
        }
        let data = enc.finish();
        let mut probs3 = vec![PROB_INIT; 64];
        let mut dec = Dec::new(&data);
        for (i, &b) in bits.iter().enumerate() {
            assert_eq!(dec.bit(&mut probs3, i % 64), b, "bit {}", i);
        }
    }

    #[test]
    fn tree_roundtrip() {
        let mut probs = vec![PROB_INIT; 64];
        let mut enc = Enc::new();
        for v in 0..16 {
            enc_tree(&mut enc, &mut probs, 0, 4, v);
        }
        let data = enc.finish();
        let mut probs2 = vec![PROB_INIT; 64];
        let mut dec = Dec::new(&data);
        for v in 0..16 {
            assert_eq!(dec_tree(&mut dec, &mut probs2, 0, 4), v as u32);
        }
    }
}
