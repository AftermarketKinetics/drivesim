//! One robot's motors in the world: each physics step, every rotor gets
//! its motor's torque for its present speed, equal and opposite on the
//! chassis, plus the gearbox coupling (`gear_coupling.rs`), which needs
//! the accelerations over the previous step. `Sim` keeps one per robot.

use crate::gear_coupling::gear_coupling;
use crate::mixer::Duties;
use crate::motor::rotor_torque;
use crate::spawn::{RobotHandles, forward_sign, side_of, v};
use drivesim_robot::{RobotModel, Rotor, Side};
use rapier3d::prelude::*;

/// The motor state carried between steps. Per rotor (wheels, then
/// weapons): (rotor, chassis) absolute angular velocity about the rotor's
/// axis after the last step, and the accelerations over it.
#[derive(Default)]
pub struct Drivetrain {
    axis_w: Vec<(f32, f32)>,
    axis_alpha: Vec<(f32, f32)>,
}

fn rotors(model: &RobotModel) -> impl Iterator<Item = &Rotor> {
    model.wheels.iter().chain(&model.weapons)
}

/// Rotor speed relative to the chassis about its axis, rad/s.
pub fn rotor_speed(world: &PhysicsWorld, robot: &RobotHandles, rotor: &Rotor) -> f32 {
    let c = &world.bodies[robot.bodies[0]];
    let b = &world.bodies[robot.bodies[rotor.body]];
    let a = *c.rotation() * v(rotor.axis_dir).normalize();
    (b.angvel() - c.angvel()).dot(a)
}

/// (rotor, chassis) absolute angular velocity about each rotor's axis.
fn axis_speeds(world: &PhysicsWorld, model: &RobotModel, robot: &RobotHandles) -> Vec<(f32, f32)> {
    let c = &world.bodies[robot.bodies[0]];
    rotors(model)
        .map(|r| {
            let a = *c.rotation() * v(r.axis_dir).normalize();
            let b = &world.bodies[robot.bodies[r.body]];
            (b.angvel().dot(a), c.angvel().dot(a))
        })
        .collect()
}

impl Drivetrain {
    /// Before a world step: this step's motor and coupling torques on the
    /// robot's bodies (replacing last step's).
    pub fn apply(
        &mut self,
        world: &mut PhysicsWorld,
        model: &RobotModel,
        robot: &RobotHandles,
        duties: &Duties,
    ) {
        if self.axis_w.is_empty() {
            self.axis_w = axis_speeds(world, model, robot);
        }
        let chassis = robot.bodies[0];
        for &h in &robot.bodies {
            world.bodies[h].reset_torques(false);
        }
        let v_batt = model.battery_v as f32;
        let rot = *world.bodies[chassis].rotation();
        for (k, rotor) in rotors(model).enumerate() {
            let is_wheel = k < model.wheels.len();
            let (duty, s) = if is_wheel {
                let d = match side_of(rotor) {
                    Side::Left => duties.left,
                    Side::Right => duties.right,
                };
                (d, forward_sign(rotor))
            } else {
                (duties.weapon, 1.0)
            };
            let tau = s * rotor_torque(duty, s * rotor_speed(world, robot, rotor), rotor, v_batt);
            let (alpha_r, alpha_c) = self.axis_alpha.get(k).copied().unwrap_or_default();
            let (fix_r, fix_c) = gear_coupling(
                rotor.motor.rotor_inertia_kg_m2 as f32,
                rotor.ratio as f32,
                alpha_r,
                alpha_c,
            );
            let a = rot * v(rotor.axis_dir).normalize();
            world.bodies[robot.bodies[rotor.body]].add_torque(a * (tau + fix_r), true);
            world.bodies[chassis].add_torque(a * (fix_c - tau), true);
        }
    }

    /// After a world step: the accelerations over it, for the next step's
    /// coupling torques.
    pub fn observe(&mut self, world: &PhysicsWorld, model: &RobotModel, robot: &RobotHandles) {
        let after = axis_speeds(world, model, robot);
        let dt = world.integration_parameters.dt;
        self.axis_alpha = self
            .axis_w
            .iter()
            .zip(&after)
            .map(|(b, a)| ((a.0 - b.0) / dt, (a.1 - b.1) / dt))
            .collect();
        self.axis_w = after;
    }
}
