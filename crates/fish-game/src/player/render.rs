use bevy::color::palettes::css::{HOT_PINK, PINK, RED};
use bevy::prelude::*;
use fish_game_core::player::PlayerState as CorePlayerState;
use std::collections::HashMap;
use std::time::Duration;

use crate::core_adapter::{CoreState, PlayerMarker};
use crate::shared::{
    animation::{Animation, AnimationFrame, AnimationState},
    game::{GameOver, GameRestarted},
    render::FontHandles,
};

/// Cached meshes / materials for the boost-tracker dots (filled circle + ring
/// outline). Built once on startup so spawn sites stay terse.
#[derive(Resource)]
pub(super) struct BoostTrackerAssets {
    pub fill_mesh: Handle<Mesh>,
    pub border_mesh: Handle<Mesh>,
    pub fill_material: Handle<ColorMaterial>,
    pub border_material: Handle<ColorMaterial>,
}

impl FromWorld for BoostTrackerAssets {
    fn from_world(world: &mut World) -> Self {
        let fill_mesh = world.resource_mut::<Assets<Mesh>>().add(Circle::new(4.0));
        let border_mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Annulus::new(4.5, 5.0));
        let fill_material = world
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from(Color::from(PINK)));
        let border_material = world
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from(Color::from(HOT_PINK)));
        Self {
            fill_mesh,
            border_mesh,
            fill_material,
            border_material,
        }
    }
}

#[derive(Resource)]
pub(super) struct PlayerStateAnimations {
    pub map: HashMap<CorePlayerState, Animation>,
}

impl FromWorld for PlayerStateAnimations {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        let swim_1_handle = asset_server.load("sprites/player/fish1.png");
        let swim_2_handle = asset_server.load("sprites/player/fish2.png");

