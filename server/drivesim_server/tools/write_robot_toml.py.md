# write_robot_toml.py

## Function
Writes the robot file and checks it with the sim's own code. This is the
only way a robot becomes drivable (docs/drivesim.md §4).

## Interface
Tool `write_robot_toml` with input `{"robot_toml": str}`, at most 20,000
characters. It returns the checker's report (`check.py`).

Side effects:
- sets the robot's `checked` flag (and its `name`, when the check passes);
- emits a `robot{report}` event.

## Implementation
1. Replaces `bundle/robot.toml` whole.
2. Runs `drivesim-check`.
3. Records the outcome.

A failing file stays written, so the next `show_robot_toml` shows what
failed. Before an import exists, it raises `ToolError`.

## Assertions
- [ ] `checked` always reflects the file currently on disk.
- Tested: the fixture's file passes at 4.54 kg; a typo'd key fails and
  clears `checked`.
