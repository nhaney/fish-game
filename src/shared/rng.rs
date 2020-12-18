use bevy::prelude::*;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use super::game::GameRestarted;

pub struct GameRng {
    pub rng: ChaCha8Rng,
    pub seed: <ChaCha8Rng as SeedableRng>::Seed,
}

impl Default for GameRng {
    fn default() -> Self {
        let mut seed: <ChaCha8Rng as SeedableRng>::Seed = Default::default();
        thread_rng().fill(&mut seed);

        debug!("Random seed used: {:?}", seed);

        GameRng {
            rng: ChaCha8Rng::from_seed(seed),
            seed,
        }
    }
}

pub(super) fn reset_rng_on_restart(
    mut rng: ResMut<GameRng>,
    restart_events: Res<Events<GameRestarted>>,
    mut restart_reader: Local<EventReader<GameRestarted>>,
) {
    if let Some(_) = restart_reader.earliest(&restart_events) {
        let mut seed: <ChaCha8Rng as SeedableRng>::Seed = Default::default();
        thread_rng().fill(&mut seed);

        debug!("Generated random seed after restart: {:?}", seed);

        rng.rng = ChaCha8Rng::from_seed(seed);
        rng.seed = seed;
    }
}
