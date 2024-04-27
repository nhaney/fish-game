use bevy::{prelude::*, window::WindowResized};

use super::arena::Arena;
use super::MainCamera;

pub(super) fn scale_camera_to_screen_size(
    arena: Res<Arena>,
    mut resize_event_reader: EventReader<WindowResized>,
    mut query: Query<(&Camera, &MainCamera, &mut Transform)>,
) {
    if let Some(resize_event) = resize_event_reader.read().next() {
        let scale = Vec3::new(
            arena.width / resize_event.width,
            arena.height / resize_event.height,
            1.0,
        );

        let (_, _, mut camera_transform) = query
            .get_single_mut()
            .expect("Could not find camera to resize.");

        camera_transform.scale = scale;
    }
}

#[derive(Debug, Component)]
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
