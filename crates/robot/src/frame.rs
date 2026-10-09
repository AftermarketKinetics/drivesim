//! The STEP's axes -> the robot frame (x forward, y left, z up). Generic
//! STEPs disagree on which way is up; robot.toml says.

use crate::math::{M3, cross};

/// Parses `"+x"`, `"-z"` and so on into a unit vector.
pub fn axis(s: &str) -> Option<[f64; 3]> {
    let (sign, rest) = match s.trim().as_bytes() {
        [b'+', r @ ..] => (1.0, r),
        [b'-', r @ ..] => (-1.0, r),
        r => (1.0, r),
    };
    let i = match rest {
        [b'x'] | [b'X'] => 0,
        [b'y'] | [b'Y'] => 1,
        [b'z'] | [b'Z'] => 2,
        _ => return None,
    };
    let mut v = [0.0; 3];
    v[i] = sign;
    Some(v)
}

/// Rows are the robot's x, y, z expressed in STEP axes, so
/// `mat_vec(&r, p_step)` is the point in the robot frame. `None` unless
/// `up` and `forward` are perpendicular.
pub fn step_to_robot(up: &str, forward: &str) -> Option<M3> {
    let (z, x) = (axis(up)?, axis(forward)?);
    if crate::math::dot(x, z) != 0.0 {
        return None;
    }
    Some([x, cross(z, x), z])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::mat_vec;

    #[test]
    fn z_up_x_forward_is_identity() {
        let r = step_to_robot("+z", "+x").unwrap();
        assert_eq!(r, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    #[test]
    fn y_up_cad_is_turned_upright() {
        // A Y-up export with the robot facing -z.
        let r = step_to_robot("+y", "-z").unwrap();
        assert_eq!(mat_vec(&r, [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
        assert_eq!(mat_vec(&r, [0.0, 0.0, -1.0]), [1.0, 0.0, 0.0]);
        // Left is up × forward, so the frame stays right-handed.
        assert_eq!(mat_vec(&r, [-1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn parallel_or_garbage_axes_are_rejected() {
        assert!(step_to_robot("+z", "-z").is_none());
        assert!(step_to_robot("up", "+x").is_none());
    }
}
