# Rapier's "active" contacts and contact distances don't say whether a body touches; its impulse does

`ContactPair::has_any_active_contact()` is true for speculative contacts,
which rapier keeps before bodies meet. The prediction distance is larger
around a CCD body.

Found 2026-10-10 on the vertical-spinner fixture at rest:
- **The disc** sat 5.5 mm above the floor and was reported as touching
  it, with impulse 0.
- **A point's `dist` is no better.** The loaded fork tips read
  `dist` 2.5 mm and the loaded wheels 0.5 mm. Thresholding `dist` at
  1 mm dropped the forks entirely.

How to apply: "touching" means "carrying load". Count a body as in
contact only if one of its points has `data.impulse > 0` after a step.
`crates/physics/src/rest.rs` does this.
