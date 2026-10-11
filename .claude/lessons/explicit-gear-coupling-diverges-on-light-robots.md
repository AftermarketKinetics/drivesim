# The explicit gearbox correction diverges on light robots on high-ratio gearmotors

`gear_coupling.rs` applies `(r−1)²J·α_chassis` to the chassis from the previous step's acceleration. That is an explicit added inertia, and once it nears the chassis's own inertia about the rotor axis the loop feeds itself: a 0.96 kg beetle on 27:1 Repeat Max drive (2.2e-3 vs 2.2e-3 kg m²) flew around the arena at 65 m/s with every motor unpowered, and its unpowered bar spun to 28,000 rpm.

- Found 2026-10-11 from a live report ("pings around the arena without me pressing anything... why is the weapon spinning by default"). Bisected: not impact loss, not 2 kHz (4 kHz was worse), not weapon_reach, not collisions; zeroing the wheel motors' rotor inertia fixed it.
- The 1:1 ratio is necessary but not sufficient: the 12 lb fixture with its chassis inertia cut 100× stayed stable. A fixture variant with 5× wheel motor J and 0.1× chassis inertia reproduces it (tests/drive.rs).
- Fix: `coupling_scale` caps the summed self-term at 0.5 of the chassis inertia. 0.25 also worked but over-tilted the 1 lb house robot on spin-up.
- How to apply: when a user's robot misbehaves with no input, run it headless with `control_tick(None)` first. Energy from nothing means a solver/explicit-term problem, not controls.
