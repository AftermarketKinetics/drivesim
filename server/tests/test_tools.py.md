# test_tools.py

## Function
The role conversation's tools on the fixture bundle, with the real checker.

## Interface
Tests:
- `test_parts_summary_lists_parts_with_axes`
- `test_parts_summary_groups_repeats_and_puts_big_parts_first`
- `test_write_checks_and_marks_the_robot`
- `test_show_and_list_motors`
- `test_write_without_an_import_is_a_tool_error`
- `test_prompt_materials_load`, run for each material in `prompt.MATERIALS`
- `test_the_prompt_points_problems_to_discord_with_the_robot_id`
  (`arctic256` and `robot_id` are in `prompt.SYSTEM`)
- `test_the_prompt_ends_with_every_tunable_setting` (step 5 of
  `prompt.SYSTEM`, up to the schema, names each tunable `robot.toml` key)

## Implementation
`test_prompt_materials_load` writes the fixture robot with each material
the prompt names as the default, through the real Rust checker. That keeps
the prompt's list and `materials.rs` in step.

`test_parts_summary_lists_parts_with_axes` also checks the summary
carries the conversation's `robot_id`.

Fixture numbers it pins: 14 parts, wheel radius 31.8 mm, total mass
5.282 kg. They change with `make_fixture.py`.

## Assertions
- The checker binary must be built first (`make test-server` does it).
