use crate::shared::components::{Collider, SideScrollDirection, Velocity};
use bevy::prelude::*;

mod components;
mod states;
mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_player.system())
            .add_system(systems::sink_system.system())
            .add_system(systems::player_bounds_system.system())
            .add_plugin(states::PlayerStatePlugin)
            .run();
    }
}

const PLAYER_WIDTH: f32 = 32.0;
const PLAYER_HEIGHT: f32 = 32.0;

pub fn init_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(SpriteComponents {
            material: materials.add(
                asset_server
                    .load("assets/sprites/player/fish1.png")
                    .unwrap()
                    .into(),
            ),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..Default::default()
        })
        .with(components::Player {
            stats: components::PlayerStats {
                boost_speed: 1500.0,
                boost_duration: 0.1,
                boost_cooldown: 0.2,
                speed: 400.0,
                acceleration: 0.8,
                traction: 0.8,
                stop_threshold: 0.1,
            },
        })
        .with(Velocity(Vec3::zero()))
        .with(SideScrollDirection(true))
        .with(components::Sink { weight: 30.0 })
        .with(Collider {
            width: PLAYER_WIDTH,
            height: PLAYER_HEIGHT,
        })
        .with(states::idle::IdleState);
}
