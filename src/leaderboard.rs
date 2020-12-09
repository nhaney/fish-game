use serde::{Deserialize, Serialize};

use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use bevy::prelude::*;

use crate::shared::{
    game::{GameOver, Score},
    stages,
};

#[derive(Deserialize, Serialize)]
pub struct LocalScores {
    scores: Vec<u32>,
    // in case the filename is changed or something
    #[serde(skip_serializing)]
    lookup: String,
}

impl LocalScores {
    pub fn new(key: &str) -> Self {
        let filename = key.to_owned() + ".json";
        println!("Filename: {:?}", filename);

        let mut contents = String::new();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename.clone())
            .unwrap();

        file.read_to_string(&mut contents).unwrap();

        if let Ok(loaded_scores) = serde_json::from_str::<LocalScores>(&contents) {
            println!("Found existing scores in file");
            let mut existing_scores = loaded_scores.scores;
            existing_scores.sort();
            existing_scores.reverse();

            Self {
                scores: existing_scores,
                lookup: filename.to_string(),
            }
        } else {
            println!(
                "Could not load file {:?} with valid existing scores, creating a new one",
                filename
            );
            Self {
                scores: Vec::<u32>::new(),
                lookup: filename.to_string(),
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(key: &str) -> Self {
        let window = web_sys::window().unwrap();

        if let Ok(Some(local_storage)) = window.local_storage() {
            if let Ok(Some(value)) = local_storage.get_item(&self.name) {
            } else {
            }
        }
    }

    fn save_scores(&mut self) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.lookup.clone())
            .unwrap();

        let serialized_scores = serde_json::to_string(self).unwrap();
        println!("Writing {:?} to file {:?}", serialized_scores, self.lookup);
        file.write_all(serialized_scores.as_bytes()).unwrap();
    }

    pub fn get_top_ten_local_scores(&mut self) -> &[u32] {
        &self.scores.as_slice()[0..11]
    }

    pub fn add_new_score(&mut self, score: u32) {
        self.scores.push(score);
        self.scores.sort();
        self.scores.reverse();
        self.save_scores();
    }
}

impl Default for LocalScores {
    fn default() -> Self {
        Self::new("scores")
    }
}

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<LocalScores>()
            .add_system_to_stage(stages::HANDLE_EVENTS, update_local_scores_system);
    }
}

pub fn update_local_scores_system(
    score: Res<Score>,
    mut game_over_reader: Local<EventReader<GameOver>>,
    game_over_events: Res<Events<GameOver>>,
    mut local_scores: ResMut<LocalScores>,
) {
    if let Some(_game_over_event) = game_over_reader.earliest(&game_over_events) {
        println!(
            "Saving score new score ({:?}) to file {:?}",
            score.count, local_scores.lookup
        );
        local_scores.add_new_score(score.count);
    }
}
