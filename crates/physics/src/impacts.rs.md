# impacts.rs

## Function

Finds where the robots take large impacts, for the glowing marks the
sim page draws (user, 2026-10-11: "glowing orange-to-red marks where
they take large impacts, scaling based on the strength of the impact").
`web/src/impact_marks.rs` draws them.

## Interface

- `MARK_DV` 2.0 m/s: the bar for a large impact.
- `Impact { body, point, dv }`: one large impact on one body. `point` is
  the contact point in that body's frame. `dv` is the step's contact
  impulse over the mass of the body's group (the whole robot), in m/s.
- `find_impacts(world, groups, floor) -> Vec<Impact>`: after a step, for
  each contact pair between a group (`impact_loss::Group`) and anything
  but `floor` and that same group, gives each touching group body whose
  `dv` reaches `MARK_DV` one impact. The point is where the pair's
  strongest contact point lies.
  `Sim::physics_step` calls it just before `impact_loss::absorb`;
  `Sim::impacts` collects the results.

## Implementation

- **Strength as `dv` (impulse over the robot's mass),** not raw impulse,
  so the same bar and colour scale work from 1 lb to 30 lb.
- **Why 2 m/s:** measured per step on the fixture (2026-10-11). Weapon
  hits on the opponent were 3–12 m/s and the weapon striking the wall
  12. A 4.5 m/s ram with the weapon off peaked at 1.6, so a shove leaves
  no mark and a weapon hit does.
- **Per step, per pair:** a hit lasting several steps reports several
  impacts at nearly one spot. The page merges them (`MERGE_R`,
  `MERGE_S` in impact_marks.rs).
- The point comes from the solver manifolds, which hold the impulses
  actually applied. Its local point is turned to world space by the
  collider's pose, then to the body's frame. Every collider here is a
  single convex shape, so a manifold point is in its collider's frame.
- The floor is skipped, as in `impact_loss`. A landing is not a hit.

## Assertions

- No impact below `MARK_DV`; none from the floor or within one group.
- `point` is body-local.
- Tested in tests/impact.rs: a weapon hit reports impacts on the
  opponent, every one at least `MARK_DV` and within 0.5 m of its body's
  origin; a weapon-off shove reports none.
