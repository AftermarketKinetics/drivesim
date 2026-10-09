# parts_summary.py

## Function
Gives the agent the imported parts in one compact read, one line per part.
This is the evidence it proposes roles from.

## Interface
Tool `parts_summary`, with an optional input `contains` (a substring of the
tree path). It returns `{"source", "frame", "count", "parts": [str]}`.

Each line holds:
- the id, instance and product names;
- the volume (cm³);
- the bounding-box size and centre (mm, in the STEP's own axes);
- for parts with an axis: its direction, radius, width, the curved and cap
  shares, and the centre's distance from the axis.

## Implementation
Lines rather than raw JSON keep a 140-part robot to about 17 k characters,
under the loop's 30 k result cap. Before an import exists, it raises
`ToolError`.

## Assertions
- Tested: the fixture's 4 parts, `wheel_L`'s axis and radius, the filter.
