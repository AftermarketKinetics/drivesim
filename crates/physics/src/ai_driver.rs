//! The AI opponent's driver (user, 2026-10-10): turn toward the robot,
//! drive at it with the weapon at full, and back out when wedged. While
//! its weapon is slow (spinning up, or slowed by a hit) it backs off and
//! waits for it (user, 2026-10-11: "pause slightly or back off"). Pure:
//! it sees two planar poses and returns the ESC duties a radio would.

use crate::mixer::Duties;
use std::f32::consts::FRAC_PI_2;

/// Weapon speed, as a fraction of no-load, below which it stops attacking
/// to wait for the weapon, and at or above which it attacks again. The gap
/// keeps it from dithering at one threshold.
pub const WEAPON_LOW: f32 = 0.7;
pub const WEAPON_READY: f32 = 0.9;
/// While waiting: closer than this to the target it reverses away at
/// `BACKOFF_THROTTLE`, still facing it; farther, it holds still and aims.
pub const BACKOFF_M: f32 = 0.8;
pub const BACKOFF_THROTTLE: f32 = -0.6;
/// The longest it waits for its weapon before attacking anyway: the 30 lb
/// house robot's disc takes ~7 s, and one dragging on something might
/// never get there.
pub const WAIT_MAX_S: f32 = 10.0;
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AiDriver {
    /// `Some(seconds waited)` while it waits for its weapon.
    waiting: Option<f32>,
    stuck_s: f32,
    reverse_s: f32,
}

impl Default for AiDriver {
    /// A round starts with the weapon still, so waiting for it.
    fn default() -> Self {
        AiDriver {
            waiting: Some(0.0),
            stuck_s: 0.0,
            reverse_s: 0.0,
        }
    }
}

/// `a` wrapped into (−π, π].
pub fn wrap(a: f32) -> f32 {
    let t = std::f32::consts::TAU;
    let w = a.rem_euclid(t);
    if w > std::f32::consts::PI { w - t } else { w }
}

