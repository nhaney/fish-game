use bevy::prelude::*;

use crate::{
    arena::ArenaBound,
    core2d::{Position2d, Vec2, Velocity2d},
};

/// Initialize the player character and its components.
pub(crate) fn initialize_player(mut commands: Commands) {
    commands.spawn((
        Player,
        PlayerState::default(),
        ArenaBound,
        StarveTimer {
            ticks_left: 100 * 30,
        },
        BoostSupply { boosts_left: 3 },
        Position2d(Vec2 { x: 0, y: 0 }),
        Velocity2d(Vec2 { x: 0, y: 0 }),
    ));
}

/// Marker component that prevents an entity from leaving the arena.
#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Default, Component)]
pub(crate) enum PlayerState {
    #[default]
    Idle,
    Swimming,
    Boosting,
    Dead,
}

#[derive(Debug, Component)]
pub struct StarveTimer {
    ticks_left: u32,
}

#[derive(Debug, Component)]
pub struct BoostSupply {
    boosts_left: u32,
}
