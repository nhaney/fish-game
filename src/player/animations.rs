use bevy::prelude::*;

use super::{
    attributes::Player,
    events::{PlayerBonked, PlayerStarved},
};
use crate::shared::{
    arena::Arena,
    movement::{Destination, Velocity},
};

pub(super) fn player_starved_handler(
    commands: &mut Commands,
    arena: Res<Arena>,
    mut player_starved_reader: Local<EventReader<PlayerStarved>>,
    player_starved_events: Res<Events<PlayerStarved>>,
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    for player_starved_event in player_starved_reader.iter(&player_starved_events) {
        let (mut player_transform, mut player_velocity) = player_query
            .get_mut(player_starved_event.player_entity)
            .unwrap();

        let surface_point = Vec3::new(
            player_transform.translation.x,
            arena.height + arena.offset,
            1.0,
        );

        player_velocity.0 = (surface_point - player_transform.translation).normalize() * 100.0;
        player_transform.rotation = Quat::from_rotation_x(std::f32::consts::PI);

        commands.insert_one(
            player_starved_event.player_entity,
            Destination {
                point: surface_point,
                trigger_distance: 10.0,
            },
        );
    }
}
