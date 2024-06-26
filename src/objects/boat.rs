use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;

use crate::player::{
    attributes::Player,
    events::{PlayerAte, PlayerBonked, PlayerHooked},
};
use crate::shared::{
    animation::{Animation, AnimationFrame, AnimationState},
    arena::Arena,
    collision::Collider,
    game::{Difficulty, GameOver, GameRestarted, GameState},
    movement::{Destination, Follow, Velocity},
    render::RenderLayer,
    rng::GameRng,
};

use rand::Rng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug)]
enum BoatTypes {
    Dinghy,
    Fishingboat,
    Speedboat,
    Yacht,
}

#[derive(Debug, Component)]
struct BoatStats {
    num_poles: u8,
    speed: f32,
    width: f32,
    height: f32,
    worm_chance: f32,
}

#[derive(Debug, Resource)]
pub(super) struct BoatMaterials {
    boat: Handle<Image>,
    line: Color,
    worm: Animation,
    hook: Handle<Image>,
}

impl FromWorld for BoatMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        BoatMaterials {
            boat: asset_server.load("sprites/boat/boat.png"),
            line: Color::BLACK,
            worm: Animation {
                should_loop: true,
                frames: vec![
                    AnimationFrame {
                        material_handle: asset_server.load("sprites/worm/worm1.png"),
                        time: 0.5,
                    },
                    AnimationFrame {
                        material_handle: asset_server.load("sprites/worm/worm2.png"),
                        time: 0.5,
                    },
                ],
            },
            hook: asset_server.load("sprites/hook/hook.png"),
        }
    }
}

fn boat_stats_factory(difficulty: u8, rng: &mut ChaCha8Rng) -> BoatStats {
    let boat_type = match rng.gen_range(1..difficulty + 1) {
        1 => BoatTypes::Dinghy,
        2 => BoatTypes::Fishingboat,
        3 => BoatTypes::Speedboat,
        4 => BoatTypes::Yacht,
        _ => panic!("Cannot scale difficulty past 4"),
    };

    match boat_type {
        BoatTypes::Dinghy => BoatStats {
            num_poles: 1,
            speed: (rng.gen_range(30..40) + (5 * difficulty)) as f32,
            width: 45.0,
            height: 10.0,
            worm_chance: 0.5,
        },
        BoatTypes::Fishingboat => BoatStats {
            num_poles: rng.gen_range(1..3) + difficulty,
            speed: (rng.gen_range(40..50) + (5 * difficulty)) as f32,
            width: 65.0,
            height: 24.0,
            worm_chance: 0.8,
        },
        BoatTypes::Speedboat => BoatStats {
            num_poles: rng.gen_range(1..2) + difficulty,
            speed: (rng.gen_range(75..100) + (5 * difficulty)) as f32,
            width: 75.0,
            height: 16.0,
            worm_chance: 0.4,
        },
        BoatTypes::Yacht => BoatStats {
            num_poles: rng.gen_range(3..6) + difficulty,
            speed: (rng.gen_range(60..75) + (5 * difficulty)) as f32,
            width: 128.0,
            height: 64.0,
            worm_chance: 0.25,
        },
    }
}

#[derive(Component)]
pub struct Boat;

#[derive(Component)]
pub struct Worm {
    #[allow(dead_code)]
    line_entity: Entity,
}

#[derive(Component)]
pub struct Hook {
    line_entity: Entity,
}

#[derive(Component)]
pub struct Line {
    start_point: Vec3,
    end_point: Vec3,
}

#[derive(Debug, Resource)]
pub(super) struct BoatSpawner {
    pub spawn_timer: Timer,
}

// TODO: See if this can be reduced
#[allow(clippy::too_many_arguments)]
pub(super) fn boat_spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    arena: Res<Arena>,
    game_state: Res<GameState>,
    difficulty: Res<Difficulty>,
    boat_materials: Res<BoatMaterials>,
    mut rng: ResMut<GameRng>,
    mut boat_spawner: ResMut<BoatSpawner>,
) {
    if !game_state.is_running() {
        return;
    }

    boat_spawner.spawn_timer.tick(time.delta());

    if boat_spawner.spawn_timer.finished() {
        for _ in 0..rng.rng.gen_range(1..difficulty.multiplier + 1) {
            let stats = boat_stats_factory(difficulty.multiplier, &mut rng.rng);
            spawn_boat(stats, &mut commands, &boat_materials, &arena, &mut rng.rng);
        }
    }
}

