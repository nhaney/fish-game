use bevy::{prelude::*, render::camera::Camera};

use super::arena::Arena;

pub(super) fn scale_camera_to_screen_size(
    arena: Res<Arena>,
    windows: Res<Windows>,
    mut query: Query<(&Camera, &mut Transform)>,
) {
    let primary_window = windows.get_primary().unwrap();

    let scale = Vec3::new(
        arena.width / primary_window.width() as f32,
        arena.height / primary_window.height() as f32,
        1.0,
    );

    for (_, mut camera_transform) in query.iter_mut() {
        camera_transform.scale = scale;
    }
}
