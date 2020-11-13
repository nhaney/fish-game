use bevy::prelude::*;

use super::attributes::Player;
use crate::shared::{arena::Arena, collision::Collider, movement::GameTransform};

/// Keeps player in bounds of arena
pub(super) fn player_bounds_system(
    arena: Res<Arena>,
    mut query: Query<(&Player, &Collider, &mut GameTransform)>,
) {
    for (_, collider, mut transform) in query.iter_mut() {
        let new_pos = &mut transform.cur_transform.translation;

        let arena_half_width = arena.width / 2.0;
        let arena_half_height = arena.height / 2.0;

        let player_half_width = collider.width / 2.0;
        let player_half_height = collider.height / 2.0;

        if new_pos.x() - player_half_width < -arena_half_width {
            *new_pos.x_mut() = -arena_half_width + player_half_width;
        }

        if new_pos.x() + player_half_width > arena_half_width {
            *new_pos.x_mut() = arena_half_width - player_half_width;
        }

        if new_pos.y() - player_half_height < -arena_half_height {
            *new_pos.y_mut() = -arena_half_height + player_half_height;
        }

        if new_pos.y() + player_half_height > (arena_half_height + arena.offset) {
            *new_pos.y_mut() = (arena_half_height + arena.offset) - player_half_height;
        }
    }
}
