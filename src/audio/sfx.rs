use bevy::prelude::*;

use crate::player::events::{PlayerAte, PlayerBonked, PlayerBoosted, PlayerHooked, PlayerStarved};

pub(super) struct SfxFiles {
    bonked: String,
    hooked: String,
    starved: String,
    ate: String,
    boost: String,
}

impl FromResources for SfxFiles {
    fn from_resources(resources: &Resources) -> Self {
        // let asset_server = resources.get::<AssetServer>().unwrap();
        // let mut sfx = resources.get_mut::<Assets<AudioSource>>().unwrap();
        Self {
            bonked: "audio/sfx/bonked.ogg".to_string(),
            hooked: "audio/sfx/hooked.ogg".to_string(),
            starved: "audio/sfx/starved.ogg".to_string(),
            ate: "audio/sfx/ate.ogg".to_string(),
            boost: "audio/sfx/boost.ogg".to_string(),
        }
    }
}

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
    sound_files: Res<SfxFiles>,
    asset_server: Res<AssetServer>,
    mut audio: Res<Audio>,
) {
    for _ in player_hooked_reader.iter(&player_hooked_events) {
        println!("Playing hooked sound effect");
        let sfx = asset_server.load("audio/sfx/hooked.ogg");
        audio.play(sfx);
    }

    for _ in player_starved_reader.iter(&player_starved_events) {
        println!("Playing starved sound effect");
        let sfx = asset_server.load("audio/sfx/starved.ogg");
        audio.play(sfx);
    }

    for _ in player_bonked_reader.iter(&player_bonked_events) {
        println!("Playing bonked sound effect");
        let sfx = asset_server.load("audio/sfx/bonked.ogg");
        audio.play(sfx);
    }

    for _ in player_ate_reader.iter(&player_ate_events) {
        println!("Playing ate sound effect");
        let sfx = asset_server.load("audio/sfx/eat.ogg");
        audio.play(sfx);
    }

    for _ in player_boosted_reader.iter(&player_boosted_events) {
        println!("Playing boosted sound effect");
        let sfx = asset_server.load("audio/sfx/boost.ogg");
        audio.play(sfx);
    }
}
