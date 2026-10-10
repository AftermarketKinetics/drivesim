//! The driver's-eye view (docs/drivesim.md: decided 2026-10-08, the only
//! view in v1 by the user's choice 2026-10-09): fixed outside the cage at
//! standing eye height, as at an event.

use bevy::prelude::*;
use drivesim_physics::arena::ARENA_SIDE_M;

/// Standing eye height, m, and how far behind the wall the driver stands.
pub const EYE_HEIGHT_M: f32 = 1.7;
pub const BEHIND_WALL_M: f32 = 1.0;

/// Behind the -x wall, looking at the cage's centre (rapier coordinates).
pub fn spawn_camera(commands: &mut Commands, root: Entity) {
    let x = -ARENA_SIDE_M / 2.0 - BEHIND_WALL_M;
    let cam = commands
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 50f32.to_radians(),
                ..default()
            }),
            Transform::from_xyz(x, 0.0, EYE_HEIGHT_M).looking_at(Vec3::new(0.0, 0.0, 0.1), Vec3::Z),
        ))
        .id();
    commands.entity(root).add_child(cam);
}
