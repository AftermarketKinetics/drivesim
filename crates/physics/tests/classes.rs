//! simple_bot scaled to each weight class (fixtures/simple_bot_1lb, _3lb,
//! _30lb; user, 2026-10-11): each drives near its no-load speed, and
//! spins its disc up to speed without flipping itself.

mod common;

use common::radio;
use drivesim_physics::motor::no_load_speed;
use drivesim_physics::{OpponentKind, Sim};
use drivesim_robot::{RobotModel, load};
use std::path::PathBuf;

const CLASSES: [&str; 3] = ["simple_bot_1lb", "simple_bot_3lb", "simple_bot_30lb"];

fn dir(name: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures")).join(name)
}

fn model(name: &str) -> RobotModel {
    let d = dir(name);
    load(
        &std::fs::read(d.join("parts.json")).unwrap(),
        &std::fs::read_to_string(d.join("robot.toml")).unwrap(),
        include_str!("../../../data/motors.toml"),
    )
    .unwrap()
}

/// `name` in the arena facing the box, which is moved out of its way.
fn alone(name: &str) -> Sim {
    let d = dir(name);
    let mut s = Sim::new(
        model(name),
        &move |p| std::fs::read(d.join(p)).ok(),
        OpponentKind::Box,
    )
    .unwrap();
    let t = s.target.unwrap();
    s.world.bodies[t].set_translation(rapier3d::prelude::Vector::new(20.0, 20.0, 0.5), true);
    s
}

#[test]
fn each_class_weighs_just_under_its_limit() {
    for (name, lb) in CLASSES.iter().zip([1.0, 3.0, 30.0]) {
        let kg = model(name).total_mass();
        let limit = lb * 0.453_592_37;
        assert!(kg < limit && kg > 0.9 * limit, "{name}: {kg} kg");
    }
}

#[test]
fn each_class_drives_near_its_no_load_speed() {
    for name in CLASSES {
        let mut s = alone(name);
        for _ in 0..1000 {
            s.control_tick(Some(&radio(0.0, 0.4, -1.0, true)));
        }
        let wh = s.model.wheels[0].clone();
        let want = no_load_speed(0.4, &wh, s.model.battery_v as f32) * wh.radius_m as f32;
        let v = s.chassis().linvel().length();
        assert!(
            (v / want - 1.0).abs() < 0.1,
            "{name}: {v} m/s, no-load {want}"
        );
    }
}

#[test]
fn each_class_spins_up_its_disc_and_stays_flat() {
    // 8 s: the 30 lb's disc takes ~7 s. Near 10,000 rpm the 1 and 3 lb
    // discs hopped and flipped their robots at the 2 kHz step (2026-10-11),
    // hence their slower ratios; this pins them flat all the way.
    for name in CLASSES {
        let mut s = alone(name);
        let mut tilt = 0.0f32;
        for _ in 0..8000 {
            s.control_tick(Some(&radio(0.0, 0.0, 1.0, true)));
            let up = *s.chassis().rotation() * rapier3d::prelude::Vector::Z;
            tilt = tilt.max(up.z.clamp(-1.0, 1.0).acos().to_degrees());
        }
        let wp = s.model.weapons[0].clone();
        let want = no_load_speed(1.0, &wp, s.model.battery_v as f32);
        let w = s.rotor_speed(&wp);
        assert!(
            (w / want - 1.0).abs() < 0.05,
            "{name}: {w} rad/s, no-load {want}"
        );
        assert!(tilt < 10.0, "{name} tilted {tilt} degrees");
    }
}
