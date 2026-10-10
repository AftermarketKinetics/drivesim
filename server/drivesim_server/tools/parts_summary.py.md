# parts_summary.py

## Function
Gives the agent the imported parts in one compact read, one line per part.
This is the evidence it proposes roles from.

## Interface
Tool `parts_summary`, with an optional input `contains` (a substring of the
tree path). It returns `{"robot_id", "source", "frame", "count", "parts": [str]}`.

Each line holds:
- the id, instance and product names;
- the volume (cm³);
- the bounding-box size and centre (mm, in the STEP's own axes);
- for parts with an axis: its direction, radius, width, the curved and cap
  shares, and the centre's distance from the axis.

Also returned: `robot_bbox` (the whole robot's extent, mm) and a `note`
on how to read the list.

## Implementation
- **Biggest first, repeats grouped.** More than `GROUP_OVER` (2) instances
  of one product share a single line, with their count, per-instance
  volume, spread and ids. Lines are sorted by volume, a group by its
  total.
- **Why:** the first real robot (Seraphs V1, 2026-10-10) had 349 parts,
  100 of them identical stator plates plus dozens of screws, bearings and
  magnets. One line per part came to about 42 k characters, past the
  loop's 30 k result cap, so the agent saw a truncated list and couldn't
  find the weapon. Grouped and sorted, that robot is 89 lines and 17 k
  characters, with the weapon plate in the top ten.
- Pairs stay as two lines: a left and a right wheel are told apart by
  position.
- `contains` matches the instance, product or tree path.
- Before an import exists, it raises `ToolError`.
- **`robot_id`** is the robot the conversation belongs to, for the user
  to quote when the prompt sends them to arctic256 on Discord. It lives
  here, not in the prompt, so the prompt stays byte-stable for caching
  (2026-10-10, as in combat-robot-analyzer's `show_setup`).

## Assertions
- Tested: the fixture's 14 parts, `wheel_L`'s axis and radius, the
  filter, and grouping (five identical screws become one line, last;
  the wheels stay two lines; the biggest part leads), and `robot_id`.
