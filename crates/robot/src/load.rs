//! parts.json + robot.toml + the motor library -> `RobotModel`
//! (docs/drivesim.md §5). Takes text and bytes, never paths, so the same
//! loader runs on the server, in tests and in the browser.

use crate::frame::step_to_robot;
use crate::mass::{MassProps, combine};
use crate::materials::density;
use crate::math::{mat_scale, mat_vec, rotate_tensor};
use crate::model::{Body, RobotModel, Rotor, Side};
use crate::motors::{Library, parse_library};
use crate::parts::{Part, PartsDoc, parse_parts};
use crate::robot_file::{Control, RobotFile, RotorSection, parse_robot_file};
use crate::spin::spin_axis;
use std::fmt;

#[derive(Debug)]
pub enum LoadError {
    Parse(String),
    Frame {
        up: String,
        forward: String,
    },
    UnknownMaterial(String),
    UnknownMotor {
        rotor: String,
        motor: String,
    },
    NoParts {
        rotor: String,
    },
    TwoRoles {
        part: String,
        first: String,
        second: String,
    },
    TwoOverrides {
        part: String,
    },
    NoAxis {
        rotor: String,
    },
    NoChassis,
    MissingMu {
        wheel: String,
    },
    BadSide {
        wheel: String,
        side: String,
    },
    BadChannel(usize),
    BadMass(String),
    BadSpin {
        rotor: String,
        why: String,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LoadError::*;
        match self {
            Parse(e) => write!(f, "{e}"),
            Frame { up, forward } => write!(
                f,
                "up {up:?} and forward {forward:?} are not two perpendicular axes"
            ),
            UnknownMaterial(m) => write!(f, "unknown material {m:?}"),
            UnknownMotor { rotor, motor } => {
                write!(f, "{rotor}: motor {motor:?} is not in the library")
            }
            NoParts { rotor } => write!(f, "{rotor}: its patterns match no part"),
            TwoRoles {
                part,
                first,
                second,
            } => write!(f, "part {part:?} is in both {first} and {second}"),
            TwoOverrides { part } => write!(f, "part {part:?} matches more than one mass override"),
            NoAxis { rotor } => write!(f, "{rotor}: no matched part has a rotation axis"),
            NoChassis => write!(
                f,
                "every part has a rotor role: nothing is left for the chassis"
            ),
            MissingMu { wheel } => write!(f, "wheel {wheel}: needs mu"),
            BadSide { wheel, side } => {
                write!(f, "wheel {wheel}: side {side:?} is not left or right")
            }
            BadChannel(c) => write!(f, "channel {c} is not 1..=16"),
            BadMass(why) => write!(f, "{why}"),
            BadSpin { rotor, why } => write!(f, "{rotor}: {why}"),
        }
    }
}

pub fn load(
    parts_json: &[u8],
    robot_toml: &str,
    motors_toml: &str,
) -> Result<RobotModel, LoadError> {
    let doc = parse_parts(parts_json).map_err(|e| LoadError::Parse(e.to_string()))?;
    let file =
        parse_robot_file(robot_toml).map_err(|e| LoadError::Parse(format!("robot.toml: {e}")))?;
    let lib =
        parse_library(motors_toml).map_err(|e| LoadError::Parse(format!("motors.toml: {e}")))?;
    let r = step_to_robot(&file.robot.up, &file.robot.forward).ok_or_else(|| LoadError::Frame {
        up: file.robot.up.clone(),
        forward: file.robot.forward.clone(),
    })?;

    let masses = part_masses(&doc, &file)?;
    // Each part's mass properties, rotated into the robot frame.
    let props: Vec<MassProps> = doc
        .parts
        .iter()
        .zip(&masses)
        .map(|(p, &m)| MassProps {
            mass: m,
            com: mat_vec(&r, p.com_m),
            inertia: rotate_tensor(&r, &mat_scale(&p.inertia_m5, m / p.volume_m3)),
        })
        .collect();

    // Roles: every rotor claims its parts; the rest is chassis.
    let mut role: Vec<Option<String>> = vec![None; doc.parts.len()];
    let rotors: Vec<(&RotorSection, bool)> = file
        .wheel
        .iter()
        .map(|s| (s, true))
        .chain(file.weapon.iter().map(|s| (s, false)))
        .collect();
    let mut claimed: Vec<Vec<usize>> = Vec::new();
    for (sec, _) in &rotors {
        let ids: Vec<usize> = (0..doc.parts.len())
            .filter(|&i| sec.parts.iter().any(|pat| doc.parts[i].matches(pat)))
            .collect();
        if ids.is_empty() {
            return Err(LoadError::NoParts {
                rotor: sec.name.clone(),
            });
        }
        for &i in &ids {
            if let Some(first) = &role[i] {
                return Err(LoadError::TwoRoles {
                    part: doc.parts[i].path.clone(),
                    first: first.clone(),
                    second: sec.name.clone(),
                });
            }
            role[i] = Some(sec.name.clone());
        }
        claimed.push(ids);
    }
    let chassis: Vec<usize> = (0..doc.parts.len())
        .filter(|&i| role[i].is_none())
        .collect();
    if chassis.is_empty() {
        return Err(LoadError::NoChassis);
    }

    let body = |name: &str, ids: &[usize]| Body {
        name: name.to_string(),
        parts: ids.to_vec(),
        meshes: ids.iter().map(|&i| doc.parts[i].mesh.clone()).collect(),
        mass: combine(&ids.iter().map(|&i| props[i]).collect::<Vec<_>>()),
    };
    let mut bodies = vec![body("chassis", &chassis)];
    let (mut wheels, mut weapons) = (Vec::new(), Vec::new());
    for ((sec, is_wheel), ids) in rotors.iter().zip(&claimed) {
        bodies.push(body(&sec.name, ids));
        let rotor = make_rotor(&doc, &lib, &r, sec, ids, bodies.len() - 1, *is_wheel)?;
        if *is_wheel {
            wheels.push(rotor);
        } else {
            weapons.push(rotor);
        }
    }

    if let Control::Mixer {
        throttle,
        turn,
        weapon,
        arm,
        ..
    } = &file.control
    {
        for &c in [Some(*throttle), Some(*turn), *weapon, *arm]
            .iter()
            .flatten()
        {
            if !(1..=16).contains(&c) {
                return Err(LoadError::BadChannel(c));
            }
        }
    }

    Ok(RobotModel {
        name: file.robot.name.clone(),
        battery_v: file.robot.battery_v,
        step_to_robot: r,
        bodies,
        wheels,
        weapons,
        control: file.control.clone(),
    })
}

