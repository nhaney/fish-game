# Refactoring of fish game, starting in 2024

## Bevy upgrade

* [x] Update bevy from 0.2 -> 0.13 and get it running!

### Things to fix after upgrade

Goals:
    * I want the codebase to be in a similar state in terms of organization and functionality, just working properly with the new version.
    * Get rid of all warnings
    * Things that are not idiomatic bevy should be changed if they are not a part of the bigger refactoring later.

* [ ] UI refactoring
    * [ ] Make entire UI be one tree under a root node and arranged with flexbox + component tags.
        * [x] Score works
        * [x] Leaderboard UI works
        * [ ] Make pause button work
        * [ ] Gameover text works
    * [ ] Move countdown text from being a part of UI to being a TextBundle2d above the player
* [ ] Use bevy primitives introduced in 0.13 instead of bevy_prototype_lyon for
    * [ ] lines (Line2d?, Rect + rope texture?)
    * [ ] trackers (Circle)
* [ ] Fix collisions
    * [ ] Debug collider drawing
        * [ ] Gizmos?
    * [ ] Fix boat collision
    * [ ] Fix hook collision
    * [ ] Fix worm collision

## Deterministic game refactoring

Goals:
    * I want to completely separate out the game logic from the presentation logic and put it in another crate.
    * I want to expose a bevy plugin to run the game.
    * I want to use the bevy plugin to run the game on a fixed timestep.
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
