use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlasBuilder};
use std::collections::HashMap;

use super::attributes::Player;
use super::states::{PlayerState, PlayerStates};

/**
Represents one frame of animation. The atlas index references the TextureAtlas
handle on the entity.
*/
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AnimationFrame {
    atlas_index: usize,
    time: f32,
}

/// Represents an entire animation
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    should_loop: bool,
    frames: Vec<AnimationFrame>,
}

/// Component that represents the current state of animation
#[derive(Debug, Clone)]
pub struct AnimationState {
    animation: Animation,
    timer: Timer,
    frame_index: u32,
    speed_multiplier: f32,
}

/// creates the player state -> animation mapping
fn create_player_animations(
    texture_atlas: &TextureAtlas,
    asset_server: Res<AssetServer>,
) -> HashMap<PlayerStates, Animation> {
    let swim_1_handle = asset_server.get_handle("sprites/player/fish1.png");
    let swim_2_handle = asset_server.get_handle("sprites/player/fish2.png");

    [
        (
            PlayerStates::Idle,
            Animation {
                should_loop: false,
                frames: vec![AnimationFrame {
                    atlas_index: texture_atlas.get_texture_index(&swim_1_handle).unwrap(),
                    time: 999.9,
                }],
            },
        ),
        (
            PlayerStates::Swim,
            Animation {
                should_loop: false,
                frames: vec![
                    AnimationFrame {
                        atlas_index: texture_atlas.get_texture_index(&swim_1_handle).unwrap(),
                        time: 0.3,
                    },
                    AnimationFrame {
                        atlas_index: texture_atlas.get_texture_index(&swim_2_handle).unwrap(),
                        time: 0.3,
                    },
                ],
            },
        ),
        (
            PlayerStates::Boost,
            Animation {
                should_loop: false,
                frames: vec![
                    AnimationFrame {
                        atlas_index: texture_atlas.get_texture_index(&swim_1_handle).unwrap(),
                        time: 0.1,
                    },
                    AnimationFrame {
                        atlas_index: texture_atlas.get_texture_index(&swim_2_handle).unwrap(),
                        time: 0.1,
                    },
                ],
            },
        ),
    ]
    .iter()
    .cloned()
    .collect()
}

#[derive(Default)]
pub struct PlayerSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

pub(super) fn start_atlas_load(
    mut player_sprite_handles: ResMut<PlayerSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    println!("Asynchronously loading player sprites...");
    player_sprite_handles.handles = asset_server.load_folder("sprites/player").unwrap();
}

/**
Adds the player sprite to a player without a sprite as soon as the textures
load.
*/
pub(super) fn load_player_atlas(
    mut commands: Commands,
    mut player_sprite_handles: ResMut<PlayerSpriteHandles>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<Without<TextureAtlasSprite, (&Player, &PlayerState, Entity)>>,
) {
    if player_sprite_handles.atlas_loaded {
        return;
    }

    let mut texture_atlas_builder = TextureAtlasBuilder::default();

    if let LoadState::Loaded = asset_server
        .get_group_load_state(player_sprite_handles.handles.iter().map(|handle| handle.id))
    {
        println!("Loaded player sprite textures.");

        for handle in player_sprite_handles.handles.iter() {
            let texture = textures.get(handle).unwrap();
            texture_atlas_builder.add_texture(handle.clone_weak().typed::<Texture>(), &texture);
        }

        let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();

        let player_animations = create_player_animations(&texture_atlas, asset_server);

        println!("Player state animations map: {:?}", player_animations);

        let texture_atlas_texture = texture_atlas.texture.clone();

        let atlas_handle = texture_atlases.add(texture_atlas);

        // adds the sprite sheet component and animation state component to the player entities
        for (_, player_state, entity) in query.iter() {
            let player_animation = player_animations.get(&PlayerStates::Swim).unwrap();
            let first_animation_frame = player_animation.frames[0];

            println!("Adding sprite sheet components to entity: {:?}", entity);
            commands.insert(
                entity,
                SpriteSheetComponents {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    sprite: TextureAtlasSprite::new(first_animation_frame.atlas_index as u32),
                    texture_atlas: atlas_handle.clone(),
                    ..Default::default()
                },
            );
            commands.insert_one(
                entity,
                AnimationState {
                    animation: player_animation.clone(),
                    timer: Timer::from_seconds(first_animation_frame.time, false),
                    frame_index: 0,
                    speed_multiplier: 1.0,
                },
            );
        }

        commands.spawn(SpriteComponents {
            material: materials.add(texture_atlas_texture.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        });

        commands.insert_resource(player_animations.clone());
        player_sprite_handles.atlas_loaded = true;
    }
}

pub(super) fn animation_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut TextureAtlasSprite)>,
) {
    for (mut animation_state, mut texture_atlas_sprite) in query.iter_mut() {
        animation_state.timer.tick(time.delta_seconds);

        if animation_state.timer.finished {
            animation_state.frame_index =
                (animation_state.frame_index + 1) % animation_state.animation.frames.len() as u32;
            animation_state.timer = Timer::from_seconds(
                animation_state.animation.frames[animation_state.frame_index as usize].time,
                false,
            );
            texture_atlas_sprite.index = animation_state.frame_index;
        }
    }
}
