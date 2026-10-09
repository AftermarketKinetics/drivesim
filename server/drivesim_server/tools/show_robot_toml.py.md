# show_robot_toml.py

## Function
The current `robot.toml`, so the agent can edit it rather than rewrite it
from memory.

## Interface
Tool `show_robot_toml`, with no input. It returns `{"robot_toml": str |
null}`.

## Implementation
Reads `bundle/robot.toml` if it exists.

## Assertions
- Read-only.
