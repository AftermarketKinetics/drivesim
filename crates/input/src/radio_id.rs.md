# radio_id.rs

## Function

Picks the EdgeTX radio out of whatever game controllers are connected,
by USB ID rather than by name: names differ between radios and browsers.

## Interface

- `RadioId { vendor, product }`.
- `EDGETX_JOYSTICK`: vendor 0x1209, product 0x4F54.
- `is_edgetx_joystick(id) -> bool`.
- `parse_web_id(&str) -> Option<RadioId>`: reads the
  `(Vendor: 1209 Product: 4f54)` part of a Chromium Gamepad API `id`.

## Implementation

- The ID is EdgeTX's own USB joystick ID (pid.codes), seen 2026-10-09 on
  the Pocket by Windows (4617 / 20308 decimal) and by Chrome. Being
  EdgeTX's, not RadioMaster's, it should match any EdgeTX radio (a TX12
  too); unverified beyond the Pocket.
- Only the Chromium `id` format is parsed. Firefox writes
  `1209-4f54-Name`; v1 targets Chrome / Edge (user, 2026-10-09).

## Assertions

- Tested: the Pocket's real Chrome `id`, the Windows decimal IDs, and a
  non-radio pad with and without IDs.
