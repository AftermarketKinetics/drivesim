# accelerate-elrs-pocket.yml

## Function

The radio model drivesim offers for download on its front page (user,
2026-10-10): an EdgeTX 2.10 model for the RadioMaster Pocket with ELRS. It
drives the sim over USB joystick and a real robot over ELRS with the same
sticks and switches.

## Interface

| CH | Control | Sim (`kind = "mixer"` defaults) |
|---|---|---|
| 1 | right stick X (`Ail`) | turn |
| 2 | right stick Y (`Ele`) | throttle |
| 3 | left stick Y (`Thr`) | weapon |
| 4 | left stick X (`Rud`) | — (Momentum's lifter) |
| 5 | SA, down | arm |
| 6–8 | SD, SE, SB | — (Momentum's assist, strike, invert) |

Served as a static file at `/radio/accelerate-elrs-pocket.yml`.

## Implementation

- **A copy of Momentum's `radio/edgetx/momentum.yml`** (2026-10-10). Only
  the header comment and the model name ("Accelerate") differ: below the
  header, `diff` shows just the name line. Its sidecar there holds the
  why of every field: role-name stick sources for 2.10, `Au` switch
  warning, no expo, and so on.
- **The copy does not follow changes to Momentum's file.** If that model
  changes and the change should reach site users, copy it again and keep
  the name.
- Install notes (free `modelNN.yml` slot, USB Storage, Classic joystick
  mode) are on the front page, `index.html`.

## Assertions

- CH1/2/3/5 match the role prompt's `[control]` defaults (turn 1,
  throttle 2, weapon 3, arm 5).
- Tested: `test_the_radio_model_is_downloadable_from_the_front_page`.