/// Mass of every part: override, else density × volume; then all scaled
/// to `total_mass_kg` if set.
fn part_masses(doc: &PartsDoc, file: &RobotFile) -> Result<Vec<f64>, LoadError> {
    let density_of = |p: &Part| -> Result<f64, LoadError> {
        let rule = file
            .materials
            .rule
            .iter()
            .find(|r| r.parts.iter().any(|pat| p.matches(pat)));
        if let Some(d) = rule.and_then(|r| r.density_kg_m3) {
            return Ok(d);
        }
        let name = rule
            .and_then(|r| r.material.as_deref())
            .unwrap_or(&file.materials.default);
        density(name).ok_or_else(|| LoadError::UnknownMaterial(name.to_string()))
    };
    let mut masses = Vec::with_capacity(doc.parts.len());
    for p in &doc.parts {
        let over: Vec<f64> = file
            .mass_overrides
            .iter()
            .filter(|(pat, _)| p.matches(pat))
            .map(|(_, &kg)| kg)
            .collect();
        let m = match over.as_slice() {
            [] => density_of(p)? * p.volume_m3,
            [kg] => *kg,
            _ => {
                return Err(LoadError::TwoOverrides {
                    part: p.path.clone(),
                });
            }
        };
        if m.is_nan() || m <= 0.0 {
            return Err(LoadError::BadMass(format!(
                "part {:?} has mass {m}",
                p.path
            )));
        }
        masses.push(m);
    }
    if let Some(total) = file.robot.total_mass_kg {
        if total.is_nan() || total <= 0.0 {
            return Err(LoadError::BadMass(format!("total_mass_kg {total}")));
        }
        let k = total / masses.iter().sum::<f64>();
        masses.iter_mut().for_each(|m| *m *= k);
    }
    Ok(masses)
}

fn make_rotor(
    doc: &PartsDoc,
    lib: &Library,
    r: &crate::math::M3,
    sec: &RotorSection,
    ids: &[usize],
    body: usize,
    is_wheel: bool,
) -> Result<Rotor, LoadError> {
    let axis_from = ids
        .iter()
        .map(|&i| &doc.parts[i])
        .filter(|p| sec.axis_part.as_deref().is_none_or(|pat| p.matches(pat)))
        .find_map(|p| p.rotation_axis.as_ref())
        .ok_or_else(|| LoadError::NoAxis {
            rotor: sec.name.clone(),
        })?;
    let motor = lib
        .get(&sec.motor)
        .ok_or_else(|| LoadError::UnknownMotor {
            rotor: sec.name.clone(),
            motor: sec.motor.clone(),
        })?
        .with(&sec.motor_override);
    let centre = mat_vec(r, axis_from.centre_m);
    let (mu, side) = if is_wheel {
        let mu = sec.mu.ok_or_else(|| LoadError::MissingMu {
            wheel: sec.name.clone(),
        })?;
        let side = match sec.side.as_deref() {
            Some("left") => Side::Left,
            Some("right") => Side::Right,
            Some(other) => {
                return Err(LoadError::BadSide {
                    wheel: sec.name.clone(),
                    side: other.to_string(),
                });
            }
            None if centre[1] >= 0.0 => Side::Left,
            None => Side::Right,
        };
        (Some(mu), Some(side))
    } else {
        (None, None)
    };
    let bad_spin = |why: String| LoadError::BadSpin {
        rotor: sec.name.clone(),
        why,
    };
    let axis_dir = if is_wheel {
        if sec.spin.is_some() {
            return Err(bad_spin(
                "a wheel's direction comes from the drive, not spin".into(),
            ));
        }
        mat_vec(r, axis_from.dir)
    } else {
        spin_axis(mat_vec(r, axis_from.dir), sec.spin.as_deref()).map_err(bad_spin)?
    };
    Ok(Rotor {
        name: sec.name.clone(),
        body,
        axis_point: centre,
        axis_dir,
        radius_m: axis_from.radius_m,
        width_m: axis_from.width_m,
        motor,
        ratio: sec.ratio,
        efficiency: sec.efficiency,
        mu,
        side,
    })
}
