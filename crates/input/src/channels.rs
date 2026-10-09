//! Joystick axes (-1..+1) -> one 16-channel CRSF frame.

/// CRSF channel values for -100%, 0 and +100%: what an ELRS receiver hands
/// the flight controller. Mirrors momentum-core's `crsf` constants.
pub const CRSF_MIN: u16 = 172;
pub const CRSF_MID: u16 = 992;
pub const CRSF_MAX: u16 = 1811;

/// The 16 channels of one CRSF frame.
pub type Frame = [u16; 16];

/// Axis `i` becomes channel `i` (ch1 = axis 0). Channels with no axis sit
/// at centre. `None` in means no radio this tick, and gives `None`: the
/// control sees signal loss, exactly as when a receiver stops getting
/// frames.
pub fn frame_from_axes(axes: Option<&[f32]>) -> Option<Frame> {
    let axes = axes?;
    let mut frame = [CRSF_MID; 16];
    for (ch, &v) in frame.iter_mut().zip(axes) {
        *ch = to_crsf(v);
    }
    Some(frame)
}

/// The inverse of momentum-core's `crsf::normalize`, which scales each
/// half of the range separately (the midpoint is not centred: 820 below,
/// 819 above). Out-of-range values clamp; NaN reads as centre.
fn to_crsf(v: f32) -> u16 {
    if v.is_nan() {
        return CRSF_MID;
    }
    let v = v.clamp(-1.0, 1.0);
    let half = if v < 0.0 {
        (CRSF_MID - CRSF_MIN) as f32
    } else {
        (CRSF_MAX - CRSF_MID) as f32
    };
    (CRSF_MID as f32 + v * half).round() as u16
}

/// A channel value back to -1..+1: momentum-core's `crsf::normalize`,
/// for controls that read the frame (the mixer).
pub fn normalize(raw: u16) -> f32 {
    let d = raw as f32 - CRSF_MID as f32;
    let half = if d < 0.0 {
        (CRSF_MID - CRSF_MIN) as f32
    } else {
        (CRSF_MAX - CRSF_MID) as f32
    };
    (d / half).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ends_and_centre_hit_the_crsf_values() {
        let f = frame_from_axes(Some(&[-1.0, 0.0, 1.0])).unwrap();
        assert_eq!(&f[..3], &[CRSF_MIN, CRSF_MID, CRSF_MAX]);
    }

    #[test]
    fn axis_n_is_channel_n_plus_one_and_the_rest_centre() {
        let f = frame_from_axes(Some(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0])).unwrap();
        assert_eq!(f[7], CRSF_MAX);
        assert!(f[8..].iter().all(|&c| c == CRSF_MID));
    }

    #[test]
    fn half_stick_lands_halfway_on_each_side() {
        let f = frame_from_axes(Some(&[-0.5, 0.5])).unwrap();
        assert_eq!(f[0], 582);
        assert_eq!(f[1], 1402);
    }

    #[test]
    fn out_of_range_clamps_and_nan_centres() {
        let f = frame_from_axes(Some(&[-3.0, 2.0, f32::NAN])).unwrap();
        assert_eq!(&f[..3], &[CRSF_MIN, CRSF_MAX, CRSF_MID]);
    }

    #[test]
    fn normalize_inverts_the_frame() {
        let f = frame_from_axes(Some(&[-1.0, -0.25, 0.0, 0.6, 1.0])).unwrap();
        for (raw, v) in f.iter().zip([-1.0, -0.25, 0.0, 0.6, 1.0]) {
            assert!((normalize(*raw) - v).abs() < 1.0 / 819.0);
        }
    }

    #[test]
    fn no_radio_is_no_frame() {
        assert_eq!(frame_from_axes(None), None);
    }

    #[test]
    fn extra_axes_beyond_sixteen_are_ignored() {
        let axes = [1.0f32; 20];
        assert_eq!(frame_from_axes(Some(&axes)).unwrap(), [CRSF_MAX; 16]);
    }
}
