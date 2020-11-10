use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlasBuilder};
use std::collections::HashMap;

use super::attributes::Player;
use super::states::{PlayerState, PlayerStates};
use crate::shared::animation::{Animation, AnimationFrame, AnimationState};

#[derive(Default)]
pub(super) struct PlayerStateAnimations {
    map: HashMap<PlayerStates, Animation>,
}

/// creates the player state -> animation mapping to store in the PlayerStateAnimations resource
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
                should_loop: true,
                frames: vec![AnimationFrame {
                    atlas_index: texture_atlas.get_texture_index(&swim_1_handle).unwrap(),
                    time: 999.9,
                }],
            },
        ),
        (
            PlayerStates::Swim,
            Animation {
                should_loop: true,
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
                should_loop: true,
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
    mut player_state_animations: ResMut<PlayerStateAnimations>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
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

        player_state_animations.map = create_player_animations(&texture_atlas, asset_server);

        let atlas_handle = texture_atlases.add(texture_atlas);

        // adds the sprite sheet component and animation state component to the player entities
        for (_, player_state, entity) in query.iter() {
            let player_animation = player_state_animations
                .map
                .get(&player_state.current_state)
                .unwrap();
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

        player_sprite_handles.atlas_loaded = true;
    }
}

/// TODO: State changes might work better as events in the future
pub(super) fn player_state_animation_change_system(
    player_state_animations: Res<PlayerStateAnimations>,
    mut last_entity_states: Local<HashMap<Entity, PlayerStates>>,
    mut query: Query<(&mut AnimationState, &PlayerState, Entity)>,
) {
    for (mut animation_state, player_state, entity) in query.iter_mut() {
        // On the first iteration per entity, its map entry will be empty
        let cur_player_state = &player_state.current_state;
        if let Some(prev_player_state) = last_entity_states.get(&entity) {
            if cur_player_state != prev_player_state {
                println!(
                    "State change detected in animation system from {:?} to {:?}",
                    prev_player_state, cur_player_state
                );

                let next_animation = player_state_animations.map.get(&cur_player_state).unwrap();

                animation_state.timer.duration = next_animation.frames[0].time;
                animation_state.timer.reset();
                // TODO: Find out how to remove this clone
                animation_state.animation = next_animation.clone();
                animation_state.frame_index = 0;

                *last_entity_states.get_mut(&entity).unwrap() = *cur_player_state;
            }
        } else {
            last_entity_states.insert(entity, *cur_player_state);
        }
    }
}
