//! Draws the robot and the target box where the physics puts them.
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

/// Wheel markings: the tyre in this many sectors about its axis,
/// alternating `TYRE` and `STRIPE`, so a wheel is seen to roll, spin or
/// lock. Plain cylinders looked the same turning or not, and rolling read
/// as sliding (user, 2026-10-10).
const WHEEL_SECTORS: usize = 6;
const TYRE: [f32; 3] = [0.07, 0.07, 0.08];
const STRIPE: [f32; 3] = [0.95, 0.75, 0.15];

/// A rotor's axis in the robot frame, for marking it.
pub struct Axis {
    pub point: Vec3,
    pub dir: Vec3,
}

/// A flat-shaded mesh from STL bytes, vertices in the robot frame. With
/// `marked`, each triangle is coloured by the sector its centroid lies in
/// about that axis (as vertex colours, which multiply the material's).
fn part_mesh(bytes: &[u8], r: &drivesim_robot::math::M3, marked: Option<&Axis>) -> Option<Mesh> {
    let tris = parse_stl(bytes)?;
    let mut pos = Vec::with_capacity(tris.len() * 3);
    let mut nrm = Vec::with_capacity(tris.len() * 3);
    let mut col: Vec<[f32; 4]> = Vec::new();
    // Two directions across the axis, to measure each centroid's angle.
    let across = marked.map(|a| {
        let u = a.dir.any_orthonormal_vector();
        (u, a.dir.cross(u))
    });
    for t in &tris {
        let p = t.map(|v| mat_vec(r, v.map(f64::from)).map(|c| c as f32));
        let (a, b, c) = (Vec3::from(p[0]), Vec3::from(p[1]), Vec3::from(p[2]));
        let n = (b - a).cross(c - a).normalize_or_zero();
        pos.extend_from_slice(&p);
        nrm.extend([n.to_array(); 3]);
        if let (Some(axis), Some((u, w))) = (marked, across) {
            let d = (a + b + c) / 3.0 - axis.point;
            let angle = d.dot(w).atan2(d.dot(u)).rem_euclid(std::f32::consts::TAU);
            let sector = (angle / std::f32::consts::TAU * WHEEL_SECTORS as f32) as usize;
            let [r, g, b] = if sector.is_multiple_of(2) { TYRE } else { STRIPE };
            col.extend([[r, g, b, 1.0]; 3]);
        }
    }
    let n = pos.len() as u32;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nrm)
    .with_inserted_indices(Indices::U32((0..n).collect()));
    if !col.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
    }
    Some(mesh)
}

pub fn spawn_robot_visuals(
    commands: &mut Commands,
    root: Entity,
    bundle: &Bundle,
    sim: &SimRes,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    let model = &sim.sim.model;
    for (i, (body, handle)) in model.bodies.iter().zip(&sim.sim.robot.bodies).enumerate() {
        let e = commands
            .spawn((
                BodyLink(*handle),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        commands.entity(root).add_child(e);
        let wheel = model.wheels.iter().find(|w| w.body == i).map(|w| Axis {
            point: Vec3::from(w.axis_point.map(|c| c as f32)),
            dir: Vec3::from(w.axis_dir.map(|c| c as f32)).normalize(),
        });
        for (part, path) in body.parts.iter().zip(&body.meshes) {
            let Some(mesh) = bundle
                .meshes
                .get(path)
                .and_then(|b| part_mesh(b, &model.step_to_robot, wheel.as_ref()))
            else {
                continue;
            };
            // A marked wheel takes its colours from the mesh alone.
            let [r, g, b] = if wheel.is_some() {
                [1.0; 3]
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
