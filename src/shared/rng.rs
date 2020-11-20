use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub struct GameRng {
    pub rng: ChaCha8Rng,
    pub seed: <ChaCha8Rng as SeedableRng>::Seed,
}

impl Default for GameRng {
    fn default() -> Self {
        let mut seed: <ChaCha8Rng as SeedableRng>::Seed = Default::default();
        thread_rng().fill(&mut seed);

        println!("Random seed used: {:?}", seed);

        GameRng {
            rng: ChaCha8Rng::from_seed(seed),
            seed,
        }
    }
}
