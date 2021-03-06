use bevy::prelude::*;

use crate::shared::render::RenderLayer;

pub const DEFAULT_ARENA_WIDTH: f32 = 640.0;
pub const DEFAULT_ARENA_HEIGHT: f32 = 360.0;
pub const DEFAULT_ARENA_OFFSET: f32 = -50.0;

// TODO: Make this API cleaner
pub struct Arena {
    pub width: f32,
    pub height: f32,
    pub offset: f32,
}

pub fn initialize_arena(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // initializes arena resource and its sprite representation
    let arena = Arena {
        width: DEFAULT_ARENA_WIDTH,
        height: DEFAULT_ARENA_HEIGHT,
        offset: DEFAULT_ARENA_OFFSET,
    };

    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb_u8(173, 216, 230).into()),
            sprite: Sprite {
                size: Vec2::new(arena.width, arena.height),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, arena.offset, 0.0)),
            ..Default::default()
        })
        .with(RenderLayer::Background);

    commands.insert_resource(arena);
}
