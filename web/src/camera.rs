//! The driver's-eye view (docs/drivesim.md: decided 2026-10-08, the only
//! view in v1 by the user's choice 2026-10-09): fixed outside the cage at
//! standing eye height, as at an event.

use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use drivesim_physics::arena::ARENA_SIDE_M;

/// Standing eye height, m, and how far behind the wall the driver stands.
pub const EYE_HEIGHT_M: f32 = 1.7;
/// 1.8 m, moved back from 1.0 (user, 2026-10-10): at 1.0 the bottom of the
/// view met the floor 0.24 m inside the near wall, so a robot driven up to
/// it dropped out of sight. At 1.8 the whole floor is in view.
pub const BEHIND_WALL_M: f32 = 1.8;
/// Vertical field of view, degrees: 58, widened from 50 so a little more
/// of the cage is in view (user, 2026-10-10: "zoom out ... a little").
pub const FOV_DEG: f32 = 58.0;

/// Behind the -x wall, looking at the cage's centre (rapier coordinates).
pub fn spawn_camera(commands: &mut Commands, root: Entity) {
    let x = -ARENA_SIDE_M / 2.0 - BEHIND_WALL_M;
    let cam = commands
        .spawn((
            Camera3d::default(),
            // Bloom, so the impact marks glow (user, 2026-10-11: "make the
            // impact marks glow more"). It needs (and adds) an HDR target.
            Bloom::NATURAL,
            Projection::Perspective(PerspectiveProjection {
                fov: FOV_DEG.to_radians(),
                ..default()
            }),
            Transform::from_xyz(x, 0.0, EYE_HEIGHT_M).looking_at(Vec3::new(0.0, 0.0, 0.1), Vec3::Z),
        ))
        .id();
    commands.entity(root).add_child(cam);
}
