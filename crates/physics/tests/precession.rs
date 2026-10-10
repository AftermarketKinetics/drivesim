//! The weapon's gyroscopic behaviour, with gravity off and no contacts
//! (docs/drivesim.md §9): a torque across a spinning weapon precesses it
//! at τ/L instead of tipping it, and nothing driven conserves angular
//! momentum.

mod common;

use drivesim_physics::Sim;
use drivesim_physics::spawn::{reflected_inertia, spawn_robot, v};
use rapier3d::prelude::*;

const SPIN: f32 = 1000.0; // rad/s, ~9,500 rpm

/// The fixture robot floating in zero g, weapon spun up to `spin` about
/// its axis.
fn floating(spin: f32) -> Sim {
    let model = common::fixture_model();
    let mut world = Sim::empty_world(0.0);
    let robot = spawn_robot(&mut world, &model, &common::fixture_mesh, 0.0, 0.0, 0.0).unwrap();
    let weapon = robot.bodies[model.weapons[0].body];
    world.bodies[weapon].set_angvel(spin_axis(&model) * spin, true);
    Sim::assemble(world, robot, model, None)
}

/// The weapon's axis; the robot spawns unrotated, so robot frame = world.
fn spin_axis(model: &drivesim_robot::RobotModel) -> Vector {
    v(model.weapons[0].axis_dir).normalize()
}

/// The weapon body's inertia about its axis, as spawned: the disc plus
/// its motor's reflected inertia.
fn weapon_inertia(sim: &Sim) -> f32 {
    let wp = &sim.model.weapons[0];
    let i = &sim.model.bodies[wp.body].mass.inertia;
    let a = wp.axis_dir;
    let along: f64 = (0..3)
        .map(|r| (0..3).map(|c| a[r] * i[r][c] * a[c]).sum::<f64>())
        .sum();
    (along + reflected_inertia(wp)) as f32
}

#[test]
fn torque_across_the_spin_precesses_at_tau_over_l() {
    let mut sim = floating(SPIN);
    let tau = 0.5; // N m about +x (across the spin), on the chassis
    let a = spin_axis(&sim.model);
    let b = Vector::X;
    assert!(a.dot(b).abs() < 1e-6, "the torque must be across the spin");
    let l = weapon_inertia(&sim) * SPIN;
    let t_end = 1.0;
    let chassis = sim.robot.bodies[0];
    for _ in 0..(t_end * drivesim_physics::PHYSICS_HZ as f32) as usize {
        sim.world.bodies[chassis].reset_torques(false);
        sim.world.bodies[chassis].add_torque(Vector::new(tau, 0.0, 0.0), true);
        sim.world.step();
    }
    // dL/dt = τ: the spin axis swings toward the torque axis b. Tipping
    // about b instead would move it toward b × a.
    let axis = *sim.chassis().rotation() * a;
    let precessed = axis.dot(b).atan2(axis.dot(a));
    let tipped = axis.dot(b.cross(a)).atan2(axis.dot(a));
    let want = tau / l * t_end;
    assert!(
        (precessed / want - 1.0).abs() < 0.1,
        "precessed {precessed} rad, want {want}"
    );
    assert!(
        tipped.abs() < 0.2 * want,
        "tipped {tipped} rad about the torque axis"
    );
}

#[test]
fn without_spin_the_same_torque_just_tips_it() {
    // The whole robot rolls as one rigid body: θ = τ t² / (2 I_xx), with
    // I_xx the combined inertia about the system CoM.
    let mut sim = floating(0.0);
    let (tau, t) = (0.5f32, 0.1f32);
    let chassis = sim.robot.bodies[0];
    for _ in 0..(t * drivesim_physics::PHYSICS_HZ as f32) as usize {
        sim.world.bodies[chassis].reset_torques(false);
        sim.world.bodies[chassis].add_torque(Vector::new(tau, 0.0, 0.0), true);
        sim.world.step();
    }
    let axis = *sim.chassis().rotation() * Vector::Z;
    let rolled = (-axis.y).atan2(axis.z);
    let want = tau * t * t / (2.0 * sim.model.combined().inertia[0][0] as f32);
    assert!(
        (rolled / want - 1.0).abs() < 0.15,
        "rolled {rolled} rad, a rigid robot would roll {want}"
    );
    assert!(axis.x.abs() < 1e-3);
}

/// Total angular momentum of every robot body about the system CoM.
fn angular_momentum(sim: &Sim) -> Vector {
    let bodies: Vec<&RigidBody> = sim
        .robot
        .bodies
        .iter()
        .map(|&h| &sim.world.bodies[h])
        .collect();
    let m: f32 = bodies.iter().map(|b| b.mass()).sum();
    let com = bodies
        .iter()
        .map(|b| b.center_of_mass() * b.mass())
        .sum::<Vector>()
        / m;
    let vcom = bodies.iter().map(|b| b.linvel() * b.mass()).sum::<Vector>() / m;
    bodies
        .iter()
        .map(|b| {
            let mp = b.mass_properties().local_mprops;
            let frame = *b.rotation() * mp.principal_inertia_local_frame;
            let w_local = frame.inverse() * b.angvel();
            let spin = frame * (mp.principal_inertia() * w_local);
            spin + (b.center_of_mass() - com).cross(b.linvel() - vcom) * b.mass()
        })
        .sum()
}

#[test]
fn nothing_driven_conserves_angular_momentum() {
    let mut sim = floating(SPIN);
    let chassis = sim.robot.bodies[0];
    sim.world.bodies[chassis].set_angvel(Vector::new(0.4, -0.3, 0.2), true);
    // Rapier fills in world-frame mass properties on the first step.
    sim.world.step();
    let l0 = angular_momentum(&sim);
    for _ in 0..drivesim_physics::PHYSICS_HZ {
        sim.world.step();
    }
    let l1 = angular_momentum(&sim);
    assert!(
        (l1 - l0).length() < 0.01 * l0.length(),
        "L {l0:?} -> {l1:?}"
    );
}
