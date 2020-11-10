use bevy::prelude::*;

use crate::shared::{
    collision::Collider,
    movement::{SideScrollDirection, Velocity},
};

/**
# Boat components:
## Components always on
* Velocity (Speed of boat)
* Transform (position and orientation of boat)
* SpriteSheetComponents (image of boat)
* Animation (Animations of the boat)
* Direction (For direction the sprite is facing)
* Collider (AABB)
*/
struct Boat;

pub(super) fn spawn_boat(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let boat_material = materials.add(Color::rgb(0.8, 0.8, 0.8).into());

    println!("Spawning boat...");
    commands
        .spawn(SpriteComponents {
            material: boat_material.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite {
                size: Vec2::new(64.0, 64.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Velocity(Vec3::new(0.0, 5.0, 0.0)))
        .with(Collider {
            width: 64.0,
            height: 64.0,
        })
        .with(SideScrollDirection(true))
        .with(Boat);
}
