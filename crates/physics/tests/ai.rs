//! The AI opponent in the arena with the fixture robot: it waits out the
//! countdown, then chases, and backs out when wedged; a rival robot in
//! place of the copy drives with its own model.

mod common;

use drivesim_physics::arena::ARENA_SIDE_M;
use drivesim_physics::opponent::{BOUT_S, COUNTDOWN_S};
use drivesim_physics::{OpponentKind, Rival, Sim};
use rapier3d::prelude::{RigidBodyHandle, Vector};

fn sim_ai() -> Sim {
    Sim::new(
        common::fixture_model(),
        &common::fixture_mesh,
        OpponentKind::Ai,
    )
    .unwrap()
}

fn ai_bodies(s: &Sim) -> Vec<RigidBodyHandle> {
    s.opponent.as_ref().unwrap().robot.bodies.clone()
}

/// Moves every one of a robot's `bodies` by `dx` along x.
fn shift(s: &mut Sim, bodies: &[RigidBodyHandle], dx: f32) {
    for &h in bodies {
        let p = s.world.bodies[h].translation();
        s.world.bodies[h].set_translation(Vector::new(p.x + dx, p.y, p.z), true);
    }
}

fn ticks(s: f32) -> usize {
    (s * drivesim_physics::CONTROL_HZ as f32) as usize
}

#[test]
fn it_waits_out_the_countdown() {
    let mut s = sim_ai();
    let o0 = s.opponent_chassis().unwrap().translation();
    assert_eq!(s.countdown_left(), Some(COUNTDOWN_S));
    for _ in 0..ticks(COUNTDOWN_S) - 10 {
        s.control_tick(None);
    }
    let moved = (s.opponent_chassis().unwrap().translation() - o0).length();
    assert!(moved < 0.01, "moved {moved} m during the countdown");
    assert!(s.countdown_left().is_some());
    for _ in 0..20 {
        s.control_tick(None);
    }
    assert_eq!(s.countdown_left(), None);
}

#[test]
fn the_copy_and_the_box_have_no_countdown() {
    for kind in [OpponentKind::Copy, OpponentKind::Box] {
        let s = Sim::new(common::fixture_model(), &common::fixture_mesh, kind).unwrap();
        assert_eq!(s.countdown_left(), None, "{kind:?}");
    }
}

#[test]
fn it_reaches_an_idle_robot() {
    let mut s = sim_ai();
    s.time_s = COUNTDOWN_S;
    let mut closest = f32::MAX;
    for _ in 0..ticks(3.0) {
        s.control_tick(None);
        let d = s.chassis().translation() - s.opponent_chassis().unwrap().translation();
        closest = closest.min(d.x.hypot(d.y));
    }
    assert!(closest < 0.4, "closest {closest} m");
}

#[test]
fn it_does_not_stay_pinned_on_a_wall() {
    // The robot parked just outside the -x wall: the AI drives straight
    // at it, into the wall. With the disc at full it never stalls long
    // enough to back out (stuck recovery is tested in ai_driver.rs): each
    // hit on the wall throws it back, often onto its back (measured
    // 2026-10-10: back off 1.6 m within 4 s, up.z down to -0.93).
    let mut s = sim_ai();
    let half = ARENA_SIDE_M / 2.0;
    let robot = s.robot.bodies.clone();
    shift(&mut s, &robot, -half - 0.5 + 1.0);
    let ai = ai_bodies(&s);
    shift(&mut s, &ai, -half + 0.6 - 1.0);
    s.time_s = COUNTDOWN_S;
    let (mut nearest, mut backed) = (f32::MAX, 0.0f32);
    for _ in 0..ticks(4.0) {
        s.control_tick(None);
        let x = s.opponent_chassis().unwrap().translation().x;
        nearest = nearest.min(x);
        backed = backed.max(x - nearest);
    }
    assert!(
        nearest + half < 0.4,
        "never reached the wall: {} m",
        nearest + half
    );
    assert!(backed > 0.1, "backed off the wall {backed} m");
}

