use bevy::{prelude::*, sprite};

use super::game::{GameState, GameStates};

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct SideScrollDirection(pub bool);

#[derive(Component)]
pub struct Destination {
    pub point: Vec3,
    pub trigger_distance: f32,
}

#[derive(Event)]
pub struct DestinationReached {
    #[allow(dead_code)]
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

#[derive(Debug, Component)]
pub struct Follow {
    pub entity_to_follow: Entity,
    pub offset: Vec3,
    pub follow_global_transform: bool,
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
    }
}

//pub fn flip_transform_system(mut query: Query<(&SideScrollDirection, &mut Transform)>) {
//    for (direction, mut transform) in query.iter_mut() {
//        if direction.is_left() {
//            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
//        } else {
//            transform.rotation = Quat::from_rotation_y(0.0);
//        }
//    }
//}

pub fn flip_sprite_system(mut query: Query<(&SideScrollDirection, &mut Sprite)>) {
    for (direction, mut sprite) in query.iter_mut() {
        if direction.is_left() {
            sprite.flip_x = true;
            sprite.flip_y = false;
        } else {
            sprite.flip_x = false;
            sprite.flip_y = false;
        }
    }
}

pub fn check_distance_from_destination(
    mut commands: Commands,
    mut destination_reached_events: ResMut<Events<DestinationReached>>,
    mut query: Query<(&Destination, &mut Transform, Entity), With<Velocity>>,
) {
    for (destination, mut transform, entity) in query.iter_mut() {
        let distance_from_destination = (destination.point - transform.translation).length();

        if distance_from_destination < destination.trigger_distance {
            debug!("Destination has been reached for {:?}", entity);
            destination_reached_events.send(DestinationReached { entity });
            commands.entity(entity).remove::<(Velocity, Destination)>();
            transform.translation = destination.point;
        }
    }
}

pub fn follow_system(
    mut follower_query: Query<(&Follow, &mut Transform)>,
    transform_query: Query<&Transform, Without<Follow>>,
    global_transform_query: Query<&GlobalTransform, Without<Follow>>,
) {
    for (follow_data, mut follower_transform) in follower_query.iter_mut() {
        if follow_data.follow_global_transform {
            if let Ok(target_transform) = global_transform_query.get(follow_data.entity_to_follow) {
                follower_transform.translation =
                    target_transform.translation() + follow_data.offset;
            }
        } else if let Ok(target_transform) = transform_query.get(follow_data.entity_to_follow) {
            follower_transform.translation = target_transform.translation + follow_data.offset;
        }
    }
}
