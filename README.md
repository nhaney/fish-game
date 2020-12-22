# Stay Off the Line! Remastered

![](assets/sprites/player/fish1.png)

## Overview

A port/enhanced version of [the original Stay Off the Line!](https://github.com/nhaney/stayOffTheLine) game 
I made for the 2018 [js13k](https://js13kgames.com/) game jam.

This game is written using Rust using the fledgling but powerful [Bevy game engine](https://github.com/bevyengine/bevy).

One of the goals of this project is keeping this repo up to date with the latest versions of Bevy as the 
project keeps evolving,to be an example of a slightly complicated cross platform game running on this engine.

Play a WASM build of this game on your browser [here](https://nigelhaney.com/fish-game).

To play a native build of the game, log in to GitHub and check the recent 
[GitHub actions artifacts for this project](https://github.com/nhaney/fish-game/actions) and select your platform.

## Development

Currently I am using the python `doit` framework for performing dev and pipeline actions, but may switch to 
something else in the future.

After installing `doit` with `pip install --user doit`, run `doit list` in the root directory of the repo to 
see the commands that can be taken.

Alternatively, you can use `cargo` with the  using the `--features native` for native builds and 
`--features web` for WASM builds. Most of the `doit` commands are just wrappers for `cargo` commands.

If you hack around on the game and do something cool, feel free to open a PR!

## Road map

Below is an unsorted list of the next things that I want to accomplish on this project in no particular order:

- [ ] Update to latest bevy master (0.4+)
- [ ] Improve WASM/executable size (right now its pretty big)
- [ ] Improve development iteration speed (dynamically link bevy, prune unused features, improve CI speed)
- [ ] Implement a fixed timestep for game logic, and a normal timestep for rendering
- [ ] Clean up code using new bevy features (states)
- [ ] Set up and write some unit tests
- [ ] Implement an online leaderboard with server-side validation of score using a headless build
- [ ] Improve graphics/gameplay mechanics

