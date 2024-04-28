use bevy::prelude::*;

use crate::shared::game::{GamePaused, GameRestarted, GameState, GameStates, GameUnpaused};

#[derive(Debug, Clone, Resource)]
pub(super) struct PauseButtonMaterials {
    pause: Handle<ColorMaterial>,
    play: Handle<ColorMaterial>,
}

#[derive(Debug, Component)]
pub(super) struct PauseButton {
    is_paused: bool,
}

impl FromWorld for PauseButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        debug!("Loading pause button materials...");
        PauseButtonMaterials {
            pause: asset_server.load("sprites/ui/pause.png"),
            play: asset_server.load("sprites/ui/play.png"),
        }
    }
}

pub(super) fn pause_button_system(
    game_state: Res<GameState>,
    pause_button_materials: Res<PauseButtonMaterials>,
    mut game_paused_events: ResMut<Events<GamePaused>>,
    mut game_unpaused_events: ResMut<Events<GameUnpaused>>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &mut PauseButton),
        Changed<Interaction>,
    >,
) {
    if let GameStates::GameOver = game_state.cur_state {
        return;
    }

    for (interaction, mut material, mut pause_button) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            if pause_button.is_paused {
                *material = pause_button_materials.pause.clone();
                game_unpaused_events.send(GameUnpaused);
            } else {
                *material = pause_button_materials.play.clone();
                game_paused_events.send(GamePaused);
            }

            pause_button.is_paused = !pause_button.is_paused;
        }
    }
}

pub(super) fn add_pause_button(
    commands: &mut Commands,
    pause_button_materials: &PauseButtonMaterials,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let pause_button = commands
        .spawn(NodeBundle {
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(64.0), Val::Px(64.0)),
                        align_self: AlignSelf::FlexEnd,
                        margin: Rect {
                            top: Val::Percent(5.0),
                            right: Val::Percent(5.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    material: pause_button_materials.pause.clone(),
                    ..Default::default()
                })
                .with(PauseButton { is_paused: false });
        })
        .current_entity()
        .unwrap();

    pause_button
}

pub(super) fn reset_pause_button_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    pause_button_materials: Res<PauseButtonMaterials>,
    mut pause_button_query: Query<(&mut Handle<ColorMaterial>, &mut PauseButton)>,
) {
    if restart_reader.read().next().is_some() {
        for (mut material, mut pause_button) in pause_button_query.iter_mut() {
            *material = pause_button_materials.pause.clone();
            pause_button.is_paused = false;
        }
    }
}
