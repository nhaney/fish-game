/**
System order for this game is very important to make sure events are handled
on the same frame. What follows is a rough guideline of where various systems
are placed in this game.

1. EVENT
Handle timers ticking in this stage. The events that are emitted by them will
be handled in the next step.

2. HANDLE_EVENTS - After PRE_UPDATE
Handles the initial round of events from timers finishing/input being received
from various sources. This will also handle all of the events that resulted from a
collision during the previous frame.

3. MOVEMENT - After HANDLE_EVENTS
Handles the remaining calculations to get the final velocity for everything
that is moving.

4. FINALIZE_MOVEMENT - After MOVEMENT, the same as bevy's UPDATE
Calculates final positions of transforms after applying velocity to them

5. CALCULATE_COLLISIONS - After POST_UPDATE, but before render (because child transforms need to be updated)
Calculates any collisions that might have occurred during the frame and emits appropriate events
about them.

6. HANDLE_COLLISIONS - After CALCULATE_COLLISIONS
Handle events emitted from the result of a collision

7. PREPARE_RENDER
Handles any final events from the frame and how final frame looks (animation, audio, ui, etc.)
*/
use bevy::prelude::*;

pub const HANDLE_EVENTS: &str = "handle_events";
pub const MOVEMENT: &str = "movement";
pub const FINALIZE_MOVEMENT: &str = stage::UPDATE;
pub const PREPARE_RENDER: &str = "prepare_render";
pub const CALCULATE_COLLISIONS: &str = "calculate_collisions";
