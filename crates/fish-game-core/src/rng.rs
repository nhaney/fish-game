use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Deterministic rng used throughout the sim.
///
/// Wrapped so we can serialize the seed and derive the next seed on restart
/// from the current stream (no `thread_rng()`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRng {
    pub seed: [u8; 32],
    pub rng: ChaCha8Rng,
}

impl GameRng {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            seed,
            rng: ChaCha8Rng::from_seed(seed),
        }
    }

    /// Derive a new deterministic seed from the current stream and reset.
    /// Keeps restart deterministic without any OS entropy.
    pub fn reseed_from_self(&mut self) {
        let mut next_seed: [u8; 32] = [0; 32];
        self.rng.fill(&mut next_seed);
        self.seed = next_seed;
        self.rng = ChaCha8Rng::from_seed(next_seed);
    }
}
