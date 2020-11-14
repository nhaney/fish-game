use bevy::prelude::*;

use crate::shared::{
    arena::Arena,
    collision::Collider,
    game::Difficulty,
    movement::{GameTransform, SideScrollDirection, Velocity},
    rng::GameRng,
};

use rand::Rng;
use rand_chacha::ChaCha8Rng;

enum BoatTypes {
    Dinghy,
    Fishingboat,
    Speedboat,
    Yacht,
}

struct BoatStats {
    num_poles: u8,
    speed: f32,
    width: f32,
    height: f32,
    worm_chance: f32,
    boat_type: BoatTypes,
}

fn boat_stats_factory(difficulty: u8, rng: &mut ChaCha8Rng) -> BoatStats {
    let boat_type = match rng.gen_range(1, difficulty) {
        1 => BoatTypes::Dinghy,
        2 => BoatTypes::Fishingboat,
        3 => BoatTypes::Speedboat,
        4 => BoatTypes::Yacht,
        _ => panic!("Cannot scale difficulty past 4"),
    };

    match boat_type {
        BoatTypes::Dinghy => BoatStats {
            num_poles: rng.gen_range(0, 2) + difficulty,
            speed: (rng.gen_range(30, 40) + (5 * difficulty)) as f32,
            width: 32 as f32,
            height: 5 as f32,
            boat_type,
        },
        BoatTypes::Fishingboat => BoatStats {
            num_poles: rng.gen_range(1, 3) + difficulty,
            speed: (rng.gen_range(40, 50) + (5 * difficulty)) as f32,
            width: 48 as f32,
            height: 24 as f32,
            boat_type,
        },
        BoatTypes::Speedboat => BoatStats {
            num_poles: rng.gen_range(1, 2) + difficulty,
            speed: (rng.gen_range(75, 100) + (5 * difficulty)) as f32,
            width: 32 as f32,
            height: 16 as f32,
            boat_type,
        },
        BoatTypes::Yacht => BoatStats {
            num_poles: rng.gen_range(3, 6) + difficulty,
            speed: (rng.gen_range(60, 75) + (5 * difficulty)) as f32,
            width: 64 as f32,
            height: 64 as f32,
            boat_type,
        },
    }
}

pub struct Boat;

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
) {
    boat_spawner.spawn_timer.tick(time.delta_seconds);

    if boat_spawner.spawn_timer.finished {
        println!("Spawning a boat!");
        for _ in 1..rng.rng.gen_range(1, difficulty.multiplier) {
            let stats = boat_stats_factory(difficulty.multiplier, &mut rng.rng);
            spawn_boat(stats, &mut commands, &mut materials, &arena, &mut rng.rng);
        }
    }
}

fn spawn_boat(
    stats: BoatStats,
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
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
    let x_velocity = if facing_right { 50.0 } else { -50.0 };

    let boat_material = materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen()).into());

    // spawn boat
    commands
        .spawn((
            Velocity(Vec3::new(x_velocity, 0.0, 0.0)),
            Collider {
                width: 64.0,
                height: 64.0,
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
                size: Vec2::new(64.0, 64.0),
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
            spawn_lines_and_hooks(boat_start_pos, &stats, rng, parent, materials);
        });
}

const POLE_HEIGHT: f32 = 5.0;
const FISHING_LINE_WIDTH: f32 = 1.0;

fn spawn_lines_and_hooks(starting_pos: Vec3, boat_stats: &BoatStats, rng: &mut ChaCha8Rng, parent: &mut ChildBuilder, materials: &mut Assets<ColorMaterial>) {
    // all poles start above the top of the boat at the same y position
    let start_y = starting_pos.y() + (boat_stats.height / 2.0) + POLE_HEIGHT;

    let line_material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());

    for i in 1..boat_stats.num_poles + 1 {
        let x_offset = i as f32 * (boat_stats.width / (boat_stats.num_poles + 1) as f32);
        let start_x = starting_pos.x() - (boat_stats.width / 2.0) + x_offset;

        let line_length = rng.gen_range(75, 510) as f32;
        let line_angle = rng.gen_range(45, 91) as f32;

        let end_x = line_length * (std::f32::consts::PI * (line_angle / 180.0)).cos();
        let end_y = line_length * (std::f32::consts::PI * (line_angle / 180.0)).sin();

        let mid_point = Vec3::new(start_x + end_x / 2.0, start_y + end_y / 2.0, 0.0);

        parent.spawn(SpriteComponents {
            material: line_material.clone(),
            sprite: Sprite {
                size: Vec2::new(FISHING_LINE_WIDTH, line_length),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(GameTransform {
            cur_transform: Transform {
                translation: mid_point,                    
                rotation: Quat::from_rotation_z(std::f32::consts::PI * (line_angle / 180.0)),
                ..Default::default()
            },
            prev_transform: Transform::default()
        })
        .with_children(|parent| {                
            spawn_hooks_and_worm(parent, end_point, materials);
        });
    }
}

fn spawn_hooks_and_worm(mut parent: &mut ChildBuilder, end_point: Vec3, materials: &mut Assets<ColorMaterial>) {
    // spawns the hooks and the worm
}
