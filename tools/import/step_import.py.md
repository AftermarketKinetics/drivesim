# step_import.py

## Function

The importer's entry point: a STEP assembly in, a parts directory out
(docs/drivesim.md §4). It runs inside the import sandbox on the server
and on the command line in development.

## Interface

```
python tools/import/step_import.py robot.step OUT_DIR
```

Prints one line per part: id, instance, product, volume, and its axis
evidence. Writes:

- `OUT_DIR/parts.json`:
  - top level: `format` (1), `source` (the STEP's basename), `units`,
    `frame`, `parts`;
  - per part: `id`, `instance`, `product`, `path`, `color`, `mesh`,
    `triangles`, `bbox_m` (`[min, max]`), `volume_m3`, `com_m`,
    `inertia_m5`, and `rotation_axis` (the dict from `rotation_axis.py`,
    plus `radius_m`, `width_m`, `centre_m` and `com_offset_m`), or
    `null`.
- `OUT_DIR/meshes/NNN.stl`: one per part, named by `id`.

`import_step(step_path, out_dir) -> dict` does the same and returns the
`parts.json` document; the tests call it directly.

## Implementation

- Composes the four modules beside it; it holds no geometry code of its
  own beyond the bounding box taken from the mesh points.
- Everything stays in the STEP's own frame. Which way is up and forward
  is the robot file's job (`up`, `forward`), because generic STEPs
  disagree.
- OCC prints transfer banners to stdout; the per-part lines are the only
  output meant for people.

## Assertions

- `format` changes whenever a key is renamed or its meaning changes.
- Part `id`s are dense from 0 and match the mesh file names.
- No roles or materials appear in `parts.json`.
