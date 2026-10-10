//! The torques a lumped reflected inertia leaves out (docs/drivesim.md §6).
//!
//! `spawn.rs` gives each rotor body its motor's inertia reflected through
//! the gearing, `J·r²`, so the rotor responds to motor torque as a real
//! gearmotor's output does. But a lump on the output also takes the
//! motor rotor's reaction as if it all came back through the output.
//! The motor rotor actually turns at `r·ω_rotor − (r−1)·ω_chassis`:
//!
//! ```text
//! KE_real  = ½ J (r ω_w − (r−1) ω_c)²
//! KE_lump  = ½ J r² ω_w²
//! ```
//!
//! The difference, as torques about the axis, is what this returns. It
//! is applied each physics step from the previous step's accelerations.

/// Torques (on the rotor body, on the chassis), N m, about the rotor's
/// axis, that turn the lumped `J·r²` into a real gearmotor. `alpha_rotor`
/// and `alpha_chassis` are the absolute angular accelerations of the two
/// bodies about that axis, rad/s².
pub fn gear_coupling(j_motor: f32, ratio: f32, alpha_rotor: f32, alpha_chassis: f32) -> (f32, f32) {
    let (r, j) = (ratio, j_motor);
    let cross = r * (r - 1.0) * j;
    (
        cross * alpha_chassis,
        cross * alpha_rotor - (r - 1.0) * (r - 1.0) * j * alpha_chassis,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rigid_rotation_costs_only_the_motor_rotor() {
        // Everything turning together: the motor rotor turns with it at
        // the same rate, so the whole robot carries J, not J r².
        let (j, r, a) = (5e-6, 22.5, 100.0);
        let (on_rotor, on_chassis) = gear_coupling(j, r, a, a);
        let lump = j * r * r * a;
        assert!(((lump - on_rotor - on_chassis) / (j * a) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn spinning_up_against_a_still_chassis_reacts_r_minus_1_of_r_less() {
        // A wheel spun up with the chassis held: the lump puts all of
        // J r² α back on the chassis, the real gearbox only J r α.
        let (j, r, a) = (5e-6, 22.5, 1000.0);
        let (on_rotor, on_chassis) = gear_coupling(j, r, a, 0.0);
        assert_eq!(on_rotor, 0.0);
        assert!(((j * r * r * a - on_chassis) / (j * r * a) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn direct_drive_needs_no_correction() {
        assert_eq!(gear_coupling(5e-6, 1.0, 50.0, 20.0), (0.0, 0.0));
    }
}
