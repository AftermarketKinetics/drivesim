# test_tools.py

## Function
The role conversation's tools on the fixture bundle, with the real checker.

## Interface
Tests:
- `test_parts_summary_lists_parts_with_axes`
- `test_write_checks_and_marks_the_robot`
- `test_show_and_list_motors`
- `test_write_without_an_import_is_a_tool_error`
- `test_prompt_materials_load`, run for each material in `prompt.MATERIALS`

## Implementation
`test_prompt_materials_load` writes the fixture robot with each material
the prompt names as the default, through the real Rust checker. That keeps
the prompt's list and `materials.rs` in step.

Fixture numbers it pins: 14 parts, wheel radius 31.8 mm, total mass
5.282 kg. They change with `make_fixture.py`.

## Assertions
- The checker binary must be built first (`make test-server` does it).
