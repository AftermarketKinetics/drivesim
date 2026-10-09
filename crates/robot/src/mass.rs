//! Rigid-body mass properties and combining parts into one body.

use crate::math::{M3, V3, ZERO, add, mat_add, parallel_axis, scale, sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MassProps {
    /// kg.
    pub mass: f64,
    /// Centre of mass, m.
    pub com: V3,
    /// Inertia about `com`, kg m², same axes as `com`.
    pub inertia: M3,
}

impl MassProps {
    pub const NONE: MassProps = MassProps {
        mass: 0.0,
        com: ZERO,
        inertia: [[0.0; 3]; 3],
    };
}

/// One body from many: the CoM is the mass-weighted mean, and each part's
/// inertia moves to it by the parallel-axis theorem.
pub fn combine(parts: &[MassProps]) -> MassProps {
    let mass: f64 = parts.iter().map(|p| p.mass).sum();
    if mass <= 0.0 {
        return MassProps::NONE;
    }
    let com = scale(
        parts
            .iter()
            .fold(ZERO, |acc, p| add(acc, scale(p.com, p.mass))),
        1.0 / mass,
    );
    let inertia = parts.iter().fold([[0.0; 3]; 3], |acc, p| {
        mat_add(
            &acc,
            &mat_add(&p.inertia, &parallel_axis(p.mass, sub(p.com, com))),
        )
    });
    MassProps { mass, com, inertia }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(m: f64, at: V3) -> MassProps {
        MassProps {
            mass: m,
            com: at,
            inertia: [[0.0; 3]; 3],
        }
    }

    #[test]
    fn two_point_masses_make_a_dumbbell() {
        let b = combine(&[point(1.0, [1.0, 0.0, 0.0]), point(1.0, [-1.0, 0.0, 0.0])]);
        assert_eq!(b.mass, 2.0);
        assert_eq!(b.com, [0.0, 0.0, 0.0]);
        // m r² about y and z, nothing about the bar's own axis.
        assert_eq!(b.inertia[0][0], 0.0);
        assert_eq!(b.inertia[1][1], 2.0);
        assert_eq!(b.inertia[2][2], 2.0);
    }

    #[test]
    fn off_diagonal_terms_appear_for_skewed_layouts() {
        let b = combine(&[point(1.0, [1.0, 1.0, 0.0]), point(1.0, [-1.0, -1.0, 0.0])]);
        assert_eq!(b.inertia[0][1], -2.0);
    }

    #[test]
    fn nothing_is_nothing() {
        assert_eq!(combine(&[]), MassProps::NONE);
    }
}
