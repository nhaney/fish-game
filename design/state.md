# State in ECS Options

## Option 1 - Component per state

### Fish

#### `IdleState`:
Starts in idle state with a `IdleState` component.
    * When there is input in the `IdleState` system, do a state transition
      to `SwimState`
    * If boost is pressed, transition to `BoostState` occurs

#### `SwimState`:
Starts after user has made a directional input from `IdleState`
    * When there is no input, do a state transition to `IdleState`
    * If boost is pressed, transition to `BoostState` occurs

#### `BoostState`:
Starts after the user has pressed boost in either `IdleState` or `SwimState`
    * Keep track of the state that the player was in before the boost
    * When the boost is over,
        * Transition back into the state that it was before
        * Add a boost cooldown component so that the player cannot boost

### Analysis

Main problem is that there can be more than one state at a time with this approach so there
needs to something that can accurately determine which state an entity is in by just observing
its events. Once events are sent, they cannot be cancelled or else there will be a mismatch between listeners
and the actual state (needs cancellation events to be accurate). There needs to be a single source of truth
about the state of an entity, so this will be hard to make work.

## Option 2 - Single State Component

### Fish

State enum looks like:

```rust
enum PlayerStates {
    Idle,
    Swim,
    Boost,
    ...
}

struct PlayerState {
    cur_state: PlayerStates,
}

impl PlayerState {
    valid_transitions
    fn start_boost(...) {
        // send transition state event
    }
}
```

Single entry point into the movement system that calls a `match` based
on what the `State` is.

How does the boost transition work?

System that querys for player state that only acts when a player is in a certain state.

Example:

```rust
fn player_movement(...) {
    if let Boost(_) = player_state.cur_state {
        return
    }
    ...
}

fn boost_movement(...) {
    match player_state.cur_state {
        Boost => do something
        NotBoost => dont
    }
}
```

When does the state change in the component occur? Right away. 
Extra data that will be needed for the state will be added when the change occurs. For example the
direction of the boost for the BoostState.

How does the boost timer work?

#### Boost timer cooldown as a component

*Option 1: Boost timer is passed into system as a query (entity/boost timer) that handles movement.*

Pros:
    * Simple to implement and fast
Cons:
    * Querying for for "debuff" seems like it should be in its own system
        * Counter: this debuff is directly related to preventing a state transition
            * What if there are many states and debuffs that affect them? What if there is a debuff where
              the player is "frozen" and can't transition to swim? Would that be another query?

*Option 2: System that looks for state transition events that occurred after the movement system*

Pros:
    * Event reader for the state transition event and the debuff query keeps this logic in only one place
    * Would be easy to add something like a "frozen" state with this approach.
Cons:
    * Would this need to "cancel" the event somehow? What if there is another system which needs the boost
      event? How would this reconcile the event? Could the event be removed?
        * All of the state transition events could be deleted via the Bevy API, but this would have to be
          guaranteed to run directly after it is produced or else it would get to other listeners

*Option 3: Boost timer is inserted after space is pressed, can this exception be caught if this component already exists?*

Research:
    * Would have to be a thread local system + the component that already exists would be dropped. No go.


*Option 4: System that runs before state transition systems that sends event if a state change cannot occur for an entity*

System for boost timer runs before the state change. If the timer is up, the boost timer is removed. If it is not, a
`StateTransitionLocked` event gets fired indicating what state transition is "locked" for this iteration. The lock
should be able to prevent transition from a specific state to another, a set a of states to a particular state,
or any state to a particular state.
```rust
struct StateTransitionBlock {
    from: HashSet<PlayerState>
    to: PlayerState,
    for: Entity
}
```

Pros:
    * Only need to have a local event reader for state transition block events in any state changing
      system.
    * Could be extended to prevent other state changes from occuring in the future. (Ex. Idle -> Swim, Any -> Dead, etc.)

Cons:
    * Events produced every iteration when a "blocker" is occuring instead of only once when an "event" occurs, so kind
      of goes against the idea that this is acutally an event occuring, it is more of a game function that is not emergent.
    * Seems overengineered after thinking about it more, but may be the most extensible solution?

Could the StateTransitionBlock be a component instead? No, a component being added every frame would probably be too much.

*Option 5: StateTransitionBlock inside the state component*

System that runs the boost timer adds a key inside a state transition block field on the state component

Example:
```rust
enum PlayerStates {
    Idle,
    Swim,
    Boost(BoostData),
}

struct PlayerState {
    current_state: PlayerStates,
    blocked_transitions: HashSet(PlayerStates),
}
```

## Option 3 - State Manager Resource + State Components

### Fish


State manager is a global resource that keeps track of all the different states
that a player can be in. A player entity can submit a state transition request
and the manager will do the proper commands to change the state with a set
of components that the state needs. Then the manager sends out a state change
event and the individual.


