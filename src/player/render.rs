use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;

use super::attributes::BoostSupply;
use super::states::{PlayerState, PlayerStates};
use crate::shared::{
    animation::{Animation, AnimationFrame, AnimationState},
    game::{GameOver, GameRestarted},
    movement::Follow,
    render::RenderLayer,
};

#[derive(Resource)]
pub(super) struct PlayerStateAnimations {
    pub map: HashMap<PlayerStates, Animation>,
}

impl FromWorld for PlayerStateAnimations {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();

        let swim_1_handle = materials.add(asset_server.load("sprites/player/fish1.png").into());
        let swim_2_handle = materials.add(asset_server.load("sprites/player/fish2.png").into());

        PlayerStateAnimations {
            map: [
                (
                    PlayerStates::Idle,
                    Animation {
                        should_loop: true,
                        frames: vec![AnimationFrame {
                            material_handle: swim_1_handle.clone(),
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
                                material_handle: swim_1_handle.clone(),
                                time: 0.2,
                            },
                            AnimationFrame {
                                material_handle: swim_2_handle.clone(),
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
                                material_handle: swim_1_handle,
                                time: 0.1,
                            },
                            AnimationFrame {
                                material_handle: swim_2_handle,
                                time: 0.1,
                            },
                        ],
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        }
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
                // debug!(
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
#[derive(Component)]
pub(super) struct BoostTracker;

pub(super) fn spawn_player_boost_trackers(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    player_width: f32,
    player_height: f32,
    // TODO: Use this instead of hard coding 3 boosts
    _max_boosts: u8,
    player_entity: Entity,
) {
    // calculate the positions of each of the boost trackers relative to the player
    // given the player's size and number of boosts
    let pink_color = materials.add(Color::PINK.into());
    let hot_pink_color = materials.add(Color::rgb_u8(255, 105, 180).into());

    debug!("Adding boost trackers for player {:?}...", player_entity);

    let extended_width = player_width;
    let tracker_height = player_height;

    // let mut tracker_positions: Vec<Vec3> = Vec::new();

    // for i in 0..max_boosts {
    //     let x_offset = (i as f32 * (extended_width / max_boosts as f32)) - (extended_width / 2.0);
    //     tracker_positions.push(Vec3::new(x_offset, tracker_height, 1.0));
    // }
    // TODO: Make this more generic to handle more
    let tracker_positions = vec![
        Vec3::new(-extended_width / 2.0, tracker_height, 0.0),
        Vec3::new(0.0, tracker_height, 0.0),
        Vec3::new(extended_width / 2.0, tracker_height, 0.0),
    ];

    let mut boost_trackers: Vec<Entity> = Vec::new();

    for tracker_position in tracker_positions {
        // make the border and the tracker child components of the player
        let tracker_shape = shapes::Circle {
            radius: 4.0,
            center: Vec2::ZERO,
        };

        let tracker_border_shape = shapes::Circle {
            radius: 5.0,
            center: Vec2::ZERO,
        };

        let tracker = commands
            .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&tracker_shape),
                    ..default()
                },
                Fill::color(pink_color.clone()),
                BoostTracker,
                RenderLayer::Player,
                Follow {
                    entity_to_follow: player_entity,
                    offset: tracker_position,
                    follow_global_transform: false,
                },
            ))
            .id();

        let tracker_border = commands
            .spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&tracker_border_shape),
                    ..default()
                },
                Stroke::new(hot_pink_color.clone(), 2.0),
                RenderLayer::Player,
            ))
            .id();

        commands.entity(tracker).push_children(&[tracker_border]);
        boost_trackers.push(tracker);
    }

    debug!("Added boost tracker entities: {:?}", boost_trackers);

    commands.entity(player_entity).insert(BoostTrackerDisplay {
        trackers: boost_trackers,
    });
}

#[derive(Debug, Component)]
pub(super) struct BoostTrackerDisplay {
    trackers: Vec<Entity>,
}

// TODO: Update to also handle changes to max boosts here, make this use "changed" instead of updating every
// frame
pub(super) fn update_tracker_display_from_boost_supply(
    player_query: Query<(&BoostSupply, &BoostTrackerDisplay)>,
    mut tracker_query: Query<&mut Visibility, With<BoostTracker>>,
) {
    for (boost_supply, boost_tracker_display) in player_query.iter() {
        for i in 0..boost_supply.max_boosts {
            let tracker_entity = boost_tracker_display.trackers[i as usize];
            let mut tracker_visibility = tracker_query.get_mut(tracker_entity).unwrap();

            if (i + 1) <= boost_supply.count {
                tracker_visibility = Visibility::Hidden;
            } else {
                tracker_visibility = Visibility::Visible;
            }
        }
    }
}

pub(super) fn despawn_trackers_on_gameover_or_restart(
    mut commands: Commands,
    mut game_over_reader: EventReader<GameOver>,
    mut game_restarted_reader: EventReader<GameRestarted>,
    boost_tracker_query: Query<Entity, With<BoostTracker>>,
    player_query: Query<Entity, With<BoostTrackerDisplay>>,
) {
    if game_over_reader.read().next() {
        for boost_tracker in boost_tracker_query.iter() {
            commands.entity(boost_tracker).despawn_recursive();
        }

        for player_entity in player_query.iter() {
            commands
                .entity(player_entity)
                .remove::<BoostTrackerDisplay>();
        }
    }

    if game_restarted_reader.read().next().is_some() {
        for boost_tracker in boost_tracker_query.iter() {
            commands.entity(boost_tracker).despawn_recursive();
        }

        for player_entity in player_query.iter() {
            commands
                .entity(player_entity)
                .remove::<BoostTrackerDisplay>();
        }
    }
}
