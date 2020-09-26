use crate::arena::Arena;
use crate::shared::{Collider, SideScrollDirection, Velocity};
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
const PLAYER_WIDTH: f32 = 32.0;
const PLAYER_HEIGHT: f32 = 32.0;

#[derive(Debug)]
enum PlayerState {
    Normal,
    IsBoosting(BoostState),
    Dead,
}

#[derive(Debug)]
struct BoostState {
    pub boost_velocity: Vec3,
    pub boost_timer: Timer,
}

impl BoostState {
    pub fn new(boost_velocity: Vec3, boost_duration: f32) -> Self {
        BoostState {
            boost_velocity,
            boost_timer: Timer::from_seconds(boost_duration, false),
        }
    }
}

#[derive(Debug)]
struct PlayerStats {
    boost_speed: f32,
    boost_duration: f32,
    speed: f32,
    acceleration: f32,
    traction: f32,
    stop_threshold: f32,
}

#[derive(Debug)]
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
            sprite: Sprite::new(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..Default::default()
        })
        .with(Player {
            state: PlayerState::Normal,
            stats: PlayerStats {
                boost_speed: 1250.0,
                boost_duration: 0.2,
                speed: 400.0,
                acceleration: 1.0,
                traction: 1.0,
                stop_threshold: 0.1,
            },
        })
        .with(Velocity(Vec3::zero()))
        .with(SideScrollDirection(true))
        .with(Sink { weight: 5.0 })
        .with(Collider {
            width: PLAYER_WIDTH,
            height: PLAYER_HEIGHT,
        });
}

// TODO: Change to use specific player command events
pub fn normal_player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Mut<Player>,
    mut velocity: Mut<Velocity>,
    mut facing: Mut<SideScrollDirection>,
) {
    if let PlayerState::Normal = player.state {
        let mut target_speed = Vec3::zero();

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            *target_speed.x_mut() -= player.stats.speed;
            facing.0 = false;
        }

        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            *target_speed.x_mut() += player.stats.speed;
            facing.0 = true;
        }

        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            *target_speed.y_mut() += player.stats.speed;
        }

        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            *target_speed.y_mut() -= player.stats.speed;
        }

        if keyboard_input.pressed(KeyCode::Space) {
            start_boost(&mut player, &facing, &target_speed);
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
    }
}

fn start_boost(player: &mut Player, facing: &SideScrollDirection, target_speed: &Vec3) {
    // if not moving, boost in the direction that the player is facing
    let boost_direction = if *target_speed == Vec3::zero() {
        if facing.is_right() {
            Vec3::unit_x()
        } else {
            -Vec3::unit_x()
        }
    } else {
        target_speed.normalize()
    };

    // change state and specify velocity of the boost
    player.state = PlayerState::IsBoosting(BoostState::new(
        boost_direction * player.stats.boost_speed,
        player.stats.boost_duration,
    ));

    println!("Started boost state: {:?}", player.state);
}

pub fn boost_player_movement_system(
    time: Res<Time>,
    mut player: Mut<Player>,
    mut velocity: Mut<Velocity>,
) {
    if let PlayerState::IsBoosting(ref mut boost_state) = player.state {
        velocity.0 = boost_state.boost_velocity;

        boost_state.boost_timer.tick(time.delta_seconds);

        if boost_state.boost_timer.finished {
            end_boost(&mut player);
        }
    }
}

fn end_boost(player: &mut Player) {
    println!("Finished boost");
    player.state = PlayerState::Normal;
}

pub fn sink_system(mut velocity: Mut<Velocity>, sink: &Sink) {
    *velocity.0.y_mut() -= sink.weight;
}

pub fn player_bounds_system(
    arena: Res<Arena>,
    _player: &Player,
    mut transform: Mut<Transform>,
    collider: &Collider,
) {
    let mut new_pos = transform.translation().clone();

    let arena_half_width = arena.width / 2.0;
    let arena_half_height = arena.height / 2.0;

    let player_half_width = collider.width / 2.0;
    let player_half_height = collider.height / 2.0;

    if new_pos.x() - player_half_width < -arena_half_width {
        *new_pos.x_mut() = -arena_half_width + player_half_width;
        println!("Repositioned to {:?}", new_pos);
    }

    if new_pos.x() + player_half_width > arena_half_width {
        *new_pos.x_mut() = arena_half_width - player_half_width;
        println!("Repositioned to {:?}", new_pos);
    }

    if new_pos.y() - player_half_height < -arena_half_height {
        *new_pos.y_mut() = -arena_half_height + player_half_height;
        println!("Repositioned to {:?}", new_pos);
    }

    if new_pos.y() + player_half_height > (arena_half_height + arena.offset) {
        *new_pos.y_mut() = (arena_half_height + arena.offset) - player_half_height;
        println!("Repositioned to {:?}", new_pos);
    }

    transform.set_translation(new_pos);
}
