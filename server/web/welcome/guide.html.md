# welcome/guide.html

## Function
The guides, "How it works", "Set up your radio (EdgeTX + ELRS)" and
"drivesim physics/compensations" (user, 2026-10-11: "write this into the
homepage under a dropdown"), as one HTML fragment shared by the signed-in front page and the public
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
- [ ] Its physics card lists what `crates/physics` does now, matching
  docs/drivesim.md §6: rates `PHYSICS_RATES_HZ` (2/3/4 kHz, default
  `PHYSICS_HZ`), the 12-hull chassis, `weapon_reach`, friction (floor 1,
  body 0.3), the motor model, the reflected inertia, the wheels'
  gyroscopic term being off, `gear_coupling` and its `smoothing`,
  `impact_loss` (5 g, `KEEP` 0.5), `MARK_DV` 2 m/s, the arena and roof,
  and the AI's behaviour. Update it with any of those.
- [ ] Its "Physics" slider mention matches `web/index.html` (2–4 kHz).
- [ ] Its "Share your robot" step matches `server/web/robot.html`'s
  publish card and what `access.py` shares (bundle only, never the
  conversation).
