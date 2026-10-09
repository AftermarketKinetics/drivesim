//! The loaded robot, in the robot frame (x forward, y left, z up, metres),
//! ready for the physics crate to build bodies from.

use crate::mass::MassProps;
use crate::math::{M3, V3};
use crate::motors::MotorSpec;
use crate::robot_file::Control;

#[derive(Clone, Debug)]
pub struct RobotModel {
    pub name: String,
    pub battery_v: f64,
    /// STEP -> robot frame rotation; apply to mesh vertices.
    pub step_to_robot: M3,
    /// `bodies[0]` is the chassis; each rotor names its own body.
    pub bodies: Vec<Body>,
    pub wheels: Vec<Rotor>,
    pub weapons: Vec<Rotor>,
    pub control: Control,
}

#[derive(Clone, Debug)]
pub struct Body {
    pub name: String,
    /// Part ids from parts.json, and their mesh paths (bundle-relative).
    pub parts: Vec<usize>,
    pub meshes: Vec<String>,
    pub mass: MassProps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// A wheel or weapon: a body turning about an axis fixed in the chassis.
#[derive(Clone, Debug)]
pub struct Rotor {
    pub name: String,
    pub body: usize,
    /// A point on the axis and its unit direction (sign: largest component
    /// positive in STEP axes, carried through the frame change).
    pub axis_point: V3,
    pub axis_dir: V3,
    pub radius_m: f64,
    pub width_m: f64,
    pub motor: MotorSpec,
    pub ratio: f64,
    pub efficiency: f64,
    /// Wheels only.
    pub mu: Option<f64>,
    pub side: Option<Side>,
}

impl RobotModel {
    pub fn total_mass(&self) -> f64 {
        self.bodies.iter().map(|b| b.mass.mass).sum()
    }

    pub fn combined(&self) -> MassProps {
        crate::mass::combine(&self.bodies.iter().map(|b| b.mass).collect::<Vec<_>>())
    }
}
