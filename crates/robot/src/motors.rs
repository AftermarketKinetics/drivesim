//! The motor library (data/motors.toml) and per-robot overrides.

use serde::Deserialize;
use std::collections::BTreeMap;

/// Brushless (three-phase, behind an ESC) or brushed. The motor model is
/// the same DC model for both; `kind` is for choosing and showing motors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MotorKind {
    #[default]
    Brushless,
    Brushed,
}

/// One motor (plus its ESC's current limit), SI-ish units as motor
/// datasheets give them.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotorSpec {
    #[serde(default)]
    pub kind: MotorKind,
    /// No-load speed constant, rpm per volt.
    pub kv_rpm_per_v: f64,
    /// Winding resistance, ohm: phase-to-phase (brushless) or terminal
    /// (brushed).
    pub r_ohm: f64,
    /// ESC / motor current limit, A.
    pub i_max_a: f64,
    /// Unused by the motor model; ignored for brushed motors.
    #[serde(default = "seven")]
    pub pole_pairs: u32,
    /// Rotor inertia about its own axis, kg m²; adds to the driven body.
    #[serde(default)]
    pub rotor_inertia_kg_m2: f64,
    /// Where the numbers came from.
    #[serde(default)]
    pub source: String,
    /// Fields that are placeholders, not measured or datasheet values.
    #[serde(default)]
    pub provisional: Vec<String>,
}

fn seven() -> u32 {
    7
}

/// Any subset of `MotorSpec`'s numbers, from robot.toml.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotorOverride {
    pub kv_rpm_per_v: Option<f64>,
    pub r_ohm: Option<f64>,
    pub i_max_a: Option<f64>,
    pub pole_pairs: Option<u32>,
    pub rotor_inertia_kg_m2: Option<f64>,
}

impl MotorSpec {
    pub fn with(&self, o: &MotorOverride) -> MotorSpec {
        MotorSpec {
            kv_rpm_per_v: o.kv_rpm_per_v.unwrap_or(self.kv_rpm_per_v),
            r_ohm: o.r_ohm.unwrap_or(self.r_ohm),
            i_max_a: o.i_max_a.unwrap_or(self.i_max_a),
            pole_pairs: o.pole_pairs.unwrap_or(self.pole_pairs),
            rotor_inertia_kg_m2: o.rotor_inertia_kg_m2.unwrap_or(self.rotor_inertia_kg_m2),
            ..self.clone()
        }
    }

    /// kv in rad/s per volt, the form the motor model uses.
    pub fn kv_rad(&self) -> f64 {
        self.kv_rpm_per_v * std::f64::consts::TAU / 60.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryFile {
    motor: BTreeMap<String, MotorSpec>,
}

pub type Library = BTreeMap<String, MotorSpec>;

pub fn parse_library(text: &str) -> Result<Library, toml::de::Error> {
    Ok(toml::from_str::<LibraryFile>(text)?.motor)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIB: &str = r#"
        [motor.test-2830]
        kv_rpm_per_v = 1450
        r_ohm = 0.06
        i_max_a = 40
        provisional = ["r_ohm"]
    "#;

    #[test]
    fn parses_with_defaults() {
        let lib = parse_library(LIB).unwrap();
        let m = &lib["test-2830"];
        assert_eq!(m.pole_pairs, 7);
        assert_eq!(m.kind, MotorKind::Brushless);
        assert_eq!(m.rotor_inertia_kg_m2, 0.0);
        assert!((m.kv_rad() - 151.84).abs() < 0.01);
    }

    #[test]
    fn overrides_replace_only_what_they_name() {
        let m = parse_library(LIB).unwrap()["test-2830"].with(&MotorOverride {
            r_ohm: Some(0.03),
            ..Default::default()
        });
        assert_eq!((m.r_ohm, m.kv_rpm_per_v), (0.03, 1450.0));
    }

    #[test]
    fn kind_parses_and_rejects_unknown_values() {
        let lib = "[motor.x]\nkind = \"brushed\"\nkv_rpm_per_v = 1\nr_ohm = 1\ni_max_a = 1\n";
        assert_eq!(parse_library(lib).unwrap()["x"].kind, MotorKind::Brushed);
        assert!(parse_library(&lib.replace("brushed", "stepper")).is_err());
    }

    #[test]
    fn unknown_fields_are_errors() {
        assert!(parse_library("[motor.x]\nkv = 1\nr_ohm = 1\ni_max_a = 1\n").is_err());
    }
}
