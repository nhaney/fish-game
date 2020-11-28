use bevy::{prelude::*, render::camera::Camera, ui::camera::UI_CAMERA};

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
        if camera.name != Some(UI_CAMERA.to_string()) {
            camera_transform.scale = scale;
        }
    }
}

pub struct NonRotatingChild;

/** Rotates the child element so that compared to the parent's rotation it is the
  TODO: MAKE THIS WORK
*/
pub(super) fn readjust_rotation(
    mut query: Query<(&mut GlobalTransform, &Transform), With<NonRotatingChild>>,
) {
    for (mut global_transform, transform) in query.iter_mut() {
        // println!("Rotation of boost tracker: {:?}", transform.rotation);
        global_transform.rotation = Quat::identity();
    }
}
