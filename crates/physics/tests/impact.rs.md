# impact.rs

## Function

Tests impact loss (`impact_loss.rs`) and impact reporting (`impacts.rs`)
on the fixture. Each run is
measured against the same run with the loss off (`Sim::floor = None`).

## Interface

Tests:
- `a_hit_throws_the_opponent_slower`: spun-up weapon, full throttle into
  the copy for 1.5 s. The opponent's top speed with the loss is below 0.7×
  its speed without (measured 3.7 vs 6.7 m/s, 2026-10-11).
- `the_weapon_barely_bounces_the_robot_off_a_wall`: copy moved away,
  full throttle into the +x wall for 4 s. The fastest recoil with the loss
  is below 0.3× its recoil without (measured 0.5 vs 5.8 m/s).

- `a_weapon_hit_is_reported_on_the_opponent_and_a_shove_is_not`: the hit
  reports impacts on the opponent's bodies, each at least `MARK_DV`
  with a body-local point within 0.5 m; 3 s of weapon-off full throttle
  into the copy reports none.

## Implementation

The bounds are relative, not absolute: a single hit is chaotic (see
tests/drive.rs.md), so the tests check that the loss bites, not exact
speeds.

## Assertions

- Both runs of a test use identical inputs; only `floor` differs.
