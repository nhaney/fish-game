use bevy::prelude::*;

use crate::shared::game::{GamePaused, GameRestarted, GameState, GameStates, GameUnpaused};

#[derive(Debug, Clone)]
pub(super) struct PauseButtonMaterials {
    pause: Handle<ColorMaterial>,
    play: Handle<ColorMaterial>,
}

pub(super) struct PauseButton {
    is_paused: bool,
}

impl FromResources for PauseButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();

        debug!("Loading pause button materials...");
        PauseButtonMaterials {
            pause: materials.add(asset_server.load("sprites/ui/pause.png").into()),
            play: materials.add(asset_server.load("sprites/ui/play.png").into()),
            // pause: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            // play: materials.add(Color::rgb(0.15, 0.55, 0.0).into()),
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
        Mutated<Interaction>,
    >,
) {
    if let GameStates::GameOver = game_state.cur_state {
        return;
    }

    for (interaction, mut material, mut pause_button) in interaction_query.iter_mut() {
        if let Interaction::Clicked = *interaction {
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
            style: Style {
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            visible: Visible {
                is_visible: false,
                ..Default::default()
            },
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
    restart_events: Res<Events<GameRestarted>>,
    mut restart_reader: Local<EventReader<GameRestarted>>,
    pause_button_materials: Res<PauseButtonMaterials>,
    mut pause_button_query: Query<(&mut Handle<ColorMaterial>, &mut PauseButton)>,
) {
    if restart_reader.earliest(&restart_events).is_some() {
        for (mut material, mut pause_button) in pause_button_query.iter_mut() {
            *material = pause_button_materials.pause.clone();
            pause_button.is_paused = false;
        }
    }
}
