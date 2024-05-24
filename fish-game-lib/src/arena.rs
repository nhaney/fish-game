use bevy::prelude::*;

pub const DEFAULT_ARENA_WIDTH: i32 = 640;
pub const DEFAULT_ARENA_HEIGHT: i32 = 360;

/// Resource that represents the area that the player is allowed to move around in.
#[derive(Debug, Resource)]
pub struct Arena {
    pub width: i32,
    pub height: i32,
}

pub(crate) fn initialize_arena(mut commands: Commands) {
    // initializes arena resource and its sprite representation
    let arena = Arena {
        width: DEFAULT_ARENA_WIDTH,
        height: DEFAULT_ARENA_HEIGHT,
    };

    commands.insert_resource(arena);
}

/// Component that prevents an entity from leaving the arena.
#[derive(Debug, Component)]
pub(crate) struct ArenaBound;
