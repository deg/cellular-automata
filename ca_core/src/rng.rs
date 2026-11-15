/// A small, fast pseudo-random generator (xorshift64*), for random fills.
///
/// Not cryptographic, and not meant to be: demos need cheap, reproducible noise, and pulling
/// in a crate (plus `getrandom`'s web shims) for that would cost more than it gives.
pub struct Rng(u64);

impl Rng {
    /// A generator started from `seed`. Equal seeds give equal sequences.
    pub fn new(seed: u64) -> Self {
        // splitmix64 spreads every bit of the seed through the state, so neighbouring seeds
        // give unrelated sequences. (An XOR with `| 1` used to merge seeds 2k and 2k+1, so
        // every other Restart repeated the previous random row.)
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // xorshift has a fixed point at zero, so steer clear of it.
        Self(if z == 0 { 1 } else { z })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reproducibility is the point of seeding: the same seed must give the same fill.
    #[test]
    fn equal_seeds_give_equal_sequences() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        assert!((0..100).all(|_| a.next_u64() == b.next_u64()));
    }

    /// Demos bump the seed by one for each new random start, so neighbouring seeds must give
    /// different sequences, or every other Restart shows the same random row again.
    #[test]
    fn neighbouring_seeds_differ() {
        let firsts: std::collections::HashSet<u64> =
            (0..1_000).map(|s| Rng::new(s).next_u64()).collect();
        assert_eq!(firsts.len(), 1_000);
    }

    /// A random fill at density p should come out near p, or "30% random" would mislead.
    /// Also catches a seed of 0 sticking at xorshift's fixed point.
    #[test]
    fn chance_matches_its_probability() {
        let mut r = Rng::new(0);
        let hits = (0..100_000).filter(|_| r.chance(0.3)).count();
        assert!((29_000..31_000).contains(&hits), "{hits} hits");
    }
}
