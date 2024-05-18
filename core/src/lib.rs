use bevy::prelude::*;

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
    pub inputs_processed: Vec<(u32, Vec<FishGameInputEvent>)>,
    pub tick: u32,
}

impl FishGame {
    pub fn init(_config: FishGameConfig) -> Self {
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

fn create_bevy_app_for_simulation(_config: FishGameConfig) -> App {
    let mut app = App::new();

    app.init_resource::<FishGameState>()
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
        .push((next_tick, input_events.read().cloned().collect()));

    state.tick = next_tick;
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
