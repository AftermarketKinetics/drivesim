//! The generic control: arcade mixing of a CRSF frame into motor duties,
//! for any robot whose robot.toml says `kind = "mixer"`.

use drivesim_input::{Frame, normalize};

/// ESC commands for one control tick; `None` = no pulses (freewheel).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Duties {
    pub left: Option<f32>,
    pub right: Option<f32>,
    pub weapon: Option<f32>,
}

/// Channel numbers are 1-based, as in robot.toml.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixerConfig {
    pub throttle: usize,
    pub turn: usize,
    pub weapon: Option<usize>,
    pub arm: Option<usize>,
    pub reverse_turn: bool,
}

/// No frame (signal loss) or arm switch low: every ESC gets no pulses.
/// Turn stick right turns clockwise seen from above: the left side speeds
/// up. The weapon channel is a throttle: stick fully down is off.
pub fn mix(frame: Option<&Frame>, cfg: &MixerConfig) -> Duties {
    let Some(f) = frame else {
        return Duties::default();
    };
    let ch = |n: usize| normalize(f[n - 1]);
    if let Some(arm) = cfg.arm
        && ch(arm) <= 0.5
    {
        return Duties::default();
    }
    let t = ch(cfg.throttle);
    let s = if cfg.reverse_turn {
        -ch(cfg.turn)
    } else {
        ch(cfg.turn)
    };
    Duties {
        left: Some((t + s).clamp(-1.0, 1.0)),
        right: Some((t - s).clamp(-1.0, 1.0)),
        weapon: cfg.weapon.map(|w| (ch(w) + 1.0) / 2.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drivesim_input::{CRSF_MAX, CRSF_MID, CRSF_MIN};

    const CFG: MixerConfig = MixerConfig {
        throttle: 2,
        turn: 1,
        weapon: Some(3),
        arm: Some(5),
        reverse_turn: false,
    };

    fn frame(turn: u16, throttle: u16, weapon: u16, arm: u16) -> Frame {
        let mut f = [CRSF_MID; 16];
        f[0] = turn;
        f[1] = throttle;
        f[2] = weapon;
        f[4] = arm;
        f
    }

    #[test]
    fn full_forward_drives_both_sides() {
        let d = mix(Some(&frame(CRSF_MID, CRSF_MAX, CRSF_MIN, CRSF_MAX)), &CFG);
        assert_eq!(
            (d.left, d.right, d.weapon),
            (Some(1.0), Some(1.0), Some(0.0))
        );
    }

    #[test]
    fn turn_right_speeds_up_the_left_side() {
        let d = mix(Some(&frame(CRSF_MAX, CRSF_MID, CRSF_MIN, CRSF_MAX)), &CFG);
        assert_eq!((d.left, d.right), (Some(1.0), Some(-1.0)));
    }

    #[test]
    fn disarmed_or_no_signal_is_no_pulses() {
        let off = Duties::default();
        assert_eq!(
            mix(Some(&frame(CRSF_MAX, CRSF_MAX, CRSF_MAX, CRSF_MIN)), &CFG),
            off
        );
        assert_eq!(mix(None, &CFG), off);
    }

    #[test]
    fn weapon_stick_is_a_throttle() {
        let d = mix(Some(&frame(CRSF_MID, CRSF_MID, CRSF_MAX, CRSF_MAX)), &CFG);
        assert_eq!(d.weapon, Some(1.0));
    }
}
