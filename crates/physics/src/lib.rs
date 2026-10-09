//! The rapier3d world for a loaded robot (docs/drivesim.md §6).
//!
//! `spawn` turns a `RobotModel` into bodies and revolute joints, using
//! `shapes` for collision geometry; `arena` adds the floor, walls and the
//! target box; `motor` is the DC motor model; `mixer` the generic control;
//! `sim` steps it all at a fixed rate. Bevy-free and file-free: meshes
//! arrive through a fetch closure, so it runs in tests and in WASM.

pub mod arena;
pub mod mixer;
pub mod motor;
pub mod shapes;
pub mod sim;
pub mod spawn;

pub use sim::{CONTROL_HZ, PHYSICS_HZ, Sim};
