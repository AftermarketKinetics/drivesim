//! The arena: NHRL's 12 lb cage (also its 30 lb cage) and its four walls
//! (docs/drivesim.md §6). Source: the NHRL wiki, "NHRL
//! Facilities" and "Cage Design" (both last edited 2025-08), read
//! 2026-10-09: 16' x 16' usable floor of plywood painted with dead-flat
//! paint, double-walled polycarbonate walls on a 6x6 timber skirt, a roof.

use rapier3d::prelude::*;

/// Inside width of the square arena, m: 16 ft (NHRL wiki).
pub const ARENA_SIDE_M: f32 = 16.0 * 0.3048;
/// The wiki gives no wall height; the cage is roofed, so the walls only
/// need to be taller than any robot can climb. PROVISIONAL. The roof sits
/// on top of them.
pub const WALL_H_M: f32 = 2.4;
/// Floor friction factor; colliders combine friction by multiplying, so a
/// tyre's robot.toml `mu` is its grip on this floor (painted plywood).
pub const FLOOR_MU: f32 = 1.0;

/// The floor, four walls and a roof on them, so nothing thrown can leave
/// (user, 2026-10-10). Returns the floor's collider, for contact queries.
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
    w.colliders.insert(
        ColliderBuilder::cuboid(half + 2.0 * t, half + 2.0 * t, t)
            .translation(Vector::new(0.0, 0.0, WALL_H_M + t))
            .friction(0.3),
    );
    floor
}
