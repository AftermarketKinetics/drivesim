//! Draws the box opponent where the physics puts it.

use crate::robot_render::BodyLink;
use bevy::prelude::*;
use drivesim_physics::target::BOX_HALF_M;
use rapier3d::prelude::RigidBodyHandle;

/// The box, `color`, as a child of `root` linked to its body.
pub fn spawn_target_visual(
    commands: &mut Commands,
    root: Entity,
    handle: RigidBodyHandle,
    color: [f32; 3],
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) {
    let [hx, hy, hz] = BOX_HALF_M;
    let [r, g, b] = color;
    let e = commands
        .spawn((
            BodyLink(handle),
            Mesh3d(meshes.add(Cuboid::new(2.0 * hx, 2.0 * hy, 2.0 * hz))),
            MeshMaterial3d(mats.add(Color::srgb(r, g, b))),
            Transform::default(),
        ))
        .id();
    commands.entity(root).add_child(e);
}
