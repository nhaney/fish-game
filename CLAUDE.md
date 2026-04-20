# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## Project

"Stay Off the Line! Remastered" — a Rust port of a js13k 2018 game. The game
is a Bevy 0.13 application; targets native Linux and WebAssembly. A hosted
WASM build runs at https://nigelhaney.com/fish-game.

## Workspace layout

The repository is a Cargo workspace with three library crates and one CLI:

```
crates/fish-game-core/      Deterministic simulation kernel. No bevy, no I/O,
                            no pause/restart lifecycle. Cross-target
                            bit-identical given the same (config, inputs).
crates/fish-game-replay/    Replay recording + verification on top of the
                            core. Owns bincode serialization. Hosts the
                            determinism tests.
crates/fish-game/           Bevy presentation: rendering, audio, UI,
                            leaderboard, keyboard input. Drives the core
                            via the adapter in `src/core_adapter/mod.rs`.
tools/replay-verify/        Headless CLI that takes a recorded replay file
                            and prints PASS / FAIL — what a leaderboard
                            server would invoke to validate a submission.
```

Workspace deps and profiles live in the root `Cargo.toml`.

## Development environment

The project is built with Nix. The dev shell provides the Rust toolchain
(with `wasm32-unknown-unknown`), `clang` + `mold` for linking,
`wasm-bindgen-cli`, `binaryen`, and all X11/Vulkan/ALSA runtime libs. Enter
it with `nix develop` or let `direnv` load `.envrc` (`use flake .`).

Cargo is configured (`.cargo/config.toml`) to use `clang` + `mold` on Linux
and `wasm-server-runner` for `wasm32-unknown-unknown`.

## Common commands

The `fish-game` crate uses a non-standard default feature set:
`default = ["common", "linux", "wasm", "dev"]`. The `dev` feature enables
`bevy/dynamic_linking` for fast iterative dev builds. Release builds must
pass `--no-default-features` and select exactly one platform feature.

- Dev run (native, dynamic linking): `cargo run -p fish-game`
- Native release build:
  `cargo build -p fish-game --no-default-features --features linux --release`
- WASM dev run via wasm-server-runner:
  `cargo run -p fish-game --no-default-features --features wasm --target wasm32-unknown-unknown`
- Reproducible native build: `nix build` (output at `result/bin/fish-game`)
- Reproducible WASM build: `nix build .#wasm` (output JS/WASM at
  `result/bin/`, served by `index.html`)
- Lint: `cargo clippy --all-targets --workspace`
- Format: `cargo fmt`
- Core unit tests (collision, boost, hunger, spawn, …):
  `cargo test -p fish-game-core`
- Determinism + replay round-trip tests:
  `cargo test -p fish-game-replay`
- Replay verify CLI: `cargo run -p replay-verify -- path/to/replay.bin`

CI (`.github/workflows/ci.yml`) runs the WASM nix build, the core test
suite, the replay/determinism suite, builds `replay-verify`, and verifies
that `fish-game-core` does not depend on `bevy`.

## Architecture

### `fish-game-core` — the simulation kernel

The whole game simulation lives here as plain Rust. Public API:

```rust
let mut state = FishGameState::new(config);          // FishGameConfig
loop {
    let input = build_input_from_keyboard();         // FishGameInput
    let observed: &FishGameState = state.tick(input);
    /* render `observed` ... */
}
```

Rules of the kernel:

- **No `bevy`, no `web-sys`, no filesystem, no network.**
- **No `thread_rng` / wall-clock entropy.** Seed comes from `config.seed`;
  the live RNG is `rand_chacha::ChaCha8Rng` which is bit-identical across
  targets.
- **No `HashMap` iteration on hot paths.** Entities live in
  `slotmap::SlotMap` (deterministic iteration); cross-component lookups use
  `BTreeMap` if ever needed.
- **No `f32::sin`/`cos`/`sqrt` or `Vec3::normalize`.** Route through
  `core::math::{sinf, cosf, sqrtf, vec3_normalize_or_zero, vec3_length}`,
  which call `libm`. `glam` is enabled with the `libm` feature for the
  same reason.
- **No pause / restart lifecycle.** Pause = caller stops calling `tick`.
  Restart = caller drops the state and constructs a new `FishGameState`.
  This keeps the input log a replay records purely about gameplay.

`tick()` returns `&FishGameState` rather than an event stream. The
presentation layer takes a snapshot of the state before each tick and
diffs `prev` vs `curr` to derive transitions (boost started, worm eaten,
game over). This means the simulation has a single source of truth and
the same diff logic works for both live play and replay verification.

Tick pipeline (inside `state::tick_running`):

