//! The fixture robot in the arena, driven through the mixer exactly as
//! the radio would drive it.

mod common;

use common::radio;
use drivesim_physics::motor::no_load_speed;
use drivesim_physics::{CONTROL_HZ, Sim};

fn sim() -> Sim {
    Sim::new(common::fixture_model(), &common::fixture_mesh).unwrap()
}

fn run(sim: &mut Sim, frame: drivesim_input::Frame, seconds: f32) {
    for _ in 0..(seconds * CONTROL_HZ as f32) as usize {
        sim.control_tick(Some(&frame));
    }
}

fn heading(sim: &Sim) -> f32 {
    let x = *sim.chassis().rotation() * rapier3d::prelude::Vector::X;
    x.y.atan2(x.x)
}

#[test]
fn throttle_drives_straight_near_no_load_speed() {
    let mut s = sim();
    let x0 = s.chassis().translation().x;
    // 1 s: up to speed (it takes ~0.6 s) and still short of the box, which
    // it reaches at ~1.3 s.
    run(&mut s, radio(0.0, 0.1, -1.0, true), 1.0);
    let wheel = &s.model.wheels[0];
    let want = no_load_speed(0.1, wheel, s.model.battery_v as f32) * wheel.radius_m as f32;
    let v = s.chassis().linvel().x;
    assert!(s.chassis().translation().x - x0 > 1.0, "barely moved");
    // Only the dragging chassis edge and winding resistance hold it below.
    assert!(
        (v / want - 1.0).abs() < 0.05,
        "speed {v} m/s, no-load {want}"
    );
    assert!(heading(&s).abs() < 0.1, "veered {} rad", heading(&s));
}

#[test]
fn turn_right_turns_clockwise() {
    let mut s = sim();
    run(&mut s, radio(0.3, 0.0, -1.0, true), 0.5);
    assert!(
        s.chassis().angvel().z < -0.5,
        "yaw rate {}",
        s.chassis().angvel().z
    );
}

#[test]
fn disarmed_does_not_move() {
    let mut s = sim();
    let x0 = s.chassis().translation().x;
    run(&mut s, radio(0.0, 1.0, 1.0, false), 0.5);
    assert!((s.chassis().translation().x - x0).abs() < 0.01);
    assert!(s.rotor_speed(&s.model.weapons[0].clone()).abs() < 1.0);
}

#[test]
fn weapon_spins_up_to_no_load_and_the_robot_stays_upright() {
    let mut s = sim();
    run(&mut s, radio(0.0, 0.0, 1.0, true), 3.0);
    let wp = s.model.weapons[0].clone();
    let want = no_load_speed(1.0, &wp, s.model.battery_v as f32);
    let w = s.rotor_speed(&wp);
    assert!(
        (w / want - 1.0).abs() < 0.05,
        "weapon {w} rad/s, no-load {want}"
    );
    let up = *s.chassis().rotation() * rapier3d::prelude::Vector::Z;
    assert!(up.z > 0.9, "tipped: up = {up:?}");
}

#[test]
fn driving_into_the_box_pushes_it() {
    let mut s = sim();
    let box0 = s.world.bodies[s.target].translation().x;
    run(&mut s, radio(0.0, 0.15, -1.0, true), 3.0);
    let moved = s.world.bodies[s.target].translation().x - box0;
    assert!(moved > 0.05, "box moved {moved} m");
}
