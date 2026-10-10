//! The fixture robot in the arena, driven through the mixer exactly as
//! the radio would drive it.

mod common;

use common::radio;
use drivesim_physics::motor::no_load_speed;
use drivesim_physics::spawn::forward_sign;
use drivesim_physics::{CONTROL_HZ, Sim};

fn sim() -> Sim {
    Sim::new(common::fixture_model(), &common::fixture_mesh).unwrap()
}

/// The fixture with the box moved far out of its way.
fn sim_alone() -> Sim {
    let mut s = sim();
    let t = s.target;
    s.world.bodies[t].set_translation(rapier3d::prelude::Vector::new(20.0, 20.0, 0.1), true);
    s
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
    let mut s = sim_alone();
    let x0 = s.chassis().translation().x;
    // 1 s at 40%, on an empty floor (with VytaFlex grip it reached the box
    // inside 1 s).
    run(&mut s, radio(0.0, 0.4, -1.0, true), 1.0);
    let wheel = &s.model.wheels[0];
    let want = no_load_speed(0.4, wheel, s.model.battery_v as f32) * wheel.radius_m as f32;
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
fn full_turn_is_bounded_by_the_wheels_and_slowed_by_the_forks() {
    // A pivot turn's ceiling is both wheels at no-load, opposite ways:
    // 2 v / track. The fork tips carry ~25% of the weight and drag
    // sideways, so the fixture settles well under it (measured 24.4 of
    // 37.2 rad/s, 2026-10-10), as a wedge-front robot does.
    let mut s = sim();
    run(&mut s, radio(1.0, 0.0, -1.0, true), 2.0);
    let (l, r) = (&s.model.wheels[0], &s.model.wheels[1]);
    let track = (l.axis_point[1] - r.axis_point[1]).abs() as f32;
    let v = no_load_speed(1.0, l, s.model.battery_v as f32) * l.radius_m as f32;
    let ratio = -s.chassis().angvel().z / (2.0 * v / track);
    assert!(
        (0.5..1.02).contains(&ratio),
        "yaw at {:.0}% of 2 v / track",
        ratio * 100.0
    );
}

#[test]
fn wheels_roll_without_ringing() {
    // Steady at 40%: each wheel's rim speed is the ground speed, and no
    // wheel runs past its no-load speed. A wheel too light for the step
    // (no reflected motor inertia, spawn.rs) rang between the current
    // limits here, one side braking at full current while it overspun
    // (rims 30-60% off the ground speed). Rapier's soft contacts creep
    // ~3.5% even when rolling steadily, so the bound is 5%.
    let mut s = sim();
    run(&mut s, radio(0.0, 0.4, -1.0, true), 0.8);
    let v = s.chassis().linvel().x;
    for wh in s.model.wheels.clone() {
        let rim = forward_sign(&wh) * s.rotor_speed(&wh) * wh.radius_m as f32;
        let limit = no_load_speed(0.4, &wh, s.model.battery_v as f32) * wh.radius_m as f32;
        assert!(
            (rim / v - 1.0).abs() < 0.05,
            "{}: rim {rim} m/s, ground {v}",
            wh.name
        );
        assert!(
            rim <= limit * 1.01,
            "{}: rim {rim} m/s past no-load {limit}",
            wh.name
        );
    }
}

#[test]
fn slamming_throttle_does_not_flip_it() {
    // Full throttle from rest, then straight to full reverse and back.
    // With the motor's inertia lumped onto the wheel and no gearbox
    // correction (gear_coupling.rs), each slam kicked the chassis ~22x too
    // hard and the fixture flipped end over end (2026-10-10).
    let mut s = sim();
    let mut lowest_up = 1.0f32;
    for throttle in [1.0, -1.0, 1.0] {
        for _ in 0..500 {
            s.control_tick(Some(&radio(0.0, throttle, -1.0, true)));
            let up = *s.chassis().rotation() * rapier3d::prelude::Vector::Z;
            lowest_up = lowest_up.min(up.z);
        }
    }
    assert!(lowest_up > 0.9, "tipped to up.z = {lowest_up}");
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
    run(&mut s, radio(0.0, 0.5, -1.0, true), 3.0);
    let moved = s.world.bodies[s.target].translation().x - box0;
    assert!(moved > 0.05, "box moved {moved} m");
}

#[test]
fn the_fixture_rests_on_its_wheels_and_fork_tips() {
    let r = drivesim_physics::rest::settle(&common::fixture_model(), &common::fixture_mesh, 2.0)
        .unwrap();
    // CoM ~58 mm ahead of the axle: it tips forward until the fork tips
    // (2 mm up, 230 mm ahead of the axle) meet the floor, atan(2/230) ≈ 0.5°.
    assert!(r.tilt_deg < 1.0, "tilt {}", r.tilt_deg);
    assert!(r.touching.contains(&"wheel_left".to_string()));
    assert!(r.touching.contains(&"wheel_right".to_string()));
    assert!(r.touching.contains(&"chassis".to_string()));
    assert!(!r.touching.contains(&"disc".to_string()));
    assert!(r.residual_speed < 0.01);
}
