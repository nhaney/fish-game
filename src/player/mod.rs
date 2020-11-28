use crate::shared::{
    collision::Collider,
    game::GameRestarted,
    movement::{SideScrollDirection, Velocity},
    stages,
};
use bevy::prelude::*;
use std::collections::HashSet;

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
        app.add_startup_system(init_player)
            // systems that handle input/movement
            .add_system_to_stage(stages::CALCULATE_VELOCITY, states::swim_movement_system)
            .add_system_to_stage(stages::CALCULATE_VELOCITY, states::boost_movement_system)
            .add_system_to_stage(stages::CALCULATE_VELOCITY, states::boost_cooldown_system)
            .add_system_to_stage(stages::CALCULATE_VELOCITY, movement::sink_system)
            // systems that handle collision
            .add_system_to_stage(stages::CORRECT_MOVEMENT, collision::player_bounds_system)
            .add_system_to_stage(
                stages::CORRECT_MOVEMENT,
                collision::player_hook_collision_system,
            )
            .add_system_to_stage(
                stages::CORRECT_MOVEMENT,
                collision::player_worm_collision_system,
            )
            .add_system_to_stage(
                stages::CORRECT_MOVEMENT,
                collision::player_boat_collision_system,
            )
            // systems that handle presentation
            .init_resource::<render::PlayerSpriteHandles>()
            .init_resource::<render::PlayerStateAnimations>()
            .add_startup_system(render::start_atlas_load.system())
            .add_system(render::load_player_atlas)
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                render::player_state_animation_change_system,
            )
            //events
            .add_event::<events::PlayerHooked>()
            .add_event::<events::PlayerStarved>()
            .add_event::<events::PlayerBonked>()
            .add_event::<events::PlayerAte>()
            //attributes
            .add_system_to_stage(stage::LAST, attributes::add_boost_system)
            .add_system_to_stage(stage::LAST, attributes::hunger_countdown_system)
            .add_system_to_stage(
                stage::LAST,
                render::update_tracker_display_from_boost_supply,
            )
            .add_system_to_stage(stage::LAST, reset_player);
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
