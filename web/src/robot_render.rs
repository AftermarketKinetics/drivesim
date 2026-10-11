//! Draws the robot and its opponent copy where the physics puts them.
//!
//! Every sim entity is a child of `SimRoot`, which turns rapier's z-up
//! world into Bevy's y-up one, so each body's Transform is its rapier pose
//! unchanged. Bodies share the robot frame at spawn (physics `spawn.rs`), so
//! mesh vertices in the robot frame are body-local.

use crate::bundle_load::Bundle;
use crate::sim_step::SimRes;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use drivesim_physics::spawn::RobotHandles;
use drivesim_robot::math::mat_vec;
use drivesim_robot::stl::parse_stl;
use rapier3d::prelude::RigidBodyHandle;

/// Rapier world (z up) -> Bevy world (y up).
#[derive(Component)]
pub struct SimRoot;

/// An entity drawn at a rapier body's pose.
#[derive(Component)]
pub struct BodyLink(pub RigidBodyHandle);

const DEFAULT_COLOR: [f32; 3] = [0.55, 0.57, 0.6];

/// Wheels are plain dark tyres, never tinted. Their striped sectors went
/// (user, 2026-10-11: they clashed with the impact marks).
const TYRE: [f32; 3] = [0.07, 0.07, 0.08];

/// A flat-shaded mesh from STL bytes, vertices in the robot frame.
fn part_mesh(bytes: &[u8], r: &drivesim_robot::math::M3) -> Option<Mesh> {
    let tris = parse_stl(bytes)?;
    let mut pos = Vec::with_capacity(tris.len() * 3);
    let mut nrm = Vec::with_capacity(tris.len() * 3);
    for t in &tris {
        let p = t.map(|v| mat_vec(r, v.map(f64::from)).map(|c| c as f32));
        let (a, b, c) = (Vec3::from(p[0]), Vec3::from(p[1]), Vec3::from(p[2]));
        let n = (b - a).cross(c - a).normalize_or_zero();
        pos.extend_from_slice(&p);
        nrm.extend([n.to_array(); 3]);
    }
    let n = pos.len() as u32;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nrm)
    .with_inserted_indices(Indices::U32((0..n).collect()));
    Some(mesh)
}

/// One robot's visuals: `robot` is either the driven robot or the
/// opponent copy, both of `model`. Wheels are `TYRE`; with `tint`, every
/// other part takes that colour instead of its CAD colour.
#[allow(clippy::too_many_arguments)]
pub fn spawn_robot_visuals(
    commands: &mut Commands,
    root: Entity,
    bundle: &Bundle,
    model: &drivesim_robot::RobotModel,
    robot: &RobotHandles,
    tint: Option<[f32; 3]>,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    for (i, (body, handle)) in model.bodies.iter().zip(&robot.bodies).enumerate() {
        let e = commands
            .spawn((
                BodyLink(*handle),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        commands.entity(root).add_child(e);
        let wheel = model.wheels.iter().any(|w| w.body == i);
        for (part, path) in body.parts.iter().zip(&body.meshes) {
            let Some(mesh) = bundle
                .meshes
                .get(path)
                .and_then(|b| part_mesh(b, &model.step_to_robot))
            else {
                continue;
            };
            let [r, g, b] = if wheel {
                TYRE
            } else if let Some(t) = tint {
                t
            } else {
                bundle
                    .part_colors
                    .get(part)
                    .copied()
                    .flatten()
                    .unwrap_or(DEFAULT_COLOR)
            };
            let child = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(mats.add(StandardMaterial {
                        base_color: Color::srgb(r, g, b),
                        perceptual_roughness: 0.6,
                        metallic: 0.2,
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    })),
                ))
                .id();
            commands.entity(e).add_child(child);
        }
    }
}

pub fn sync_bodies(sim: Option<Res<SimRes>>, mut q: Query<(&BodyLink, &mut Transform)>) {
    let Some(sim) = sim else { return };
    for (link, mut t) in &mut q {
        let Some(b) = sim.sim.world.bodies.get(link.0) else {
            continue;
        };
        let p = b.translation();
        let r = b.rotation();
        t.translation = Vec3::new(p.x, p.y, p.z);
        t.rotation = Quat::from_xyzw(r.x, r.y, r.z, r.w);
    }
}
