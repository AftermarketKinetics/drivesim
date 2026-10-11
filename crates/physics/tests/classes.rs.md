# classes.rs

## Function

Tests simple_bot scaled to each weight class (user, 2026-10-11:
fixtures/simple_bot_1lb, _3lb and _30lb, the house robots beside the
12 lb original): that each is a real robot of its class that drives and
spins up cleanly in the sim.

## Interface

Tests:
- `each_class_weighs_just_under_its_limit`: total mass between 90% and
  100% of 1, 3 and 30 lb (measured 0.442, 1.32, 13.195 kg: 97%).
- `each_class_drives_near_its_no_load_speed`: 40% throttle for 1 s with
  the box moved away. Speed is within 10% of `motor::no_load_speed`
  times the wheel radius.
- `each_class_spins_up_its_disc_and_stays_flat`: full weapon for 8 s.
  The disc ends within 5% of no-load, and the chassis never tilts 10°.

## Implementation

- **8 s, not 3:** the 30 lb's disc (4.9× the 12 lb's inertia) takes
  about 7 s to reach speed. The 1 and 3 lb get there in under 1 s.
- **Tilt is the maximum over the whole run**, not the end state. At 9,900
  rpm the 1 lb tumbled and still read upright at 3 s by chance. The disc
  ratios that pass (1 lb 4.5, 5,900 rpm; 3 lb 3.5, 7,600 rpm) come from
  this test. Faster, the small discs hopped and flipped their robots at
  the 2 kHz step (lesson `rapier-fast-weapon-needs-small-dt`).
- The bundles are loaded from `fixtures/` with the repo's motor
  library, as the server's house seeding serves them.

## Assertions

- Expected speeds come from `motor::no_load_speed`, never from a
  previous run.
