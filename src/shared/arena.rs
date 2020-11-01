use bevy::prelude::*;

const DEFAULT_ARENA_WIDTH: f32 = 800.0;
const DEFAULT_ARENA_HEIGHT: f32 = 600.0;
const DEFAULT_ARENA_OFFSET: f32 = -50.0;

// TODO: Make this API cleaner
pub struct Arena {
    pub width: f32,
    pub height: f32,
    pub offset: f32,
}

pub fn initialize_arena(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // initializes arena resource and its sprite representation
    let arena = Arena {
        width: DEFAULT_ARENA_WIDTH,
        height: DEFAULT_ARENA_HEIGHT,
        offset: DEFAULT_ARENA_OFFSET,
    };

    commands.spawn(SpriteComponents {
        material: materials.add(Color::rgb_u8(173, 216, 230).into()),
        transform: Transform::from_translation(Vec3::new(0.0, arena.offset, 0.0)),
        sprite: Sprite {
            size: Vec2::new(arena.width, arena.height),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(arena);
}