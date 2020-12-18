use serde::{Deserialize, Serialize};

#[cfg(feature = "native")]
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
    pub scores: Vec<u32>,
    // in case the filename is changed or something
    #[serde(skip)]
    lookup: String,
}

impl LocalScores {
    #[cfg(feature = "native")]
    pub fn new(key: &str) -> Self {
        let filename = key.to_owned() + ".json";
        debug!("Filename: {:?}", filename);

        let mut contents = String::new();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename.clone())
            .unwrap();

        file.read_to_string(&mut contents).unwrap();

        Self::load_scores_from_json(&contents, &filename)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(key: &str) -> Self {
        let window = web_sys::window().unwrap();

        if let Ok(Some(local_storage)) = window.local_storage() {
            if let Ok(Some(value)) = local_storage.get_item(key) {
                Self::load_scores_from_json(&value, key)
            } else {
                debug!("Key {:?} not found in local storage", key);
                Self::load_scores_from_json("", key)
            }
        } else {
            panic!("Could not get local storage")
        }
    }

    fn load_scores_from_json(scores_json: &str, lookup: &str) -> Self {
        if let Ok(loaded_scores) = serde_json::from_str::<LocalScores>(&scores_json) {
            debug!("Found existing scores in file");
            let mut existing_scores = loaded_scores.scores;
            existing_scores.sort();
            existing_scores.reverse();

            Self {
                scores: existing_scores,
                lookup: lookup.to_string(),
            }
        } else {
            debug!(
                "Could not load json {:?} with valid existing scores, creating a new one",
                scores_json
            );
            Self {
                scores: Vec::<u32>::new(),
                lookup: lookup.to_string(),
            }
        }
    }

    #[cfg(feature = "native")]
    fn save_scores(&mut self) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.lookup.clone())
            .unwrap();

        let serialized_scores = serde_json::to_string_pretty::<Self>(&self).unwrap();
        debug!("Writing {:?} to file {:?}", serialized_scores, self.lookup);
        file.write_all(serialized_scores.as_bytes()).unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    fn save_scores(&mut self) {
        let window = web_sys::window().unwrap();

        if let Ok(Some(local_storage)) = window.local_storage() {
            let serialized_scores = serde_json::to_string_pretty::<Self>(&self).unwrap();
            if let Ok(_) = local_storage.set_item(&self.lookup, &serialized_scores) {
                debug!("Updated scores in local storage to {:?}", serialized_scores);
            } else {
                panic!("Could not save in local storage")
            }
        } else {
            panic!("Could not get local storage")
        }
    }

    pub fn high_score(&self) -> Option<u32> {
        if self.scores.len() == 0 {
            None
        } else {
            Some(self.scores[0])
        }
    }

    pub fn add_new_score(&mut self, score: u32, score_saved_events: &mut Events<ScoreSaved>) {
        self.scores.push(score);
        self.scores.sort();
        self.scores.reverse();
        self.save_scores();

        score_saved_events.send(ScoreSaved {
            score,
            score_index: self.scores.iter().position(|&r| r == score).unwrap(),
        });
    }
}

impl Default for LocalScores {
    fn default() -> Self {
        Self::new("scores")
    }
}

#[derive(Debug)]
pub struct ScoreSaved {
    score: u32,
    score_index: usize,
}

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<LocalScores>()
            .add_system_to_stage(stages::HANDLE_EVENTS, update_local_scores_system.system())
            .add_event::<ScoreSaved>();
    }
}

pub fn update_local_scores_system(
    score: Res<Score>,
    mut game_over_reader: Local<EventReader<GameOver>>,
    game_over_events: Res<Events<GameOver>>,
    mut local_scores: ResMut<LocalScores>,
    mut score_saved_events: ResMut<Events<ScoreSaved>>,
) {
    if let Some(_game_over_event) = game_over_reader.earliest(&game_over_events) {
        debug!(
            "Saving score new score ({:?}) to file {:?}",
            score.count, local_scores.lookup
        );
        local_scores.add_new_score(score.count, &mut score_saved_events);
    }
}
