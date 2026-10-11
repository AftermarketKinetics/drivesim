# weapon_reach.rs

## Function

Picks which of a weapon body's parts get collision hulls (user,
2026-10-10: "do the automatic one"): only the parts that reach near the
weapon's outer radius. Parts buried inside the robot (spindle, pulley,
axle, spacers) can't touch anything, but every hull on the weapon is
swept at weapon speed every physics step, which is most of a fight's
physics cost.

## Interface

- `REACH_FRACTION` 0.5.
- `part_reach(part, point, dir) -> f32`: the farthest any vertex lies
  from the axis through `point` along unit `dir`, m.
- `reaching_parts(parts, point, dir) -> Vec<Vec<Tri>>`: the parts
  reaching at least `REACH_FRACTION` of the farthest one's reach, in
  their original order. `spawn.rs` calls it for weapon bodies before
  `shapes::body_shapes`.

## Implementation

- **Reach is radial:** distance from the spin axis, ignoring position
  along it. A tall spindle is still "inside".
- **Why 0.5:** measured 2026-10-10 on Obliterator 6 v72, whose weapon is
  11 parts. Its 7 stacked bar pieces reach 54–100% of the outer radius;
  the pulley and spindle 29–30%; the axle and spacer 6–7%. 0.5 keeps
  every bar piece and drops all four internals. Every other local robot
  (simple_bot, Seraphs V1) has a one-part weapon and is unchanged.
  Obliterator against its AI copy at 2 kHz went from 2.4× to 2.86× real
  time natively.
- Only collision changes: the dropped parts stay in the body's mass and
  inertia, which come from the model, not the colliders.
- **What a caller could get wrong:** a part near the axis that sticks out
  axially (a hub below a bar that could scrape a wedge) is dropped too.
  The threshold is radial only.

## Assertions

- The farthest-reaching part is always kept, so a weapon never ends up
  with no collision.
- Tested inline: reach ignores the axial distance; inner parts are
  dropped and outer kept, in order; a single part is always kept.
