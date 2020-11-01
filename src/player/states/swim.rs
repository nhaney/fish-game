use super::super::components::Player;
use super::super::systems::move_player_from_input;
use super::boost::{BoostStartStates, BoostStarted, BoostState};
use super::idle::{IdleStartStates, IdleStarted, IdleState};
use crate::shared::components::{SideScrollDirection, Velocity};
use bevy::prelude::*;

/// The valid states to transition to a idle from
#[derive(Debug)]
pub(super) enum SwimStartStates {
    Idle(IdleState),
    Boost(BoostState),
}

/// An event indicating a boost was started
#[derive(Debug)]
pub(super) struct SwimStarted {
    pub(super) entity: Entity,
    pub(super) from_state: SwimStartStates,
}

/// The state that represents idle
#[derive(Clone, Debug)]
pub(super) struct SwimState;

/**
Replaces the previous state component with the `SwimState` component when
a `SwimStarted` event is read.
*/
pub(super) fn swim_starter(
    mut commands: Commands,
    mut listener: Local<EventReader<SwimStarted>>,
    swim_started_events: Res<Events<SwimStarted>>,
) {
    for swim_started_event in listener.iter(&swim_started_events) {
        commands.insert_one(swim_started_event.entity, SwimState);

        match swim_started_event.from_state {
            SwimStartStates::Idle(_) => {
                commands.remove_one::<IdleState>(swim_started_event.entity);
            }
            SwimStartStates::Boost(_) => {
                commands.remove_one::<BoostState>(swim_started_event.entity);
            }
        }

        println!(
            "Started swim state for entity {:?}",
            swim_started_event.entity
        )
    }
}

pub(super) fn swim_player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut idle_started_events: ResMut<Events<IdleStarted>>,
    mut boost_started_events: ResMut<Events<BoostStarted>>,
    player: &Player,
    mut velocity: Mut<Velocity>,
    mut facing: Mut<SideScrollDirection>,
    entity: Entity,
    state: &SwimState,
) {
    let target_speed = move_player_from_input(&keyboard_input, player, &mut velocity, &mut facing);

    println!("Target speed in swim movement system: {:?}", target_speed);

    if keyboard_input.pressed(KeyCode::Space) {
        let boost_direction = if target_speed == Vec3::zero() {
            if facing.is_right() {
                Vec3::unit_x()
            } else {
                -Vec3::unit_x()
            }
        } else {
            target_speed.normalize()
        };

        boost_started_events.send(BoostStarted {
            entity,
            from_state: BoostStartStates::Swim(state.clone()),
            target_state: BoostState {
                boost_velocity: boost_direction * player.stats.boost_speed,
                boost_timer: Timer::from_seconds(player.stats.boost_duration, false),
                prev_state: BoostStartStates::Swim(state.clone()),
            },
        })
    } else if target_speed == Vec3::zero() {
        idle_started_events.send(IdleStarted {
            entity,
            from_state: IdleStartStates::Swim(state.clone()),
        })
    }
}
