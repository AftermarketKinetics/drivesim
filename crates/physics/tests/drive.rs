//! The fixture robot in the arena, driven through the mixer exactly as
//! the radio would drive it.

mod common;

use common::radio;
use drivesim_physics::motor::no_load_speed;
use drivesim_physics::spawn::forward_sign;
use drivesim_physics::target::BOX_HALF_M;
use drivesim_physics::{CONTROL_HZ, OpponentKind, Sim};

fn sim() -> Sim {
    Sim::new(
        common::fixture_model(),
        &common::fixture_mesh,
        OpponentKind::Copy,
    )
    .unwrap()
}

fn sim_box() -> Sim {
    Sim::new(
        common::fixture_model(),
        &common::fixture_mesh,
        OpponentKind::Box,
    )
    .unwrap()
}

/// The fixture on its own: the opponent copy moved far out of its way.
fn sim_alone() -> Sim {
    let mut s = sim();
    let bodies = s.opponent.as_ref().unwrap().robot.bodies.clone();
    for h in bodies {
        let p = s.world.bodies[h].translation();
        s.world.bodies[h].set_translation(
            rapier3d::prelude::Vector::new(p.x + 20.0, p.y + 20.0, p.z),
            true,
        );
    }
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
fn the_opponent_is_a_copy_facing_the_robot() {
    let s = sim();
    let o = s.opponent.as_ref().unwrap();
    assert_eq!(o.robot.bodies.len(), s.robot.bodies.len());
    let x = *s.opponent_chassis().unwrap().rotation() * rapier3d::prelude::Vector::X;
    assert!(x.x < -0.99, "opponent faces {x:?}");
}

#[test]
fn the_box_opponent_sits_at_the_copys_spot_as_heavy_as_the_robot() {
    let s = sim_box();
    assert!(s.opponent.is_none());
    let b = s.opponent_chassis().unwrap();
    let p = b.translation();
    assert!((p.x - 1.0).abs() < 1e-6 && p.y.abs() < 1e-6 && (p.z - BOX_HALF_M[2]).abs() < 1e-6);
    let want = s.model.total_mass() as f32;
    assert!(
        (b.mass() / want - 1.0).abs() < 1e-3,
        "box {} kg, robot {want} kg",
        b.mass()
    );
}

#[test]
fn driving_into_the_box_shoves_it() {
    let mut s = sim_box();
    let o0 = s.opponent_chassis().unwrap().translation();
    run(&mut s, radio(0.0, 1.0, -1.0, true), 3.0);
    let moved = s.opponent_chassis().unwrap().translation() - o0;
    assert!(moved.x > 0.3, "box moved {moved:?}");
}

/// The weapon spun up to full speed in place, 2.5 s.
fn spun_up(mut s: Sim) -> Sim {
    run(&mut s, radio(0.0, 0.0, 1.0, true), 2.5);
    s
}

#[test]
fn a_spinning_weapon_does_not_launch_the_robot_on_open_floor() {
    // Full throttle with the disc at speed and nothing in front. Spinning
    // front edge down (the importer's arbitrary axis sign, before robot.toml
    // had `spin`) the robot yawed itself up to 5 rad/s and pitched 22°
    // within 0.7 s; front edge up it stays flat.
    let mut s = spun_up(sim_alone());
    let (mut top, mut yaw) = (0.0f32, 0.0f32);
    for _ in 0..700 {
        s.control_tick(Some(&radio(0.0, 1.0, 1.0, true)));
        top = top.max(s.chassis().translation().z);
        yaw = yaw.max(s.chassis().angvel().z.abs());
    }
    assert!(top < 0.02 && yaw < 1.0, "rose {top} m, yawed {yaw} rad/s");
}

#[test]
fn a_hit_throws_the_opponent_up_not_the_robot() {
    // Front edge up: the hit throws the opponent higher than the robot
    // (measured 1.88 m against 0.32 m at 4 kHz; 0.56 m against 0.39 m at
    // 2 kHz, 2026-10-10). Front edge down it was the other way round: the
    // opponent stayed down and the robot flew. The single hit is chaotic,
    // so the check is only which one goes higher, not by how much. Impact
    // loss is off: it checks the spin's direction, and with the loss
    // (tests/impact.rs) neither rises 0.1 m.
    let mut s = sim();
    s.floor = None;
    let mut s = spun_up(s);
    let (mut robot_top, mut opponent_top) = (0.0f32, 0.0f32);
    for _ in 0..1500 {
        s.control_tick(Some(&radio(0.0, 1.0, 1.0, true)));
        robot_top = robot_top.max(s.chassis().translation().z);
        opponent_top = opponent_top.max(s.opponent_chassis().unwrap().translation().z);
    }
    assert!(
        opponent_top > 0.5 && opponent_top > robot_top,
        "opponent rose {opponent_top} m, robot {robot_top} m"
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
fn driving_into_the_opponent_shoves_it() {
    // Full throttle into the braked copy for 3 s: it is shoved back
    // (measured 1.28 m, 2026-10-10). At half throttle it deflected 2 m
    // sideways off the forks instead, so the check is full throttle.
    let mut s = sim();
    let o0 = s.opponent_chassis().unwrap().translation();
    run(&mut s, radio(0.0, 1.0, -1.0, true), 3.0);
    let moved = s.opponent_chassis().unwrap().translation() - o0;
    assert!(moved.x > 0.3, "opponent moved {moved:?}");
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
