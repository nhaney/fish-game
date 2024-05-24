# fish-game-core

Contains the main deterministic simulation logic of the fish game.

To initialize the game, use the `FishGame::init` method and provide a configuration:

```rs
let config = FishGameConfig { random_seed: 100 };
let mut fish_game = FishGame::init(config);
```

To update the game, call the `FishGame::update` method:

```rs
let input = FishGameInputState { inputs: [ FishGameInput.UpPressed, FishGameInput.DownPressed ] };

// Updates the fish game based on the input and provides the current state after the update.
let new_state = fish_game.update(input);
```

## TODO

* [x] Setup cargo workspace between app and core library that can share dependencies and other stuff as well as allow both to be opened in the same project.
* [ ] Extract player logic + arena from app and put it into core library.
* [ ] Set up glue code between core library and app and allow for player movement with interpolation
* [ ] Set up integration tests on core library that test for determinism.
* [ ] Extract boat logic from app and put it into core library.
* [ ] Extract collision detection logic and put it into core library
* [ ] Allow for debug rendering of colliders with gizmos by optionally putting collider dimensions in state.
* [ ] Add frame-by-frame execution for debugging
