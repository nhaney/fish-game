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
