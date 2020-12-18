use bevy::prelude::*;
use std::collections::HashSet;

use super::attributes::{BoostSupply, Player, PlayerStats};
use super::events::PlayerBoosted;
use super::movement::move_player_from_input;
use crate::shared::{
    game::GameState,
    movement::{SideScrollDirection, Velocity},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(super) enum PlayerStates {
    Idle,
    Swim,
    Boost,
}

#[derive(Debug)]
pub(super) struct PlayerState {
    pub(super) current_state: PlayerStates,
    pub(super) blocked_transitions: HashSet<PlayerStates>,
}

/// State transition methods and helpers
impl PlayerState {
    /// Returns whether a player can transition to the desired state from their current state
    fn can_transition_to(&self, target_state: PlayerStates) -> bool {
        if self.current_state == target_state {
            return false;
        }
        if self.blocked_transitions.contains(&target_state) {
            return false;
        }

        match self.current_state {
            PlayerStates::Idle => {
                target_state == PlayerStates::Swim || target_state == PlayerStates::Boost
            }
            PlayerStates::Swim => {
                target_state == PlayerStates::Idle || target_state == PlayerStates::Boost
            }
            PlayerStates::Boost => {
                target_state == PlayerStates::Idle || target_state == PlayerStates::Swim
            }
        }
    }

    pub(super) fn start_swim(&mut self) {
        if self.can_transition_to(PlayerStates::Swim) {
            // debug!(
            //     "Transitioning from state {:?} to {:?}.",
            //     self.current_state,
            //     PlayerStates::Swim
            // );
            self.current_state = PlayerStates::Swim;
        }
    }

    pub(super) fn start_idle(&mut self) {
        if self.can_transition_to(PlayerStates::Idle) {
            // debug!(
            //     "Transitioning from state {:?} to {:?}.",
            //     self.current_state,
            //     PlayerStates::Idle
            // );
            self.current_state = PlayerStates::Idle;
        }
    }

    /**
    Start a boost in the direction the player is trying to move. If they are not moving, boost in
    the direction they are facing. Changes the state and starts the boost cooldown.
    */
    pub(super) fn start_boost(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        player_stats: &PlayerStats,
        facing: &SideScrollDirection,
        target_speed: &Vec3,
        boost_supply: &mut BoostSupply,
        player_boosted_events: &mut Events<PlayerBoosted>,
    ) {
        if self.can_transition_to(PlayerStates::Boost) {
            // debug!(
            //     "Transitioning from state {:?} to {:?}",
            //     self.current_state,
            //     PlayerStates::Boost
            // );
            if boost_supply.use_boost() {
                let boost_direction = if *target_speed == Vec3::zero() {
                    if facing.is_right() {
                        Vec3::unit_x()
                    } else {
                        -Vec3::unit_x()
                    }
                } else {
                    target_speed.normalize()
                };

                let prev_state = self.current_state;

                self.current_state = PlayerStates::Boost;

                commands.insert(
                    entity,
                    (
                        BoostCooldown {
                            timer: Timer::from_seconds(player_stats.boost_cooldown, false),
                            did_release: false,
                        },
                        BoostData {
                            velocity: boost_direction * player_stats.boost_speed,
                            timer: Timer::from_seconds(player_stats.boost_duration, false),
                            prev_state,
                        },
                    ),
                );

                // emit event that player boosted
                player_boosted_events.send(PlayerBoosted { player: entity });
            } else {
                // if the player was unable to boost, require that they release the boost
                // button before attempting again
                commands.insert_one(
                    entity,
                    BoostCooldown {
                        timer: Timer::from_seconds(0.0, false),
                        did_release: false,
                    },
                );
            }
        }
    }
}

/// Data that is assigned to an entity to represent their boost after entering a boosting state
#[derive(Debug)]
pub(super) struct BoostData {
    velocity: Vec3,
    timer: Timer,
    prev_state: PlayerStates,
}

/**
Cooldown component that is applied after boosting that must expire before
boosting is allowed again
*/
#[derive(Debug)]
pub(super) struct BoostCooldown {
    timer: Timer,
    did_release: bool,
}

/// Moves the player when they are not in the Boost state
pub(super) fn swim_movement_system(
    mut commands: &mut Commands,
    mut boost_events: ResMut<Events<PlayerBoosted>>,
    game_state: Res<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &Player,
        &mut Velocity,
        &mut SideScrollDirection,
        Entity,
        &mut PlayerState,
        &mut BoostSupply,
    )>,
) {
    if !game_state.is_running() {
        return;
    }

    for (player, mut velocity, mut facing, entity, mut state, mut boost_supply) in query.iter_mut()
    {
        if state.current_state != PlayerStates::Idle && state.current_state != PlayerStates::Swim {
            continue;
        }

        let target_speed =
            move_player_from_input(&keyboard_input, player, &mut velocity, &mut facing);

        if keyboard_input.just_pressed(KeyCode::Space) {
            state.start_boost(
                &mut commands,
                entity,
                &player.stats,
                &facing,
                &target_speed,
                &mut boost_supply,
                &mut boost_events,
            );
        } else if target_speed != Vec3::zero() {
            state.start_swim();
        } else {
            state.start_idle();
        }
    }
}

/// Movement system for a boosting player
pub(super) fn boost_movement_system(
    commands: &mut Commands,
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&mut BoostData, &mut PlayerState, &mut Velocity, Entity)>,
) {
    if !game_state.is_running() {
        return;
    }

    for (mut boost_data, mut player_state, mut velocity, entity) in query.iter_mut() {
        if player_state.current_state != PlayerStates::Boost {
            continue;
        }

        velocity.0 = boost_data.velocity;

        boost_data.timer.tick(time.delta_seconds());

        if boost_data.timer.finished() {
            // debug!("Boost finished!");
            commands.remove_one::<BoostData>(entity);
            match boost_data.prev_state {
                PlayerStates::Idle => player_state.start_idle(),
                PlayerStates::Swim => player_state.start_swim(),
                _ => panic!("Player boosted from an invalid state"),
            }
        }
    }
}

/**
Keeps track of the cooldown of the boost. The player must wait the duration
of the boost cooldown and must release the boost button before being able
to boost again.
*/
pub(super) fn boost_cooldown_system(
    commands: &mut Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    game_state: Res<GameState>,
    mut query: Query<(&mut BoostCooldown, &mut PlayerState, Entity)>,
) {
    if !game_state.is_running() {
        return;
    }

    for (mut boost_cooldown, mut player_state, entity) in query.iter_mut() {
        boost_cooldown.timer.tick(time.delta_seconds());

        boost_cooldown.did_release =
            boost_cooldown.did_release || !keyboard_input.pressed(KeyCode::Space);

        if boost_cooldown.timer.finished() && boost_cooldown.did_release {
            // debug!("Boost cooldown finished. Boost can be used again.");
            commands.remove_one::<BoostCooldown>(entity);
            player_state
                .blocked_transitions
                .remove(&PlayerStates::Boost);
        } else {
            // insert this every iteration of the timer so that if another system removes this first
            // this debuff will still be applied.
            player_state.blocked_transitions.insert(PlayerStates::Boost);
        }
    }
}
