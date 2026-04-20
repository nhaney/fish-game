//! Determinism tests for `fish-game-core`, gated by the replay crate.
//!
//! These are the "the same `(config, inputs)` produces the same hash" tests
//! that protect cross-target replay verification. They live here, not in
//! `fish-game-core`, because they exercise the recording / verification API
//! and `bincode` serialization — both of which are this crate's job.
//!
//! On CI, this file runs as a separate target (`tests/determinism.rs`) so a
//! failure clearly points at "the determinism contract regressed", not at
//! "some unrelated unit test".

use fish_game_core::{FishGameConfig, FishGameInput, FishGameState};
use fish_game_replay::{record, verify, Replay};

fn seeded_config(seed_byte: u8) -> FishGameConfig {
    let mut seed = [0u8; 32];
    for (i, s) in seed.iter_mut().enumerate() {
        *s = seed_byte.wrapping_add(i as u8);
    }
    FishGameConfig::default().with_seed(seed)
}

/// A reproducible (no `thread_rng`) input sequence for stress tests. We don't
/// pull in `rand` here — instead we walk a small lookup table keyed on the
/// step index. The exact values don't matter; what matters is the sequence is
/// the same on every machine that runs this test.
fn deterministic_inputs(n: usize) -> Vec<FishGameInput> {
    (0..n)
        .map(|i| {
            let bits = i as u32;
            FishGameInput {
                move_left: bits & 0b0001 != 0,
                move_right: bits & 0b0010 != 0,
                move_up: bits & 0b0100 != 0,
                move_down: bits & 0b1000 != 0,
                boost_pressed: bits & 0b1_0000 != 0,
                // Spike a "just pressed" every 17 ticks so the boost path runs.
                boost_just_pressed: i % 17 == 0,
            }
        })
        .collect()
}

#[test]
fn same_seed_same_inputs_same_hash() {
    let cfg = seeded_config(7);
    let mut a = FishGameState::new(cfg.clone());
    let mut b = FishGameState::new(cfg);

    for input in deterministic_inputs(2_000) {
        a.tick(input);
        b.tick(input);
    }

    assert_eq!(
        a.hash(),
        b.hash(),
        "two states with the same seed + inputs must produce the same hash",
    );
    assert_eq!(a.score.count, b.score.count);
}

#[test]
fn different_seed_diverges() {
    let mut a = FishGameState::new(seeded_config(1));
    let mut b = FishGameState::new(seeded_config(2));

    // Idle inputs still diverge because boat spawns are rng-driven.
    for _ in 0..600 {
        a.tick(FishGameInput::default());
        b.tick(FishGameInput::default());
    }
    assert_ne!(a.hash(), b.hash(), "different seeds must diverge");
}

#[test]
fn replay_roundtrip_bincode_then_verify() {
    let cfg = seeded_config(42);
    let inputs = deterministic_inputs(500);

    let recorded = record(cfg, inputs);
    let bytes = recorded.encode().expect("encode");
    let decoded = Replay::decode(&bytes).expect("decode");

    assert_eq!(recorded.final_hash, decoded.final_hash);
    assert_eq!(recorded.final_score, decoded.final_score);
    assert_eq!(recorded.inputs.len(), decoded.inputs.len());

    let result = verify(&decoded);
    assert!(result.hash_matches, "hash: {} vs {}", result.actual_hash, decoded.final_hash);
    assert!(result.score_matches, "score: {} vs {}", result.actual_score, decoded.final_score);
}

#[test]
fn verify_detects_tampered_input_log() {
    let cfg = seeded_config(11);
    let inputs = deterministic_inputs(300);
    let mut recorded = record(cfg, inputs);

    // Flip one input bit ⇒ the simulation's path diverges ⇒ hash mismatch.
    let last = recorded.inputs.len() - 1;
    recorded.inputs[last].move_left = !recorded.inputs[last].move_left;

    let result = verify(&recorded);
    assert!(
        !result.hash_matches,
        "verifier must catch a tampered input log",
    );
}

#[test]
fn verify_detects_tampered_expected_hash() {
    let cfg = seeded_config(13);
    let inputs = deterministic_inputs(200);
    let mut recorded = record(cfg, inputs);

    recorded.final_hash = recorded.final_hash.wrapping_add(1);

    let result = verify(&recorded);
    assert!(!result.hash_matches);
    assert!(result.score_matches, "score should still match — only hash was tampered");
}
