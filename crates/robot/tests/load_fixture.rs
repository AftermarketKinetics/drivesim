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
fn masses_come_from_densities() {
    let m = fixture();
    let by = |n: &str| m.bodies.iter().find(|b| b.name == n).unwrap().mass.mass;
    assert!(close(by("chassis"), 0.2 * 0.16 * 0.04 * 2700.0));
    assert!(close(
        by("wheel_left"),
        PI * 0.03f64.powi(2) * 0.02 * 1200.0
    ));
    assert!(close(by("disc"), PI * 0.08f64.powi(2) * 0.006 * 7850.0));
}

#[test]
fn chassis_inertia_is_the_boxs() {
    let c = &fixture().bodies[0].mass;
    let m = c.mass;
    assert!(close(
        c.inertia[0][0],
        m / 12.0 * (0.16f64.powi(2) + 0.04f64.powi(2))
    ));
    assert!(close(
        c.inertia[2][2],
        m / 12.0 * (0.2f64.powi(2) + 0.16f64.powi(2))
    ));
    assert!(c.inertia[0][1].abs() < 1e-12);
    assert!(close(c.com[2], 0.03));
}

#[test]
fn wheels_and_weapon_get_their_axes() {
    let m = fixture();
    let l = &m.wheels[0];
    assert_eq!(l.side, Some(Side::Left));
    assert_eq!(m.wheels[1].side, Some(Side::Right));
    assert_eq!(l.axis_dir, [0.0, 1.0, 0.0]);
    assert!(close(l.radius_m, 0.03) && close(l.axis_point[1], 0.1));
    let w = &m.weapons[0];
    assert_eq!(w.axis_dir, [0.0, 0.0, 1.0]);
    assert!(close(w.radius_m, 0.08));
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
    let m = edited(
        "[[wheel]]",
        "[mass_overrides]\n\"chassis\" = 1.5\n\n[[wheel]]",
    )
    .unwrap();
    assert!(close(m.bodies[0].mass.mass, 1.5));
}

#[test]
fn a_y_up_step_is_turned_upright() {
    // Claiming the fixture is Y-up puts its +y (wheel_L's side) on top.
    let m = edited("up = \"+z\"", "up = \"+y\"").unwrap();
    assert!(close(m.wheels[0].axis_point[2], 0.1));
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
        edited("mu = 0.9", "side = \"up\"\nmu = 0.9"),
        Err(BadSide { .. })
    ));
    assert!(matches!(edited("arm = 5", "arm = 17"), Err(BadChannel(17))));
    assert!(matches!(
        edited("up = \"+z\"", "up = \"+x\""),
        Err(Frame { .. })
    ));
    assert!(matches!(edited("mu = 0.9", "mu_typo = 0.9"), Err(Parse(_))));
}
