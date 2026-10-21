//! Seeded pseudo-random generator (SplitMix64).
//!
//! Hand-written rather than taken from `rand`: the same seed must give the
//! same game whatever the dependency versions.

const SEED_DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// How a seed is shown to the player, to copy and type in: base 36, in capitals.
pub fn seed_code(mut seed: u64) -> String {
    let mut digits = Vec::new();
    loop {
        digits.push(SEED_DIGITS[(seed % 36) as usize]);
        seed /= 36;
        if seed == 0 {
            break;
        }
    }
    digits.reverse();
    String::from_utf8(digits).unwrap()
}

/// The seed a code stands for, whatever its case and surrounding spaces.
pub fn parse_seed_code(code: &str) -> Option<u64> {
    let code = code.trim();
    if code.is_empty() {
        return None;
    }
    code.chars().try_fold(0u64, |seed, c| {
        seed.checked_mul(36)?.checked_add(c.to_digit(36)? as u64)
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    fn seed_codes_read_back() {
        for seed in [0, 35, 36, 1_234_567, 36u64.pow(8) - 1, u64::MAX] {
            assert_eq!(parse_seed_code(&seed_code(seed)), Some(seed));
        }
        assert_eq!(seed_code(36u64.pow(8) - 1), "ZZZZZZZZ");
        assert_eq!(parse_seed_code(" omarchy "), parse_seed_code("OMARCHY"));
        for bad in ["", "  ", "no-seed", "3W5E11264SGSG"] {
            assert_eq!(parse_seed_code(bad), None, "{bad}");
        }
    }

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
