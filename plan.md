# Plan: Move game-over logic to the presentation layer

## Context

Today the core sim continues to run after `GamePhase::GameOver`: `state.tick`
dispatches to `tick_game_over` (`crates/fish-game-core/src/state.rs:109-180`),
which in turn drives `boat::step_boats`, `boat::step_reeling_hooks`, and
`boat::despawn_offscreen_boats`. That post-death wind-down — non-winning boats
reversing off-arena at 2× speed, the winning boat dragging the hook back to its
rod tip at 300 units/sec, and boats despawning once fully off-screen — is
purely a visual flourish. It has no effect on score, no replayable input, and
is not what determinism testing is protecting. Keeping it in core bloats the
kernel's surface, ties the simulation to a notion of "after death" it doesn't
need, and forces the adapter to keep syncing positions from core every tick
even when nothing gameplay-relevant is happening.

This plan moves all of that to the Bevy presentation layer. Once the core
declares `GamePhase::GameOver` (winner / cause already attached to
`CoreEvent::GameOver`), the core stops ticking entirely — the adapter takes
over driving boat motion and reel-in animation in Bevy. The reel math itself
is extracted into a small standalone module (`crates/fish-game-core/src/reel.rs`)
so a future feature — moving / reeling hooks **during** gameplay — can pull
the same primitive back in without re-deriving it.

The trigger for this work was the user request: *"Can you now move the game
over logic (reeling the fish, boats leaving, etc.) to strictly the
presentation layer? Keep the reeling logic (maybe separate) because we may
want that as a gameplay element in the future."*

## Decisions confirmed up front

1. **Core `tick()` becomes a no-op in `GamePhase::GameOver`.** No more
   `tick_game_over`, no more wind-down. The state freezes at the moment of
   game over.
2. **Reel math survives, but as a pure module.** New `reel.rs` with
   `ReelMotion::toward(from, to, speed)` and `advance(pos, motion, dt) ->
   (Vec3, bool /* arrived */)`. Zero Bevy, zero state — easy to call from
   the future "moving hook" feature without resurrecting `tick_game_over`.
3. **Replays are unaffected by visual wind-down.** Replays already only
   record `(config, inputs)` and rebuild events by re-ticking. With the
   wind-down gone, replay verification stops at the same `GameOver` tick
   it always did — but expected `final_hash` values change because
   post-death boat/hook positions no longer mutate state. We'll re-record
   the determinism fixtures.

## Code changes by category

### 1. Core: shrink the simulation kernel

`crates/fish-game-core/src/state.rs`

- `tick(input)`: dispatch on phase
  - `GamePhase::Running` → `tick_running` (unchanged).
  - `GamePhase::GameOver` → clear `events`, return `&self`. Do not advance
    score, hunger, boats, or hooks. Do not emit any new events.
- Delete `tick_game_over` (currently `state.rs:~140-180`).
- `enter_game_over` keeps its existing job: stamp `phase = GameOver`,
  populate `game_over_boat` / `game_over_cause`, despawn worms, freeze the
  player. Remove the call to `boat::trigger_boat_exit` (it now lives in
  the adapter).

`crates/fish-game-core/src/boat.rs`

