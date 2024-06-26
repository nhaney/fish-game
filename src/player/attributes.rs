use bevy::prelude::*;
use std::collections::HashSet;

use super::events::{PlayerAte, PlayerStarved};
use crate::shared::game::GameState;

#[derive(Debug)]
pub(crate) struct PlayerStats {
    pub boost_speed: f32,
    pub boost_duration: f32,
    pub boost_cooldown: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub traction: f32,
    pub stop_threshold: f32,
}

#[derive(Debug, Component)]
pub(crate) struct Player {
    pub stats: PlayerStats,
}

#[derive(Debug, Component)]
pub(super) struct BoostSupply {
    pub max_boosts: u8,
    pub count: u8,
}

impl BoostSupply {
    pub fn use_boost(&mut self) -> bool {
        match self.count > 0 {
            true => {
                self.count -= 1;
                true
            }
            false => {
                debug!("Cannot use boost because there are none left.");
                false
            }
        }
    }

    pub fn add_boost(&mut self) -> bool {
        match self.count < self.max_boosts {
            true => {
                self.count += 1;
                true
            }
            false => {
                debug!(
                    "Could not add boost because already at max boosts ({:?})",
                    self.max_boosts
                );
                false
            }
        }
    }
}

// TODO: Could this component be shared? It might be cool for other things to sink in the future
#[derive(Default, Component)]
pub(super) struct Sink {
    pub weight: f32,
}

#[derive(Default, Component)]
pub(crate) struct HungerCountdown {
    pub time_left: f32,
    pub extra_time_per_worm: f32,
}

pub(super) fn hunger_countdown_system(
    game_state: Res<GameState>,
    mut player_ate_reader: EventReader<PlayerAte>,
    time: Res<Time>,
    mut starved_event_writer: EventWriter<PlayerStarved>,
    mut query: Query<(&mut HungerCountdown, Entity)>,
) {
    if !game_state.is_running() {
        return;
    }

    let mut players_to_add_time_for: HashSet<Entity> = HashSet::new();

    for ate_event in player_ate_reader.read() {
        players_to_add_time_for.insert(ate_event.player_entity);
    }

    for (mut hunger_countdown, player_entity) in query.iter_mut() {
        if players_to_add_time_for.contains(&player_entity) {
            debug!(
                "Adding extra time because player ate a worm. Time left: {:?}",
                hunger_countdown.time_left
            );
            hunger_countdown.time_left += hunger_countdown.extra_time_per_worm;
        }

        hunger_countdown.time_left -= time.delta_seconds();

        if hunger_countdown.time_left < 0.0 {
            // emit starved event for entity
            debug!("Player starved!");
            starved_event_writer.send(PlayerStarved { player_entity });
        }
    }
}

pub(super) fn add_boost_system(
    mut player_ate_reader: EventReader<PlayerAte>,
    mut query: Query<&mut BoostSupply>,
) {
    for ate_event in player_ate_reader.read() {
        if let Ok(mut boost_supply) = query.get_mut(ate_event.player_entity) {
            debug!("Adding extra time because player ate a worm.");
            boost_supply.add_boost();
        }
    }
}
