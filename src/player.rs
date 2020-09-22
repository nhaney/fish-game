use crate::shared::Velocity;
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
    swim_speed: f32,
}

pub struct Player {
    state: PlayerState,
    stats: PlayerStats,
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
                swim_speed: 20.0,
            },
        })
        .with(Velocity(Vec3::zero()));
}

pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Velocity)>,
) {
    for (player, mut velocity) in &mut query.iter() {
        let mut x_direction = 0.0;
        let mut y_direction = 0.0;

        velocity.0 = Vec3::zero();

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            x_direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            x_direction += 1.0;
        }

        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            y_direction += 1.0;
        }

        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            y_direction -= 1.0;
        }

        // move the paddle horizontally
        *velocity.0.x_mut() += x_direction * player.stats.swim_speed;
        *velocity.0.y_mut() += y_direction * player.stats.swim_speed;

        println!("New velocity: {:?}", velocity.0);
    }
}
