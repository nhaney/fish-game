use std::collections::HashMap;

use super::super::components::Player;
use super::idle::{IdleStartStates, IdleStarted, IdleState};
use super::swim::{SwimStartStates, SwimStarted, SwimState};
use crate::shared::components::Velocity;
use bevy::prelude::*;

/// The valid states to transition to a boost from
#[derive(Clone, Debug)]
pub(super) enum BoostStartStates {
    Idle(IdleState),
    Swim(SwimState),
}

/// An event indicating a boost was started
#[derive(Debug)]
pub(super) struct BoostStarted {
    pub(super) entity: Entity,
    pub(super) from_state: BoostStartStates,
    pub(super) target_state: BoostState,
}

/// The state that represents boosting
#[derive(Clone, Debug)]
pub(super) struct BoostState {
    pub(super) boost_velocity: Vec3,
    pub(super) boost_timer: Timer,
    pub(super) prev_state: BoostStartStates,
}

/// Cooldown component that is applied after boosting before boosting again
#[derive(Debug)]
pub(super) struct BoostCooldown {
    pub(super) timer: Timer,
    pub(super) did_release: bool,
}

/**
Replaces the previous state component with the boost state component when
a `BoostStarted` event is read. Only change the state if the entity does not
have a `BoostCooldown` debuff.
*/
pub(super) fn boost_starter(
    mut commands: Commands,
    mut listener: Local<EventReader<BoostStarted>>,
    boost_started_events: Res<Events<BoostStarted>>,
    mut query: Query<Without<BoostCooldown, Entity>>,
) {
    let mut read_event: bool = false;
    let mut entity_events: HashMap<Entity, &BoostStarted> = HashMap::new();

    for boost_started_event in listener.iter(&boost_started_events) {
        read_event = true;
        entity_events.insert(boost_started_event.entity, boost_started_event);
    }

    if read_event {
        for entity in &mut query.iter() {
            if entity_events.contains_key(&entity) {
                // change state and specify velocity of the boost
                let boost_event = entity_events.get(&entity).unwrap();

                commands.insert_one(entity, boost_event.target_state.clone());

                match boost_event.from_state {
                    BoostStartStates::Idle(_) => {
                        commands.remove_one::<IdleState>(entity);
                    }
                    BoostStartStates::Swim(_) => {
                        commands.remove_one::<SwimState>(entity);
                    }
                }

                println!(
                    "Started boost state {:?} for entity {:?}",
                    entity, boost_event.target_state
                );
            }
        }
    }
}

/**
Moves the player when they are in a boost state. Ends the boost state and creates a state transition event
based on the previous state when the boost timer runs out.
*/
pub(super) fn boost_player_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut idle_started_events: ResMut<Events<IdleStarted>>,
    mut swim_started_events: ResMut<Events<SwimStarted>>,
    player: &Player,
    mut velocity: Mut<Velocity>,
    mut boost_state: Mut<BoostState>,
    entity: Entity,
) {
    velocity.0 = boost_state.boost_velocity;

    boost_state.boost_timer.tick(time.delta_seconds);

    if boost_state.boost_timer.finished {
        println!("Finished boost for entity {:?}, transitioning state back to what it was before boost and starting cooldown timer.", entity);
        commands.insert_one(
            entity,
            BoostCooldown {
                timer: Timer::from_seconds(player.stats.boost_cooldown, false),
                did_release: false,
            },
        );

        match boost_state.prev_state {
            BoostStartStates::Idle(_) => idle_started_events.send(IdleStarted {
                entity,
                from_state: IdleStartStates::Boost(boost_state.clone()),
            }),
            BoostStartStates::Swim(_) => swim_started_events.send(SwimStarted {
                entity,
                from_state: SwimStartStates::Boost(boost_state.clone()),
            }),
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
    mut boost_cooldown: Mut<BoostCooldown>,
    entity: Entity,
) {
    boost_cooldown.timer.tick(time.delta_seconds);

    boost_cooldown.did_release =
        boost_cooldown.did_release || !keyboard_input.pressed(KeyCode::Space);

    if boost_cooldown.timer.finished && boost_cooldown.did_release {
        println!("Boost cooldown finished for entity {:?}", entity);
        commands.remove_one::<BoostCooldown>(entity);
    }
}
