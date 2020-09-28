use bevy::prelude::*;

use super::components::{state, Player, PlayerStats, Sink};
use crate::shared::{
    arena::Arena,
    components::{Collider, SideScrollDirection, Velocity},
};

const PLAYER_WIDTH: f32 = 32.0;
const PLAYER_HEIGHT: f32 = 32.0;

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
            stats: PlayerStats {
                boost_speed: 1500.0,
                boost_duration: 0.1,
                boost_cooldown: 0.2,
                speed: 400.0,
                acceleration: 1.0,
                traction: 0.05,
                stop_threshold: 0.1,
            },
        })
        .with(Velocity(Vec3::zero()))
        .with(SideScrollDirection(true))
        .with(Sink { weight: 5.0 })
        .with(Collider {
            width: PLAYER_WIDTH,
            height: PLAYER_HEIGHT,
        })
        .with(state::NormalState);
}

// TODO: Change to use specific player command events
pub fn normal_player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    player: &Player,
    mut velocity: Mut<Velocity>,
    mut facing: Mut<SideScrollDirection>,
    _state: &state::NormalState,
) {
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

pub fn start_boost_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<
        Without<state::BoostCooldown, (&state::NormalState, &Player, &SideScrollDirection, Entity)>,
    >,
) {
    for (_state, player, facing, entity) in &mut query.iter() {
        let mut target_speed = Vec3::zero();

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            *target_speed.x_mut() -= player.stats.speed;
        }

        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            *target_speed.x_mut() += player.stats.speed;
        }

        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            *target_speed.y_mut() += player.stats.speed;
        }

        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            *target_speed.y_mut() -= player.stats.speed;
        }

        if keyboard_input.pressed(KeyCode::Space) {
            state::start_boost(&mut commands, entity, player, &facing, &target_speed);
        }
    }
}

pub fn boost_player_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut player: Mut<Player>,
    mut velocity: Mut<Velocity>,
    mut boost_state: Mut<state::BoostState>,
    entity: Entity,
) {
    velocity.0 = boost_state.boost_velocity;

    boost_state.boost_timer.tick(time.delta_seconds);

    if boost_state.boost_timer.finished {
        state::end_boost(&mut commands, &mut player, entity);
    }
}

pub fn boost_cooldown_system(
    mut commands: Commands,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut boost_cooldown: Mut<state::BoostCooldown>,
    entity: Entity,
) {
    boost_cooldown.timer.tick(time.delta_seconds);

    boost_cooldown.did_release =
        boost_cooldown.did_release || !keyboard_input.pressed(KeyCode::Space);

    if boost_cooldown.timer.finished && boost_cooldown.did_release {
        println!("Boost cooldown finished");
        commands.remove_one::<state::BoostCooldown>(entity);
    }
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