/// The fixture renamed "rival" and scaled to `kg`: a different robot
/// with the same meshes.
fn rival_model(kg: f64) -> drivesim_robot::RobotModel {
    let d = common::fixture_dir();
    let toml = std::fs::read_to_string(d.join("robot.toml"))
        .unwrap()
        .replacen(
            "[robot]\nname = \"simple_bot\"\n",
            &format!("[robot]\nname = \"rival\"\ntotal_mass_kg = {kg}\n"),
            1,
        );
    assert!(
        toml.contains("\"rival\""),
        "fixture robot.toml changed shape"
    );
    drivesim_robot::load(
        &std::fs::read(d.join("parts.json")).unwrap(),
        &toml,
        include_str!("../../../data/motors.toml"),
    )
    .unwrap()
}

#[test]
fn a_rival_drives_with_its_own_model() {
    let rival = Rival {
        model: rival_model(9.0),
        mesh: &common::fixture_mesh,
    };
    let mut s = Sim::versus(
        common::fixture_model(),
        &common::fixture_mesh,
        OpponentKind::Ai,
        Some(rival),
    )
    .unwrap();
    s.control_tick(None); // rapier sums collider masses on the first step
    let o = s.opponent.as_ref().unwrap();
    assert_eq!(o.model.name, "rival");
    assert_eq!(s.model.name, "simple_bot");
    let mass = |bodies: &[RigidBodyHandle], s: &Sim| -> f32 {
        bodies.iter().map(|&h| s.world.bodies[h].mass()).sum()
    };
    let (mine, theirs) = (mass(&s.robot.bodies, &s), mass(&o.robot.bodies, &s));
    assert!((theirs - 9.0).abs() < 0.05, "rival {theirs} kg");
    assert!(
        (mine - theirs).abs() > 1.0,
        "robot {mine} kg, rival {theirs} kg"
    );
    // Heavier on the same motors, it still reaches the robot.
    s.time_s = COUNTDOWN_S;
    let mut closest = f32::MAX;
    for _ in 0..ticks(4.0) {
        s.control_tick(None);
        let d = s.chassis().translation() - s.opponent_chassis().unwrap().translation();
        closest = closest.min(d.x.hypot(d.y));
    }
    assert!(closest < 0.45, "closest {closest} m");
}

#[test]
fn the_bout_runs_three_minutes_from_fight_then_disarms_both() {
    let mut s = sim_ai();
    assert_eq!(s.bout_left(), Some(BOUT_S));
    s.time_s = COUNTDOWN_S + 60.0;
    assert_eq!(s.bout_left(), Some(BOUT_S - 60.0));
    s.time_s = COUNTDOWN_S + BOUT_S - 0.0005;
    s.control_tick(Some(&common::radio(0.0, 1.0, 1.0, true)));
    assert!(s.duties.left.is_some(), "still armed before time");
    s.control_tick(Some(&common::radio(0.0, 1.0, 1.0, true)));
    assert_eq!(s.bout_left(), Some(0.0));
    assert_eq!(s.duties, drivesim_physics::mixer::Duties::default());
    assert_eq!(
        s.opponent.as_ref().unwrap().duties,
        drivesim_physics::mixer::Duties::default()
    );
}

#[test]
fn the_copy_and_the_box_are_untimed() {
    for kind in [OpponentKind::Copy, OpponentKind::Box] {
        let s = Sim::new(common::fixture_model(), &common::fixture_mesh, kind).unwrap();
        assert_eq!(s.bout_left(), None, "{kind:?}");
    }
}

#[test]
fn ai_vs_ai_drives_the_robot_without_a_radio() {
    let mut s = Sim::new(
        common::fixture_model(),
        &common::fixture_mesh,
        OpponentKind::AiVsAi,
    )
    .unwrap();
    let r0 = s.chassis().translation();
    for _ in 0..ticks(COUNTDOWN_S) - 10 {
        s.control_tick(None);
    }
    let moved = (s.chassis().translation() - r0).length();
    assert!(moved < 0.01, "moved {moved} m during the countdown");
    assert!(s.bout_left().is_some());
    let (mut closest, mut hits) = (f32::MAX, 0);
    s.take_impacts();
    // 5 s: both first wait ~1 s for their discs, and back off after hits.
    // Front to front, the two discs keep the centres ~0.43 m apart.
    for _ in 0..ticks(5.0) {
        s.control_tick(None);
        let gap = s.chassis().translation() - s.opponent_chassis().unwrap().translation();
        closest = closest.min(gap.length());
        hits += s.take_impacts().len();
    }
    assert!(s.duties.weapon == Some(1.0), "duties {:?}", s.duties);
    assert!(closest < 0.5, "came within {closest} m");
    assert!(hits > 0, "no impacts");
}
