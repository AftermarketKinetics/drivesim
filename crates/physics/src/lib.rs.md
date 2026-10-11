# lib.rs (drivesim-physics)

## Function

The rapier3d world for a loaded robot (docs/drivesim.md §6): bodies,
joints, motors, the arena and an opponent (a copy of the robot or a
rival robot, held
still or AI-driven, or a box), stepped at a fixed rate.

## Interface

Modules `ai_driver`, `arena`, `drivetrain`, `gear_coupling`, `impact_loss`, `impacts`, `mixer`, `motor`, `opponent`, `rest`, `shapes`, `sim`, `spawn`, `target`, `weapon_reach`; re-exports
`Sim`, `OpponentKind`, `Rival`, `PHYSICS_HZ`, `PHYSICS_RATES_HZ`,
`CONTROL_HZ`.

## Implementation

Bevy-free and file-free: meshes arrive through a fetch closure, so the
same code runs under `cargo test` and in the browser.

## Assertions

- No file I/O and no platform APIs in `src/`.
