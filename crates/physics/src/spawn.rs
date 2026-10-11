//! A `RobotModel` -> rapier bodies, colliders and revolute joints.
//!
//! Every body is created at the same pose, with its local frame equal to
//! the robot frame, so model coordinates (CoMs, axis points) are used
//! unchanged as local coordinates and joint anchors. Mass properties come
//! from the model (the importer's exact B-rep numbers) plus each rotor's
//! motor inertia reflected through its gearing; colliders carry no mass.

use crate::shapes::body_shapes;
use crate::weapon_reach::reaching_parts;
use drivesim_robot::math::mat_vec;
use drivesim_robot::stl::{Tri, parse_stl};
use drivesim_robot::{RobotModel, Side};
use rapier3d::prelude::*;
use std::fmt;

/// Chassis and weapon friction against the (μ = 1) floor.
pub const BODY_MU: f32 = 0.3;

pub struct RobotHandles {
    /// Parallel to `model.bodies`; `[0]` is the chassis.
    pub bodies: Vec<RigidBodyHandle>,
    pub wheel_joints: Vec<ImpulseJointHandle>,
    pub weapon_joints: Vec<ImpulseJointHandle>,
}

#[derive(Debug)]
pub enum SpawnError {
    MissingMesh(String),
    BadMesh(String),
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpawnError::MissingMesh(p) => write!(f, "mesh {p} is not in the bundle"),
            SpawnError::BadMesh(p) => write!(f, "mesh {p} is not a usable binary STL"),
        }
    }
}

/// The motor's own rotor as the driven body feels it, kg m², about the
/// rotor axis: `J_motor · ratio²`. At 22.5:1 a 2830's rotor outweighs a
/// 68 g wheel's inertia ~80 times; without it the wheel is so light that
/// the explicit motor torque (from last step's speed) overshoots and the
/// wheel's speed rings between the current limits.
pub fn reflected_inertia(rotor: &drivesim_robot::Rotor) -> f64 {
    rotor.motor.rotor_inertia_kg_m2 * rotor.ratio * rotor.ratio
}

pub fn v(a: [f64; 3]) -> Vector {
    Vector::new(a[0] as f32, a[1] as f32, a[2] as f32)
}

