//! The arena: a steel floor, four walls and the free target box
//! (docs/drivesim.md §6).

use rapier3d::prelude::*;

/// Inside width of the square arena, m. PROVISIONAL: 16 ft, Momentum's
/// placeholder; the NHRL 12 lb cage size is an open question (§10).
pub const ARENA_SIDE_M: f32 = 4.88;
pub const WALL_H_M: f32 = 0.6;
/// Floor friction; colliders combine friction by multiplying, so a tyre's
/// `mu` against this floor is exactly its robot.toml value.
pub const FLOOR_MU: f32 = 1.0;

/// The target: 12 lb, 300 × 300 × 150 mm.
pub const BOX_MASS_KG: f32 = 5.44;
pub const BOX_HALF_M: [f32; 3] = [0.15, 0.15, 0.075];
pub const BOX_MU: f32 = 0.4;

/// Returns the floor's collider, for contact queries.
pub fn add_arena(w: &mut PhysicsWorld) -> ColliderHandle {
    let half = ARENA_SIDE_M / 2.0;
    let floor = w.colliders.insert(
        ColliderBuilder::cuboid(half + 0.5, half + 0.5, 0.05)
            .translation(Vector::new(0.0, 0.0, -0.05))
            .friction(FLOOR_MU)
            .friction_combine_rule(CoefficientCombineRule::Multiply),
    );
    let t = 0.05;
    for (x, y, hx, hy) in [
        (half + t, 0.0, t, half + 2.0 * t),
        (-half - t, 0.0, t, half + 2.0 * t),
        (0.0, half + t, half + 2.0 * t, t),
        (0.0, -half - t, half + 2.0 * t, t),
    ] {
        w.colliders.insert(
            ColliderBuilder::cuboid(hx, hy, WALL_H_M / 2.0)
                .translation(Vector::new(x, y, WALL_H_M / 2.0))
                .friction(0.3),
        );
    }
    floor
}

/// The free box, resting on the floor at (`x`, `y`).
pub fn add_target(w: &mut PhysicsWorld, x: f32, y: f32) -> RigidBodyHandle {
    let b = w
        .bodies
        .insert(RigidBodyBuilder::dynamic().translation(Vector::new(x, y, BOX_HALF_M[2])));
    let [hx, hy, hz] = BOX_HALF_M;
    w.colliders.insert_with_parent(
        ColliderBuilder::cuboid(hx, hy, hz)
            .mass(BOX_MASS_KG)
            .friction(BOX_MU)
            .friction_combine_rule(CoefficientCombineRule::Multiply),
        b,
        &mut w.bodies,
    );
    b
}
