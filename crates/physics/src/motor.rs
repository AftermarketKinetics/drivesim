//! A brushless motor behind its ESC, as a DC motor (carried over from
//! Momentum's 2D sim, 2026-10-09): complementary PWM makes the winding see
//! `duty · V`, so a duty below the back-EMF brakes rather than coasts.

use drivesim_robot::Rotor;

/// Torque on the rotor about its axis, N m, for an ESC command and the
/// rotor's speed relative to the chassis (rad/s, same sign convention as
/// the duty). `None` = no pulses: the ESC lets the motor freewheel.
pub fn rotor_torque(duty: Option<f32>, rotor_w: f32, rotor: &Rotor, battery_v: f32) -> f32 {
    let Some(d) = duty else { return 0.0 };
    let m = &rotor.motor;
    let (kv, ratio) = (m.kv_rad() as f32, rotor.ratio as f32);
    let back_emf = rotor_w * ratio / kv;
    let i_max = m.i_max_a as f32;
    let current =
        ((d.clamp(-1.0, 1.0) * battery_v - back_emf) / m.r_ohm as f32).clamp(-i_max, i_max);
    // SI torque constant = 1 / kv (rad/s per volt).
    current / kv * ratio * rotor.efficiency as f32
}

/// Rotor speed at which `rotor_torque` falls to zero for a duty.
pub fn no_load_speed(duty: f32, rotor: &Rotor, battery_v: f32) -> f32 {
    duty * battery_v * rotor.motor.kv_rad() as f32 / rotor.ratio as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use drivesim_robot::motors::MotorSpec;

    fn rotor() -> Rotor {
        Rotor {
            name: "w".into(),
            body: 1,
            axis_point: [0.0; 3],
            axis_dir: [0.0, 1.0, 0.0],
            radius_m: 0.03,
            width_m: 0.02,
            motor: MotorSpec {
                kv_rpm_per_v: 1450.0,
                r_ohm: 0.06,
                i_max_a: 40.0,
                pole_pairs: 7,
                rotor_inertia_kg_m2: 0.0,
                source: String::new(),
                provisional: vec![],
            },
            ratio: 6.0,
            efficiency: 0.85,
            mu: Some(0.9),
            side: None,
        }
    }

    #[test]
    fn stall_torque_is_current_limited() {
        let r = rotor();
        let t = rotor_torque(Some(1.0), 0.0, &r, 22.2);
        let kv = r.motor.kv_rad() as f32;
        assert!((t - 40.0 / kv * 6.0 * 0.85).abs() < 1e-5);
    }

    #[test]
    fn torque_vanishes_at_no_load_speed() {
        let r = rotor();
        let w = no_load_speed(0.5, &r, 22.2);
        assert!(rotor_torque(Some(0.5), w, &r, 22.2).abs() < 1e-4);
    }

    #[test]
    fn low_duty_brakes_a_fast_rotor_and_none_freewheels() {
        let r = rotor();
        let w = no_load_speed(1.0, &r, 22.2);
        assert!(rotor_torque(Some(0.0), w, &r, 22.2) < 0.0);
        assert_eq!(rotor_torque(None, w, &r, 22.2), 0.0);
    }
}
