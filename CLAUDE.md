# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

"Stay Off the Line! Remastered" — a Rust port of a js13k 2018 game, written with the [Bevy](https://github.com/bevyengine/bevy) 0.13 engine. Targets native Linux and WebAssembly. A hosted WASM build runs at https://nigelhaney.com/fish-game.

## Development environment

The project is built with Nix. The dev shell provides the Rust toolchain (with `wasm32-unknown-unknown`), `clang` + `mold` for linking, `wasm-bindgen-cli`, `binaryen`, and all X11/Vulkan/ALSA runtime libs. Enter it with `nix develop` or let `direnv` load `.envrc` (`use flake .`).

Cargo is configured (`.cargo/config.toml`) to use `clang` + `mold` on Linux and `wasm-server-runner` as the runner for `wasm32-unknown-unknown`.

## Common commands

Cargo uses a non-standard default feature set: `default = ["common", "linux", "wasm", "dev"]`. The `dev` feature enables `bevy/dynamic_linking` for fast iterative dev builds. Release builds must pass `--no-default-features` and select exactly one platform feature.

- Dev run (native, with dynamic linking): `cargo run`
- Native release build: `cargo build --no-default-features --features linux --release`
- WASM dev run in browser via wasm-server-runner: `cargo run --no-default-features --features wasm --target wasm32-unknown-unknown`
- Reproducible native build via Nix: `nix build` (output at `result/bin/fish-game`, assets copied alongside)
- Reproducible WASM build via Nix: `nix build .#wasm` (output JS/WASM at `result/bin/`, served by `index.html`)
- Lint: `cargo clippy --all-targets`
- Format: `cargo fmt`
- Tests: `cargo test` (there are no tests currently; the `wasm` Nix build explicitly sets `doCheck = false`)

CI (`.github/workflows/ci.yml`) only runs `nix build .#wasm` on push and uploads the artifacts — there is no lint/test gate.

## Architecture

The game is a Bevy `App` composed of plugins registered in `src/main.rs`:

- `shared::SharedPlugin` — cross-cutting resources, events, system-set ordering, camera, arena
- `leaderboard::LeaderboardPlugin` — local high-score persistence
- `player::PlayerPlugin` — the fish entity, its states, movement, collisions
- `objects::ObjectPlugins` — boats, hooks, worms
- `ui::UIPlugin` — HUD + game-over screens (composed of `gamehud` and `gameover` sub-plugins)
- `audio::AudioPlugin` — SFX playback

### System-set pipeline (critical)

Frame ordering is the single most important architectural concept. `SharedPlugin` configures a strict ordering of custom `SystemSet`s (defined in `src/shared/stages.rs`) on Bevy's `Update` schedule:

```
EmitEventsSet → HandleEventsSet → MovementSet → FinalizeMovementSet
  → CalculateCollisionsSet → AdjustPositionsSet → PrepareRenderSet
```

Each plugin adds its systems into the appropriate set so that, within a single frame: timers tick → events are handled → velocity is computed → transforms are finalized → collisions are detected → positions are adjusted → rendering/animation/audio/UI are prepared. When adding a new system, pick the set whose semantics match (see comments in `src/shared/stages.rs`) rather than inventing new ordering.

### Events as the glue between plugins

Plugins are decoupled by `Event`s, not by direct calls. Key events:

- Player lifecycle: `PlayerHooked`, `PlayerStarved`, `PlayerBonked`, `PlayerAte`, `PlayerBoosted` (in `player::events`)
- Game lifecycle: `GameOver`, `GamePaused`, `GameUnpaused`, `GameRestarted` (in `shared::game`)
- Generic: `DestinationReached` (in `shared::movement`), `ScoreSaved` (in `leaderboard`)

Emit events in `EmitEventsSet` and consume them in `HandleEventsSet` on the same frame. Collision systems emit in `CalculateCollisionsSet`; those events are then consumed on the *next* frame's `HandleEventsSet`.

### Platform conditionals

Leaderboard persistence is split with `#[cfg(target_arch = "wasm32")]` / `#[cfg(not(...))]`: WASM uses `web-sys` local storage; native writes `scores.json` to the working directory. Any new persistence or platform API must follow the same pattern, and `web-sys` / `wasm-bindgen` are gated behind the `wasm` feature in `Cargo.toml`.

### Player restart pattern

On `GameRestarted`, `reset_player` (`src/player/mod.rs`) despawns the current player recursively and respawns a new entity with fresh UI children. Many subsystems (rng, difficulty, score, game state) have their own `reset_*_on_restart` systems in `shared::game`. When adding stateful resources, add a matching reset system to the `PrepareRenderSet`.

### Rendering notes

2D sprites use `ImagePlugin::default_nearest()` for pixel-art scaling. Shape drawing uses `bevy_prototype_lyon` (`ShapePlugin`) — see the fishing line between boat and hook. A `RenderLayer` component (`shared::render`) is adjusted per frame to enforce z-ordering, and `scale_camera_to_screen_size` keeps the arena framed on resize.

## Conventions

- Each subsystem lives in a directory with a `mod.rs` that defines a `Plugin` struct and registers that module's resources, events, and systems. Follow this pattern when adding a new subsystem.
- Keep systems small and single-purpose; wire them into the correct `SystemSet`.
- `pub(crate)` / `pub(super)` are used deliberately to keep internals private — match the existing visibility when extending a module.
- Design docs for the architecture live in `design/v1/` (stages, state, animations) and `design/v2/` (planned refactors/improvements). `design/TODO.md` tracks outstanding work.
