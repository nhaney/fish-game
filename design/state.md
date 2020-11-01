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

## Option 2 - Single State Component

### Fish

State enum looks like:

```rust
enum State {
    Idle,
    Swim,
    Boost,
    ...
}
```

Single entry point into the movement system that calls a `match` based
on what the `State` is.

## Option 3 - State Manager Resource + State Components

### Fish

State manager is a global resource that keeps track of all the different states
that a player can be in. A player entity can submit a state transition request
and the manager will do the proper commands to change the state with a set
of components that the state needs. Then the manager sends out a state change
event and the individual.


