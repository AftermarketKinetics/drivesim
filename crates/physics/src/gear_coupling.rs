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

/// The rotors' summed chassis self-term `(r−1)²J`, projected on each
/// rotor's axis, against the chassis's own inertia about that axis; the
/// largest over the axes. `rotors` are (unit axis in the chassis frame,
/// motor J, ratio); `chassis` is the chassis inertia about its CoM in the
/// same frame. 0 when no rotor has a gearbox.
pub fn self_ratio(rotors: &[([f32; 3], f32, f32)], chassis: [[f32; 3]; 3]) -> f32 {
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
            added / about(*d)
        })
        .fold(0.0, f32::max)
}

/// How far each step moves the accelerations the correction uses toward
/// the step's measured ones (`smoothed`), for a robot at `self_ratio` g.
///
/// The correction is explicit: the chassis self-term `K = g·I` is an
/// added inertia fed back a step late, and unfiltered it diverges once g
/// nears 1. With a one-pole filter `α_f += β (α − α_f)`, the loop
/// `α = (τ − K α_f)/I` gives `α_f ← (1 − β(1+g)) α_f + β τ/I`. At
/// `β = 1/(1+g)` that settles on the implicit answer `τ/(I+K)` in one
/// step, and it is stable for any g. A heavy robot (g ≈ 0.1) is barely
/// filtered; the 0.96 kg beetle on 27:1 (g = 1.8, 2026-10-11) gets 0.35.
pub fn smoothing(self_ratio: f32) -> f32 {
    1.0 / (1.0 + self_ratio)
}

/// `prev` moved `beta` of the way toward `measured`.
pub fn smoothed(prev: f32, measured: f32, beta: f32) -> f32 {
    prev + beta * (measured - prev)
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

    #[test]
    fn smoothing_settles_on_the_implicit_answer_in_one_step() {
        // A chassis self-term K = g I fed back a step late through the
        // filter: α = (τ − K α_f)/I lands on τ/(I + K) at once.
        for g in [0.1f32, 1.0, 1.8, 10.0] {
            let (i, tau) = (2e-3f32, 0.5f32);
            let beta = smoothing(g);
            let mut f = 0.0;
            for _ in 0..3 {
                let a = (tau - g * i * f) / i;
                f = smoothed(f, a, beta);
                assert!(
                    (f / (tau / (i * (1.0 + g))) - 1.0).abs() < 1e-4,
                    "g {g}: {f}"
                );
            }
        }
    }

    const BOX: [[f32; 3]; 3] = [[2e-3, 0.0, 0.0], [0.0, 2e-3, 0.0], [0.0, 0.0, 4e-3]];
    const Y: [f32; 3] = [0.0, 1.0, 0.0];

    #[test]
    fn self_ratio_sums_the_wheels_on_their_axis() {
        // Two wheels on 27:1, J 1.6e-6, on a 2e-3 chassis: 2·26²·J / I.
        let g = self_ratio(&[(Y, 1.6e-6, 27.0); 2], BOX);
        assert!((g - 2.0 * 26.0 * 26.0 * 1.6e-6 / 2e-3).abs() < 1e-4, "{g}");
    }

    #[test]
    fn a_rotor_across_the_wheels_adds_nothing_to_their_axis() {
        let x = [1.0, 0.0, 0.0];
        let wheels = self_ratio(&[(Y, 1.6e-6, 27.0); 2], BOX);
        let both = self_ratio(&[(Y, 1.6e-6, 27.0), (Y, 1.6e-6, 27.0), (x, 1e-9, 2.0)], BOX);
        assert!((wheels - both).abs() < 1e-6);
    }

    #[test]
    fn no_gearbox_is_not_filtered() {
        assert_eq!(smoothing(self_ratio(&[(Y, 1.6e-6, 1.0); 2], BOX)), 1.0);
    }

    #[test]
    fn direct_drive_needs_no_correction() {
        assert_eq!(gear_coupling(5e-6, 1.0, 50.0, 20.0), (0.0, 0.0));
    }
}
