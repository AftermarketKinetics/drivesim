//! A robot bundle -> a `RobotModel` (docs/drivesim.md §5).
//!
//! `parts` reads the importer's parts.json, `robot_file` reads robot.toml
//! and `motors` the motor library; `load` combines them, using `frame`
//! (STEP axes -> robot frame), `materials` (densities), `glob` (part-name
//! patterns), `mass` (combining parts into bodies) and `spin` (which way a
//! weapon turns), into the types in
//! `model`. `stl` reads the meshes the model points at. Text and bytes in,
//! never paths, so it runs on the server, in tests and in WASM.

pub mod frame;
pub mod glob;
pub mod load;
pub mod mass;
pub mod materials;
pub mod math;
pub mod model;
pub mod motors;
pub mod parts;
pub mod robot_file;
pub mod spin;
pub mod stl;

pub use load::{LoadError, load};
pub use model::{Body, RobotModel, Rotor, Side};
