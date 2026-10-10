//! Draws NHRL's 12 lb cage (physics `arena.rs`): the painted plywood
//! floor, polycarbonate walls with distance markings, and overhead lights.

use crate::floor_texture::floor_image;
use crate::wall_markings::spawn_wall_markings;
use bevy::prelude::*;
use drivesim_physics::arena::{ARENA_SIDE_M, WALL_H_M};

/// Floor, walls and overhead lights, under `root` (rapier coordinates, z up).
pub fn spawn_arena(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let s = ARENA_SIDE_M;
    let floor = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(s, s, 0.02))),
            MeshMaterial3d(mats.add(StandardMaterial {
                // Dead-flat grey paint over plywood sheets (floor_texture.rs).
                base_color_texture: Some(images.add(floor_image())),
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
    spawn_overhead_lights(commands, root, meshes, mats);
    spawn_wall_markings(commands, root, meshes, mats);
}

/// Overhead lights height above the floor, m: over the cage roof.
pub const LIGHT_HEIGHT_M: f32 = 3.5;
/// A 3 × 3 grid over the cage.
const LIGHT_GRID: usize = 3;
/// Luminous power per lamp, lm. 150,000 (about 1,000 lx straight below
/// each) left the cage "really dark" (user, 2026-10-10).
const LIGHT_LUMENS: f32 = 600_000.0;
/// A light truss is many lamps, which together act like even light from
/// straight above: a downward directional light, lx, with no shadows.
const TRUSS_LUX: f32 = 6_000.0;
/// Fill, cd/m², so faces the overhead light only grazes (the robot's and
/// the box's sides, all the driver sees of them) are not black.
const AMBIENT_BRIGHTNESS: f32 = 600.0;

/// The cage's overhead lighting, as at an event: a grid of lamps above the
/// roof, each drawn as a small glowing panel. No shadow maps (user,
/// 2026-10-10).
fn spawn_overhead_lights(
    commands: &mut Commands,
    root: Entity,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    let panel = meshes.add(Cuboid::new(0.6, 0.3, 0.04));
    let glow = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(8.0, 8.0, 7.5),
        unlit: true,
        ..default()
    });
    commands.insert_resource(GlobalAmbientLight {
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });
    let truss = commands
        .spawn((
            DirectionalLight {
                illuminance: TRUSS_LUX,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, LIGHT_HEIGHT_M).looking_at(Vec3::ZERO, Vec3::X),
        ))
        .id();
    commands.entity(root).add_child(truss);
    let pitch = ARENA_SIDE_M / LIGHT_GRID as f32;
    for i in 0..LIGHT_GRID {
        for j in 0..LIGHT_GRID {
            let x = -ARENA_SIDE_M / 2.0 + pitch * (i as f32 + 0.5);
            let y = -ARENA_SIDE_M / 2.0 + pitch * (j as f32 + 0.5);
            let lamp = commands
                .spawn((
                    PointLight {
                        intensity: LIGHT_LUMENS,
                        range: 12.0,
                        radius: 0.2,
                        shadow_maps_enabled: false,
                        ..default()
                    },
                    Mesh3d(panel.clone()),
                    MeshMaterial3d(glow.clone()),
                    Transform::from_xyz(x, y, LIGHT_HEIGHT_M),
                ))
                .id();
            commands.entity(root).add_child(lamp);
        }
    }
}
