use crate::shared::{SideScrollDirection, Velocity};
use bevy::prelude::*;

/******************************************************************************
# Fish components:
## Components always on
* Velocity (Speed of fish)
* Transform (position and orientation of fish)
* Player
  - state
  - stats - boost length, speed, etc.
* Collider (AABB)
## display
* Sprite (image of fish)
* Animation (Animations of the fish)
* Direction (For direction the sprite is facing)
## Conditional Components
* Reeling (happens when hooked)
* Starving (happens when timer runs out)
* Sink (makes player sink in the water)
******************************************************************************/
enum PlayerState {
    Alive,
    IsBoosting,
    Dead,
}

struct PlayerStats {
    boost_length: f32,
    speed: f32,
    acceleration: f32,
    traction: f32,
    stop_threshold: f32,
}

pub struct Player {
    state: PlayerState,
    stats: PlayerStats,
}

pub struct Sink {
    weight: f32,
}

pub fn init_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the game's entities to our world
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/player/fish1.png").unwrap().into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(32.0, 32.0)),
            ..Default::default()
        })
        .with(Player {
            state: PlayerState::Alive,
            stats: PlayerStats {
                boost_length: 500.0,
                speed: 500.0,
                acceleration: 0.8,
                traction: 0.2,
                stop_threshold: 0.1,
            },
        })
        .with(Velocity(Vec3::zero()))
        .with(SideScrollDirection(true))
        .with(Sink { weight: 5.0 });
}

pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Velocity, &mut SideScrollDirection)>,
) {
    for (player, mut velocity, mut facing) in &mut query.iter() {
        let mut direction = Vec3::zero();

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            *direction.x_mut() -= 1.0;
            facing.0 = false;
        }

        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            *direction.x_mut() += 1.0;
            facing.0 = true;
        }

        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            *direction.y_mut() += 1.0;
        }

        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            *direction.y_mut() -= 1.0;
        }

        let mut target_speed = Vec3::zero();

        // determine target speed based on whether there is an input or not
        if direction.x() < 0.0 {
            *target_speed.x_mut() = -player.stats.speed;
        }

        if direction.x() > 0.0 {
            *target_speed.x_mut() = player.stats.speed;
        }

        if direction.y() < 0.0 {
            *target_speed.y_mut() = -player.stats.speed;
        }

        if direction.y() > 0.0 {
            *target_speed.y_mut() = player.stats.speed;
        }

        // determine whether to apply traction or regular acceleration
        let a = if target_speed == Vec3::zero() {
            player.stats.traction
        } else {
            player.stats.acceleration
        };

        // calculate new player velocity based on acceleration
        velocity.0 = a * target_speed + (1.0 - a) * velocity.0;

        if velocity.0.length() < player.stats.stop_threshold {
            velocity.0 = Vec3::zero();
        }

        println!("New velocity: {:?}", velocity.0);
    }
}

pub fn sink_system(mut velocity: Mut<Velocity>, sink: &Sink) {
    *velocity.0.y_mut() -= sink.weight;
}

pub fn player_bounds_system() {}
