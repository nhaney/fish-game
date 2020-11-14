use crate::shared::{
    collision::Collider,
    movement::{GameTransform, SideScrollDirection, Velocity},
};
use bevy::prelude::*;
use std::collections::HashSet;

mod attributes;
mod collision;
mod movement;
mod render;
mod states;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        println!("Building player plugin...");
        app.add_startup_system(init_player.system())
            // systems that handle input/movement
            .add_system_to_stage(stage::PRE_UPDATE, states::swim_movement_system.system())
            .add_system_to_stage(stage::PRE_UPDATE, states::boost_movement_system.system())
            .add_system_to_stage(stage::PRE_UPDATE, states::boost_cooldown_system.system())
            .add_system_to_stage(stage::PRE_UPDATE, movement::sink_system.system())
            // systems that handle collision
            .add_system_to_stage(stage::POST_UPDATE, collision::player_bounds_system.system())
            // systems that handle presentation
            .init_resource::<render::PlayerSpriteHandles>()
            .init_resource::<render::PlayerStateAnimations>()
            .add_startup_system(render::start_atlas_load.system())
            .add_system_to_stage(stage::LAST, render::load_player_atlas.system())
            .add_system_to_stage(
                stage::LAST,
                render::player_state_animation_change_system.system(),
            );
    }
}

const PLAYER_WIDTH: f32 = 32.0;
const PLAYER_HEIGHT: f32 = 32.0;

fn init_player(mut commands: Commands) {
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
        GameTransform {
            cur_transform: Transform::default(),
            prev_transform: Transform::default(),
        },
    ));
}
