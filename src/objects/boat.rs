use bevy::prelude::*;

use crate::shared::{
    arena::Arena,
    collision::Collider,
    movement::{GameTransform, SideScrollDirection, Velocity},
    rng::GameRng,
};

use rand::Rng;
use rand_chacha::ChaCha8Rng;

pub struct Boat;

pub(super) struct BoatSpawner {
    pub spawn_timer: Timer,
}

pub(super) fn boat_spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    arena: Res<Arena>,
    mut rng: ResMut<GameRng>,
    mut boat_spawner: ResMut<BoatSpawner>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    boat_spawner.spawn_timer.tick(time.delta_seconds);

    if boat_spawner.spawn_timer.finished {
        println!("Spawning a boat!");
        spawn_boat(&mut commands, &mut materials, arena, &mut rng.rng);
    }
}

fn spawn_boat(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    arena: Res<Arena>,
    rng: &mut ChaCha8Rng,
) {
    let facing_right: bool = rng.gen();

    let boat_width = 64.0;

    let x_pos = match facing_right {
        // going from the right to the left
        true => -(arena.width / 2.0) - boat_width + 1.0,
        false => (arena.width / 2.0) + boat_width - 1.0,
    };

    let x_velocity = if facing_right { 50.0 } else { -50.0 };

    let boat_material = materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen()).into());
    commands
        .spawn(SpriteComponents {
            material: boat_material.clone(),
            sprite: Sprite {
                size: Vec2::new(64.0, 64.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Velocity(Vec3::new(x_velocity, 0.0, 0.0)))
        .with(Collider {
            width: 64.0,
            height: 64.0,
        })
        .with(SideScrollDirection(facing_right))
        .with(Boat)
        .with(GameTransform {
            cur_transform: Transform::from_translation(Vec3::new(
                x_pos,
                (arena.height / 2.0) + arena.offset,
                0.0,
            )),
            prev_transform: Transform::default(),
        });
}