fn spawn_boat(
    stats: BoatStats,
    commands: &mut Commands,
    boat_materials: &BoatMaterials,
    arena: &Arena,
    rng: &mut ChaCha8Rng,
) {
    let facing_right: bool = rng.gen();

    let boat_start_pos = Vec3::new(
        match facing_right {
            // going from the right to the left
            true => -(arena.width / 2.0) - stats.width + 1.0,
            false => (arena.width / 2.0) + stats.width - 1.0,
        },
        (arena.height / 2.0) + arena.offset,
        0.0,
    );

    let boat_start_rotation = match facing_right {
        true => Quat::from_rotation_y(0.0),
        false => Quat::from_rotation_y(std::f32::consts::PI),
    };

    let velocity = Vec3::new(
        match facing_right {
            // going from the right to the left
            true => stats.speed,
            // going from the left to the right
            false => -stats.speed,
        },
        0.0,
        0.0,
    );

    // spawn boat
    commands
        .spawn((
            Velocity(velocity),
            Collider {
                width: stats.width,
                height: stats.height,
            },
            //SideScrollDirection(facing_right),
            Boat,
            RenderLayer::Objects,
            SpriteBundle {
                texture: boat_materials.boat.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(stats.width, stats.height)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: boat_start_pos,
                    rotation: boat_start_rotation,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            spawn_lines(&stats, rng, parent, boat_materials);
        });
}

const ROD_LENGTH: f32 = 5.0;
const POLE_HEIGHT: f32 = 10.0;
const FISHING_LINE_WIDTH: f32 = 1.0;
const HOOK_SIZE: f32 = 16.0;
const WORM_SIZE: f32 = 16.0;

fn spawn_lines(
    boat_stats: &BoatStats,
    rng: &mut ChaCha8Rng,
    parent: &mut ChildBuilder,
    boat_materials: &BoatMaterials,
) {
    // all poles start above the top of the boat at the same y position
    let hook_material = boat_materials.hook.clone();
    let worm_animation = boat_materials.worm.clone();

    for i in 1..boat_stats.num_poles + 1 {
        // start point of the rod from the start of the boat
        let rod_offset = i as f32 * (boat_stats.width / (boat_stats.num_poles + 1) as f32);

        // the start point of the rod
        let rod_start_point = Vec3::new(
            -(boat_stats.width / 2.0) + rod_offset,
            boat_stats.height / 2.0,
            0.0,
        );

        // the point that the rod angles
        let rod_angle_point = Vec3::new(rod_start_point.x, rod_start_point.y + POLE_HEIGHT, 0.0);

        // the start point of the line, behind the rod angle point
        let line_start_point = Vec3::new(rod_angle_point.x - ROD_LENGTH, rod_angle_point.y, 0.0);

        // TODO: Make this more dynamic based on the boat/arena.
        let line_length = rng.gen_range(50 + boat_stats.height as u32..325) as f32;
        let line_angle = rng.gen_range(225..271) as f32;

        let line_end_point = Vec3::new(
            line_start_point.x + line_length * (std::f32::consts::PI * (line_angle / 180.0)).cos(),
            line_start_point.y + line_length * (std::f32::consts::PI * (line_angle / 180.0)).sin(),
            0.0,
        );

        let line_mid_point = Vec3::new(
            (line_start_point.x + line_end_point.x) / 2.0,
            (line_start_point.y + line_end_point.y) / 2.0,
            0.0,
        );

        // spawn the rod
        let mut builder = PathBuilder::new();
        builder.move_to([rod_start_point.x, rod_start_point.y].into());
        builder.line_to([rod_angle_point.x, rod_start_point.y].into());
        builder.line_to([line_start_point.x, line_start_point.y].into());
        let rod = builder.build();

        parent.spawn((
            ShapeBundle {
                path: rod,
                ..default()
            },
            Stroke {
                color: boat_materials.line.clone(),
                options: StrokeOptions::default()
                    .with_line_width(FISHING_LINE_WIDTH)
                    .with_line_cap(LineCap::Round)
                    .with_line_join(LineJoin::Round),
            },
        ));

        // spawn the line that connects the start and end points
        builder = PathBuilder::new();
        builder.move_to([line_start_point.x, line_start_point.y].into());
        builder.line_to([line_end_point.x, line_end_point.y].into());

        // TODO: Is this needed? I don't think we need to close it...
        builder.close();

        let line = builder.build();

        let line_entity = parent
            .spawn((
                ShapeBundle {
                    path: line,
                    ..default()
                },
                Stroke {
                    color: boat_materials.line.clone(),
                    options: StrokeOptions::default()
                        .with_line_width(FISHING_LINE_WIDTH)
                        .with_line_cap(LineCap::Round)
                        .with_line_join(LineJoin::Round),
                },
                Line {
                    start_point: line_start_point,
                    end_point: line_end_point,
                },
            ))
            .id();

        // spawn the hook at the end point of the line
        let mut hook_point = line_end_point;
        hook_point.y -= HOOK_SIZE / 2.0;

        parent.spawn((
            Hook { line_entity },
            Collider {
                width: HOOK_SIZE,
                height: HOOK_SIZE,
            },
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(HOOK_SIZE, HOOK_SIZE)),
                    ..default()
                },
                texture: hook_material.clone(),
                transform: Transform::from_translation(hook_point),
                ..Default::default()
            },
        ));

        if rng.gen_bool(boat_stats.worm_chance as f64) {
            // spawn a worm on the line between the endpoint and the mid point
            let worm_distance_from_mid = rng.gen_range(0..(line_length / 2.0) as u32) as f32;

            let worm_pos = line_mid_point
                - ((line_mid_point - line_end_point).normalize() * worm_distance_from_mid);

            let worm_initial_animation_frame = worm_animation.frames[0].clone();
            parent.spawn((
                Worm { line_entity },
                Collider {
                    width: WORM_SIZE,
                    height: WORM_SIZE,
                },
                AnimationState::from_animation(&worm_animation, rng.gen::<f32>() * 2.0),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(WORM_SIZE, WORM_SIZE)),
                        ..default()
                    },
                    texture: worm_initial_animation_frame.material_handle.clone(),
                    transform: Transform::from_translation(worm_pos),
                    ..Default::default()
                },
            ));
        }
    }
}

