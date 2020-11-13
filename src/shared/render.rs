use bevy::prelude::*;

use super::arena::Arena;

pub(super) fn scale_game_transform_to_screen_size(
    arena: Res<Arena>,
    window: 
    mut query: Query<(&GameTransform, &mut Transform)>,
) {
}
