# tools/__init__.py

## Function
Collects the role conversation's tools into `TOOLS`.

## Interface
`TOOLS: dict[str, Tool]` holds `parts_summary`, `list_motors`,
`show_robot_toml` and `write_robot_toml`. The module also re-exports `Ctx`,
`Tool`, `ToolError` and `validate`.

## Implementation
One module per tool, each exporting `TOOL`. The agent reads parts, motors
and the current file, and writes the file. It never touches the STEP or
runs the importer.

## Assertions
- [ ] Every tool's `input_schema` has `additionalProperties: false`.
