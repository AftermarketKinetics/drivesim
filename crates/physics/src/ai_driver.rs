//! The AI opponent's driver (user, 2026-10-10): turn toward the robot,
//! drive at it with the weapon at full, and back out when wedged. Pure:
//! it sees two planar poses and returns the ESC duties a radio would.

use crate::mixer::Duties;
use std::f32::consts::FRAC_PI_2;

/// Heading gain: stick per rad of heading error.
pub const KP: f32 = 2.0;
/// Yaw damping: stick per rad/s of yaw rate.
pub const KD: f32 = 0.15;
/// Most turn stick it uses. Full stick spins the fixture at ~24 rad/s
/// (tests/drive.rs), far past what aiming needs.
pub const MAX_TURN: f32 = 0.7;
/// Commanding at least this throttle while slower than `STUCK_SPEED_M_S`
/// for `STUCK_S` counts as wedged.
pub const STUCK_THROTTLE: f32 = 0.5;
pub const STUCK_SPEED_M_S: f32 = 0.1;
pub const STUCK_S: f32 = 1.0;
/// How long it backs out once wedged, and the sticks it backs out with:
/// full reverse, turning so the next approach comes from another angle.
pub const REVERSE_S: f32 = 0.5;
pub const REVERSE_LEFT: f32 = -1.0;
pub const REVERSE_RIGHT: f32 = -0.4;

/// A robot on the floor: position (m), heading (rad, CCW from +x),
/// forward speed (m/s along its heading) and yaw rate (rad/s, CCW).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub heading: f32,
    pub speed: f32,
    pub yaw_rate: f32,
}

/// The driver's memory between control ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AiDriver {
    stuck_s: f32,
    reverse_s: f32,
}

/// `a` wrapped into (−π, π].
pub fn wrap(a: f32) -> f32 {
    let t = std::f32::consts::TAU;
    let w = a.rem_euclid(t);
    if w > std::f32::consts::PI { w - t } else { w }
}

impl AiDriver {
    /// One control tick of `dt` s: the duties to chase `target` from `me`.
    pub fn tick(&mut self, me: &Pose, target: &Pose, dt: f32) -> Duties {
        if self.reverse_s > 0.0 {
            self.reverse_s -= dt;
            return duties(REVERSE_LEFT, REVERSE_RIGHT);
        }
        let err = wrap((target.y - me.y).atan2(target.x - me.x) - me.heading);
        // CCW stick: positive turns left, toward a target on the left.
        let ccw = (KP * err - KD * me.yaw_rate).clamp(-MAX_TURN, MAX_TURN);
        // Full ahead when lined up, none at 90° off or more: turn in place first.
        let throttle = (1.0 - err.abs() / FRAC_PI_2).max(0.0);
        if throttle >= STUCK_THROTTLE && me.speed.abs() < STUCK_SPEED_M_S {
            self.stuck_s += dt;
        } else {
            self.stuck_s = 0.0;
        }
        if self.stuck_s >= STUCK_S {
            self.stuck_s = 0.0;
            self.reverse_s = REVERSE_S;
        }
        duties(throttle - ccw, throttle + ccw)
    }
}

/// Wheel duties as arcade mixing would clamp them, weapon at full.
fn duties(left: f32, right: f32) -> Duties {
    Duties {
        left: Some(left.clamp(-1.0, 1.0)),
        right: Some(right.clamp(-1.0, 1.0)),
        weapon: Some(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1e-3;

    fn at(x: f32, y: f32) -> Pose {
        Pose {
            x,
            y,
            ..Pose::default()
        }
    }

    #[test]
    fn a_target_on_the_left_turns_left() {
        let d = AiDriver::default().tick(&at(0.0, 0.0), &at(0.0, 1.0), DT);
        assert!(d.left < d.right, "{d:?}");
    }

    #[test]
    fn a_target_dead_ahead_is_full_throttle_and_weapon() {
        let d = AiDriver::default().tick(&at(0.0, 0.0), &at(2.0, 0.0), DT);
        assert_eq!(
            (d.left, d.right, d.weapon),
            (Some(1.0), Some(1.0), Some(1.0))
        );
    }

    #[test]
    fn a_target_behind_turns_in_place() {
        let d = AiDriver::default().tick(&at(0.0, 0.0), &at(-2.0, 0.1), DT);
        let (l, r) = (d.left.unwrap(), d.right.unwrap());
        assert!((l + r).abs() < 1e-6 && r > 0.0, "{d:?}");
    }

    #[test]
    fn wedged_for_a_second_backs_out_then_chases_again() {
        let mut ai = AiDriver::default();
        let (me, target) = (at(0.0, 0.0), at(2.0, 0.0));
        let ticks = |s: f32| (s / DT).round() as usize;
        let reversing = |d: Duties| d.left == Some(REVERSE_LEFT) && d.right == Some(REVERSE_RIGHT);
        let mut n = 0;
        while !reversing(ai.tick(&me, &target, DT)) {
            n += 1;
            assert!(n <= ticks(STUCK_S) + 2, "never backed out");
        }
        assert!(n + 2 >= ticks(STUCK_S), "backed out after {n} ticks");
        let mut r = 1;
        while reversing(ai.tick(&me, &target, DT)) {
            r += 1;
            assert!(r <= ticks(REVERSE_S) + 2, "never stopped backing out");
        }
        assert!(r + 2 >= ticks(REVERSE_S), "backed out for {r} ticks");
    }

    #[test]
    fn moving_resets_the_stuck_timer() {
        let mut ai = AiDriver::default();
        let moving = Pose {
            speed: 1.0,
            ..at(0.0, 0.0)
        };
        for _ in 0..3000 {
            assert_eq!(ai.tick(&moving, &at(2.0, 0.0), DT).left, Some(1.0));
        }
    }

    #[test]
    fn wrap_stays_in_minus_pi_to_pi() {
        assert!((wrap(3.0 * std::f32::consts::PI) - std::f32::consts::PI).abs() < 1e-5);
        assert!((wrap(-0.5) + 0.5).abs() < 1e-6);
    }
}
