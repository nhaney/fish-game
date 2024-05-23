use bevy::prelude::*;

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

mod player;

#[derive(Debug, Default, Clone, Copy)]
pub struct FishGameConfig {
    pub random_seed: u32,
}

pub struct FishGame {
    app: App,
}

#[derive(Default)]
pub struct FishGameInputState {
    pub inputs: Vec<FishGameInputEvent>,
}

#[derive(Debug, Copy, Clone, Event)]
pub enum FishGameInputEvent {
    UpPressed,
    DownPressed,
    LeftPressed,
    RightPressed,
    BoostPressed,
}

#[derive(Debug, Default, Resource)]
pub struct FishGameState {
    pub game_config: FishGameConfig,
    pub inputs_processed: Vec<(u32, Vec<FishGameInputEvent>)>,
    pub tick: u32,
    pub player_info: PlayerInfo,
}

#[derive(Debug, Default)]
pub struct PlayerInfo {
    pos: Vec2,
    state: PlayerState,
}

#[derive(Debug, Default)]
pub enum PlayerState {
    #[default]
    Idle,
    Swimming,
    Boosting,
}

#[derive(Resource)]
struct GameRng {
    pub rng: ChaCha8Rng,
    pub seed: <ChaCha8Rng as SeedableRng>::Seed,
}

impl Default for GameRng {
    fn default() -> Self {
        let mut seed: <ChaCha8Rng as SeedableRng>::Seed = Default::default();
        rand::thread_rng().fill(&mut seed);

        debug!("Random seed used: {:?}", seed);

        GameRng {
            rng: ChaCha8Rng::from_seed(seed),
            seed,
        }
    }
}

impl FishGame {
    pub fn init(_config: &FishGameConfig) -> Self {
        let bevy_app = create_bevy_app_for_simulation(_config);
        FishGame { app: bevy_app }
    }

    pub fn update(&mut self, _input_state: FishGameInputState) -> &FishGameState {
        let input_events = _input_state.inputs.into_iter();

        self.app.world.send_event_batch(input_events);

        self.app.update();

        self.app
            .world
            .get_resource::<FishGameState>()
            .expect("Could not find state resource on internal bevy app world.")
    }
}

fn create_bevy_app_for_simulation(_config: &FishGameConfig) -> App {
    let mut app = App::new();

    app.init_resource::<FishGameState>()
        .init_resource::<GameRng>()
        .add_event::<FishGameInputEvent>()
        .add_systems(Update, update_state_resource);

    app
}

fn update_state_resource(
    mut input_events: EventReader<FishGameInputEvent>,
    mut state: ResMut<FishGameState>,
) {
    // Update the tick of the simulation.
    let next_tick = state.tick + 1;

    state
        .inputs_processed
        .push((next_tick, input_events.read().copied().collect()));

    state.tick = next_tick;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_count_and_inputs_per_tick_are_recorded_in_state() {
        let mut fish_game = FishGame::init(&FishGameConfig::default());

        let tick_one_inputs = vec![
            FishGameInputEvent::UpPressed,
            FishGameInputEvent::BoostPressed,
        ];

        let tick_two_inputs = vec![
            FishGameInputEvent::LeftPressed,
            FishGameInputEvent::DownPressed,
        ];

        let tick_one_state = fish_game.update(FishGameInputState {
            inputs: tick_one_inputs,
        });

        assert!(tick_one_state.tick == 1);
        assert!(tick_one_state.inputs_processed.len() == 1);
        assert!(tick_one_state.inputs_processed[0].0 == 1);
        assert!(tick_one_state.inputs_processed[0].1.len() == 2);

        let tick_two_state = fish_game.update(FishGameInputState {
            inputs: tick_two_inputs,
        });

        assert!(tick_two_state.tick == 2);
        assert!(tick_two_state.inputs_processed.len() == 2);
        assert!(tick_two_state.inputs_processed[1].0 == 2);
        assert!(tick_two_state.inputs_processed[1].1.len() == 2);
    }
}
