//! Draws NHRL's 12 lb cage (physics `arena.rs`): the painted plywood
//! floor, polycarbonate walls, and the target box (synced like a body).

use crate::robot_render::BodyLink;
use bevy::prelude::*;
use drivesim_physics::arena::{ARENA_SIDE_M, BOX_HALF_M, WALL_H_M};
use rapier3d::prelude::RigidBodyHandle;

/// Floor, walls and lights, under `root` (rapier coordinates, z up).
pub fn spawn_arena(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    let s = ARENA_SIDE_M;
    let floor = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(s, s, 0.02))),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::srgb(0.32, 0.33, 0.34), // dead-flat grey paint
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, -0.01),
        ))
        .id();
    commands.entity(root).add_child(floor);
    let wall_mat = mats.add(StandardMaterial {
        base_color: Color::srgba(0.75, 0.85, 0.95, 0.12),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        ..default()
    });
    let t = 0.03;
    for (x, y, sx, sy) in [
        (s / 2.0 + t / 2.0, 0.0, t, s),
        (-s / 2.0 - t / 2.0, 0.0, t, s),
        (0.0, s / 2.0 + t / 2.0, s, t),
        (0.0, -s / 2.0 - t / 2.0, s, t),
    ] {
        let w = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(sx, sy, WALL_H_M))),
                MeshMaterial3d(wall_mat.clone()),
                Transform::from_xyz(x, y, WALL_H_M / 2.0),
            ))
            .id();
        commands.entity(root).add_child(w);
    }
    let sun = commands
        .spawn((
            DirectionalLight {
                shadow_maps_enabled: true,
                illuminance: 9000.0,
                ..default()
            },
            Transform::from_xyz(-1.5, -2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Z),
        ))
        .id();
    commands.entity(root).add_child(sun);
}

pub fn spawn_target(
    commands: &mut Commands,
    root: Entity,
    handle: RigidBodyHandle,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    let [hx, hy, hz] = BOX_HALF_M;
    let e = commands
        .spawn((
            BodyLink(handle),
            Mesh3d(meshes.add(Cuboid::new(2.0 * hx, 2.0 * hy, 2.0 * hz))),
            MeshMaterial3d(mats.add(Color::srgb(0.85, 0.55, 0.15))),
            Transform::default(),
        ))
        .id();
    commands.entity(root).add_child(e);
}
