## To get same features as original:

- [x]  Clean up project modules (player/shared)
- [x]  Consolidate into shared and player plugins with system stages
- [x]  Implement death timer and display
- [x]  Implement score
- [x]  Implement restart
- [x]  Implement enemies
   * hooks
   * rods
   * lines
   * exit animation
   * reel animation/death
- [x]  Implement worms
- [x]  Implement simple animation (fish and worms)
- [ ]  Implement sfx
- [x]  Implement high score saving

## Additional features

- [ ]  Fixed updates
- [ ]  Music
- [ ]  Customizable controls
- [ ]  Main menu
- [ ]  Customizable fish
- [ ]  Replay
- [ ]  Leaderboards
- [ ]  Improved graphics
- [ ]  New obstacles (Birds, sharks, etc)
- [ ]  Power-ups (Invincibility, life vest)

## Goal List - week of 12/14
- [x] More ergonomic build commands for native/web dev and deploy builds
- [x] Draw fishing poles + rods
- [x] Add audio - might be hard on WASM?
- [ ] Clean up code/warnings
- [ ] Build + deploy pipeline/workflow to put WASM build on my site
  - Use Github actions
    - On push, run tests, linting, and format check
    - On tag, run the above and do a release on github with native build (linux for now) and 
the wasm build
    - After making a release, pull down and add to site and push to netlify
- [ ] Balance difficulty
- [x] Update bevy version
  x Update system syntax
  x Update stages/plan for fixed update stages (implement in the future)
  x Add in logging instead of println! calls

