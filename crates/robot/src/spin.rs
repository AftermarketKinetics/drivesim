//! A weapon's spin direction, from robot.toml's `spin` (docs/drivesim.md
//! §5). Positive weapon duty spins a rotor positively about its
//! `axis_dir`, so choosing the direction means choosing that vector's
//! sign. The importer's sign is arbitrary (largest component positive),
//! and it left the fixture's vertical disc spinning front edge down, which
//! threw the robot itself into the air on every hit (2026-10-10).

/// Which way `spin` asks for, in the robot frame (x forward, y left, z
/// up). The default for each kind of weapon:
/// - axis across the robot (a vertical disc, a drum): `"front-up"`, the
///   front edge moving up, which throws the opponent up and the attacker
///   down;
/// - axis upright (a horizontal spinner): `"ccw"` seen from above.
pub const VERTICAL_DEFAULT: &str = "front-up";
pub const HORIZONTAL_DEFAULT: &str = "ccw";

/// `dir` (a unit axis in the robot frame) signed so that positive spin
/// about it is `spin`. Errors name what the axis allows.
pub fn spin_axis(dir: [f64; 3], spin: Option<&str>) -> Result<[f64; 3], String> {
    let flipped = [-dir[0], -dir[1], -dir[2]];
    let upright = dir[2].abs() >= 0.5;
    if upright {
        // Positive spin about +z is counter-clockwise seen from above.
        let want_up = match spin.unwrap_or(HORIZONTAL_DEFAULT) {
            "ccw" => true,
            "cw" => false,
            other => {
                return Err(format!(
                    "spin {other:?}: a weapon on an upright axis spins \"cw\" or \"ccw\" (seen from above)"
                ));
            }
        };
        return Ok(if (dir[2] > 0.0) == want_up {
            dir
        } else {
            flipped
        });
    }
    if dir[1].abs() < 0.5 {
        return Err(
            "its axis points along the robot, so neither \"front-up\" nor \"cw\" applies".into(),
        );
    }
    // A point at the rotor's front (+x of the axis) moves along dir × x,
    // whose z component is -dir.y: front-up needs dir.y < 0.
    let want_front_up = match spin.unwrap_or(VERTICAL_DEFAULT) {
        "front-up" => true,
        "front-down" => false,
        other => {
            return Err(format!(
                "spin {other:?}: a weapon on a cross axis spins \"front-up\" or \"front-down\""
            ));
        }
    };
    Ok(if (dir[1] < 0.0) == want_front_up {
        dir
    } else {
        flipped
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vertical_disc_defaults_to_its_front_edge_moving_up() {
        assert_eq!(spin_axis([0.0, 1.0, 0.0], None), Ok([-0.0, -1.0, -0.0]));
        assert_eq!(
            spin_axis([0.0, -1.0, 0.0], Some("front-up")),
            Ok([0.0, -1.0, 0.0])
        );
        assert_eq!(
            spin_axis([0.0, -1.0, 0.0], Some("front-down")),
            Ok([-0.0, 1.0, -0.0])
        );
    }

    #[test]
    fn a_horizontal_spinner_defaults_to_ccw_from_above() {
        assert_eq!(spin_axis([0.0, 0.0, 1.0], None), Ok([0.0, 0.0, 1.0]));
        assert_eq!(
            spin_axis([0.0, 0.0, 1.0], Some("cw")),
            Ok([-0.0, -0.0, -1.0])
        );
    }

    #[test]
    fn a_direction_that_does_not_fit_the_axis_is_an_error() {
        assert!(spin_axis([0.0, 1.0, 0.0], Some("cw")).is_err());
        assert!(spin_axis([0.0, 0.0, 1.0], Some("front-up")).is_err());
        assert!(spin_axis([1.0, 0.0, 0.0], None).is_err());
    }
}
