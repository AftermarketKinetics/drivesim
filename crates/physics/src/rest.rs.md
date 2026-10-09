# rest.rs

## Function

Reports how a robot rests on the floor: the evidence the role
conversation shows a user, so a wrong role or axis (a robot lying on its
side, a weapon on the floor) shows up as a wrong number before anyone
drives (docs/drivesim.md §4).

## Interface

- `settle(model, mesh, seconds) -> Result<Rest, SpawnError>`.
- `Rest { tilt_deg, com_height_m, touching, residual_speed }`:
  `touching` names the bodies in floor contact, in model order.
- `NUDGE_RAD_S`: 0.3.

## Implementation

- The robot is dropped on an empty arena with no power, given a small
  pitch-rate nudge, and stepped for `seconds` at the physics rate.
- **Why the nudge:** the fixture, with its CoM exactly above the axle,
  balanced on its wheels at 0.5° forever (2026-10-09). No real robot
  does that, so a check reporting it would mislead. Which way it then
  falls is arbitrary (forward).
- Floor contact comes from the narrow phase's active contact pairs with
  the floor collider.

## Assertions

- Tested (tests/drive.rs): the fixture settles at about 5.7°
  (atan(10/100), a chassis edge down), on both wheels and the chassis,
  with the disc clear of the floor.
