use bevy::prelude::*;

use super::game::{GameState, GameStates};

pub struct Velocity(pub Vec3);

pub struct SideScrollDirection(pub bool);

pub struct Destination {
    pub point: Vec3,
    pub trigger_distance: f32,
}

pub struct DestinationReached {
    entity: Entity,
}

impl SideScrollDirection {
    pub fn is_right(&self) -> bool {
        self.0
    }

    pub fn is_left(&self) -> bool {
        !self.0
    }
}

pub fn movement_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    if let GameStates::Paused = game_state.cur_state {
        return;
    }

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += time.delta_seconds() * velocity.0.x;
        transform.translation.y += time.delta_seconds() * velocity.0.y;
        transform.translation.z = 1.0;
    }
}

pub fn flip_transform_system(mut query: Query<(&SideScrollDirection, &mut Transform)>) {
    for (direction, mut transform) in query.iter_mut() {
        if direction.is_left() {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        } else {
            transform.rotation = Quat::from_rotation_y(0.0);
        }
    }
}

pub fn check_distance_from_destination(
    commands: &mut Commands,
    mut destination_reached_events: ResMut<Events<DestinationReached>>,
    mut query: Query<(&Destination, &mut Transform, Entity), With<Velocity>>,
) {
    for (destination, mut transform, entity) in query.iter_mut() {
        let distance_from_destination = (destination.point - transform.translation).length();

        if distance_from_destination < destination.trigger_distance {
            println!("Destination has been reached for {:?}", entity);
            destination_reached_events.send(DestinationReached { entity });
            commands.remove::<(Velocity, Destination)>(entity);
            transform.translation = destination.point;
        }
    }
}
