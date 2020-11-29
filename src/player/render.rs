use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlasBuilder};
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;

use super::attributes::{BoostSupply, Player};
use super::states::{PlayerState, PlayerStates};
use crate::shared::{
    animation::{Animation, AnimationFrame, AnimationState},
    collision::Collider,
    render::NonRotatingChild,
};

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
                        time: 0.2,
                    },
                    AnimationFrame {
                        atlas_index: texture_atlas.get_texture_index(&swim_2_handle).unwrap(),
                        time: 0.2,
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
    pub atlas_loaded: bool,
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
    commands: &mut Commands,
    mut player_sprite_handles: ResMut<PlayerSpriteHandles>,
    mut player_state_animations: ResMut<PlayerStateAnimations>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    query: Query<
        (&PlayerState, &Collider, &BoostSupply, Entity),
        (With<Player>, Without<TextureAtlasSprite>),
    >,
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
        for (player_state, collider, boost_supply, entity) in query.iter() {
            let player_animation = player_state_animations
                .map
                .get(&player_state.current_state)
                .unwrap();
            let first_animation_frame = player_animation.frames[0];

            println!("Adding sprite sheet components to entity: {:?}", entity);
            commands.insert(
                entity,
                SpriteSheetBundle {
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
                // println!(
                //     "State change detected in animation system from {:?} to {:?}",
                //     prev_player_state, cur_player_state
                // );

                let next_animation = player_state_animations.map.get(&cur_player_state).unwrap();

                animation_state
                    .timer
                    .set_duration(next_animation.frames[0].time);
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

/**
    Design:
    * Boost supply tracker component manages the visibility (Draw.is_visible)
      of the three (or more) sprite entities (can these all be on the same entity? no.)
      that represent the number of boosts available.
*/
pub(super) struct BoostTracker;

pub(super) fn spawn_player_boost_trackers(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    player_width: f32,
    player_height: f32,
    max_boosts: u8,
    player_entity: Entity,
) {
    // calculate the positions of each of the boost trackers relative to the player
    // given the player's size and number of boosts
    let pink_color = materials.add(Color::PINK.into());
    let hot_pink_color = materials.add(Color::rgb_u8(255, 105, 180).into());

    println!("Adding boost trackers for player {:?}...", player_entity);

    let extended_width = player_width;
    let tracker_height = player_height;

    let mut tracker_positions: Vec<Vec3> = Vec::new();

    for i in 0..max_boosts {
        let x_offset = (i as f32 * (extended_width / max_boosts as f32)) - (extended_width / 2.0);
        tracker_positions.push(Vec3::new(x_offset, tracker_height, 1.0));
    }

    let mut boost_trackers: Vec<Entity> = Vec::new();

    for tracker_position in tracker_positions {
        // make the border and the tracker child components of the player
        let tracker = commands
            .spawn(primitive(
                pink_color.clone(),
                &mut meshes,
                ShapeType::Circle(5.0),
                TessellationMode::Fill(&FillOptions::default()),
                tracker_position,
            ))
            .with(BoostTracker)
            .with(NonRotatingChild)
            .current_entity()
            .unwrap();

        let tracker_border = commands
            .spawn(primitive(
                hot_pink_color.clone(),
                &mut meshes,
                ShapeType::Circle(6.0),
                TessellationMode::Stroke(&StrokeOptions::default()),
                tracker_position,
            ))
            .with(NonRotatingChild)
            .current_entity()
            .unwrap();

        commands.push_children(player_entity, &[tracker, tracker_border]);
        boost_trackers.push(tracker);
    }

    println!("Added boost tracker entities: {:?}", boost_trackers);

    commands.insert_one(
        player_entity,
        BoostTrackerDisplay {
            trackers: boost_trackers,
        },
    );
}

pub(super) struct BoostTrackerDisplay {
    trackers: Vec<Entity>,
}

// TODO: Update to also handle changes to max boosts here, make this use "changed" instead of updating every
// frame
pub(super) fn update_tracker_display_from_boost_supply(
    player_query: Query<(&BoostSupply, &BoostTrackerDisplay)>,
    mut tracker_query: Query<(&mut Draw, &GlobalTransform), With<BoostTracker>>,
) {
    for (boost_supply, boost_tracker_display) in player_query.iter() {
        for i in 0..boost_supply.max_boosts {
            let tracker_entity = boost_tracker_display.trackers[i as usize];
            let (mut tracker_draw, transform) = tracker_query.get_mut(tracker_entity).unwrap();

            if (i + 1) <= boost_supply.count {
                tracker_draw.is_visible = true;
            } else {
                tracker_draw.is_visible = false;
            }
        }
    }
}
