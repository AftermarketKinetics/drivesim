//! Impact loss (`impact_loss.rs`): a hit and a wall strike keep only part
//! of their velocity change, measured against the same run without it.

mod common;

use common::radio;
use drivesim_physics::{OpponentKind, Sim};

/// The fixture against its copy, weapon spun up; `alone` moves the copy
/// out of the way, so the robot drives into the +x wall.
fn spun_up(loss: bool, alone: bool) -> Sim {
    let mut s = Sim::new(
        common::fixture_model(),
        &common::fixture_mesh,
        OpponentKind::Copy,
    )
    .unwrap();
    if !loss {
        s.floor = None;
    }
    if alone {
        for h in s.opponent.as_ref().unwrap().robot.bodies.clone() {
            let p = s.world.bodies[h].translation();
            s.world.bodies[h].set_translation(
                rapier3d::prelude::Vector::new(p.x + 20.0, p.y + 20.0, p.z),
                true,
            );
        }
    }
    for _ in 0..2500 {
        s.control_tick(Some(&radio(0.0, 0.0, 1.0, true)));
    }
    s
}

/// The opponent's top speed over 1.5 s of full throttle into it.
fn hit_speed(loss: bool) -> f32 {
    let mut s = spun_up(loss, false);
    let mut top = 0.0f32;
    for _ in 0..1500 {
        s.control_tick(Some(&radio(0.0, 1.0, 1.0, true)));
        top = top.max(s.opponent_chassis().unwrap().linvel().length());
    }
    top
}

/// The robot's fastest recoil off the +x wall over 4 s of full throttle.
fn wall_recoil(loss: bool) -> f32 {
    let mut s = spun_up(loss, true);
    let mut back = 0.0f32;
    for _ in 0..4000 {
        s.control_tick(Some(&radio(0.0, 1.0, 1.0, true)));
        back = back.max(-s.chassis().linvel().x);
    }
    back
}

#[test]
fn a_hit_throws_the_opponent_slower() {
    // Measured 6.7 m/s without the loss, 3.7 with it (2026-10-11).
    let (with, without) = (hit_speed(true), hit_speed(false));
    assert!(
        with < 0.7 * without,
        "{with} m/s with the loss, {without} without"
    );
}

#[test]
fn the_weapon_barely_bounces_the_robot_off_a_wall() {
    // Measured 5.8 m/s without the loss, 0.5 with it: the weapon walks
    // the robot up the wall instead (2026-10-11).
    let (with, without) = (wall_recoil(true), wall_recoil(false));
    assert!(
        with < 0.3 * without,
        "{with} m/s with the loss, {without} without"
    );
}
