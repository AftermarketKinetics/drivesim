//! The loader on the fixture bundle (fixtures/simple_bot): masses from
//! densities, roles from patterns, axes and sides, the frame change, and
//! each error a robot.toml mistake should produce.

use drivesim_robot::{LoadError, RobotModel, Side, load};
use std::f64::consts::PI;

const PARTS: &[u8] = include_bytes!("../../../fixtures/simple_bot/parts.json");
const ROBOT: &str = include_str!("../../../fixtures/simple_bot/robot.toml");
const MOTORS: &str = include_str!("../../../data/motors.toml");

fn fixture() -> RobotModel {
    load(PARTS, ROBOT, MOTORS).unwrap()
}

fn edited(from: &str, to: &str) -> Result<RobotModel, LoadError> {
    assert!(ROBOT.contains(from), "fixture lacks {from:?}");
    load(PARTS, &ROBOT.replacen(from, to, 1), MOTORS)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

#[test]
fn masses_come_from_densities_and_overrides() {
    let m = fixture();
    let by = |n: &str| m.bodies.iter().find(|b| b.name == n).unwrap().mass.mass;
    assert!(close(
        by("wheel_left"),
        PI * 0.03175f64.powi(2) * 0.025 * 1000.0
    ));
    // The toothed disc has no closed form; its density times the volume
    // the importer measured (tools/import/tests checks that volume).
    let disc_v = drivesim_robot::parts::parse_parts(PARTS)
        .unwrap()
        .parts
        .iter()
        .find(|p| p.instance == "weapon")
        .unwrap()
        .volume_m3;
    assert!(close(by("disc"), disc_v * 7850.0));
    // Everything else is the chassis: aluminium base and rails, a UHMW
    // top, AR500 forks and rear bar, and the overridden electronics.
    let al = (0.25 * 0.4 * 0.003 + 2.0 * 0.28 * 0.01 * 0.05) * 2700.0;
    let uhmw = 0.21 * 0.4 * 0.003 * 940.0;
    let ar500 = (2.0 * 0.5 * 0.08 * 0.038 * 0.025 + 0.006 * 0.4 * 0.04) * 7850.0;
    let overridden = 0.35 + 0.12 + 2.0 * 0.17;
    assert!(close(by("chassis"), al + uhmw + ar500 + overridden));
}

#[test]
fn the_wheel_has_a_cylinders_inertia() {
    let m = fixture();
    let w = &m
        .bodies
        .iter()
        .find(|b| b.name == "wheel_left")
        .unwrap()
        .mass;
    let (r, t) = (0.03175f64, 0.025f64);
    assert!(close(w.inertia[1][1], w.mass * r * r / 2.0));
    assert!(close(
        w.inertia[0][0],
        w.mass * (3.0 * r * r + t * t) / 12.0
    ));
    assert!(w.inertia[0][1].abs() < 1e-12);
    // The balanced toothed disc's CoM is on its axis.
    let d = &m.bodies.iter().find(|b| b.name == "disc").unwrap().mass;
    assert!((d.com[0] - 0.15).abs() < 1e-6 && (d.com[2] - 0.095).abs() < 1e-6);
}

#[test]
fn wheels_and_weapon_get_their_axes() {
    let m = fixture();
    let l = &m.wheels[0];
    assert_eq!(l.side, Some(Side::Left));
    assert_eq!(m.wheels[1].side, Some(Side::Right));
    assert_eq!(l.axis_dir, [0.0, 1.0, 0.0]);
    assert!(close(l.radius_m, 0.03175) && close(l.axis_point[1], 0.22));
    let w = &m.weapons[0];
    // spin = "front-up": the front edge rises, so the axis points -y.
    assert_eq!(w.axis_dir, [0.0, -1.0, 0.0]);
    assert!(close(w.radius_m, 0.09f64.hypot(0.015)));
    assert_eq!(w.mu, None);
}

#[test]
fn total_mass_scales_every_part_and_keeps_the_com() {
    let before = fixture().combined();
    let m = edited("battery_v = 22.2", "battery_v = 22.2\ntotal_mass_kg = 5.44").unwrap();
    let after = m.combined();
    assert!(close(after.mass, 5.44));
    for k in 0..3 {
        assert!((after.com[k] - before.com[k]).abs() < 1e-12);
    }
}

#[test]
fn mass_override_replaces_density() {
    let m = edited("\"esc\" = 0.12", "\"esc\" = 0.12\n\"weapon\" = 1.5").unwrap();
    assert!(close(
        m.weapons.iter().map(|w| m.bodies[w.body].mass.mass).sum(),
        1.5
    ));
}

#[test]
fn a_y_up_step_is_turned_upright() {
    // Claiming the fixture is Y-up puts its +y (wheel_L's side) on top.
    // Its disc axis then stands upright, where "front-up" means nothing, so
    // the spin line goes too.
    let text =
        ROBOT
            .replacen("up = \"+z\"", "up = \"+y\"", 1)
            .replacen("spin = \"front-up\"", "", 1);
    let m = load(PARTS, &text, MOTORS).unwrap();
    assert!(close(m.wheels[0].axis_point[2], 0.22));
}

#[test]
fn mistakes_are_named_errors() {
    use LoadError::*;
    assert!(matches!(
        edited("ratio = 4.0", "ratio = 4.0\naxis_part = \"nothing*\""),
        Err(NoAxis { .. })
    ));
    assert!(matches!(
        edited("badass-2820-1820kv", "no-such-motor"),
        Err(UnknownMotor { .. })
    ));
    assert!(matches!(
        edited("parts = [\"weapon\"]\nmotor", "parts = [\"zzz\"]\nmotor"),
        Err(NoParts { .. })
    ));
    assert!(matches!(
        edited("parts = [\"wheel_R\"]", "parts = [\"wheel_*\"]"),
        Err(TwoRoles { .. })
    ));
    assert!(matches!(
        edited("default = \"al6061\"", "default = \"cheese\""),
        Err(UnknownMaterial(_))
    ));
    assert!(matches!(
        edited("mu = 1.4", "side = \"up\"\nmu = 1.4"),
        Err(BadSide { .. })
    ));
    assert!(matches!(edited("arm = 5", "arm = 17"), Err(BadChannel(17))));
    assert!(matches!(
        edited("spin = \"front-up\"", "spin = \"cw\""),
        Err(BadSpin { .. })
    ));
    assert!(matches!(
        edited("mu = 1.4", "spin = \"ccw\"\nmu = 1.4"),
        Err(BadSpin { .. })
    ));
    assert!(matches!(
        edited("up = \"+z\"", "up = \"+x\""),
        Err(Frame { .. })
    ));
    assert!(matches!(edited("mu = 1.4", "mu_typo = 1.4"), Err(Parse(_))));
}
