//! The arena holds whatever is thrown in it: a robot launched at the roof
//! or a top corner stays inside (user, 2026-10-10: the roof).

mod common;

use drivesim_physics::arena::{ARENA_SIDE_M, WALL_H_M};
use drivesim_physics::{CONTROL_HZ, OpponentKind, Sim};
use rapier3d::prelude::Vector;

/// Throws every body of the driven robot at `v` (m/s) and runs `seconds`
/// with no radio; returns the furthest the chassis got from the centre
/// sideways and up.
fn throw(v: Vector, seconds: f32) -> (f32, f32) {
    let mut s = Sim::new(
        common::fixture_model(),
        &common::fixture_mesh,
        OpponentKind::Box,
    )
    .unwrap();
    for &h in &s.robot.bodies {
        s.world.bodies[h].set_linvel(v, true);
    }
    let (mut side, mut top) = (0.0f32, 0.0f32);
    for _ in 0..(seconds * CONTROL_HZ as f32) as usize {
        s.control_tick(None);
        let p = s.chassis().translation();
        side = side.max(p.x.abs()).max(p.y.abs());
        top = top.max(p.z);
    }
    (side, top)
}

#[test]
fn a_robot_thrown_straight_up_stays_under_the_roof() {
    // 30 m/s up would reach 46 m with no roof.
    let (side, top) = throw(Vector::new(0.0, 0.0, 30.0), 3.0);
    assert!(top < WALL_H_M, "rose to {top} m");
    assert!(
        side < ARENA_SIDE_M / 2.0,
        "left the floor sideways: {side} m"
    );
}

#[test]
fn a_robot_thrown_at_a_top_corner_stays_inside() {
    let (side, top) = throw(Vector::new(-15.0, 15.0, 15.0), 3.0);
    assert!(top < WALL_H_M, "rose to {top} m");
    assert!(side < ARENA_SIDE_M / 2.0, "got {side} m from the centre");
}
