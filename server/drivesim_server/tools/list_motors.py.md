# list_motors.py

## Function
Shows the agent the motor library that `robot.toml` can name.

## Interface
Tool `list_motors`, with no input. It returns `{"motors": [{"id", "kind",
"kv_rpm_per_v", "i_max_a", "provisional", "source"}]}`, sorted by id.

## Implementation
Reads `settings.motors_toml` with `tomllib`. Its `provisional` and `source`
fields let the agent tell the user which numbers are placeholders.
`kind` is filled as `"brushless"` when absent, matching the Rust default.

## Assertions
- Tested: the library's ids are listed.
