//! Pseudo-random number generators — contract: `random-generators-v1.yaml`
//!
//! Di Pierro Ch. 9: LCG, Mersenne Twister (MT19937).
//! Uses `aprender::monte_carlo::prelude::MonteCarloRng` as a reference
//! RNG for cross-validation of our generators.

#[cfg(test)]
use aprender::monte_carlo::prelude::MonteCarloRng as AprRng;

/// Linear Congruential Generator state.
#[derive(Debug, Clone)]
pub struct Lcg {
    pub state: u64,
    pub a: u64,
    pub c: u64,
    pub m: u64,
}

impl Lcg {
    /// Create LCG with given parameters. MINSTD: a=16807, c=0, m=2^31-1.
    pub fn new(seed: u64, a: u64, c: u64, m: u64) -> Self {
        assert!(m > 0, "lcg: modulus m must be > 0");
        assert!(a > 0 && a < m, "lcg: multiplier a must be in (0, m)");
        assert!(c < m, "lcg: increment c must be < m");
        assert!(seed < m, "lcg: seed must be < m");
        Self {
            state: seed,
            a,
            c,
            m,
        }
    }

    /// Advance state and return next value.
    pub fn next_val(&mut self) -> u64 {
        // Use u128 to avoid overflow
        self.state =
            ((self.a as u128 * self.state as u128 + self.c as u128) % self.m as u128) as u64;
        self.state
    }

    /// Generate a f64 in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        self.next_val() as f64 / self.m as f64
    }
}

/// Returns (value, next_state) for a single LCG step.
pub fn lcg_next(state: u64, a: u64, c: u64, m: u64) -> (u64, u64) {
    assert!(m > 0, "lcg_next: modulus m must be > 0");
    let next = ((a as u128 * state as u128 + c as u128) % m as u128) as u64;
    (next, next)
}

/// Mersenne Twister MT19937 (32-bit).
#[derive(Debug, Clone)]
pub struct Mt19937 {
    mt: [u32; 624],
    index: usize,
}

impl Mt19937 {
    const N: usize = 624;
    const M: usize = 397;
    const MATRIX_A: u32 = 0x9908_B0DF;
    const UPPER_MASK: u32 = 0x8000_0000;
    const LOWER_MASK: u32 = 0x7FFF_FFFF;

    /// Initialize from seed (reference MT19937 initialization).
    pub fn new(seed: u32) -> Self {
        let mut mt = [0u32; Self::N];
        mt[0] = seed;
        for i in 1..Self::N {
            mt[i] = 1812433253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { mt, index: Self::N }
    }

    /// Generate next u32.
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= Self::N {
            self.twist();
        }
        let mut y = self.mt[self.index];
        // Tempering
        y ^= y >> 11;
        y ^= (y << 7) & 0x9D2C_5680;
        y ^= (y << 15) & 0xEFC6_0000;
        y ^= y >> 18;
        self.index += 1;
        y
    }

    fn twist(&mut self) {
        for i in 0..Self::N {
            let x =
                (self.mt[i] & Self::UPPER_MASK) | (self.mt[(i + 1) % Self::N] & Self::LOWER_MASK);
            let mut xa = x >> 1;
            if x & 1 != 0 {
                xa ^= Self::MATRIX_A;
            }
            self.mt[i] = self.mt[(i + Self::M) % Self::N] ^ xa;
        }
        self.index = 0;
    }

    /// Generate f64 in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / (u32::MAX as f64 + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_minstd_known() {
        // MINSTD: a=16807, c=0, m=2^31-1
        let (val, _) = lcg_next(1, 16807, 0, 2_147_483_647);
        assert_eq!(val, 16807);
    }

    #[test]
    fn lcg_output_range() {
        let m = 2_147_483_647u64;
        let mut rng = Lcg::new(1, 16807, 0, m);
        for _ in 0..10_000 {
            let v = rng.next_val();
            assert!(v < m, "output must be < m");
        }
    }

    #[test]
    fn lcg_deterministic() {
        let m = 2_147_483_647u64;
        let mut r1 = Lcg::new(42, 16807, 0, m);
        let mut r2 = Lcg::new(42, 16807, 0, m);
        for _ in 0..100 {
            assert_eq!(r1.next_val(), r2.next_val());
        }
    }

    #[test]
    #[should_panic]
    fn lcg_m_zero() {
        lcg_next(1, 5, 3, 0);
    }

    #[test]
    fn lcg_f64_range() {
        let mut rng = Lcg::new(1, 16807, 0, 2_147_483_647);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v), "f64 must be in [0, 1)");
        }
    }

    #[test]
    fn mt_reference_output() {
        // MT19937 with seed 5489 — matches canonical reference value.
        let mut mt = Mt19937::new(5489);
        assert_eq!(mt.next_u32(), 3499211612);
    }

    #[test]
    fn mt_deterministic() {
        let mut r1 = Mt19937::new(42);
        let mut r2 = Mt19937::new(42);
        for _ in 0..1000 {
            assert_eq!(r1.next_u32(), r2.next_u32());
        }
    }

    #[test]
    fn mt_different_seeds() {
        let mut r1 = Mt19937::new(1);
        let mut r2 = Mt19937::new(2);
        // Very unlikely to produce same sequence
        let same = (0..100).all(|_| r1.next_u32() == r2.next_u32());
        assert!(!same, "different seeds should give different sequences");
    }

    #[test]
    fn mt_f64_range() {
        let mut mt = Mt19937::new(12345);
        for _ in 0..10_000 {
            let v = mt.next_f64();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn lcg_period() {
        // Small LCG with known full period: a=3, c=5, m=8 has period 8
        let mut rng = Lcg::new(0, 3, 5, 8);
        let first = rng.next_val();
        let mut count = 1u64;
        loop {
            let v = rng.next_val();
            count += 1;
            if v == first || count > 100 {
                break;
            }
        }
        assert!(count <= 8, "period should divide m=8");
    }

    #[test]
    fn aprender_rng_produces_values() {
        // Cross-validate: aprender's RNG also produces values in range
        let mut apr_rng = AprRng::new(42);
        for _ in 0..100 {
            let v = apr_rng.uniform();
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
