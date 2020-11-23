use bevy::prelude::*;

use crate::objects::boat::Hook;
use crate::player::events::{PlayerAte, PlayerBonked, PlayerHooked, PlayerStarved};

#[derive(Debug, Copy, Clone)]
pub enum GameStates {
    Running,
    // Paused,
    GameOver,
}

pub struct GameState {
    pub cur_state: GameStates,
    pub prev_state: GameStates,
}

impl GameState {
    fn transition(&mut self, dest_state: GameStates) {
        self.prev_state = self.cur_state;
        self.cur_state = dest_state;
        println!(
            "Game state transitioned from {:?} to {:?}",
            self.prev_state, self.cur_state
        );
    }
}

// Events
pub struct GameOver {
    pub winning_boat: Option<Entity>,
}

pub struct GamePaused;

const MAX_DIFFICULTY: u8 = 4;
const SCORE_PER_WORM: u8 = 5;

pub struct Score {
    pub count: u32,
    pub timer: Timer,
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
    game_state: Res<GameState>,
    mut score: ResMut<Score>,
    mut player_ate_reader: Local<EventReader<PlayerAte>>,
    player_ate_events: Res<Events<PlayerAte>>,
) {
    if let GameStates::GameOver = game_state.cur_state {
        return;
    }

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

pub(super) fn finalize_score(
    score: Res<Score>,
    mut game_over_reader: Local<EventReader<GameOver>>,
    game_over_events: Res<Events<GameOver>>,
) {
    if let Some(_game_over_event) = game_over_reader.earliest(&game_over_events) {
        println!("Final score: {:?}", score.count);
    }
}

pub(super) fn end_game_system(
    mut player_hooked_reader: Local<EventReader<PlayerHooked>>,
    player_hooked_events: Res<Events<PlayerHooked>>,
    mut player_starved_reader: Local<EventReader<PlayerStarved>>,
    player_starved_events: Res<Events<PlayerStarved>>,
    mut player_bonked_reader: Local<EventReader<PlayerBonked>>,
    player_bonked_events: Res<Events<PlayerBonked>>,
    mut game_over_events: ResMut<Events<GameOver>>,
    mut game_state: ResMut<GameState>,
    hook_query: Query<(&Hook, &Parent)>,
) {
    for hook_event in player_hooked_reader.iter(&player_hooked_events) {
        println!("Ending game because the player got hooked.");
        let (_, winning_boat) = hook_query.get(hook_event.hook_entity).unwrap();
        game_over_events.send(GameOver {
            winning_boat: Some(winning_boat.0),
        });
        game_state.transition(GameStates::GameOver);
    }

    for _ in player_starved_reader.iter(&player_starved_events) {
        println!("Ending game because the player starved.");
        game_over_events.send(GameOver { winning_boat: None });
        game_state.transition(GameStates::GameOver);
    }

    for bonked_event in player_bonked_reader.iter(&player_bonked_events) {
        println!("Ending game because the player bonked.");
        game_over_events.send(GameOver {
            winning_boat: Some(bonked_event.boat_entity),
        });
        game_state.transition(GameStates::GameOver);
    }
}