//! The rapier3d world for a loaded robot (docs/drivesim.md §6).
//!
//! `spawn` turns a `RobotModel` into bodies and revolute joints, using
//! `shapes` for collision geometry (on a weapon, only the parts
//! `weapon_reach` picks); `arena` adds the floor, walls and roof;
//! `motor` is the DC motor model and `gear_coupling` its gearbox
//! correction, both applied per robot by `drivetrain`; `mixer` the generic
//! control; `sim` steps the robot and its opponent (a copy of it or a
//! `Rival` robot, held still or driven by `ai_driver` through `opponent`,
//! or the `target` box) at a fixed rate, with `impact_loss` taking energy
//! out of each impact and `impacts` reporting the large ones; `rest` reports how a robot sits on the
//! floor, for checking a robot file. Bevy-free and file-free: meshes
//! arrive through a fetch closure, so it runs in tests and in WASM.

pub mod ai_driver;
pub mod arena;
pub mod drivetrain;
pub mod gear_coupling;
pub mod impact_loss;
pub mod impacts;
pub mod mixer;
pub mod motor;
pub mod opponent;
pub mod rest;
pub mod shapes;
pub mod sim;
pub mod spawn;
pub mod target;
pub mod weapon_reach;

pub use sim::{CONTROL_HZ, OpponentKind, PHYSICS_HZ, PHYSICS_RATES_HZ, Rival, Sim};
