use bevy::prelude::*;

use crate::player::events::{PlayerAte, PlayerBonked, PlayerBoosted, PlayerHooked, PlayerStarved};

pub(super) struct SfxHandles {
    bonked: Handle<AudioSource>,
    hooked: Handle<AudioSource>,
    starved: Handle<AudioSource>,
    eat: Handle<AudioSource>,
    boost: Handle<AudioSource>,
}

impl FromResources for SfxHandles {
    fn from_resources(resources: &Resources) -> Self {
        let asset_server = resources.get::<AssetServer>().unwrap();
        Self {
            bonked: asset_server.load("audio/sfx/bonked.ogg"),
            hooked: asset_server.load("audio/sfx/hooked.ogg"),
            starved: asset_server.load("audio/sfx/starved.ogg"),
            eat: asset_server.load("audio/sfx/eat.ogg"),
            boost: asset_server.load("audio/sfx/boost.ogg"),
        }
    }
}

// TODO: See if this can be reduced
#[allow(clippy::too_many_arguments)]
pub(super) fn play_sfx_system(
    mut player_hooked_reader: Local<EventReader<PlayerHooked>>,
    player_hooked_events: Res<Events<PlayerHooked>>,
    mut player_starved_reader: Local<EventReader<PlayerStarved>>,
    player_starved_events: Res<Events<PlayerStarved>>,
    mut player_bonked_reader: Local<EventReader<PlayerBonked>>,
    player_bonked_events: Res<Events<PlayerBonked>>,
    mut player_ate_reader: Local<EventReader<PlayerAte>>,
    player_ate_events: Res<Events<PlayerAte>>,
    mut player_boosted_reader: Local<EventReader<PlayerBoosted>>,
    player_boosted_events: Res<Events<PlayerBoosted>>,
    sfx_handles: Res<SfxHandles>,
    audio: Res<Audio>,
) {
    for _ in player_hooked_reader.iter(&player_hooked_events) {
        debug!("Playing hooked sound effect");
        audio.play(sfx_handles.hooked.clone());
    }

    for _ in player_starved_reader.iter(&player_starved_events) {
        debug!("Playing starved sound effect");
        audio.play(sfx_handles.starved.clone());
    }

    for _ in player_bonked_reader.iter(&player_bonked_events) {
        debug!("Playing bonked sound effect");
        audio.play(sfx_handles.bonked.clone());
    }

    for _ in player_ate_reader.iter(&player_ate_events) {
        debug!("Playing ate sound effect");
        audio.play(sfx_handles.eat.clone());
    }

    for _ in player_boosted_reader.iter(&player_boosted_events) {
        debug!("Playing boosted sound effect");
        audio.play(sfx_handles.boost.clone());
    }
}