/// Spawns the robot with its lowest point on the floor at (`x`, `y`),
/// facing `yaw` (rad, CCW from +x). `mesh` fetches a bundle file by its
/// bundle-relative path.
pub fn spawn_robot(
    w: &mut PhysicsWorld,
    model: &RobotModel,
    mesh: &dyn Fn(&str) -> Option<Vec<u8>>,
    x: f32,
    y: f32,
    yaw: f32,
) -> Result<RobotHandles, SpawnError> {
    let r = model.step_to_robot;
    let wheel_of = |body: usize| model.wheels.iter().find(|wh| wh.body == body);

    // Every non-wheel body's meshes, in the robot frame.
    let mut tris: Vec<Vec<Vec<Tri>>> = Vec::new();
    for (i, body) in model.bodies.iter().enumerate() {
        let mut per = Vec::new();
        if wheel_of(i).is_none() {
            for path in &body.meshes {
                let bytes = mesh(path).ok_or_else(|| SpawnError::MissingMesh(path.clone()))?;
                let t = parse_stl(&bytes).ok_or_else(|| SpawnError::BadMesh(path.clone()))?;
                per.push(
                    t.iter()
                        .map(|tri| tri.map(|p| mat_vec(&r, p.map(f64::from)).map(|c| c as f32)))
                        .collect(),
                );
            }
        }
        tris.push(per);
    }

    // Rest the lowest point on the floor (wheels: their rim, assuming a
    // horizontal axle).
    let mesh_min = tris
        .iter()
        .flatten()
        .flatten()
        .flatten()
        .map(|p| p[2])
        .fold(f32::INFINITY, f32::min);
    let wheel_min = model
        .wheels
        .iter()
        .map(|wh| (wh.axis_point[2] - wh.radius_m) as f32)
        .fold(f32::INFINITY, f32::min);
    let lift = -mesh_min.min(wheel_min) + 5e-4;

    let mut bodies = Vec::new();
    for (i, body) in model.bodies.iter().enumerate() {
        let m = &body.mass;
        let mut inertia = Matrix::from_cols(v(m.inertia[0]), v(m.inertia[1]), v(m.inertia[2]));
        for rotor in model
            .wheels
            .iter()
            .chain(&model.weapons)
            .filter(|r| r.body == i)
        {
            let a = v(rotor.axis_dir).normalize();
            inertia +=
                Matrix::from_cols(a * a.x, a * a.y, a * a.z) * reflected_inertia(rotor) as f32;
        }
        let is_weapon = model.weapons.iter().any(|wp| wp.body == i);
        let handle = w.bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(Vector::new(x, y, lift))
                .rotation(Vector::new(0.0, 0.0, yaw))
                .additional_mass_properties(MassProperties::with_inertia_matrix(
                    v(m.com),
                    m.mass as f32,
                    inertia,
                ))
                .ccd_enabled(is_weapon)
                // A wheel body carries its motor's reflected inertia, which
                // overstates its angular momentum ~ratio times: keep that
                // out of the gyroscopic term (a real wheel's is negligible).
                .gyroscopic_forces_enabled(wheel_of(i).is_none()),
        );
        if let Some(wh) = wheel_of(i) {
            let axis = v(wh.axis_dir).normalize();
            w.colliders.insert_with_parent(
                ColliderBuilder::cylinder(wh.width_m as f32 / 2.0, wh.radius_m as f32)
                    .position(Pose::from_parts(
                        v(wh.axis_point),
                        Rotation::from_rotation_arc(Vector::Y, axis),
                    ))
                    .density(0.0)
                    .friction(wh.mu.unwrap_or(1.0) as f32)
                    .friction_combine_rule(CoefficientCombineRule::Multiply),
                handle,
                &mut w.bodies,
            );
        } else {
            // A weapon collides only through the parts that reach near its
            // outer radius (weapon_reach.rs); the rest stay in its mass.
            let reaching;
            let parts: &[Vec<Tri>] = match model.weapons.iter().find(|wp| wp.body == i) {
                Some(wp) => {
                    let (p, d) = (v(wp.axis_point), v(wp.axis_dir).normalize());
                    reaching = reaching_parts(&tris[i], [p.x, p.y, p.z], [d.x, d.y, d.z]);
                    &reaching
                }
                None => &tris[i],
            };
            for shape in body_shapes(parts) {
                w.colliders.insert_with_parent(
                    ColliderBuilder::new(shape)
                        .density(0.0)
                        .friction(BODY_MU)
                        .friction_combine_rule(CoefficientCombineRule::Multiply),
                    handle,
                    &mut w.bodies,
                );
            }
        }
        bodies.push(handle);
    }

    let mut joint = |rotor: &drivesim_robot::Rotor| {
        let p = v(rotor.axis_point);
        let j = RevoluteJointBuilder::new(v(rotor.axis_dir).normalize())
            .local_anchor1(p)
            .local_anchor2(p)
            .contacts_enabled(false);
        w.impulse_joints
            .insert(bodies[0], bodies[rotor.body], j, true)
    };
    let wheel_joints = model.wheels.iter().map(&mut joint).collect();
    let weapon_joints = model.weapons.iter().map(&mut joint).collect();
    Ok(RobotHandles {
        bodies,
        wheel_joints,
        weapon_joints,
    })
}

/// +1 if positive spin about the wheel's axis rolls the robot forward:
/// a contact under the hub moves the hub along `axis × z`.
pub fn forward_sign(wheel: &drivesim_robot::Rotor) -> f32 {
    if wheel.axis_dir[1] >= 0.0 { 1.0 } else { -1.0 }
}

pub fn side_of(wheel: &drivesim_robot::Rotor) -> Side {
    wheel.side.unwrap_or(Side::Left)
}
