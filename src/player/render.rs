use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_prototype_lyon::prelude::*;
use std::collections::HashMap;

use super::attributes::{BoostSupply, HungerCountdown, Player};
use super::states::{PlayerState, PlayerStates};
use crate::shared::{
    animation::{Animation, AnimationFrame, AnimationState},
    game::{GameOver, GameRestarted},
    render::RenderLayer,
};

#[derive(Resource)]
pub(super) struct PlayerStateAnimations {
    pub map: HashMap<PlayerStates, Animation>,
}

impl FromWorld for PlayerStateAnimations {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        // let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();

        let swim_1_handle = asset_server.load("sprites/player/fish1.png");
        let swim_2_handle = asset_server.load("sprites/player/fish2.png");

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
                    .set_duration(Duration::from_secs_f32(next_animation.frames[0].time));
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
pub(super) struct BoostTracker {
    index: u8,
}

#[derive(Component)]
pub(super) struct BoostTrackerBorder;

pub(super) fn spawn_player_boost_trackers(
    commands: &mut Commands,
    player_width: f32,
    player_height: f32,
    max_boosts: u8,
    player_entity: Entity,
) {
    // let tracker_mesh = Mesh2dHandle(meshes.add(Circle { radius: 3.0 }));
    let tracker_color = Color::PINK;

    // let tracker_border_mesh = Mesh2dHandle(meshes.add(Circle { radius: 5.0 }));
    let tracker_border_color = Color::rgb_u8(255, 105, 180);

    debug!("Adding boost trackers for player {:?}...", player_entity);

    // Trackers should extend 1.5 times the player's width
    let extended_width = player_width * 1.5;
    // Trackers should be 1 player height heigher than the player.
    let tracker_height = player_height;

    let mut tracker_positions: Vec<Vec2> = Vec::new();

    for i in 0..max_boosts {
        // Calculate the positions of each of the boost trackers relative to the player given the
        // total number of boosts.
        // Taken from https://bevyengine.org/examples/2D%20Rendering/2d-shapes/. I didn't come up
        // with this math on my own lmao.
        let x_offset = -extended_width / 2. + i as f32 / (max_boosts - 1) as f32 * extended_width;
        tracker_positions.push(Vec2::new(x_offset, tracker_height));
    }

    let mut boost_trackers: Vec<Entity> = Vec::new();

    for (i, tracker_position) in tracker_positions.into_iter().enumerate() {
        let tracker_border_shape = GeometryBuilder::build_as(&shapes::Circle {
            radius: 5.0,
            center: Vec2::ZERO,
        });

        let tracker_border_entity = commands
            .spawn((
                ShapeBundle {
                    path: tracker_border_shape,
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(tracker_position.x, tracker_position.y, 5.0),
                        ..default()
                    },
                    ..default()
                },
                Stroke::color(tracker_border_color),
                BoostTrackerBorder,
                RenderLayer::Player,
            ))
            .id();

        let tracker_shape = GeometryBuilder::build_as(&shapes::Circle {
            radius: 4.0,
            center: Vec2::ZERO,
        });

        let tracker_entity = commands
            .spawn((
                ShapeBundle {
                    path: tracker_shape,
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(tracker_position.x, tracker_position.y, 5.0),
                        ..default()
                    },
                    ..default()
                },
                Fill::color(tracker_color),
                BoostTracker { index: i as u8 },
                RenderLayer::Player,
            ))
            .id();

        boost_trackers.extend_from_slice(&[tracker_border_entity, tracker_entity]);
    }

    commands
        .entity(player_entity)
        .push_children(boost_trackers.as_slice());

    debug!(
        "Added boost tracker entities that follow player: {:?}",
        boost_trackers
    );
}

pub(super) fn update_tracker_display_from_boost_supply(
    player_query: Query<&BoostSupply, With<Player>>,
    mut tracker_query: Query<(&mut Visibility, &BoostTracker)>,
) {
    for boost_supply in player_query.iter() {
        let boosts_left = boost_supply.count;

        for (mut tracker_vis, tracker) in tracker_query.iter_mut() {
            if boosts_left > tracker.index {
                *tracker_vis = Visibility::Visible;
            } else {
                *tracker_vis = Visibility::Hidden;
            }
        }
    }
}

pub(super) fn despawn_trackers_on_gameover_or_restart(
    mut commands: Commands,
    mut game_over_reader: EventReader<GameOver>,
    mut game_restarted_reader: EventReader<GameRestarted>,
    boost_tracker_query: Query<Entity, Or<(With<BoostTracker>, With<BoostTrackerBorder>)>>,
) {
    if game_over_reader.read().next().is_some() {
        for boost_tracker in boost_tracker_query.iter() {
            commands.entity(boost_tracker).despawn_recursive();
        }
    }

    if game_restarted_reader.read().next().is_some() {
        for boost_tracker in boost_tracker_query.iter() {
            commands.entity(boost_tracker).despawn_recursive();
        }
    }
}

#[derive(Component)]
pub(super) struct PlayerCountdownText;

pub(super) fn add_countdown_text(mut commands: Commands, player_entity: Entity) {
    commands.entity(player_entity).with_children(|builder| {
        builder.spawn((
            Text2dBundle {
                transform: Transform::from_xyz(0., 40., 1.),
                text: Text::from_section(
                    "30.0".to_string(),
                    TextStyle {
                        // TODO: Change from default font
                        font_size: 30.0,
                        ..Default::default()
                    },
                )
                .with_justify(JustifyText::Center),
                ..default()
            },
            PlayerCountdownText,
        ));
    });
}

pub(super) fn update_coundown_text_system(
    mut text_query: Query<&mut Text, With<PlayerCountdownText>>,
    player_query: Query<&HungerCountdown, With<Player>>,
) {
    for mut text in text_query.iter_mut() {
        for hunger_countdown in player_query.iter() {
            text.sections[0].value = format!("{:.1}", hunger_countdown.time_left);
            if hunger_countdown.time_left < 5.0 {
                text.sections[0].style.color = Color::RED;
            } else {
                text.sections[0].style.color = Color::PINK;
            }
        }
    }
}

pub(super) fn hide_countdown_on_game_over(
    mut game_over_reader: EventReader<GameOver>,
    mut countdown_text_query: Query<&mut Visibility, With<PlayerCountdownText>>,
) {
    if game_over_reader.read().next().is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            *countdown_text_visiblity = Visibility::Hidden;
        }
    }
}

pub(super) fn show_countdown_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    mut countdown_text_query: Query<&mut Visibility, With<PlayerCountdownText>>,
) {
    if restart_reader.read().next().is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            *countdown_text_visiblity = Visibility::Visible;
        }
    }
}
