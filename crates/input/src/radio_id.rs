//! Recognising an EdgeTX radio among connected game controllers.

/// USB vendor and product ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadioId {
    pub vendor: u16,
    pub product: u16,
}

/// EdgeTX's USB joystick ID (pid.codes vendor 0x1209, product 0x4F54).
/// Seen 2026-10-09 on a RadioMaster Pocket, by Windows (4617 / 20308) and
/// by Chrome ("Vendor: 1209 Product: 4f54"). It belongs to EdgeTX, not to
/// the radio model, so any EdgeTX radio should report it.
pub const EDGETX_JOYSTICK: RadioId = RadioId {
    vendor: 0x1209,
    product: 0x4F54,
};

pub fn is_edgetx_joystick(id: RadioId) -> bool {
    id == EDGETX_JOYSTICK
}

/// The IDs in a Chromium Gamepad API `id` string, e.g.
/// `"Radiomaster Pocket Joystick (Vendor: 1209 Product: 4f54)"`. `None` when
/// the string carries no IDs (Firefox writes `1209-4f54-Name` instead,
/// which is not supported: v1 targets Chrome / Edge).
pub fn parse_web_id(id: &str) -> Option<RadioId> {
    let hex_after = |key: &str| {
        let rest = &id[id.find(key)? + key.len()..];
        let hex: String = rest
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        u16::from_str_radix(&hex, 16).ok()
    };
    Some(RadioId {
        vendor: hex_after("Vendor:")?,
        product: hex_after("Product:")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_id_of_the_pocket_parses() {
        let id = parse_web_id("Radiomaster Pocket Joystick (Vendor: 1209 Product: 4f54)");
        assert_eq!(id, Some(EDGETX_JOYSTICK));
        assert!(is_edgetx_joystick(id.unwrap()));
    }

    #[test]
    fn windows_decimal_ids_are_the_same_radio() {
        assert!(is_edgetx_joystick(RadioId {
            vendor: 4617,
            product: 20308
        }));
    }

    #[test]
    fn other_pads_are_not_radios() {
        let xbox = parse_web_id("Xbox 360 Controller (XInput STANDARD GAMEPAD)");
        assert_eq!(xbox, None);
        let other = parse_web_id("Some Pad (Vendor: 045e Product: 028e)").unwrap();
        assert!(!is_edgetx_joystick(other));
    }
}
