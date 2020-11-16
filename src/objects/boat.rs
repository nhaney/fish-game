use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashSet;

use crate::shared::{
    arena::Arena,
    collision::Collider,
    game::Difficulty,
    movement::{GameTransform, SideScrollDirection, Velocity},
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

#[derive(Debug)]
struct BoatStats {
    num_poles: u8,
    speed: f32,
    width: f32,
    height: f32,
    worm_chance: f32,
    boat_type: BoatTypes,
}

fn boat_stats_factory(difficulty: u8, rng: &mut ChaCha8Rng) -> BoatStats {
    let boat_type = match rng.gen_range(1, difficulty + 1) {
        1 => BoatTypes::Dinghy,
        2 => BoatTypes::Fishingboat,
        3 => BoatTypes::Speedboat,
        4 => BoatTypes::Yacht,
        _ => panic!("Cannot scale difficulty past 4"),
    };

    match boat_type {
        BoatTypes::Dinghy => BoatStats {
            num_poles: 1,
            speed: (rng.gen_range(30, 40) + (5 * difficulty)) as f32,
            width: 45 as f32,
            height: 10 as f32,
            worm_chance: 0.5,
            boat_type,
        },
        BoatTypes::Fishingboat => BoatStats {
            num_poles: rng.gen_range(1, 3) + difficulty,
            speed: (rng.gen_range(40, 50) + (5 * difficulty)) as f32,
            width: 65 as f32,
            height: 24 as f32,
            worm_chance: 0.8,
            boat_type,
        },
        BoatTypes::Speedboat => BoatStats {
            num_poles: rng.gen_range(1, 2) + difficulty,
            speed: (rng.gen_range(75, 100) + (5 * difficulty)) as f32,
            width: 75 as f32,
            height: 16 as f32,
            worm_chance: 0.4,
            boat_type,
        },
        BoatTypes::Yacht => BoatStats {
            num_poles: rng.gen_range(3, 6) + difficulty,
            speed: (rng.gen_range(60, 75) + (5 * difficulty)) as f32,
            width: 128 as f32,
            height: 64 as f32,
            worm_chance: 0.25,
            boat_type,
        },
    }
}

pub struct Boat;

pub struct Worm;

pub struct Hook;

pub(super) struct BoatSpawner {
    pub spawn_timer: Timer,
}

pub(super) fn boat_spawner_system(
    mut commands: Commands,
    time: Res<Time>,
    arena: Res<Arena>,
    difficulty: Res<Difficulty>,
    mut rng: ResMut<GameRng>,
    mut boat_spawner: ResMut<BoatSpawner>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    boat_spawner.spawn_timer.tick(time.delta_seconds);

    if boat_spawner.spawn_timer.finished {
        println!("Spawning a boat!");
        for _ in 0..rng.rng.gen_range(1, difficulty.multiplier + 1) {
            println!("Generating stats...");
            let stats = boat_stats_factory(difficulty.multiplier, &mut rng.rng);
            println!("Generating a boat with stats: {:?}", stats);
            spawn_boat(
                stats,
                &mut commands,
                &mut materials,
                &mut meshes,
                &arena,
                &mut rng.rng,
            );
        }
    }
}

fn spawn_boat(
    stats: BoatStats,
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
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

    let velocity = Vec3::new(
        match facing_right {
            // going from the right to the left
            true => stats.speed,
            false => -stats.speed,
        },
        0.0,
        0.0,
    );

    let boat_material = materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen()).into());

    // spawn boat
    commands
        .spawn((
            Velocity(velocity),
            Collider {
                width: stats.width,
                height: stats.height,
            },
            SideScrollDirection(facing_right),
            Boat,
            GameTransform {
                cur_transform: Transform::from_translation(boat_start_pos),
                prev_transform: Transform::default(),
            },
        ))
        .with_bundle(SpriteComponents {
            material: boat_material.clone(),
            sprite: Sprite {
                size: Vec2::new(stats.width, stats.height),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(
                // TODO: Fix this from flashing when spawned so this can be removed
                9999.0,
                boat_start_pos.y(),
                0.0,
            )),
            ..Default::default()
        })
        .with_children(|parent| {
            spawn_lines(&stats, rng, parent, materials, meshes);
        });
}

