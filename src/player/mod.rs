use crate::shared::{
    collision::Collider,
    movement::{SideScrollDirection, Velocity},
    stages,
};
use bevy::prelude::*;
use std::collections::HashSet;

mod attributes;
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
            .add_system_to_stage(
                stages::CALCULATE_VELOCITY,
                states::swim_movement_system,
            )
            .add_system_to_stage(
                stages::CALCULATE_VELOCITY,
                states::boost_movement_system,
            )
            .add_system_to_stage(
                stages::CALCULATE_VELOCITY,
                states::boost_cooldown_system,
            )
            .add_system_to_stage(stages::CALCULATE_VELOCITY, movement::sink_system)
            // systems that handle collision
            .add_system_to_stage(
                stages::CORRECT_MOVEMENT,
                collision::player_bounds_system,
            )
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
            .add_system_to_stage(stage::LAST, attributes::hunger_countdown_system);
    }
}

const PLAYER_WIDTH: f32 = 32.0;
const PLAYER_HEIGHT: f32 = 32.0;

fn init_player(commands: &mut Commands) {
    commands.spawn((
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
            max_boosts: 3,
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
    ));
}
