//! `robot.toml`: what a STEP cannot say (docs/drivesim.md §5). Written by
//! the role conversation; every table rejects unknown keys so a typo is an
//! error, not a silently ignored setting.

use crate::motors::MotorOverride;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RobotFile {
    pub robot: RobotSection,
    pub materials: Materials,
    #[serde(default)]
    pub mass_overrides: BTreeMap<String, f64>,
    #[serde(default)]
    pub wheel: Vec<RotorSection>,
    #[serde(default)]
    pub weapon: Vec<RotorSection>,
    pub control: Control,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RobotSection {
    pub name: String,
    /// The STEP's up and forward axes, e.g. "+z", "-y".
    pub up: String,
    pub forward: String,
    /// Battery voltage under load, V.
    pub battery_v: f64,
    /// Scale every part so the robot weighs this, kg.
    pub total_mass_kg: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Materials {
    pub default: String,
    /// First matching rule wins, in file order.
    #[serde(default)]
    pub rule: Vec<MaterialRule>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialRule {
    pub parts: Vec<String>,
    pub material: Option<String>,
    pub density_kg_m3: Option<f64>,
}

/// A wheel or a weapon: parts that turn together on one axis.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotorSection {
    pub name: String,
    pub parts: Vec<String>,
    /// The part whose rotation axis is the joint's; default: the first
    /// matched part that has one.
    pub axis_part: Option<String>,
    pub motor: String,
    #[serde(default)]
    pub motor_override: MotorOverride,
    /// Motor turns per rotor turn.
    pub ratio: f64,
    #[serde(default = "one")]
    pub efficiency: f64,
    /// Wheels only: tyre friction coefficient.
    pub mu: Option<f64>,
    /// Wheels only: "left" or "right"; default from the axle's y.
    pub side: Option<String>,
}

fn one() -> f64 {
    1.0
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Control {
    /// Arcade mixing; channels are 1-based, as the radio shows them.
    Mixer {
        throttle: usize,
        turn: usize,
        weapon: Option<usize>,
        /// Switch channel that must be high for anything to move.
        arm: Option<usize>,
        #[serde(default)]
        reverse_turn: bool,
    },
    /// Momentum's firmware core in the loop (docs/drivesim.md §6).
    MomentumCore,
}

pub fn parse_robot_file(text: &str) -> Result<RobotFile, toml::de::Error> {
    toml::from_str(text)
}
