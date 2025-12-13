//! A tiny pseudo-random number generator.
//!
//! This module provides `SplitMix64`, a small PRNG suitable for emulator tasks such as
//! power-on RAM/register randomization. It is **not** cryptographically secure.

/// A small PRNG based on SplitMix64.
///
/// `SplitMix64` maintains a 64-bit state. Each call advances the state by adding a fixed
/// odd constant (mod 2⁶⁴), then scrambles the result to produce the output.
///
/// Any `seed` is valid, including 0. Different seeds yield different sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a new generator seeded with `seed`.
    ///
    /// Note: `seed` does not need to be non-zero.
    #[inline]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advances the generator and returns the next pseudo-random `u64`.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        // Advance the internal state (mod 2^64).
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);

        // Scramble using a mix of shifts, xors, and multiplications.
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns the next pseudo-random `u32`.
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    /// Returns the next pseudo-random `u8`.
    #[inline]
    pub fn next_u8(&mut self) -> u8 {
        self.next_u64() as u8
    }

    /// Fills `buf` with pseudo-random bytes.
    ///
    /// This is convenient for power-on initialization of RAM or internal latches.
    #[inline]
    pub fn fill_bytes(&mut self, buf: &mut [u8]) {
        // Fill in chunks for speed.
        let mut i = 0;
        while i + 8 <= buf.len() {
            let bytes = self.next_u64().to_le_bytes();
            buf[i..i + 8].copy_from_slice(&bytes);
            i += 8;
        }

        // Fill the tail, if any.
        while i < buf.len() {
            buf[i] = self.next_u8();
            i += 1;
        }
    }

    /// Returns `true` or `false` with approximately equal probability.
    #[inline]
    pub fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) != 0
    }
}

impl Default for SplitMix64 {
    fn default() -> Self {
        // A default seed is convenient for callers that don't care about determinism.
        // If you want reproducibility, construct with `SplitMix64::new(your_seed)`.
        Self::new(0x6A09_E667_F3BC_C909)
    }
}

#[cfg(test)]
mod tests {
    use super::SplitMix64;

    #[test]
    fn seed_zero_is_valid() {
        let mut rng = SplitMix64::new(0);
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b);
    }

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = SplitMix64::new(123);
        let mut b = SplitMix64::new(123);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
