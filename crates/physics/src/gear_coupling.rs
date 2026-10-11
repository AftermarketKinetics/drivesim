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

/// The largest the chassis self-term `(r−1)²J`, summed over a robot's
/// rotors, may be against the chassis's own inertia about a rotor axis.
/// The term is explicit (last step's acceleration), and an explicit added
/// inertia near the body's own can diverge: a 0.96 kg beetle on 27:1
/// drive motors sat at 1:1 and blew up with every motor unpowered
/// (2026-10-11); capped at 0.75 or 0.5 it was still. 0.25 left the 1 lb
/// house robot tilting 11° on spin-up (4.6° at 0.5). The 12 lb fixture is
/// at ~1:9 and unaffected.
pub const MAX_SELF_FRACTION: f32 = 0.5;

/// How much of the correction a robot can take stably: 1, or less when
/// its rotors' summed chassis self-term would pass `MAX_SELF_FRACTION` of
/// the chassis's inertia about any rotor's axis. `rotors` are (unit axis in
/// the chassis frame, motor J, ratio); `chassis` is the chassis inertia
/// about its CoM in the same frame.
pub fn coupling_scale(rotors: &[([f32; 3], f32, f32)], chassis: [[f32; 3]; 3]) -> f32 {
    let about = |d: [f32; 3]| -> f32 {
        (0..3)
            .map(|i| (0..3).map(|j| d[i] * chassis[i][j] * d[j]).sum::<f32>())
            .sum()
    };
    rotors
        .iter()
        .map(|(d, _, _)| {
            let added: f32 = rotors
                .iter()
                .map(|(e, j, r)| {
                    let c = d[0] * e[0] + d[1] * e[1] + d[2] * e[2];
                    (r - 1.0) * (r - 1.0) * j * c * c
                })
                .sum();
            if added > 0.0 {
                (MAX_SELF_FRACTION * about(*d) / added).min(1.0)
            } else {
                1.0
            }
        })
        .fold(1.0, f32::min)
}

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

    const BOX: [[f32; 3]; 3] = [[2e-3, 0.0, 0.0], [0.0, 2e-3, 0.0], [0.0, 0.0, 4e-3]];
    const Y: [f32; 3] = [0.0, 1.0, 0.0];

    #[test]
    fn a_heavy_chassis_keeps_the_whole_correction() {
        // Two wheels on 27:1, J 1.6e-6: (26)² J ≈ 1.1e-3 each, against 0.02.
        let heavy = [[0.02, 0.0, 0.0], [0.0, 0.02, 0.0], [0.0, 0.0, 0.04]];
        assert_eq!(coupling_scale(&[(Y, 1.6e-6, 27.0); 2], heavy), 1.0);
    }

    #[test]
    fn a_light_chassis_is_scaled_to_the_cap() {
        // The same wheels on a 2e-3 chassis (the 2026-10-11 beetle).
        let k = coupling_scale(&[(Y, 1.6e-6, 27.0); 2], BOX);
        let added = 2.0 * 26.0 * 26.0 * 1.6e-6 * k;
        assert!((added / 2e-3 - MAX_SELF_FRACTION).abs() < 1e-4, "scale {k}");
    }

    #[test]
    fn a_rotor_across_the_wheels_adds_nothing_to_their_axis() {
        let x = [1.0, 0.0, 0.0];
        let wheels = coupling_scale(&[(Y, 1.6e-6, 27.0); 2], BOX);
        let both = coupling_scale(&[(Y, 1.6e-6, 27.0), (Y, 1.6e-6, 27.0), (x, 1e-9, 2.0)], BOX);
        assert!((wheels - both).abs() < 1e-6);
    }

    #[test]
    fn direct_drive_needs_no_correction() {
        assert_eq!(gear_coupling(5e-6, 1.0, 50.0, 20.0), (0.0, 0.0));
    }
}