const POLE_HEIGHT: f32 = 5.0;
const FISHING_LINE_WIDTH: f32 = 1.0;
const HOOK_SIZE: f32 = 16.0;
const WORM_SIZE: f32 = 10.0;

fn spawn_lines(
    boat_stats: &BoatStats,
    rng: &mut ChaCha8Rng,
    parent: &mut ChildBuilder,
    materials: &mut Assets<ColorMaterial>,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    // all poles start above the top of the boat at the same y position
    let start_y = (boat_stats.height / 2.0) + POLE_HEIGHT;

    let line_material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let hook_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let worm_material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());

    for i in 1..boat_stats.num_poles + 1 {
        let x_offset = i as f32 * (boat_stats.width / (boat_stats.num_poles + 1) as f32);

        let start_x = -(boat_stats.width / 2.0) + x_offset;

        let start_point = Vec3::new(start_x, start_y, 0.0);

        let line_length = rng.gen_range(100, 325) as f32;
        let line_angle = rng.gen_range(225, 271) as f32;

        let end_point = Vec3::new(
            line_length * (std::f32::consts::PI * (line_angle / 180.0)).cos(),
            line_length * (std::f32::consts::PI * (line_angle / 180.0)).sin(),
            0.0,
        );

        let mid_point = Vec3::new(
            (start_point.x() + end_point.x()) / 2.0,
            (start_point.y() + end_point.y()) / 2.0,
            0.0,
        );

        // spawn the hook at the end point of the line
        parent
            .spawn((
                Hook,
                GameTransform {
                    cur_transform: Transform::from_translation(end_point),
                    prev_transform: Transform::default(),
                },
                Collider {
                    width: HOOK_SIZE,
                    height: HOOK_SIZE,
                },
            ))
            .with_bundle(SpriteComponents {
                sprite: Sprite {
                    size: Vec2::new(HOOK_SIZE, HOOK_SIZE),
                    ..Default::default()
                },
                material: hook_material.clone(),
                ..Default::default()
            });

        if rng.gen_bool(boat_stats.worm_chance as f64) {
            // spawn a worm on the line between the endpoint and the mid point
            let worm_distance_from_mid = rng.gen_range(0, (line_length / 2.0) as u32) as f32;

            let worm_pos =
                mid_point - ((mid_point - end_point).normalize() * worm_distance_from_mid);

            parent
                .spawn((
                    Worm,
                    GameTransform {
                        cur_transform: Transform::from_translation(worm_pos),
                        prev_transform: Transform::default(),
                    },
                    Collider {
                        width: WORM_SIZE,
                        height: WORM_SIZE,
                    },
                ))
                .with_bundle(SpriteComponents {
                    sprite: Sprite {
                        size: Vec2::new(WORM_SIZE, WORM_SIZE),
                        ..Default::default()
                    },
                    material: worm_material.clone(),
                    ..Default::default()
                });
        }

        // spawn the line between the start and end points
        let mut builder = PathBuilder::new();
        builder.move_to(point(start_point.x(), start_point.y()));
        builder.line_to(point(end_point.x(), end_point.y()));
        builder.close();

        let line = builder.build();

        parent.spawn(
            line.stroke(
                line_material.clone(),
                meshes,
                Vec3::new(0.0, 0.0, 0.0),
                &StrokeOptions::default()
                    .with_line_width(FISHING_LINE_WIDTH)
                    .with_line_cap(LineCap::Round)
                    .with_line_join(LineJoin::Round),
            ),
        );
    }
}

pub(super) fn despawn_boat_system(
    mut commands: Commands,
    arena: Res<Arena>,
    boat_query: Query<(&Boat, &GameTransform, Entity)>,
    hook_query: Query<(&Hook, &GameTransform, &Parent)>,
) {
    let mut boats_off_screen: HashSet<Entity> = HashSet::new();

    for (_, transform, entity) in boat_query.iter() {
        let boat_x = transform.cur_transform.translation.x();

        if boat_x < -(arena.width / 2.0) && boat_x > (arena.width / 2.0) {
            boats_off_screen.insert(entity);
        }
    }

    if boats_off_screen.is_empty() {
        return;
    }

    for (_, transform, parent) in hook_query.iter() {
        if !boats_off_screen.contains(parent) {
            continue;
        }

        let hook_x = transform.cur_transform.translation.x();
    }
}
