//! Events emitted by the simulation during a single `tick`.
//!
//! Core emits a typed, ordered stream of `CoreEvent`s whenever a transition
//! worth reacting to happens (player hooked, worm eaten, boat spawned, …).
//! Presentation iterates [`FishGameState::events`] and forwards each event to
//! its Bevy equivalent — it never re-derives transitions from state diffs.
//!
//! The event stream is ephemeral: it lives on [`FishGameState`] but is
//! cleared at the top of every [`FishGameState::tick`] call, and is
//! `#[serde(skip)]` so replays never store it. Events do NOT contribute to
//! [`FishGameState::hash`] — the hash is of the simulation state, and events
//! are a faithful description of how we got there.
//!
//! [`FishGameState`]: crate::state::FishGameState
//! [`FishGameState::tick`]: crate::state::FishGameState::tick
//! [`FishGameState::hash`]: crate::state::FishGameState::hash

use crate::boat::{BoatId, HookId, LineId, WormId};
use crate::state::GameOverCause;

/// A single transition observed during a tick. Ordered within a tick by
/// emission time (i.e. the order the core tick pipeline produced them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreEvent {
    // --- Player transitions ---------------------------------------------
    /// Player successfully consumed a boost charge this tick.
    PlayerBoosted,
    /// Player ate a worm. The worm's `WormDespawned` is emitted immediately
    /// after so presentation can clean up its sprite.
    PlayerAte { worm: WormId },
    /// Player collided with a hook; game ends as Hooked.
    PlayerHooked { hook: HookId, boat: Option<BoatId> },
    /// Player collided with a boat; game ends as Bonked.
    PlayerBonked { boat: BoatId },
    /// Hunger ran out; game ends as Starved.
    PlayerStarved,
    /// Umbrella "game ended" transition. Emitted exactly once, alongside the
    /// specific cause event above so UI listeners that only care about "is
    /// the run over" have a single subscription.
    GameOver { cause: GameOverCause },

    // --- Scoring / pacing -----------------------------------------------
    /// The 1-second score tick fired; `new_score` is the post-increment value.
    ScoreIncremented { new_score: u32 },
    /// Difficulty multiplier advanced.
    DifficultyIncreased { new_multiplier: u8 },

    // --- Entity lifecycle ------------------------------------------------
    //
    // Emitted so the Bevy adapter can mirror core slotmaps into ECS
    // entities without having to diff slotmap membership.

    BoatSpawned(BoatId),
    BoatDespawned(BoatId),
    HookSpawned(HookId),
    HookDespawned(HookId),
    LineSpawned(LineId),
    LineDespawned(LineId),
    WormSpawned(WormId),
    WormDespawned(WormId),
}
