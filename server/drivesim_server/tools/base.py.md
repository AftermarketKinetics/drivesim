# base.py

## Function
What a tool is, what it gets to work with, and input validation. Copied from
the Combat Robot Analyzer (2026-10-09) with drivesim's `Ctx`.

## Interface
- `ToolError(Exception)`: a failure the agent should read and act on; the loop returns its message as an `is_error` tool result.
- `Ctx` (dataclass): `robot_id`, `robot_dir: Path`, `settings: Settings`, `store` (`db.Store`, typed `Any`), `emit: async (event) -> None`; property `bundle` = `robot_dir / "bundle"`.
- `Tool` (frozen): `name`, `description`, `input_schema`, `run: async (Ctx, args) -> Any`; `definition()` is the API tool definition.
- `validate(schema, value, path="input")`: raises `ToolError("<path>: ...")` naming the field.

## Implementation
- `validate` implements just enough JSON Schema: `type`, `enum`, `required`, `properties`, `items`, `additionalProperties` (false or a sub-schema). Other keywords are ignored, so range checks belong in the tool.
- Needed because tools stream with `eager_input_streaming`, and streamed tool input is not validated by the API.
- `bool` is rejected where `number`/`integer` is expected.

## Assertions
- [ ] `validate` rejects unknown fields whenever a schema says `additionalProperties: false`.
- [ ] Tools never run the importer or a shell; the only subprocess any tool starts is `drivesim-check`, through `check.py`.
