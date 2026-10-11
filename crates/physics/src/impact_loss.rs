//! Energy lost in impacts (user, 2026-10-11: "more energy lost to
//! destruction/heat/sound"): after a physics step in which a body took an
//! impact, it keeps only `KEEP` of the velocity change the step gave it.
//! Rapier can't cap a contact's impulse, so the loss is taken afterwards.

use rapier3d::prelude::*;
use std::collections::HashMap;

/// The fraction of an impact's velocity change a body keeps (user,
/// 2026-10-11: "everything 50%, floor exempt").
pub const KEEP: f32 = 0.5;
/// A contact is an impact when its impulse alone would accelerate the
/// body's group above this, m/s². Driving and shoving stay under ~1 g, so
/// they are untouched.
pub const IMPACT_ACCEL: f32 = 5.0 * 9.81;

/// Bodies that move together (a robot: chassis, wheels, weapons) and so
/// share one verdict, and the rotors whose spin about `axis` (unit, in
/// the first body's frame) is left alone: a weapon still loses all the
/// speed the hit took from it.
pub struct Group {
    pub bodies: Vec<RigidBodyHandle>,
    pub rotors: Vec<(RigidBodyHandle, Vector)>,
}

/// Each group's bodies' (linvel, angvel), taken before a step.
pub type Velocities = Vec<Vec<(Vector, Vector)>>;

pub fn snapshot(world: &PhysicsWorld, groups: &[Group]) -> Velocities {
    groups
        .iter()
        .map(|g| {
            g.bodies
                .iter()
                .map(|h| (world.bodies[*h].linvel(), world.bodies[*h].angvel()))
                .collect()
        })
        .collect()
}

/// After the step: every group that took an impact, from anything but
/// `floor` and itself, keeps `KEEP` of its bodies' velocity change.
pub fn absorb(
    world: &mut PhysicsWorld,
    groups: &[Group],
    before: &Velocities,
    floor: ColliderHandle,
) {
    let mut group_of = HashMap::new();
    for (i, g) in groups.iter().enumerate() {
        for h in &g.bodies {
            group_of.insert(*h, i);
        }
    }
    let group = |c: ColliderHandle| {
        world.colliders[c]
            .parent()
            .and_then(|b| group_of.get(&b).copied())
    };
    let mut impulse = vec![0.0f32; groups.len()];
    for pair in world.narrow_phase.contact_pairs() {
        let (c1, c2) = (pair.collider1, pair.collider2);
        if c1 == floor || c2 == floor {
            continue;
        }
        let (g1, g2) = (group(c1), group(c2));
        if g1 == g2 {
            continue;
        }
        let j = pair.total_impulse_magnitude();
        for g in [g1, g2].into_iter().flatten() {
            impulse[g] += j;
        }
    }
    let dt = world.integration_parameters.dt;
    for (i, g) in groups.iter().enumerate() {
        let mass: f32 = g.bodies.iter().map(|h| world.bodies[*h].mass()).sum();
        if impulse[i] < mass * IMPACT_ACCEL * dt {
            continue;
        }
        let rot = *world.bodies[g.bodies[0]].rotation();
        for (h, (v0, w0)) in g.bodies.iter().zip(&before[i]) {
            let b = &mut world.bodies[*h];
            let (dv, dw) = (b.linvel() - *v0, b.angvel() - *w0);
            let spin = match g.rotors.iter().find(|(r, _)| r == h) {
                Some((_, axis)) => {
                    let a = rot * *axis;
                    a * dw.dot(a)
                }
                None => Vector::ZERO,
            };
            b.set_linvel(*v0 + dv * KEEP, false);
            b.set_angvel(*w0 + spin + (dw - spin) * KEEP, false);
        }
    }
}
