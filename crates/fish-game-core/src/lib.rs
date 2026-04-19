//! Deterministic simulation core for Stay Off the Line! Remastered.
//!
//! This crate holds the entire game simulation with **no Bevy dependency**.
//! The flow is:
//!
//! 1. The Bevy presentation layer constructs a [`FishGameState`] from a
//!    [`FishGameConfig`].
//! 2. On each fixed-update tick it translates keyboard state into a
//!    [`FishGameInput`] and calls [`FishGameState::tick`].
//! 3. `tick` mutates the state in place and returns `&FishGameState`. The
//!    presentation layer reads the reference, diffs it against its previous
//!    snapshot, and drives SFX/UI/animations from the observable transitions.
//! 4. For replay verification, record the `(config, inputs)` pair and hash
//!    the final state — cross-target determinism is a hard invariant of this
//!    crate. See [`replay`].
//!
//! Core forbids:
//! - `bevy`, `web-sys`, filesystem / network access
//! - `thread_rng` / wall-clock entropy
//! - `HashMap` iteration on hot paths (use `SlotMap`/`BTreeMap`)
//! - `f32::sin`/`cos`/`sqrt`/`Vec3::normalize` (route through [`math`])

pub mod boat;
pub mod collision;
pub mod config;
pub mod input;
pub mod math;
pub mod player;
pub mod replay;
pub mod rng;
pub mod state;

pub use config::{ArenaConfig, FishGameConfig, PlayerStatsConfig};
pub use input::FishGameInput;
pub use replay::{record, verify, Replay, VerifyResult};
pub use state::{FishGameState, GameOverCause, GamePhase};

#[cfg(test)]
mod tests;