pub(super) fn despawn_boat_system(
    mut commands: Commands,
    arena: Res<Arena>,
    boat_query: Query<(&Boat, &Collider, &Transform, Entity)>,
    hook_query: Query<(&Hook, &Collider, &GlobalTransform, &Parent)>,
) {
    let mut boats_off_screen: Vec<Entity> = Vec::new();

    for (_, collider, transform, entity) in boat_query.iter() {
        let boat_x = transform.translation.x;

        if (boat_x + collider.width) < -(arena.width / 2.0)
            || (boat_x - collider.width) > (arena.width / 2.0)
        {
            boats_off_screen.push(entity);
        }
    }

    if boats_off_screen.is_empty() {
        return;
    }

    let mut off_screen_hooks: HashMap<Entity, usize> = HashMap::new();
    let mut total_hooks: HashMap<Entity, usize> = HashMap::new();

    for (_, collider, transform, parent) in hook_query.iter() {
        if !boats_off_screen.contains(parent) {
            continue;
        }

        let hook_x = transform.translation().x;

        if (hook_x + collider.width) < -(arena.width / 2.0)
            || (hook_x - collider.width) > (arena.width / 2.0)
        {
            let off_screen_counter = off_screen_hooks.entry(parent.get()).or_insert(0);
            *off_screen_counter += 1;
        }

        let total_counter = total_hooks.entry(parent.get()).or_insert(0);
        *total_counter += 1
    }

    for (boat, count) in off_screen_hooks.into_iter() {
        let total = *total_hooks.get(&boat).unwrap();
        if count == total {
            commands.entity(boat).despawn_recursive();
        }
    }
}

pub(super) fn boat_exit_system(
    mut commands: Commands,
    mut game_over_reader: EventReader<GameOver>,
    mut query: Query<(&Boat, &mut Velocity, &mut Transform, Entity)>,
) {
    if let Some(game_over_event) = game_over_reader.read().next() {
        for (_, mut velocity, mut transform, entity) in query.iter_mut() {
            if Some(entity) == game_over_event.winning_boat {
                // remove the velocity of the boat that got the fish so it stays on screen
                commands.entity(entity).remove::<Velocity>();
                continue;
            }

            // turn boat around if they havent passed halfway across the screen
            if transform.translation.x < 0.0 {
                transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
                velocity.0.x = -velocity.0.x.abs();
            } else {
                transform.rotation = Quat::from_rotation_y(0.0);
                velocity.0.x = velocity.0.x.abs();
            }

            velocity.0.x *= 2.;
        }
    }
}

pub(super) fn despawn_worms_on_game_over(
    mut commands: Commands,
    mut game_over_reader: EventReader<GameOver>,
    query: Query<Entity, With<Worm>>,
) {
    if game_over_reader.read().next().is_some() {
        for worm_entity in query.iter() {
            commands.entity(worm_entity).despawn_recursive();
        }
    }
}

