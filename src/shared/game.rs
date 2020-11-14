use bevy::prelude::*;
use std::cmp;

struct GameState {
    score: u32,
}

// Events
struct GameFinished;

struct GamePaused;

const MAX_DIFFICULTY: u8 = 4;

pub struct Difficulty {
    multiplier: u8,
    timer: Timer,
}

pub(super) fn difficulty_scaling_system(time: Res<Time>, difficulty: ResMut<Difficulty>) {
    difficulty.timer.tick(time.delta_seconds);

    if difficulty.timer.finished {
        difficulty.multiplier = cmp::min(difficulty.multiplier + 1, MAX_DIFFICULTY);
    }
}