```
score + difficulty timers     → tick_score_and_difficulty
boost cooldown                → tick_boost_cooldown
hunger countdown              → tick_hunger          (may set GameOver)
boat spawner                  → tick_boat_spawner
input → velocity + facing     → tick_player_input_movement
boost integration             → tick_player_boost
sink                          → apply_sink
position integration          → integrate_positions
arena clamp                   → clamp_player_to_arena
collisions                    → check_collisions     (may set GameOver)
```

Once `phase == GameOver`, subsequent ticks run `tick_game_over` which only
finishes off-screen boat motion and reel-ins — the player is frozen and no
more spawns or scoring happen.

### `fish-game-replay` — replay record + verify

Tiny crate. `Replay { config, inputs, final_hash, final_score, target_triple }`
plus `record(config, inputs) -> Replay` and `verify(&Replay) -> VerifyResult`.
The determinism contract ("same `(config, inputs)` ⇒ same hash on every
target") is enforced by tests in `tests/determinism.rs`. Anyone tightening
this guarantee should add a regression test here.

### `fish-game` — Bevy presentation

A Bevy 0.13 `App` composed of plugins registered in `src/main.rs`:

- `core_adapter::CorePlugin` — owns `CoreState` (`state` + `prev` snapshot),
  drives `state.tick(input)` from `FixedUpdate`, diffs prev/curr to emit
  the legacy Bevy events (`PlayerHooked`, `PlayerAte`, `GameOver`, …), and
  spawns/despawns Bevy entities to mirror the core's slotmaps via a
  `CoreEntityMap: BTreeMap<CoreId, Entity>`. Pause and restart lifecycle
  live here, not in core.
- `shared::SharedPlugin` — system-set ordering, camera, arena, cross-cutting
  Bevy events.
- `leaderboard::LeaderboardPlugin` — local high-score persistence (native
  writes `scores.json`; WASM uses `web-sys` local storage, gated by
  `target_arch`).
- `player::PlayerPlugin`, `objects::ObjectPlugins`, `ui::UIPlugin`,
  `audio::AudioPlugin` — render layer for the entities the core simulates.

#### System-set pipeline

`SharedPlugin` configures a strict ordering of custom `SystemSet`s
(`src/shared/stages.rs`) on Bevy's `Update` schedule:

```
EmitEventsSet → HandleEventsSet → MovementSet → FinalizeMovementSet
  → CalculateCollisionsSet → AdjustPositionsSet → PrepareRenderSet
```

Picking the right set is more important than inventing a new one — see
the comments in `stages.rs`.

#### Events glue

Plugins are decoupled by `Event`s. Key events (synthesized by the adapter
from state diffs, not by core):

- Player lifecycle: `PlayerHooked`, `PlayerStarved`, `PlayerBonked`,
  `PlayerAte`, `PlayerBoosted` (`player::events`)
- Game lifecycle: `GameOver`, `GamePaused`, `GameUnpaused`,
  `GameRestarted` (`shared::game`)
- Generic: `DestinationReached` (`shared::movement`), `ScoreSaved`
  (`leaderboard`)

Emit in `EmitEventsSet`, consume in `HandleEventsSet` on the same frame.
Collision systems emit in `CalculateCollisionsSet`; those events are
consumed on the *next* frame's `HandleEventsSet`.

#### Rendering

2D sprites use `ImagePlugin::default_nearest()` for pixel-art scaling.
Shape drawing uses `bevy_prototype_lyon` (`ShapePlugin`) — see the
fishing line between boat and hook. A `RenderLayer` component
(`shared::render`) is adjusted per frame to enforce z-ordering, and
`scale_camera_to_screen_size` keeps the arena framed on resize.

#### Restart pattern

On `GameRestarted`, the adapter rebuilds `CoreState` with a fresh seed.
`reset_player` (`src/player/mod.rs`) handles Bevy-side cleanup. Many
subsystems also have their own `reset_*_on_restart` systems. When adding
stateful presentation resources, add a matching reset system to
`PrepareRenderSet`.

## Conventions

- Each Bevy subsystem lives in a directory with a `mod.rs` that defines a
  `Plugin` struct and registers that module's resources, events, and
  systems. Follow this pattern when adding a new subsystem.
- Keep systems small and single-purpose; wire them into the correct
  `SystemSet`.
- `pub(crate)` / `pub(super)` are used deliberately to keep internals
  private — match the existing visibility when extending a module.
- New gameplay logic belongs in `fish-game-core` with a unit test in
  `crates/fish-game-core/src/tests.rs`. If the change could affect
  determinism (RNG draws, math, slotmap ordering), add a regression test
  in `crates/fish-game-replay/tests/determinism.rs`.
- Design docs live in `design/v1/` (stages, state, animations) and
  `design/v2/` (planned refactors). `design/TODO.md` tracks outstanding
  work.
