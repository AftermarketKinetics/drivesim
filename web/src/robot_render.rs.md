# robot_render.rs

## Function

Draws the robot's parts, and anything else linked to a rapier body, where
the physics puts them.

## Interface

- `SimRoot`: a component marking the entity that turns rapier's z-up world
  into Bevy's y-up one.
- `Axis { point, dir }`: a rotor axis in the robot frame, for marking.
- `BodyLink(RigidBodyHandle)`: a component drawing an entity at that
  body's pose.
- `spawn_robot_visuals(commands, root, bundle, model, robot, tint,
  meshes, mats)`: one entity per body of `robot` (the driven robot's
  handles or the opponent's), with one mesh child per part. With `tint`,
  every part except the marked wheels takes that colour; the opponent is
  drawn orange that way.
- `sync_bodies`: a system copying each linked body's pose into its
  `Transform`.

## Implementation

- **Mesh vertices are body-local without any per-body offset.** Every
  body is created at the robot frame's pose (physics `spawn.rs`), so STEP
  vertices turned by `model.step_to_robot` are already in each body's
  frame.
- **Flat shading:** each triangle gets its own three vertices and its face
  normal, so faceted CAD tessellation reads as crisp edges. Faces are
  drawn double-sided, so a part with inward-wound faces still shows.
- A part's colour is its CAD colour, or grey when the STEP has none.
- **Wheels are marked:** each wheel's triangles are coloured, as vertex
  colours, by which of `WHEEL_SECTORS` (6) sectors about its axis their
  centroid lies in. Sectors alternate near-black tyre and amber stripe,
  so a wheel is seen to roll, spin or lock. Plain cylinders looked the
  same turning or not, and rolling read as sliding (user, 2026-10-10).
  Marked wheels ignore their CAD colour. Large cap triangles can cross a
  sector edge, so the pattern is blocky on the sidewalls.
- A part whose STL won't parse is skipped rather than failing the load.
  The physics already loaded it, so this can't happen for a checked
  bundle.

## Assertions

- `sync_bodies` copies poses unchanged; the rotation lives only on
  `SimRoot`.
- The bodies of `model.bodies` and `sim.robot.bodies` correspond by
  index.