pub(super) fn worm_eaten_system(
    mut commands: Commands,
    mut player_ate_reader: EventReader<PlayerAte>,
    query: Query<(&Worm, Entity)>,
) {
    for player_ate_event in player_ate_reader.read() {
        for (_, entity) in query.iter() {
            if entity == player_ate_event.worm_entity {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub(super) fn reset_boats_on_restart(
    mut commands: Commands,
    mut boat_spawner: ResMut<BoatSpawner>,
    mut restart_reader: EventReader<GameRestarted>,
    boat_query: Query<Entity, With<Boat>>,
) {
    if restart_reader.read().next().is_some() {
        debug!("Despawning all boats and restarting spawner because of restart event.");
        // despawn all boats
        for boat_entity in boat_query.iter() {
            commands.entity(boat_entity).despawn_recursive();
        }
        // reset spawner
        boat_spawner.spawn_timer = Timer::from_seconds(5.0, TimerMode::Repeating);
    }
}

/**
    1. Attach player as a child entity of the hook and remove the player's velocity
    2. Give the hook a velocity and destination
*/
// TODO: Refactor to use a predefined type?
#[allow(clippy::type_complexity)]
pub(super) fn player_hooked_handler(
    mut commands: Commands,
    mut player_hooked_reader: EventReader<PlayerHooked>,
    hook_query: Query<(&Hook, &Transform), With<Hook>>,
    line_query: Query<&Line>,
    mut player_query: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Hook>)>,
) {
    for player_hooked_event in player_hooked_reader.read() {
        let player_entity = player_hooked_event.player_entity;
        let hook_entity = player_hooked_event.hook_entity;

        // Set the players velocity to zero, move it to the position of the hook, and make it a child of
        // the hook.
        let (mut player_transform, mut player_velocity) =
            player_query.get_mut(player_entity).unwrap();
        player_transform.translation = Vec3::ZERO;
        player_velocity.0 = Vec3::ZERO;

        // make player follow hook back
        // TODO: Can't we just parent the player to the hook for simplicity here?
        commands
            .entity(player_hooked_event.player_entity)
            .insert((Follow {
                entity_to_follow: player_hooked_event.hook_entity,
                offset: Vec3::ZERO,
                follow_global_transform: true,
            },));

        let (hook, hook_transform) = hook_query.get(hook_entity).unwrap();
        let line_entity = hook.line_entity;
        let line_start = line_query.get(line_entity).unwrap().start_point;
        let reel_velocity = (line_start - hook_transform.translation).normalize() * 300.0;

        commands.entity(hook_entity).insert((
            Destination {
                point: line_start,
                trigger_distance: 10.0,
            },
            Velocity(reel_velocity),
        ));
    }
}

// When a hook changes position, redraw the line that the hook is on
pub(super) fn redraw_line_when_hook_moves(
    mut commands: Commands,
    boat_materials: Res<BoatMaterials>,
    hook_query: Query<(&Hook, &Transform), Changed<Transform>>,
    mut line_query: Query<&mut Line>,
) {
    for (hook_info, changed_transform) in hook_query.iter() {
        let line_entity = hook_info.line_entity;

        let mut line = line_query.get_mut(line_entity).unwrap();
        line.end_point = changed_transform.translation;

        // the line should connect to the top of the hook
        line.end_point.y += HOOK_SIZE / 2.0;

        let mut builder = PathBuilder::new();
        builder.move_to([line.start_point.x, line.start_point.y].into());
        builder.line_to([line.end_point.x, line.end_point.y].into());

        // TODO: Do we need to close this?
        builder.close();

        let line = builder.build();

        commands.entity(line_entity).insert((
            ShapeBundle {
                path: line,
                ..default()
            },
            Stroke {
                color: boat_materials.line.clone(),
                options: StrokeOptions::default()
                    .with_line_width(FISHING_LINE_WIDTH)
                    .with_line_cap(LineCap::Round)
                    .with_line_join(LineJoin::Round),
            },
        ));
    }
}

pub(super) fn player_bonked_handler(
    mut commands: Commands,
    mut player_bonked_reader: EventReader<PlayerBonked>,
    boat_query: Query<(&Collider, &Transform), With<Boat>>,
    mut player_query: Query<(&Transform, &mut Velocity), With<Player>>,
) {
    for player_bonked_event in player_bonked_reader.read() {
        let player_entity = player_bonked_event.player_entity;
        let boat_entity = player_bonked_event.boat_entity;

        let (boat_collider, boat_transform) = boat_query.get(boat_entity).unwrap();
        let (player_transform, mut player_velocity) = player_query.get_mut(player_entity).unwrap();

        let target_point = Vec3::new(
            player_transform.translation.x,
            boat_transform.translation.y + (boat_collider.height / 2.0),
            1.0,
        );

        player_velocity.0 = (target_point - player_transform.translation).normalize() * 50.0;

        commands.entity(player_entity).insert(Destination {
            point: target_point,
            trigger_distance: 10.0,
        });
    }
}