        PlayerStateAnimations {
            map: [
                (
                    CorePlayerState::Idle,
                    Animation {
                        should_loop: true,
                        frames: vec![AnimationFrame {
                            material_handle: swim_1_handle.clone(),
                            time: 999.9,
                        }],
                    },
                ),
                (
                    CorePlayerState::Swim,
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
                    CorePlayerState::Boost,
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

/// Swap the player's animation when its core state transitions. State lives
/// in the core — we track the last observed state in a `Local`.
pub(super) fn player_state_animation_change_system(
    core: Res<CoreState>,
    player_state_animations: Res<PlayerStateAnimations>,
    mut last_state: Local<Option<CorePlayerState>>,
    mut query: Query<&mut AnimationState, With<PlayerMarker>>,
) {
    let Ok(mut animation_state) = query.single_mut() else {
        return;
    };
    let current = core.state.player.state;

    if *last_state == Some(current) {
        return;
    }

    if let Some(next_animation) = player_state_animations.map.get(&current) {
        animation_state
            .timer
            .set_duration(Duration::from_secs_f32(next_animation.frames[0].time));
        animation_state.timer.reset();
        animation_state.animation = next_animation.clone();
        animation_state.frame_index = 0;
    }

    *last_state = Some(current);
}

#[derive(Component)]
pub(super) struct BoostTracker {
    index: u8,
}

#[derive(Component)]
pub(super) struct BoostTrackerBorder;

pub(super) fn spawn_player_boost_trackers(
    commands: &mut Commands,
    assets: &BoostTrackerAssets,
    player_width: f32,
    player_height: f32,
    max_boosts: u8,
    player_entity: Entity,
) {
    debug!("Adding boost trackers for player {:?}...", player_entity);

    let extended_width = player_width * 1.5;
    let tracker_height = player_height;

    let mut tracker_positions: Vec<Vec2> = Vec::new();

    for i in 0..max_boosts {
        let x_offset = -extended_width / 2. + i as f32 / (max_boosts - 1) as f32 * extended_width;
        tracker_positions.push(Vec2::new(x_offset, tracker_height));
    }

    let mut boost_trackers: Vec<Entity> = Vec::new();

    for (i, tracker_position) in tracker_positions.into_iter().enumerate() {
        let transform = Transform::from_xyz(tracker_position.x, tracker_position.y, 1.0);

        let tracker_border_entity = commands
            .spawn((
                Mesh2d(assets.border_mesh.clone()),
                MeshMaterial2d(assets.border_material.clone()),
                transform,
                BoostTrackerBorder,
            ))
            .id();

        let tracker_entity = commands
            .spawn((
                Mesh2d(assets.fill_mesh.clone()),
                MeshMaterial2d(assets.fill_material.clone()),
                transform,
                BoostTracker { index: i as u8 },
            ))
            .id();

        boost_trackers.extend_from_slice(&[tracker_border_entity, tracker_entity]);
    }

    commands
        .entity(player_entity)
        .add_children(boost_trackers.as_slice());
}

pub(super) fn update_tracker_display_from_boost_supply(
    core: Res<CoreState>,
    mut tracker_query: Query<(&mut Visibility, &BoostTracker)>,
) {
    let boosts_left = core.state.player.boosts_remaining;
    for (mut tracker_vis, tracker) in tracker_query.iter_mut() {
        if boosts_left > tracker.index {
            *tracker_vis = Visibility::Visible;
        } else {
            *tracker_vis = Visibility::Hidden;
        }
    }
}

pub(super) fn despawn_trackers_on_gameover_or_restart(
    mut commands: Commands,
    mut game_over_reader: MessageReader<GameOver>,
    mut game_restarted_reader: MessageReader<GameRestarted>,
    boost_tracker_query: Query<Entity, Or<(With<BoostTracker>, With<BoostTrackerBorder>)>>,
) {
    if game_over_reader.read().next().is_some() || game_restarted_reader.read().next().is_some() {
        for boost_tracker in boost_tracker_query.iter() {
            commands.entity(boost_tracker).despawn();
        }
    }
}

#[derive(Component)]
pub(super) struct PlayerCountdownText;

pub(super) fn add_countdown_text(
    mut commands: Commands,
    fonts: Res<FontHandles>,
    player_entity: Entity,
) {
    commands.entity(player_entity).with_children(|builder| {
        builder.spawn((
            Text2d::new("30.0"),
            TextFont {
                font: fonts.main_font.clone(),
                font_size: 70.0,
                ..Default::default()
            },
            TextColor(Color::from(PINK)),
            TextLayout::new_with_justify(Justify::Center),
            Transform {
                translation: Vec3::new(0., 50., 1.),
                scale: Vec3::ONE * 0.25,
                ..default()
            },
            Visibility::Visible,
            PlayerCountdownText,
        ));
    });
}

pub(super) fn update_coundown_text_system(
    core: Res<CoreState>,
    mut text_query: Query<(&mut Text2d, &mut TextColor), With<PlayerCountdownText>>,
) {
    let tick_rate = core.state.config.tick_rate as f32;
    let seconds_left = core.state.player.hunger_ticks_remaining as f32 / tick_rate;
    for (mut text, mut color) in text_query.iter_mut() {
        **text = format!("{:.1}", seconds_left);
        color.0 = if seconds_left < 5.0 {
            Color::from(RED)
        } else {
            Color::from(PINK)
        };
    }
}

pub(super) fn hide_countdown_on_game_over(
    mut game_over_reader: MessageReader<GameOver>,
    mut countdown_text_query: Query<&mut Visibility, With<PlayerCountdownText>>,
) {
    if game_over_reader.read().next().is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            *countdown_text_visiblity = Visibility::Hidden;
        }
    }
}

pub(super) fn show_countdown_on_restart(
    mut restart_reader: MessageReader<GameRestarted>,
    mut countdown_text_query: Query<&mut Visibility, With<PlayerCountdownText>>,
) {
    if restart_reader.read().next().is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            *countdown_text_visiblity = Visibility::Visible;
        }
    }
}
