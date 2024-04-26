use bevy::prelude::*;

use super::{attributes::Player, events::PlayerStarved};
use crate::shared::{
    arena::Arena,
    movement::{Destination, SideScrollDirection, Velocity},
};

pub(super) fn player_starved_handler(
    commands: &mut Commands,
    arena: Res<Arena>,
    mut player_starved_reader: EventReader<PlayerStarved>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &SideScrollDirection), With<Player>>,
) {
    for player_starved_event in player_starved_reader.read() {
        let (mut player_transform, mut player_velocity, player_facing) = player_query
            .get_mut(player_starved_event.player_entity)
            .unwrap();

        let surface_point = Vec3::new(
            player_transform.translation.x,
            (arena.height / 2.0) + arena.offset,
            player_transform.translation.z,
        );

        player_velocity.0 = Vec3::unit_y() * 100.0;

        // flip depending on the direction the player is facing - probably a more mathy way to do this
        if player_facing.is_right() {
            player_transform.rotation = Quat::from_rotation_x(std::f32::consts::PI);
        } else {
            player_transform.rotation = Quat::from_rotation_z(std::f32::consts::PI);
        }
        commands.remove_one::<SideScrollDirection>(player_starved_event.player_entity);

        commands.insert_one(
            player_starved_event.player_entity,
            Destination {
                point: surface_point,
                trigger_distance: 1.0,
            },
        );
    }
}
