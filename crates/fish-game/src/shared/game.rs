use bevy::prelude::*;

// Game lifecycle events. The adapter in `core_adapter` is responsible for
// emitting these — they are derived from `CoreState` / `CoreControl`, never
// from legacy Bevy state.

#[derive(Default, Message)]
pub struct GameOver {
    pub winning_boat: Option<Entity>,
}

#[derive(Message)]
pub struct GamePaused;

#[derive(Message)]
pub struct GameUnpaused;

#[derive(Message)]
pub struct GameRestarted;
