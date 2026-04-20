//! Deterministic simulation core for Stay Off the Line! Remastered.
//!
//! This crate is the simulation kernel: no Bevy, no rendering, no audio, no
//! pause/restart lifecycle, no replay I/O. The presentation layer constructs a
//! [`FishGameState`] from a [`FishGameConfig`], translates per-tick input into
//! a [`FishGameInput`], and observes the returned `&FishGameState`.
//!
//! Cross-target determinism is the central invariant. The same
//! `(FishGameConfig, sequence-of-FishGameInput)` MUST produce the same
//! [`FishGameState::hash`] on native x86-64, ARM, and `wasm32`. To preserve
//! that, core forbids:
//! - `bevy`, `web-sys`, filesystem / network access
//! - `thread_rng` / wall-clock entropy (rng comes from `config.seed`)
//! - `HashMap` iteration on hot paths (use `SlotMap`/`BTreeMap`)
//! - `f32::sin` / `cos` / `sqrt` / `Vec3::normalize` (route through [`math`]
//!   and `glam`'s `libm` feature)
//!
//! Replay recording + verification lives in the separate `fish-game-replay`
//! crate so core has zero serialization concerns beyond the `Serialize` /
//! `Deserialize` derives on its public types.

pub mod boat;
pub mod collision;
pub mod config;
pub mod input;
pub mod math;
pub mod player;
pub mod rng;
pub mod state;

pub use config::{ArenaConfig, FishGameConfig, PlayerStatsConfig};
pub use input::FishGameInput;
pub use state::{FishGameState, GameOverCause, GamePhase};

#[cfg(test)]
mod tests;
