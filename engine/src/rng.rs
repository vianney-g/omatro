//! Seeded pseudo-random generator (SplitMix64).
//!
//! Hand-written rather than taken from `rand`: the same seed must give the
//! same game whatever the dependency versions.

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
    /// Chaos Monkey: `chance(n)` succeeds `luck` times in `n`.
    luck: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: seed,
            luck: 1,
        }
    }

    /// Chances of `chance` become `luck` in `n` (1 by default).
    pub fn set_luck(&mut self, luck: u64) {
        self.luck = luck.max(1);
    }

    /// An independent stream of the game seed, one per use (shuffles, shop…),
    /// so that drawing from one never shifts the others.
    pub fn stream(seed: u64, name: &str) -> Self {
        // FNV-1a hash of the name, mixed into the seed.
        let hash = name.bytes().fold(0xCBF2_9CE4_8422_2325_u64, |h, b| {
            (h ^ b as u64).wrapping_mul(0x0100_0000_01B3)
        });
        Rng::new(Rng::new(seed ^ hash).next_u64())
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Integer in `0..n`.
    pub fn below(&mut self, n: u64) -> u64 {
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }

    /// True with a 1 in `n` chance (`luck` in `n` with Chaos Monkey).
    pub fn chance(&mut self, n: u64) -> bool {
        self.below(n) < self.luck
    }

    /// Float in `0..1`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn streams_are_independent() {
        let first = |seed, name| Rng::stream(seed, name).next_u64();
        assert_eq!(first(42, "shop"), first(42, "shop"));
        assert_ne!(first(42, "shop"), first(42, "deck"));
        assert_ne!(first(42, "shop"), first(43, "shop"));
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(7);
        for n in 1..50 {
            assert!(rng.below(n) < n);
        }
    }

    #[test]
    fn luck_raises_chances() {
        let mut rng = Rng::new(7);
        rng.set_luck(4);
        assert!((0..100).all(|_| rng.chance(4)));
    }
}
