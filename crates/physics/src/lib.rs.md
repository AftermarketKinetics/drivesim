# lib.rs (drivesim-physics)

## Function

The rapier3d world for a loaded robot (docs/drivesim.md §6): bodies,
joints, motors, the arena and the target box, stepped at a fixed rate.

## Interface

Modules `arena`, `gear_coupling`, `mixer`, `motor`, `rest`, `shapes`, `sim`, `spawn`; re-exports
`Sim`, `PHYSICS_HZ`, `CONTROL_HZ`.

## Implementation

Bevy-free and file-free: meshes arrive through a fetch closure, so the
same code runs under `cargo test` and in the browser.

## Assertions

- No file I/O and no platform APIs in `src/`.
