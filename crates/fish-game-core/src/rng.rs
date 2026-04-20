use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Deterministic rng used throughout the sim.
///
/// `ChaCha8Rng` is bit-identical across targets given the same seed. We keep
/// the original seed alongside the live rng so it can be hashed without poking
/// at the ChaCha internals (which aren't part of its stable API).
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
}