impl AiDriver {
    /// One control tick of `dt` s: the duties to chase `target` from `me`,
    /// with its weapon at `weapon` of its no-load speed (1 with no weapon).
    pub fn tick(&mut self, me: &Pose, target: &Pose, weapon: f32, dt: f32) -> Duties {
        self.waiting = match self.waiting {
            Some(w) if weapon < WEAPON_READY && w < WAIT_MAX_S => Some(w + dt),
            Some(_) => None,
            None if weapon < WEAPON_LOW => Some(0.0),
            None => None,
        };
        if self.reverse_s > 0.0 {
            self.reverse_s -= dt;
            return duties(REVERSE_LEFT, REVERSE_RIGHT);
        }
        let err = wrap((target.y - me.y).atan2(target.x - me.x) - me.heading);
        // CCW stick: positive turns left, toward a target on the left.
        let ccw = (KP * err - KD * me.yaw_rate).clamp(-MAX_TURN, MAX_TURN);
        if self.waiting.is_some() {
            self.stuck_s = 0.0;
            let close = (target.x - me.x).hypot(target.y - me.y) < BACKOFF_M;
            let back = if close { BACKOFF_THROTTLE } else { 0.0 };
            return duties(back - ccw, back + ccw);
        }
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

    /// A driver whose weapon is up to speed.
    fn started() -> AiDriver {
        AiDriver {
            waiting: None,
            ..AiDriver::default()
        }
    }

    fn at(x: f32, y: f32) -> Pose {
        Pose {
            x,
            y,
            ..Pose::default()
        }
    }

    #[test]
    fn it_waits_for_its_weapon_holding_still_when_far() {
        let mut ai = AiDriver::default();
        let d = ai.tick(&at(0.0, 0.0), &at(2.0, 0.0), 0.0, DT);
        assert_eq!(
            (d.left, d.right, d.weapon),
            (Some(0.0), Some(0.0), Some(1.0))
        );
        // Ready at 90%: it attacks.
        let d = ai.tick(&at(0.0, 0.0), &at(2.0, 0.0), WEAPON_READY, DT);
        assert_eq!(d.left, Some(1.0));
    }

    #[test]
    fn a_slowed_weapon_backs_it_off_a_close_target() {
        let mut ai = started();
        // 80%: between the thresholds, it keeps attacking.
        assert_eq!(
            ai.tick(&at(0.0, 0.0), &at(0.5, 0.0), 0.8, DT).left,
            Some(1.0)
        );
        // Under 70%: it reverses away, still facing the target.
        let d = ai.tick(&at(0.0, 0.0), &at(0.5, 0.0), 0.5, DT);
        assert_eq!(
            (d.left, d.right),
            (Some(BACKOFF_THROTTLE), Some(BACKOFF_THROTTLE))
        );
        // Back up to 80% is not enough; 90% is.
        assert_eq!(
            ai.tick(&at(0.0, 0.0), &at(0.5, 0.0), 0.8, DT).left,
            Some(BACKOFF_THROTTLE)
        );
        assert_eq!(
            ai.tick(&at(0.0, 0.0), &at(0.5, 0.0), 0.9, DT).left,
            Some(1.0)
        );
    }

    #[test]
    fn it_attacks_anyway_after_waiting_too_long() {
        let mut ai = AiDriver::default();
        let mut n = 0;
        while ai.tick(&at(0.0, 0.0), &at(2.0, 0.0), 0.1, DT).left != Some(1.0) {
            n += 1;
            assert!(n < ((WAIT_MAX_S + 0.1) / DT) as usize, "never attacked");
        }
        assert!(
            n + 2 >= (WAIT_MAX_S / DT) as usize,
            "attacked after {n} ticks"
        );
    }

    #[test]
    fn a_target_on_the_left_turns_left() {
        let d = started().tick(&at(0.0, 0.0), &at(0.0, 1.0), 1.0, DT);
        assert!(d.left < d.right, "{d:?}");
    }

    #[test]
    fn a_target_dead_ahead_is_full_throttle_and_weapon() {
        let d = started().tick(&at(0.0, 0.0), &at(2.0, 0.0), 1.0, DT);
        assert_eq!(
            (d.left, d.right, d.weapon),
            (Some(1.0), Some(1.0), Some(1.0))
        );
    }

    #[test]
    fn a_target_behind_turns_in_place() {
        let d = started().tick(&at(0.0, 0.0), &at(-2.0, 0.1), 1.0, DT);
        let (l, r) = (d.left.unwrap(), d.right.unwrap());
        assert!((l + r).abs() < 1e-6 && r > 0.0, "{d:?}");
    }

    #[test]
    fn wedged_for_a_second_backs_out_then_chases_again() {
        let mut ai = started();
        let (me, target) = (at(0.0, 0.0), at(2.0, 0.0));
        let ticks = |s: f32| (s / DT).round() as usize;
        let reversing = |d: Duties| d.left == Some(REVERSE_LEFT) && d.right == Some(REVERSE_RIGHT);
        let mut n = 0;
        while !reversing(ai.tick(&me, &target, 1.0, DT)) {
            n += 1;
            assert!(n <= ticks(STUCK_S) + 2, "never backed out");
        }
        assert!(n + 2 >= ticks(STUCK_S), "backed out after {n} ticks");
        let mut r = 1;
        while reversing(ai.tick(&me, &target, 1.0, DT)) {
            r += 1;
            assert!(r <= ticks(REVERSE_S) + 2, "never stopped backing out");
        }
        assert!(r + 2 >= ticks(REVERSE_S), "backed out for {r} ticks");
    }

    #[test]
    fn moving_resets_the_stuck_timer() {
        let mut ai = started();
        let moving = Pose {
            speed: 1.0,
            ..at(0.0, 0.0)
        };
        for _ in 0..3000 {
            assert_eq!(ai.tick(&moving, &at(2.0, 0.0), 1.0, DT).left, Some(1.0));
        }
    }

    #[test]
    fn wrap_stays_in_minus_pi_to_pi() {
        assert!((wrap(3.0 * std::f32::consts::PI) - std::f32::consts::PI).abs() < 1e-5);
        assert!((wrap(-0.5) + 0.5).abs() < 1e-6);
    }
}
