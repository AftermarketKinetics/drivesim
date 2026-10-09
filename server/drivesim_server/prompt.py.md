# prompt.py

## Function
The role conversation's system prompt (docs/drivesim.md §4): how to turn
imported parts into a checked `robot.toml` together with the user.

## Interface
- `SYSTEM`: frozen text, and the cache prefix after the tools.
- `MATERIALS`: the material names the prompt offers.

## Implementation
- **Process:** summarise the parts, propose roles from the geometry (a
  wheel is a rotor reaching the floor; the weapon is the rotor with the most
  mass far from its axis; everything else is chassis), ask everything
  unsure in one message with the evidence, then write and check the file
  until it's right.
- **Contents:** the whole `robot.toml` schema, the robot frame, and the
  EdgeTX Classic channel layout (ch1 turn, ch2 throttle, ch3 weapon, ch5
  arm).
- **Plausibility hints:** tip speeds of 50–120 m/s are normal for a 12 lb
  spinner, and a two-wheeled robot resting tilted on an edge or skid is
  expected.
- **Defaults:** the robot is named after the uploaded file, and `arm = 5`
  stays unless the radio has no arm switch.
- **Tuned on the first live conversation (2026-10-09),** which named the
  robot "robot", blamed a normal tilt on wheel clearance, and left out
  the arm switch.

## Assertions
- [ ] No timestamps or per-request text in `SYSTEM`; it must stay
  byte-stable for caching.
- [ ] `MATERIALS` ⊆ `crates/robot/src/materials.rs`, enforced by
  `test_prompt_materials_load`.
- [ ] The schema shown matches `crates/robot/src/robot_file.rs`. A field
  added there is added here.
