//! Replay recording + cross-target verification for `fish-game-core`.
//!
//! A [`Replay`] is the `(config, inputs, expected_hash, expected_score)` tuple
//! plus a recording-machine target hint. [`verify`] re-runs the inputs on the
//! local target and reports whether the resulting hash and score match — this
//! is what the leaderboard's score-verification path calls.
//!
//! This crate is intentionally tiny: it owns nothing the simulation does, just
//! `bincode` serialization and the verification loop. The determinism
//! invariants this crate gates ("same seed + same inputs ⇒ same hash on every
//! target") live as tests in this crate's `tests/` directory.

use serde::{Deserialize, Serialize};

use fish_game_core::{FishGameConfig, FishGameInput, FishGameState};

/// A recorded simulation. Contains everything needed to re-run the same game
/// on another target and confirm the same final state hash + score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub config: FishGameConfig,
    pub inputs: Vec<FishGameInput>,
    pub final_hash: u64,
    pub final_score: u32,
    /// Best-effort target hint of the recording machine; useful when a
    /// mismatch occurs (tells you which side diverged).
    pub target_triple: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifyResult {
    pub hash_matches: bool,
    pub score_matches: bool,
    pub actual_hash: u64,
    pub actual_score: u32,
}

impl Replay {
    pub fn encode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

/// Replay a recorded session on this target and report hash/score parity.
pub fn verify(replay: &Replay) -> VerifyResult {
    let mut state = FishGameState::new(replay.config.clone());
    for input in &replay.inputs {
        state.tick(*input);
    }
    let actual_hash = state.hash();
    let actual_score = state.score.count;
    VerifyResult {
        hash_matches: actual_hash == replay.final_hash,
        score_matches: actual_score == replay.final_score,
        actual_hash,
        actual_score,
    }
}

/// Convenience: run a sim with the given inputs and build a `Replay` from the
/// final state. Used for tests and as the recording path in the Bevy adapter.
pub fn record(config: FishGameConfig, inputs: Vec<FishGameInput>) -> Replay {
    let mut state = FishGameState::new(config.clone());
    for input in &inputs {
        state.tick(*input);
    }
    Replay {
        config,
        inputs,
        final_hash: state.hash(),
        final_score: state.score.count,
        target_triple: current_target_hint(),
    }
}

fn current_target_hint() -> String {
    // rustc doesn't expose `target_triple` at runtime in a stable form; compose
    // a coarse hint from `std::env::consts`. Good enough to tell native from
    // wasm in a mismatch report.
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}
