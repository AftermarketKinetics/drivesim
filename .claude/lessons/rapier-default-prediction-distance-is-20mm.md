# Rapier's default contact prediction margin is 20 mm, which is huge at robot scale

`IntegrationParameters::normalized_prediction_distance` defaults to 0.02 (× `length_unit` 1 m = 20 mm). Every collider within 20 mm of another gets a speculative contact manifold (up to 5 points) that goes through the solver every step. A low, flat robot (Obliterator 6 v72, 2026-10-10) had 19 hulls within that band of the floor: 83 solver contacts vs 15 for Seraphs, ~half its physics step time.

- At 4 kHz a body moving 10 m/s travels 2.5 mm per step, so the margin is ~10× more than needed; the weapon has real CCD anyway.
- Measure with rapier's `profiler` feature: `world.physics_pipeline.counters.enable()`, then read `stages.*_time.time_ms()` and `solver.ncontacts` after each `step()` (counters reset per step). Without the feature every timer reads 0.
- Hull vertex count is the other per-robot cost: narrow phase and per-step AABB update scale with it, and `body_shapes` does not cap it.
- Chosen value: 5 mm (`sim.rs`). 2 mm saved only a little more, but it flipped `a_hit_throws_the_opponent_up_not_the_robot`, and so did the hull cap alone. A sweep (margin 20/10/5/2 mm × vertex cap none/64/32) showed pass/fail scattered with no trend: that single-hit test is chaotic, and tiny contact changes decide it. Don't read one flip as a fidelity regression; sweep before concluding.
- Measured gain from 5 mm + 32-vertex hulls on Obliterator: AI mode 2.7× → 3.8× native, box 4.4× → 6.6×.
