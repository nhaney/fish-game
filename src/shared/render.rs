use bevy::{prelude::*, render::camera::Camera, ui::camera::CAMERA_UI};

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

    for (camera, mut camera_transform) in query.iter_mut() {
        if camera.name != Some(CAMERA_UI.to_string()) {
            camera_transform.scale = scale;
        }
    }
}

pub enum RenderLayer {
    Player,
    Objects,
    Background,
}

/// TODO: Use Vec2's everywhere so this can just be set once when added
pub(super) fn adjust_to_render_layer(mut query: Query<(&RenderLayer, &mut Transform)>) {
    for (render_layer, mut transform) in query.iter_mut() {
        transform.translation.z = match render_layer {
            RenderLayer::Player => 3.0,
            RenderLayer::Objects => 2.0,
            RenderLayer::Background => 1.0,
        }
    }
}
