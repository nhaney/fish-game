# System stages

## Bevy defaults:

1. FIRST
* Time resource updates
2. PRE_EVENT
* Gilrs input system?
3. EVENT
* Input systems
After:
* SCENE_STAGE
4. PRE_UPDATE
* Entity labels system
* clear screen
* free unused assets
* ui focus system? - clicks on the UI
5. UPDATE
* Most things
6. POST_UPDATE
(in order)
* Transform propagation
* Asset events
* Rendering
7. LAST
* Unused

## How I can use the defaults:

Before PRE_UPDATE: (EVENT)
* Increment timers/score and spawn things in

After PRE_UPDATE/before UPDATE: (REACT_TO_INPUT)
* Calculate changes to velocity

During UPDATE: (MOVEMENT)
* Move/rotate things to their proper spot based on velocity

After transform propagation system (POST_UPDATE):
* Calculate collisions + events and reposition things (COLLISION_REACTION)

After collision stage, before render (PREPARE_RENDER):
* Adjust everything based on the events from collision
* Everything ready to render

