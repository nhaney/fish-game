use bevy::prelude::*;

use crate::objects::boat::Hook;
use crate::player::events::{PlayerAte, PlayerBonked, PlayerHooked, PlayerStarved};

#[derive(Debug, Copy, Clone)]
pub enum GameStates {
    Running,
    Paused,
    GameOver,
}

#[derive(Debug, Copy, Clone, Resource)]
pub struct GameState {
    pub cur_state: GameStates,
    pub prev_state: GameStates,
}

impl GameState {
    fn transition(&mut self, dest_state: GameStates) {
        self.prev_state = self.cur_state;
        self.cur_state = dest_state;
        debug!(
            "Game state transitioned from {:?} to {:?}",
            self.prev_state, self.cur_state
        );
    }

    pub fn is_running(&self) -> bool {
        if let GameStates::Running = self.cur_state {
            return true;
        }
        false
    }
}

pub(super) fn restart_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut restart_events: ResMut<Events<GameRestarted>>,
) {
    if keyboard_input.just_pressed(KeyCode::R) {
        restart_events.send(GameRestarted);
    }
}

// Events
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

const MAX_DIFFICULTY: u8 = 4;
const SCORE_PER_WORM: u8 = 5;

#[derive(Default, Resource)]
pub struct Score {
    pub count: u32,
    pub timer: Timer,
}

#[derive(Default, Resource)]
pub struct Difficulty {
    pub multiplier: u8,
    pub(super) timer: Timer,
}

pub(super) fn difficulty_scaling_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut difficulty: ResMut<Difficulty>,
) {
    if !game_state.is_running() {
        return;
    }

    difficulty.timer.tick(time.delta_seconds());

    if difficulty.timer.finished() && difficulty.multiplier < MAX_DIFFICULTY {
        difficulty.multiplier += 1;
    }
}

pub(super) fn increment_score_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut score: ResMut<Score>,
    mut player_ate_reader: EventReader<PlayerAte>,
) {
    if !game_state.is_running() {
        return;
    }

    score.timer.tick(time.delta_seconds());

    if score.timer.finished() {
        score.count += 1;
        debug!("Score: {:?}", score.count);
    }

    for _ in player_ate_reader.read() {
        score.count += SCORE_PER_WORM as u32;
        debug!("Score: {:?}", score.count);
    }
}

pub(super) fn finalize_score(score: Res<Score>, mut game_over_reader: EventReader<GameOver>) {
    if let Some(_game_over_event) = game_over_reader.read().next() {
        debug!("Final score: {:?}", score.count);
    }
}

// TODO: Combine event types to reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub(super) fn end_game_system(
    mut player_hooked_reader: EventReader<PlayerHooked>,
    mut player_starved_reader: EventReader<PlayerStarved>,
    mut player_bonked_reader: EventReader<PlayerBonked>,
    mut game_over_events: EventWriter<GameOver>,
    mut game_state: ResMut<GameState>,
    hook_query: Query<(&Hook, &Parent)>,
) {
    for hook_event in player_hooked_reader.read() {
        debug!("Ending game because the player got hooked.");
        let (_, winning_boat) = hook_query.get(hook_event.hook_entity).unwrap();
        game_over_events.send(GameOver {
            winning_boat: Some(winning_boat.get()),
        });
        game_state.transition(GameStates::GameOver);
    }

    for _ in player_starved_reader.read() {
        debug!("Ending game because the player starved.");
        game_over_events.send(GameOver { winning_boat: None });
        game_state.transition(GameStates::GameOver);
    }

    for bonked_event in player_bonked_reader.read() {
        debug!("Ending game because the player bonked.");
        game_over_events.send(GameOver {
            winning_boat: Some(bonked_event.boat_entity),
        });
        game_state.transition(GameStates::GameOver);
    }
}

pub(super) fn reset_difficulty_on_restart(
    mut difficulty: ResMut<Difficulty>,
    mut restart_reader: EventReader<GameRestarted>,
) {
    if restart_reader.read().next().is_some() {
        debug!("Resetting difficulty after restart");
        difficulty.multiplier = 1;
        difficulty.timer = Timer::from_seconds(10.0, TimerMode::Repeating);
    }
}

pub(super) fn reset_score_on_restart(
    mut score: ResMut<Score>,
    mut restart_reader: EventReader<GameRestarted>,
) {
    if restart_reader.read().next().is_some() {
        debug!("Resetting score after restart");
        score.count = 0;
        score.timer = Timer::from_seconds(1.0, true);
    }
}

pub(super) fn reset_game_state_on_restart(
    mut game_state: ResMut<GameState>,
    mut restart_reader: EventReader<GameRestarted>,
) {
    if restart_reader.read().next().is_some() {
        debug!("Resetting game state after restart");
        game_state.cur_state = GameStates::Running;
        game_state.prev_state = GameStates::Running;
    }
}

pub(super) fn pause_game(
    mut game_state: ResMut<GameState>,
    mut pause_reader: EventReader<GamePaused>,
) {
    if pause_reader.read().next().is_some() {
        game_state.transition(GameStates::Paused);
    }
}

pub(super) fn unpause_game(
    mut game_state: ResMut<GameState>,
    mut unpause_reader: EventReader<GameUnpaused>,
) {
    if unpause_reader.read().next().is_some() {
        let prev_state = game_state.prev_state;
        game_state.transition(prev_state);
    }
}
