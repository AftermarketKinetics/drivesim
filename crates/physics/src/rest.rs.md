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
- **Floor contact means carrying load:** a body touches if any of its
  contact points with the floor collider has a non-zero impulse. Neither
  `has_any_active_contact` nor a point's `dist` works. Rapier keeps
  speculative contacts active before bodies meet, and reported the
  vertical-spinner fixture's disc, 5.5 mm up, as touching; its loaded
  fork tips read `dist` 2.5 mm and its wheels 0.5 mm (2026-10-10).

## Assertions

- Tested (tests/drive.rs): the fixture settles nose-down under 1° (on
  both wheels and the fork tips, which are the chassis), with the disc
  clear of the floor.
