use bevy::prelude::*;
use std::cmp;

use crate::player::events::{PlayerAte, PlayerHooked};

enum GameStates {
    Running,
    Paused,
    GameOver,
}

struct Game {
    state: GameStates,
}

// Events
struct GameFinished;

struct GamePaused;

const MAX_DIFFICULTY: u8 = 4;
const SCORE_PER_WORM: u8 = 5;

pub struct Score {
    pub(super) count: u32,
    pub(super) timer: Timer,
}

pub struct Difficulty {
    pub multiplier: u8,
    pub(super) timer: Timer,
}

pub(super) fn difficulty_scaling_system(time: Res<Time>, mut difficulty: ResMut<Difficulty>) {
    difficulty.timer.tick(time.delta_seconds);

    if difficulty.timer.finished && difficulty.multiplier < MAX_DIFFICULTY {
        difficulty.multiplier += 1;
    }
}

pub(super) fn increment_score_system(
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut player_ate_reader: Local<EventReader<PlayerAte>>,
    player_ate_events: Res<Events<PlayerAte>>,
) {
    score.timer.tick(time.delta_seconds);

    if score.timer.finished {
        score.count += 1;
        println!("Score: {:?}", score.count);
    }

    for _ in player_ate_reader.iter(&player_ate_events) {
        score.count += SCORE_PER_WORM as u32;
        println!("Score: {:?}", score.count);
    }
}
