# welcome/guide.html

## Function
The two guides, "How it works" and "Set up your radio (EdgeTX + ELRS)",
as one HTML fragment shared by the signed-in front page and the public
landing page.

## Interface
- Fetched and inserted by `index.html` and `welcome/index.html`.
- Links the radio model as `/radio/accelerate-elrs-pocket.yml` (absolute,
  so it works from both pages).

## Implementation
One file, so the two pages can't drift apart. It sits under `/welcome/`
because that path is public.

## Assertions
- [ ] Its channel table matches the radio model and the `[control]`
  defaults (ch1 turn, ch2 throttle, ch3 weapon, ch5 arm).
- [ ] Its list of the sim's four modes (user, 2026-10-11: "document the
  different modes"): Copy (the default), Box, AI (countdown, full weapon,
  3-minute bout ending in TIME and Rematch) and AI vs AI (an AI drives
  the robot too, no radio needed). Also the opponent dropdown of public
  robots by weight class for both AI modes, each switch restarting the
  round, and the impact marks with their "Marks" checkbox. All of it must match `web/index.html`,
  `web/src/opponent_toggle.rs` and `crates/physics/src/sim.rs`.
- [ ] Its "Share your robot" step matches `server/web/robot.html`'s
  publish card and what `access.py` shares (bundle only, never the
  conversation).
