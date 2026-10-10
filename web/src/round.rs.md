# round.rs

## Function

A round of the browser sim (docs/drivesim.md §8): the `Sim` built from a
loaded bundle with a chosen opponent, plus everything drawn for it.
`take_bundle` builds the first; `opponent_toggle` rebuilds it on each
switch, which puts both robots back at their start spots.

## Interface

- `OPPONENT_ORANGE` [0.95, 0.45, 0.08]: the colour of the opponent, copy
  or box.
- `Loaded(Bundle)`: resource holding the fetched bundle for rebuilds.
- `BuiltWith(OpponentKind)`: resource naming the opponent the current
  round was built with.
- `build_round(commands, root, bundle, kind, old, meshes, mats) ->
  Result<(), String>`: builds `Sim::new(…, kind)`, despawns `old`, spawns
  the robot's visuals and the opponent's (orange copy or orange box), and
  inserts `SimRes` and `BuiltWith(kind)`.
- `RoundVisuals`: the query for a round's visuals, every entity with a
  `BodyLink`.

## Implementation

- **The sim is built before anything is despawned,** so a failed build
  leaves the previous round in play and the error goes to the HUD.
- **A round's visuals are exactly the `BodyLink` entities.** The arena
  has none, so despawning them (children go with them) clears the robots
  and the box and nothing else.
- **Rebuilding is the reset** (user, 2026-10-10: switching restarts the
  round rather than swapping the opponent in place). A fresh `Sim` needs
  no removal path in the physics, and `SimRes` starts with no backlog.
- The opponent is orange (user, 2026-10-10), so it can't be mistaken for
  the robot being driven; the box takes the same colour.

## Assertions

- `SimRes` and `BuiltWith` are always inserted together, so `BuiltWith`
  names what is being stepped.
- Every visual `build_round` spawns carries a `BodyLink` (directly or
  through its parent), or a rebuild would leave it behind.
