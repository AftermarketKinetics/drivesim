//! The arena: NHRL's 12 lb cage (also its 30 lb cage), four walls and the
//! free target box (docs/drivesim.md §6). Source: the NHRL wiki, "NHRL
//! Facilities" and "Cage Design" (both last edited 2025-08), read
//! 2026-10-09: 16' x 16' usable floor of plywood painted with dead-flat
//! paint, double-walled polycarbonate walls on a 6x6 timber skirt, a roof.

use rapier3d::prelude::*;

/// Inside width of the square arena, m: 16 ft (NHRL wiki).
pub const ARENA_SIDE_M: f32 = 16.0 * 0.3048;
/// The wiki gives no wall height; the cage is roofed, so the walls only
/// need to be taller than any robot can climb. PROVISIONAL.
pub const WALL_H_M: f32 = 2.4;
/// Floor friction factor; colliders combine friction by multiplying, so a
/// tyre's robot.toml `mu` is its grip on this floor (painted plywood).
pub const FLOOR_MU: f32 = 1.0;

/// The target: 300 × 300 × 150 mm, as heavy as the robot driving it
/// (user, 2026-10-10), so pushing it is like pushing an opponent of the
/// same weight class.
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
