//! The box opponent: a free orange box, the driver's other choice besides
//! the robot's own copy (user, 2026-10-10).

use rapier3d::prelude::*;

/// 300 × 300 × 150 mm, the box the opponent copy replaced (commit 495eb8a).
pub const BOX_HALF_M: [f32; 3] = [0.15, 0.15, 0.075];
pub const BOX_MU: f32 = 0.4;

/// The free box of `mass_kg`, resting on the floor at (`x`, `y`).
pub fn add_target(w: &mut PhysicsWorld, x: f32, y: f32, mass_kg: f32) -> RigidBodyHandle {
    let b = w
        .bodies
        .insert(RigidBodyBuilder::dynamic().translation(Vector::new(x, y, BOX_HALF_M[2])));
    let [hx, hy, hz] = BOX_HALF_M;
    w.colliders.insert_with_parent(
        ColliderBuilder::cuboid(hx, hy, hz)
            .mass(mass_kg)
            .friction(BOX_MU)
            .friction_combine_rule(CoefficientCombineRule::Multiply),
        b,
        &mut w.bodies,
    );
    b
}
