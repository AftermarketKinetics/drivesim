# occ_load.py

## Function

Reads a STEP assembly into its leaf solids, each placed in the assembly
frame and carrying the names the role conversation matches on
(docs/drivesim.md §4). The importer's only contact with the STEP file.

## Interface

- `load(path) -> list[Leaf]`: every solid, in product-tree order. Raises
  `IOError` if OCC cannot read the file.
- `Leaf(instance, product, path, shape, color)`:
  - `instance`: the nearest real instance name up the tree (`wheel_L`),
    falling back to the product name.
  - `product`: the part definition's name (`Wheel`), shared by every
    instance of it.
  - `path`: instance names from the root, `/`-joined.
  - `shape`: a located `TopoDS_Solid`, in mm (OCC's STEP unit).
  - `color`: `(r, g, b)` in 0..1, or `None`.
- `real_name(name) -> bool`: false for XCAF's placeholders (`SOLID`,
  `=>[0:1:…]`, empty).

## Implementation

- Adapted from the cad-step plugin's `occshapes.load_parts`
  (`~/code/cad-step-parser-skills`), without its STEP-entity recovery, and
  with instance and product names kept apart: two instances of one
  `Wheel` product must come out as `wheel_L` and `wheel_R`.
- A product with several solids yields one leaf per solid; the second on
  get `#1`, `#2` … on both `instance` and `path`, so names stay unique.
- `TDocStd_Document` is given a plain `str`: an `ExtendedString` aborts
  the interpreter under OCCT 7.9 (found by the plugin).
- Colour lookup tries the instance colour first, then the label's, across
  surface / generic / curve kinds; any OCC exception there just means "no
  colour".

## Assertions

- Every leaf is a single solid, already located: callers never apply a
  placement themselves.
- `path` values are unique within one file.
- No roles, materials or units are decided here.
