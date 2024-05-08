# Refactoring of fish game, starting in 2024

## Bevy upgrade

* [x] Update bevy from 0.2 -> 0.13 and get it running!

### Things to fix after upgrade

Goals:
    * I want the codebase to be in a similar state in terms of organization and functionality, just working properly with the new version.
    * Get rid of all warnings
    * Small improvements are okay, but major refactoring will come later.

* [x] UI refactoring
    * [x] Make entire UI be one tree under a root node and arranged with flexbox + component tags.
        * [x] Score works
        * [x] Leaderboard UI works
        * [x] Make pause button work
        * [x] Gameover text works
    * [x] Move countdown text from being a part of UI to being a TextBundle2d above the player
        * [x] Make countdown text not flip when player flips
* [x] Fix collisions
    * [x] Debug collider drawing
        * [x] Gizmos?
    * [x] Fix boat collision
    * [x] Fix hook collision
    * [x] Fix worm collision
* [ ] Get WASM build working
    * [x] Get nix build working for native linux (default package)
    * [x] Get nix build working for wasm
        * [x] Remove bevy dylib feature from cargo.toml in prod build???
        * [x] Optimize web build based on guidelines here: https://github.com/bevyengine/bevy/tree/main/examples#webgl2-and-webgpu
        * [x] Remove unnecessary bevy features being used.
        * [x] Get rid of logging on debug builds
        * [ ] Figure out why firefox is blocking audio
            * [ ] Press play to start?
        * [x] Fix score text size + offset

## Deterministic game refactoring

Goals:
    * I want to completely separate out the game logic from the presentation logic and put it in another crate.
    * I want to expose a bevy plugin to run the game on a fixed timestep.
    * I want the game to be a pure function that takes an input and returns the entire game state as an output.

### Input/Output draft

Initialization:
```rust
/// Initialization configuration for the fish game.
struct FishGameConfig {
    pub tick_rate: u32;
}
```
Input:
```rust
/// Inputs for this tick of the fish game.
struct FishGameInput {
}
```

Output:
```rust
/// Contains all information about the state of the fish game.
struct FishGameState {
}

let fish_game = FishGameState::new(config: FishGameConfig);
let next_state = fish_game.tick(input: FishGameInput);
```

