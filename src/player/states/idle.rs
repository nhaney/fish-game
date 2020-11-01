use super::super::components::Player;
use super::super::systems::move_player_from_input;
use super::boost::{BoostStartStates, BoostStarted, BoostState};
use super::swim::{SwimStartStates, SwimStarted, SwimState};
use crate::shared::components::{SideScrollDirection, Velocity};

use bevy::prelude::*;

/// The valid states to transition to a idle from
#[derive(Debug)]
pub(super) enum IdleStartStates {
    Swim(SwimState),
    Boost(BoostState),
}

/// An event indicating a boost was started
#[derive(Debug)]
pub(super) struct IdleStarted {
    pub(super) entity: Entity,
    pub(super) from_state: IdleStartStates,
}

/// The state that represents idle. Public to crate because it is the initial state
#[derive(Clone, Debug)]
pub(crate) struct IdleState;

/**
Replaces the previous state component with the `IdleState` component when
a `IdleStarted` event is read.
*/
pub(super) fn idle_starter(
    mut commands: Commands,
    mut listener: Local<EventReader<IdleStarted>>,
    idle_started_events: Res<Events<IdleStarted>>,
) {
    for idle_started_event in listener.iter(&idle_started_events) {
        commands.insert_one(idle_started_event.entity, IdleState);

        match idle_started_event.from_state {
            IdleStartStates::Swim(_) => {
                commands.remove_one::<SwimState>(idle_started_event.entity);
            }
            IdleStartStates::Boost(_) => {
                commands.remove_one::<BoostState>(idle_started_event.entity);
            }
        }

        println!(
            "Started idle state for entity {:?}",
            idle_started_event.entity
        );
    }
}

pub(super) fn idle_player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut swim_started_events: ResMut<Events<SwimStarted>>,
    mut boost_started_events: ResMut<Events<BoostStarted>>,
    player: &Player,
    mut velocity: Mut<Velocity>,
    mut facing: Mut<SideScrollDirection>,
    entity: Entity,
    state: &IdleState,
) {
    let target_speed = move_player_from_input(&keyboard_input, player, &mut velocity, &mut facing);

    println!("Target speed in idle movement system: {:?}", target_speed);

    if keyboard_input.pressed(KeyCode::Space) {
        let boost_direction = {
            if facing.is_right() {
                Vec3::unit_x()
            } else {
                -Vec3::unit_x()
            }
        };

        boost_started_events.send(BoostStarted {
            entity,
            from_state: BoostStartStates::Idle(state.clone()),
            target_state: BoostState {
                boost_velocity: boost_direction * player.stats.boost_speed,
                boost_timer: Timer::from_seconds(player.stats.boost_duration, false),
                prev_state: BoostStartStates::Idle(state.clone()),
            },
        })
    } else if target_speed != Vec3::zero() {
        swim_started_events.send(SwimStarted {
            entity,
            from_state: SwimStartStates::Idle(state.clone()),
        })
    }
}
