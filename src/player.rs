use bevy::prelude::*;

/******************************************************************************
# Fish components:
## Components always on
* Velocity (Speed of fish)
* Position (position of fish)
* Player
  - state
  - stats - boost length, speed, etc.
* Sprite (image of fish)
* Animation (Animations of the fish)
* Direction (For direction the sprite is facing)
* Collider (AABB)
## Conditional Components
* Reeling (happens when hooked)
* Starving
* Sink
******************************************************************************/