- Delete `step_boats`, `step_reeling_hooks`, `despawn_offscreen_boats`,
  `trigger_boat_exit`, `start_reel_in`, `emit_despawned_boat`, and the
  `DespawnedBoat` struct. (All only called from `tick_game_over` /
  `check_collisions`'s game-over branch.)
- `check_collisions` (the call site that currently invokes
  `boat::start_reel_in` at `state.rs:~398`): on hook collision, just emit
  `CoreEvent::PlayerHooked { boat: <id> }` and call `enter_game_over`.
  Drop the reel-in setup — that's presentation now.
- Delete the `Hook::reel_velocity` and `Hook::reel_destination` fields.
- Delete the `Boat::exiting` and `Boat::winner` fields.

`crates/fish-game-core/src/events.rs`

- Delete `CoreEvent::BoatDespawned`, `CoreEvent::HookDespawned`, and
  `CoreEvent::LineDespawned`. The adapter despawns Bevy entities from its
  own observation of game-over now. (Worm despawn events stay — they
  fire during normal Running play, not just at game-over.)

`crates/fish-game-core/src/reel.rs` *(new)*

```rust
//! Pure reel-in motion. No Bevy, no state, no time source — caller owns
//! all of that. Used by the presentation-layer game-over wind-down today,
//! and reusable for future mid-game moving-hook features.

use crate::math::{vec3_length, vec3_normalize_or_zero};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReelMotion {
    pub destination: Vec3,
    pub velocity: Vec3,
    pub arrival_radius: f32,
}

impl ReelMotion {
    pub fn toward(from: Vec3, to: Vec3, speed: f32) -> Self {
        let dir = vec3_normalize_or_zero(to - from);
        Self {
            destination: to,
            velocity: dir * speed,
            arrival_radius: 10.0,
        }
    }
}

/// Advance `pos` toward `motion.destination` by `motion.velocity * dt`.
/// Returns `(new_pos, arrived)`. If arrived, the position is snapped to the
/// destination and the caller should drop / freeze the motion.
pub fn advance(pos: Vec3, motion: ReelMotion, dt: f32) -> (Vec3, bool) {
    let next = pos + motion.velocity * dt;
    let to_dest = motion.destination - next;
    if vec3_length(to_dest) <= motion.arrival_radius {
        (motion.destination, true)
    } else {
        (next, false)
    }
}
```

`crates/fish-game-core/src/lib.rs`

- `pub mod reel;` so the adapter can import it.

`crates/fish-game-core/src/tests.rs`

- Delete tests for `tick_game_over` / `step_reeling_hooks` /
  `despawn_offscreen_boats` (the "boats leave after death", "hook reels
  in after death" cases).
- Add `reel.rs` unit tests covering: zero-distance is immediate arrival,
  midpoint advance respects dt, arrival_radius snaps to destination,
  zero-vector source → zero velocity (no NaN).

### 2. Adapter: own the wind-down in Bevy

`crates/fish-game/src/core_adapter/mod.rs`

- `forward_core_events`: when handling `CoreEvent::GameOver`, fill
  `GameOver.winning_boat` from `core.state.game_over_boat` (currently
  hardcoded `None` at `:~255`).
- `sync_ecs_from_core`: early-return when `core.state.phase ==
  GamePhase::GameOver`. After game over, Bevy `Transform`s are owned by
  the new presentation systems, not derived from core slotmaps.
- The line-rendering block currently inside `sync_ecs_from_core` (the
  one that rebuilds line-segment Transforms from `line.start_pos` /
  `line.end_pos` each frame) moves out into a separate system —
  `sync_line_transforms_from_endpoints` — that reads the *Bevy
  Transform* of the line's parent boat and its hooked target. See §3.

`crates/fish-game/src/core_adapter/game_over.rs` *(new)*

This module owns the wind-down. Plugin wires its systems into the existing
SystemSets (`MovementSet` for motion, `PrepareRenderSet` for despawn).

```rust
use bevy::prelude::*;
use fish_game_core::reel::{advance as reel_advance, ReelMotion};
use crate::core_adapter::{CoreEntityMap, CoreState};
use crate::shared::game::{GameOver, GameRestarted};
use fish_game_core::GamePhase;

const BOAT_EXIT_SPEED_MULT: f32 = 2.0;
const REEL_SPEED: f32 = 300.0;

#[derive(Component)]
pub struct BoatVelocity(pub Vec3);          // for non-winning boats sliding off

#[derive(Component)]
pub struct ReelingHook {
    pub motion: ReelMotion,
    pub line_entity: Entity,                // line whose end_pos follows hook
    pub boat_entity: Entity,                // boat whose Transform anchors the line start
}

#[derive(Component)]
pub struct BoatChildren {
    pub hooks: Vec<Entity>,
    pub worms: Vec<Entity>,
    pub line: Option<Entity>,
}

/// One-shot system that runs the frame `GameOver` is emitted: snapshots
/// each boat's current core velocity into a Bevy `BoatVelocity`, flips
/// non-winners to 2x reverse, zeros the winner's, and converts the
/// winner's hook into a `ReelingHook` aimed at the line's start_pos
/// (== rod tip at the moment of death).
pub fn snapshot_game_over_state(
    mut commands: Commands,
    mut reader: MessageReader<GameOver>,
    core: Res<CoreState>,
    map: Res<CoreEntityMap>,
    transforms: Query<&Transform>,
) { /* ... */ }

/// Translate non-winning boats (and their child hooks/lines/worms) by
/// `velocity * dt` each tick. Skips entities without `BoatVelocity`.
pub fn drift_boats_off_arena(
    time: Res<Time>,
    mut q: Query<(&BoatVelocity, &mut Transform)>,
) { /* ... */ }

/// Advances `ReelingHook` entities via `reel::advance`. On arrival,
/// removes the component and freezes the hook + line at the rod tip.
pub fn advance_reeling_hooks(
    time: Res<Time>,
    mut commands: Commands,
    mut hooks: Query<(Entity, &mut ReelingHook, &mut Transform)>,
) { /* ... */ }

/// Despawns boats (and their `BoatChildren`) once their Transform leaves
/// the arena bounds. Mirrors core's old `despawn_offscreen_boats`.
pub fn despawn_offscreen_boats_bevy(
    mut commands: Commands,
    boats: Query<(Entity, &Transform, &BoatChildren), With<BoatVelocity>>,
    arena: Res<crate::shared::arena::ArenaBounds>,
) { /* ... */ }
```

The plugin (`CorePlugin::build`):

- `add_systems(Update, snapshot_game_over_state.in_set(HandleEventsSet))`
- `add_systems(Update, (drift_boats_off_arena, advance_reeling_hooks)
  .in_set(MovementSet))`
- `add_systems(Update, despawn_offscreen_boats_bevy.in_set(PrepareRenderSet))`
- On `GameRestarted`, components are torn down by the existing entity
  cleanup (which despawns everything keyed by `CoreEntityMap`); no
  extra reset needed.

### 3. Line rendering: derive from Bevy Transforms

`crates/fish-game/src/core_adapter/mod.rs` (line-rendering split)

- New component `LineEndpoints { start: Entity, end: Entity }` on each
  fishing-line entity, populated when the line is first spawned (during
  `sync_ecs_from_core`'s Running-phase pass).
- New system `sync_line_transforms_from_endpoints` (runs in
  `PrepareRenderSet`) that, for each `LineEndpoints`, reads the Bevy
  `Transform` of `start` (boat) and `end` (hook), and rebuilds the line
  segment Transform exactly the way the current inline block at
  `core_adapter/mod.rs:~421-446` does — but reading from Bevy
  Transforms, not core `line.start_pos` / `line.end_pos`. This means
  the line "just works" both during Running (Transforms come from core
  sync) and during GameOver (Transforms come from `BoatVelocity` /
  `ReelingHook`).
- Delete the inline line-rendering block in `sync_ecs_from_core`. The
  line's *spawn* still happens in `sync_ecs_from_core` (when the core
  first creates a line), but the per-frame Transform update moves to
  the new system.

### 4. Cleanup of now-dead test fixtures + replay hashes

`crates/fish-game-replay/tests/determinism.rs`

- Re-record the `expected_hash` constant(s). With `tick()` no-oping in
  GameOver, the hash at `tick_count` past death no longer drifts from
  the hash at the moment of death. Run the determinism test once with
  the new core, copy the new hash into the test, confirm cross-target
  stability per the existing pattern.

`crates/fish-game-replay/src/lib.rs`

- No API change. Replays still serialize `(config, inputs, final_hash,
  final_score, target_triple)`. The values change; the schema doesn't.

## Files to modify / create

**Modify:**

- `crates/fish-game-core/src/state.rs` — `tick` dispatcher; delete `tick_game_over`; trim `enter_game_over`.
- `crates/fish-game-core/src/boat.rs` — delete six wind-down functions + struct fields.
- `crates/fish-game-core/src/events.rs` — delete three despawn variants.
- `crates/fish-game-core/src/lib.rs` — add `pub mod reel;`.
- `crates/fish-game-core/src/tests.rs` — drop wind-down tests; add reel tests.
- `crates/fish-game/src/core_adapter/mod.rs` — populate `GameOver.winning_boat`; early-return `sync_ecs_from_core` in GameOver; pull line-render block into a new system; register the new `game_over` module.
- `crates/fish-game-replay/tests/determinism.rs` — refresh expected hash.

**Create:**

- `crates/fish-game-core/src/reel.rs` — pure reel motion math.
- `crates/fish-game/src/core_adapter/game_over.rs` — Bevy wind-down systems + components.

## Reused patterns

- `core::math::{vec3_length, vec3_normalize_or_zero}` for the reel
  primitive — same routing rule as the rest of core (no direct
  `Vec3::normalize` / `f32::sqrt`).
- The adapter's existing `CoreEntityMap` translation + marker components
  (`BoatMarker`, `HookMarker`, `LineMarker`) stay as they are. New
  components (`BoatVelocity`, `ReelingHook`, `BoatChildren`,
  `LineEndpoints`) sit alongside them.
- SystemSets (`HandleEventsSet`, `MovementSet`, `PrepareRenderSet`) — no
  new sets, just slot the new systems into the existing ones.
- `enter_game_over` already does the worm-despawn + player-freeze work.
  We keep that; only the boat-exit triggering moves out.

## Verification

1. **Core unit tests**: `cargo test -p fish-game-core` — passes; new
   `reel.rs` tests included.
2. **Determinism + replay**: `cargo test -p fish-game-replay` — passes
   after refreshing expected hashes once.
3. **Native run**: `cargo run -p fish-game`. Trigger each ending:
   - **Hooked**: confirm the winning boat freezes its forward motion,
     the hook reels back to the rod tip at ~300 u/s, and the line
     stays attached visually the entire time. Other boats slide off
     the arena at doubled reversed speed and despawn at the edge.
   - **Starved**: confirm all boats reverse and slide off; no reel-in.
   - **Bonked**: same as starved.
4. **Restart**: hit `R` after game over mid-wind-down. Confirm all
   boats / hooks / lines despawn cleanly and a fresh game starts.
5. **WASM**: `nix build .#wasm`, serve via `index.html`, repeat 3 + 4 in
   a browser.
6. **Lint + fmt**: `cargo clippy --all-targets --workspace`,
   `cargo fmt --check`.
7. **CI**: `.github/workflows/ci.yml` (WASM build, core/replay tests,
   `cargo tree -p fish-game-core | grep bevy` negative check) — all
   must pass.

## Commit

One commit on `claude/add-claude-documentation-EwuPT`:

> Move game-over wind-down to the presentation layer
>
> Core `tick()` becomes a no-op in `GamePhase::GameOver`. Boat-exit
> motion, hook reel-in, and offscreen despawn move to Bevy systems in
> a new `core_adapter::game_over` module. Reel math is extracted to a
> pure `fish-game-core::reel` module so a future moving-hook feature
> can reuse it without resurrecting `tick_game_over`. Drops six core
> functions, two `Hook` fields, two `Boat` fields, and three
> `CoreEvent` variants. Determinism fixtures re-recorded to reflect
> the new (smaller) post-death state.
