use bevy::prelude::*;

// Game lifecycle events. The adapter in `core_adapter` is responsible for
// emitting these — they are derived from `CoreState` / `CoreControl`, never
// from legacy Bevy state.

#[derive(Default, Event)]
pub struct GameOver {
    pub winning_boat: Option<Entity>,
}

#[derive(Event)]
pub struct GamePaused;

#[derive(Event)]
pub struct GameUnpaused;

#[derive(Event)]
pub struct GameRestarted;
