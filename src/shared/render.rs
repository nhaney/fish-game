use bevy::prelude::*;

use super::{arena::Arena, movement::GameTransform};

pub(super) fn scale_game_transform_to_screen_size(
    arena: Res<Arena>,
    windows: Res<Windows>,
    mut query: Query<(&GameTransform, &mut Transform)>,
) {
    let primary_window = windows.get_primary().unwrap();

    let scale = Vec3::new(
        primary_window.width() as f32 / arena.width,
        primary_window.height() as f32 / arena.height,
        1.0,
    );

    for (game_transform, mut render_transform) in query.iter_mut() {
        render_transform.translation = game_transform.cur_transform.translation * scale;
        render_transform.scale = scale;
    }
}
