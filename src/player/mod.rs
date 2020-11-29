use crate::shared::{
    collision::Collider,
    game::GameRestarted,
    movement::{SideScrollDirection, Velocity},
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
        app.init_resource::<render::PlayerSpriteHandles>()
            .init_resource::<render::PlayerStateAnimations>()
            // Events that indicate either a player collided with something/died
            .add_event::<events::PlayerHooked>()
            .add_event::<events::PlayerStarved>()
            .add_event::<events::PlayerBonked>()
            .add_event::<events::PlayerAte>()
            // Startup systems initialize the player and its components
            .add_startup_system(init_player)
            .add_startup_system(render::start_atlas_load.system())
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
            .add_system_to_stage(stages::PREPARE_RENDER, render::load_player_atlas)
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
) {
    let player_entity = spawn_player_entity(commands);
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
    mut player_sprite_handles: ResMut<render::PlayerSpriteHandles>,
    mut restart_reader: Local<EventReader<GameRestarted>>,
    player_query: Query<Entity, With<attributes::Player>>,
) {
    if let Some(_) = restart_reader.earliest(&restart_events) {
        println!("Despawning current player entity and creating a new one.");
        let player_entity = player_query.iter().next().unwrap();
        // despawn current player
        commands.despawn_recursive(player_entity);

        // mark that new sprites need to be added to the player
        player_sprite_handles.atlas_loaded = false;

        // spawn a new player with new ui components
        let new_player = spawn_player_entity(commands);
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

fn spawn_player_entity(commands: &mut Commands) -> Entity {
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
        ))
        .current_entity()
        .unwrap()
}
