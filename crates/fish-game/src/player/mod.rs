use bevy::prelude::*;
use fish_game_core::player::PlayerState as CorePlayerState;

use crate::core_adapter::{sync_player_transform, CoreState, PlayerMarker};
use crate::shared::{
    animation::AnimationState,
    game::GameRestarted,
    render::{FontHandles, RenderLayer},
    stages,
};

pub(crate) mod events;
mod render;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        debug!("Building player plugin...");
        app.init_resource::<render::PlayerStateAnimations>()
            .init_resource::<render::BoostTrackerAssets>()
            .add_message::<events::PlayerHooked>()
            .add_message::<events::PlayerStarved>()
            .add_message::<events::PlayerBonked>()
            .add_message::<events::PlayerAte>()
            .add_message::<events::PlayerBoosted>()
            .add_systems(Startup, init_player)
            .add_systems(
                Update,
                (
                    reset_player,
                    render::despawn_trackers_on_gameover_or_restart,
                    render::show_countdown_on_restart,
                    render::hide_countdown_on_game_over,
                )
                    .in_set(stages::HandleEventsSet),
            )
            .add_systems(
                Update,
                (sync_player_transform,).in_set(stages::FinalizeMovementSet),
            )
            .add_systems(
                Update,
                (
                    render::player_state_animation_change_system,
                    render::update_tracker_display_from_boost_supply,
                    render::update_coundown_text_system,
                )
                    .in_set(stages::PrepareRenderSet),
            );
    }
}

fn init_player(
    mut commands: Commands,
    core: Res<CoreState>,
    fonts: Res<FontHandles>,
    player_state_animations: Res<render::PlayerStateAnimations>,
    tracker_assets: Res<render::BoostTrackerAssets>,
) {
    let player_entity = spawn_player_entity(&mut commands, &core, &player_state_animations);
    render::spawn_player_boost_trackers(
        &mut commands,
        &tracker_assets,
        core.state.config.player.width,
        core.state.config.player.height,
        core.state.config.player.max_boosts,
        player_entity,
    );
    render::add_countdown_text(commands, fonts, player_entity);
}

fn reset_player(
    mut commands: Commands,
    core: Res<CoreState>,
    fonts: Res<FontHandles>,
    player_state_animations: Res<render::PlayerStateAnimations>,
    tracker_assets: Res<render::BoostTrackerAssets>,
    mut restart_reader: MessageReader<GameRestarted>,
    player_query: Query<Entity, With<PlayerMarker>>,
) {
    if restart_reader.read().next().is_some() {
        for player_entity in player_query.iter() {
            commands.entity(player_entity).despawn();
        }

        let new_player = spawn_player_entity(&mut commands, &core, &player_state_animations);
        render::spawn_player_boost_trackers(
            &mut commands,
            &tracker_assets,
            core.state.config.player.width,
            core.state.config.player.height,
            core.state.config.player.max_boosts,
            new_player,
        );
        render::add_countdown_text(commands, fonts, new_player);
    }
}

fn spawn_player_entity(
    commands: &mut Commands,
    core: &CoreState,
    player_state_animations: &render::PlayerStateAnimations,
) -> Entity {
    let player_animation = player_state_animations
        .map
        .get(&CorePlayerState::Idle)
        .unwrap();
    let first_animation_frame = player_animation.frames[0].clone();

    let width = core.state.config.player.width;
    let height = core.state.config.player.height;

    commands
        .spawn((
            PlayerMarker,
            RenderLayer::Player,
            Sprite {
                image: first_animation_frame.material_handle.clone(),
                custom_size: Some(Vec2::new(width, height)),
                ..Default::default()
            },
            AnimationState {
                animation: player_animation.clone(),
                timer: Timer::from_seconds(first_animation_frame.time, TimerMode::Once),
                frame_index: 0,
                speed_multiplier: 1.0,
            },
        ))
        .id()
}
