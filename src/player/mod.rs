use crate::shared::{
    animation::AnimationState,
    collision::Collider,
    game::GameRestarted,
    movement::{SideScrollDirection, Velocity},
    render::RenderLayer,
    stages,
};
use bevy::prelude::*;
use std::collections::HashSet;

mod animations;
pub(crate) mod attributes;
mod collision;
pub(crate) mod events;
mod movement;
mod render;
mod states;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        println!("Building player plugin...");
        // Resources for player sprites and animations
        app.init_resource::<render::PlayerStateAnimations>()
            // Events that indicate either a player collided with something/died
            .add_event::<events::PlayerHooked>()
            .add_event::<events::PlayerStarved>()
            .add_event::<events::PlayerBonked>()
            .add_event::<events::PlayerAte>()
            .add_event::<events::PlayerBoosted>()
            // Startup systems initialize the player and its components
            .add_startup_system(init_player)
            // Timer systems
            .add_system_to_stage(stage::EVENT, states::boost_cooldown_system)
            .add_system_to_stage(stage::EVENT, attributes::hunger_countdown_system)
            // systems that handle input/velocity calculation
            // systems that handle collision events
            .add_system_to_stage(stages::HANDLE_EVENTS, attributes::add_boost_system)
            .add_system_to_stage(stages::HANDLE_EVENTS, animations::player_starved_handler)
            .add_system_to_stage(stages::HANDLE_EVENTS, states::swim_movement_system)
            .add_system_to_stage(stages::MOVEMENT, states::boost_movement_system)
            .add_system_to_stage(stages::MOVEMENT, movement::sink_system)
            // This system needs to happen before render, but after final position has
            // been calculated to prevent stuttering movement
            .add_stage_before(stages::PREPARE_RENDER, "adjust_position")
            .add_system_to_stage("adjust_position", collision::player_bounds_system)
            // systems that calculate collision
            .add_system_to_stage(
                stages::CALCULATE_COLLISIONS,
                collision::player_hook_collision_system,
            )
            .add_system_to_stage(
                stages::CALCULATE_COLLISIONS,
                collision::player_worm_collision_system,
            )
            .add_system_to_stage(
                stages::CALCULATE_COLLISIONS,
                collision::player_boat_collision_system,
            )
            // systems that handle final events and presentation
            .add_system_to_stage(stages::HANDLE_EVENTS, reset_player)
            .add_system_to_stage(
                stages::HANDLE_EVENTS,
                render::despawn_trackers_on_gameover_or_restart,
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                render::player_state_animation_change_system,
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                render::update_tracker_display_from_boost_supply,
            );
    }
}

const PLAYER_WIDTH: f32 = 32.0;
const PLAYER_HEIGHT: f32 = 32.0;
const PLAYER_MAX_BOOSTS: u8 = 3;

fn init_player(
    mut commands: &mut Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    player_state_animations: Res<render::PlayerStateAnimations>,
) {
    let player_entity = spawn_player_entity(commands, &player_state_animations);
    render::spawn_player_boost_trackers(
        &mut commands,
        materials,
        meshes,
        PLAYER_WIDTH,
        PLAYER_HEIGHT,
        PLAYER_MAX_BOOSTS,
        player_entity,
    );
}

// TODO: Make this support more than one player
fn reset_player(
    mut commands: &mut Commands,
    materials: ResMut<Assets<ColorMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    restart_events: Res<Events<GameRestarted>>,
    player_state_animations: Res<render::PlayerStateAnimations>,
    mut restart_reader: Local<EventReader<GameRestarted>>,
    player_query: Query<Entity, With<attributes::Player>>,
) {
    if let Some(_) = restart_reader.earliest(&restart_events) {
        println!("Despawning current player entity and creating a new one.");
        let player_entity = player_query.iter().next().unwrap();
        // despawn current player
        commands.despawn_recursive(player_entity);

        // spawn a new player with new ui components
        let new_player = spawn_player_entity(commands, &player_state_animations);
        render::spawn_player_boost_trackers(
            &mut commands,
            materials,
            meshes,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            PLAYER_MAX_BOOSTS,
            new_player,
        );
    }
}

fn spawn_player_entity(
    commands: &mut Commands,
    player_state_animations: &render::PlayerStateAnimations,
) -> Entity {
    let player_animation = player_state_animations
        .map
        .get(&states::PlayerStates::Idle)
        .unwrap();
    let first_animation_frame = player_animation.frames[0].clone();

    commands
        .spawn((
            attributes::Player {
                stats: attributes::PlayerStats {
                    boost_speed: 1500.0,
                    boost_duration: 0.1,
                    boost_cooldown: 0.2,
                    speed: 400.0,
                    acceleration: 0.8,
                    traction: 0.8,
                    stop_threshold: 0.1,
                },
            },
            attributes::Sink { weight: 10.0 },
            attributes::HungerCountdown {
                time_left: 30.0,
                extra_time_per_worm: 3.0,
            },
            attributes::BoostSupply {
                max_boosts: PLAYER_MAX_BOOSTS,
                count: 3,
            },
            states::PlayerState {
                current_state: states::PlayerStates::Idle,
                blocked_transitions: HashSet::new(),
            },
            Velocity(Vec3::zero()),
            SideScrollDirection(true),
            Collider {
                width: PLAYER_WIDTH,
                height: PLAYER_HEIGHT,
            },
            RenderLayer::Player,
        ))
        .with_bundle(SpriteBundle {
            material: first_animation_frame.material_handle.clone(),
            sprite: Sprite {
                size: Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(AnimationState {
            animation: player_animation.clone(),
            timer: Timer::from_seconds(first_animation_frame.time, false),
            frame_index: 0,
            speed_multiplier: 1.0,
        })
        .current_entity()
        .unwrap()
}
