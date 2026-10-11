# impact_marks.rs

## Function

Glowing impact marks (user, 2026-10-11): where a robot takes a large
impact, a spot glows. A moderate hit glows orange; a hard one is deep
red and bigger. Marks never fade: they glow for the rest of the round
(user, 2026-10-11: "make them not fade at all", replacing the first
version's cooling to scorch).

## Interface

- `set_marks(on)` (wasm export): the page's "Marks" checkbox (user,
  2026-10-11: "a toggle setting for turning off the wear marks").
- `Mark`: the component on each mark entity, a child of the hit body's
  `BodyLink` entity.
- `MarkMesh`: a resource, the unit sphere every mark draws scaled.
- `setup_marks`: Startup system; inserts `MarkMesh`.
- `update_marks`: Update system, after `sync_bodies`. It takes
  `Sim::take_impacts()`, adds or reheats marks, and keeps at most
  `MAX_MARKS`.
- `heat(dv) -> f32`: 0 at `MARK_DV`, 1 at `RED_AT` × it and above, on a
  log scale.
- `MAX_MARKS` 200.

## Implementation

- **Colour by heat:** orange [1, 0.5, 0.08] to red [0.85, 0.05, 0.02],
  with emissive at `GLOW` (40) times that colour. That is far past 1, so
  the camera's bloom spreads it into a halo (user, 2026-10-11: "glow
  more"; it was 6 with no bloom).
- **A material changes only when its mark is reheated** by a merged
  impact. Nothing ages, so no per-frame material writes.
- **Log scale:** hits run from about 2 to 12 m/s
  (`drivesim_physics::impacts`), and each doubling should look a step
  hotter. `RED_AT` 5 puts full red at 10 m/s.
- **Size:** 8 mm to 25 mm radius by heat, for a 12 lb robot. Other
  robots scale by (mass / 5.44 kg)^(1/3), so an ant's marks fit an ant.
- **Merging:** one hit lasts several physics steps, and a frame runs
  dozens of them. A new impact within `MERGE_R` (1.5) radii of a mark
  under `MERGE_S` (0.15 s) old on the same body heats that mark instead.
  This also covers marks spawned earlier in the same frame.
- **Marks are children of the body entity,** so they ride with the
  robot. They are despawned with the round's visuals, which means a
  rematch or an opponent switch clears them.
- **Off hides, it doesn't stop marking.** Marks are still made while
  hidden (spawned `Visibility::Hidden`), so turning them back on shows
  every hit of the round. The flag is a static, read each frame. Every
  mark's visibility is rewritten only when it changes (`Local`).
- **Each mark has its own material,** since a reheat changes one mark's
  colour.
- Marks are spheres centred on the surface point, so half of each pokes
  out as a bead. The physics reports no surface normal to flatten them
  along.
- Merge windows use sim time (`Sim::time_s`), like the countdown and
  the bout.

## Assertions

- It only reads the sim (it drains `impacts`, nothing else).
- Never more than `MAX_MARKS` marks; the oldest go first.
- A mark never outlives its round.
