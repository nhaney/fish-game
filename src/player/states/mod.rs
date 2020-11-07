use bevy::prelude::*;
use std::collections::HashSet;

mod boost;
pub(crate) mod idle;
mod swim;

enum PlayerStates {
    Idle,
    Swim,
    Boost(BoostData),
}

struct PlayerState {
    current_state: PlayerStates,
    blocked_transitions: HashSet<PlayerStates>,
}

/// State transition methods and helpers
impl PlayerState {
    fn can_transition_to(&self, target_state: PlayerState) {
        // TODO: Add something like a map of valid transitions to act more like a state machine
        self.current_state != target_state && !self.blocked_transitions.contains(target_state)
    }

    fn start_swim(&mut self) {
        if self.can_transition_to(PlayerStates::Swim) {
            println!(
                "Transitioning from state {:?} to {:?}.",
                self.current_state,
                PlayerStates::Swim
            );
            self.current_state = PlayerStates::Swim;
        }
    }

    fn start_idle(&mut self) {
        if self.can_transition_to(PlayerStates::Idle) {
            println!(
                "Transitioning from state {:?} to {:?}.",
                self.current_state,
                PlayerStates::Idle
            );
            self.current_state = PlayerStates::Idle;
        }
    }

    /**
    Start a boost in the direction the player is trying to move. If they are not moving, boost in
    the direction they are facing. Changes the state and starts the boost cooldown.
    */
    fn start_boost(
        &mut self,
        &mut commands: Commands,
        entity: Entity,
        &player_stats: PlayerStats,
        &facing: Facing,
        &target_speed: Vec3,
    ) {
        if self.can_transition_to(PlayerStates::Boost) {
            println!(
                "Transitioning from state {:?} to {:?}.",
                self.current_state,
                PlayerStates::Boost
            );

            let boost_direction = if target_speed == Vec3::zero() {
                if facing.is_right() {
                    Vec3::unit_x()
                } else {
                    -Vec3::unit_x()
                }
            } else {
                target_speed.normalize()
            };

            self.current_state = PlayerStates::Boost(BoostData {
                velocity: boost_direction * player_stats.boost_speed,
                timer: Timer.from_seconds(player.stats.boost_duration, false),
                prev_state: self.current_state,
            });

            commands.insert_one(
                entity,
                BoostCooldown {
                    timer: Timer.from_seconds(player_stats.boost_cooldown),
                    did_release: false,
                },
            )
        }
    }
}

/// The state that represents boosting
#[derive(Clone, Debug)]
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
    pub(super) timer: Timer,
    pub(super) did_release: bool,
}

/// Moves the player when they are not in the Boost state
pub(super) fn swim_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(
        &Player,
        &mut Velocity,
        &mut facing,
        Entity,
        &mut PlayerState,
    )>,
) {
    for (player, mut velocity, mut facing, entity, mut state) in query.iter_mut() {
        if state.current_state != PlayerStates::Idle && state.current_state != PlayerStates::Swim {
            continue;
        }

        let target_speed =
            move_player_from_input(&keyboard_input, player, &mut velocity, &mut facing);

        println!("Target speed in swim movement system: {:?}", target_speed);

        if keyboard_input.pressed(KeyCode::Space) {
            state.start_boost(entity, &player.player_stats, &facing, &target_speed);
        } else if target_speed != Vec3::zero() {
            state.start_swim();
        }
    }
}

/// Movement system for a boosting player
pub(super) fn boost_player_movement_system(
    time: Res<Time>,
    mut query: Query<(&Player, &mut PlayerState, &mut Velocity)>,
) {
    for (player, mut player_state, mut velocity) in query.iter_mut() {
        if let PlayerStates::Boost(boost_data) = player_state.current_state {
            velocity.0 = boost_data.boost_velocity;

            boost_data.boost_timer.tick(time.delta_seconds);

            if boost_data.boost_timer.finished {
                match boost_data.prev_state {
                    PlayerState::Idle => self.start_idle(),
                    PlayerState::Swim => self.start_swim(),
                }
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
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut BoostCooldown, &mut PlayerState, Entity)>,
) {
    for (mut boost_cooldown, mut player_state, entity) in query.iter_mut() {
        boost_cooldown.timer.tick(time.delta_seconds);

        boost_cooldown.did_release =
            boost_cooldown.did_release || !keyboard_input.pressed(KeyCode::Space);

        if boost_cooldown.timer.finished && boost_cooldown.did_release {
            println!("Boost cooldown finished for entity {:?}", entity);
            commands.remove_one::<BoostCooldown>(entity);
            player_state.blocked_transitions.remove(PlayerStates::Boost);
        } else {
            // insert this every iteration of the timer so that if another system removes this first
            // this debuff will still be applied.
            player_state.blocked_transitions.insert(PlayerStates::Boost);
        }
    }
}
