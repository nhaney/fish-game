# System stages

## Bevy defaults:

1. FIRST
2. PRE_EVENT
3. EVENT
4. PRE_UPDATE
5. UPDATE
6. POST_UPDATE
7. LAST

## How I can use the defaults:

1. Pre-update:
* Process input events, update velocity
2. Update:
* Position updates
3. Post-update:
* Collision detection/correction
4. Last
* Animation + rendering
