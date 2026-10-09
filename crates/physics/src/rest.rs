//! How a robot rests on the floor: what the role conversation shows the
//! user so a wrong role or axis shows up as a wrong number (a robot on its
//! side, a weapon touching the floor) before anyone drives.

use crate::arena::add_arena;
use crate::sim::{GRAVITY, PHYSICS_HZ, Sim};
use crate::spawn::{SpawnError, spawn_robot};
use drivesim_robot::RobotModel;
use rapier3d::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Rest {
    /// Angle between the chassis's up axis and world up, degrees.
    pub tilt_deg: f32,
    /// Height of the whole robot's CoM above the floor, m.
    pub com_height_m: f32,
    /// Names of the bodies touching the floor, in model order.
    pub touching: Vec<String>,
    /// Chassis speed at the end, m/s: near 0 when it has settled.
    pub residual_speed: f32,
}

/// A small pitch-rate nudge at the start, rad/s: a robot whose CoM sits
/// exactly over its axle would otherwise balance there forever, which no
/// real robot does. Which way it then falls is arbitrary (forward).
pub const NUDGE_RAD_S: f32 = 0.3;

/// Drops the robot on an empty arena floor, unpowered, nudges it, and lets
/// it settle for `seconds`.
pub fn settle(
    model: &RobotModel,
    mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
    seconds: f32,
) -> Result<Rest, SpawnError> {
    let mut world = Sim::empty_world(-GRAVITY);
    let floor = add_arena(&mut world);
    let robot = spawn_robot(&mut world, model, mesh, 0.0, 0.0, 0.0)?;
    world.bodies[robot.bodies[0]].set_angvel(Vector::new(0.0, NUDGE_RAD_S, 0.0), true);
    for _ in 0..(seconds * PHYSICS_HZ as f32) as usize {
        world.step();
    }
    let chassis = &world.bodies[robot.bodies[0]];
    let up = *chassis.rotation() * Vector::Z;
    let mass: f32 = robot.bodies.iter().map(|&h| world.bodies[h].mass()).sum();
    let com = robot
        .bodies
        .iter()
        .map(|&h| world.bodies[h].center_of_mass() * world.bodies[h].mass())
        .sum::<Vector>()
        / mass;
    let touching_handles: Vec<RigidBodyHandle> = world
        .narrow_phase
        .contact_pairs_with(floor)
        .filter(|p| p.has_any_active_contact())
        .filter_map(|p| {
            let other = if p.collider1 == floor {
                p.collider2
            } else {
                p.collider1
            };
            world.colliders[other].parent()
        })
        .collect();
    let touching = robot
        .bodies
        .iter()
        .zip(&model.bodies)
        .filter(|(h, _)| touching_handles.contains(h))
        .map(|(_, b)| b.name.clone())
        .collect();
    Ok(Rest {
        tilt_deg: up.z.clamp(-1.0, 1.0).acos().to_degrees(),
        com_height_m: com.z,
        touching,
        residual_speed: chassis.linvel().length(),
    })
}
