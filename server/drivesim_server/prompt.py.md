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
  until it's right, then list the tunable settings.
- **Contents:** the whole `robot.toml` schema, the robot frame, and the
  EdgeTX Classic channel layout (ch1 turn, ch2 throttle, ch3 weapon, ch5
  arm).
- **Plausibility hints:** tip speeds of 50–120 m/s are normal for a 12 lb
  spinner, and a two-wheeled robot resting tilted on an edge or skid is
  expected.
- **Defaults:** the robot is named after the uploaded file, and `arm = 5`
  stays unless the radio has no arm switch.
- **The STEP origin is not the floor:** the first real robot's axle sat
  29 mm above the origin, and the agent took that for wheels sunk into
  the floor (2026-10-10).
- **Weapon direction:** the prompt documents `spin` and asks for it
  unless the weapon is a vertical disc or drum spinning front edge up. A
  weapon spinning the wrong way throws the robot itself on every hit
  (2026-10-10).
- **Tyres:** the `mu` guidance lists soft cast polyurethane at ~1.4, TPU
  and rubber at ~0.8–1.0 and hard plastic at ~0.3, all unmeasured. Steel
  cleats have no number: ask.
- **Sends users to help:** when something looks broken or a problem
  can't be resolved, the agent tells them to message `arctic256` on
  Discord and quote their full robot ID, from `parts_summary`'s
  `robot_id`. The ID is not in the prompt, keeping it identical for
  every robot. Added 2026-10-10 at Lily's request, matching
  combat-robot-analyzer. Tested by
  `test_the_prompt_points_problems_to_discord_with_the_robot_id`.
- **Ends with the tuning list:** once the robot is ready, the agent
  lists every `robot.toml` setting that brings the sim closer to the real
  robot (masses, materials, `mu`, motors and overrides with their
  provisional figures, ratio/efficiency, `battery_v`, spin/axis/side,
  axes, radio channels), each with its value and whether it was
  measured, given or guessed, leading with the two or three that matter
  most for that robot. Only fields the file exposes are listed, so the
  agent never offers a tweak it cannot make. Added 2026-10-10 at Lily's
  request. Tested by `test_the_prompt_ends_with_every_tunable_setting`.
- **Tuned on the first live conversation (2026-10-09),** which named the
  robot "robot", blamed a normal tilt on wheel clearance, and left out
  the arm switch.

## Assertions
- [ ] No timestamps or per-request text in `SYSTEM`; it must stay
  byte-stable for caching.
- [ ] `MATERIALS` ⊆ `crates/robot/src/materials.rs`, enforced by
  `test_prompt_materials_load`.
- [ ] The schema shown matches `crates/robot/src/robot_file.rs`. A field
  added there is added here, and to the step-5 tuning list if it is tunable.
