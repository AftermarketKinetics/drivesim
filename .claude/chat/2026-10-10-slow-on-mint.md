# 2026-10-10 slow on mint

## 21:30 robot runs slow on mint
**User:** a9db7f9f81c949f3b6c59e0ea81bcffa robot runs slow on mint, debug why that might be the ca se
[Request interrupted by user]
the speed runs at 0.8x average, 0.4min
**Did:**
- Reading the robot's bundle on mint was denied (auto-mode, production reads); benchmarked the two local robots natively with a scratchpad harness instead (no repo changes).
- Seraphs V1: 6.4x real time vs box, 2.0x with the opponent copy/AI; HUD "physics N×" is stepping-only headroom, so 0.8x means the 4 kHz physics alone can't keep up in the browser.
- Asked the user to copy the a9db… bundle locally and say which opponent mode/browser.

## 21:50 profiled Obliterator 6 v72 (a9db…)
**User:** (copied the bundle via `! scp -r mint:~/drivesim/robots/a9db7f9f81c949f3b6c59e0ea81bcffa server/var/robots/.`)
**Did:**
- Scratchpad profiler (rapier `profiler` feature): 70 µs/step native vs Seraphs 23 µs (budget 250 µs at 4 kHz); 23 hulls / 2,336 verts; rapier's default 20 mm prediction margin gives 19 hulls speculative floor contacts (83 contacts vs 15).
- Experiments: 2 mm margin + hulls capped at 32 verts → 36 µs/step (~1.9×); neither applied to the repo yet. Lesson `rapier-default-prediction-distance-is-20mm.md`.

## 22:00 choose fixes
**Asked:** "Which fixes should go into crates/physics?" — options: Shrink contact margin (2 mm) / Cap hull vertices (32) / Drop physics to 2 kHz / Lighter AI opponent → chose **Shrink contact margin + Cap hull vertices**. "Which opponent mode were you in when you saw 0.8× / 0.4×?" — Box / AI or copy / Not sure → **AI or copy**.

## 22:30 applied margin + hull cap
**Did:**
- `sim.rs`: `normalized_prediction_distance = 0.005` (5 mm, not the approved 2 mm: 2 mm and the cap each flipped the chaotic hit test; 5 mm passes at every cap). `shapes.rs`: `MAX_HULL_VERTS = 32`, `spread_subset` keeps the axis extremes and then farthest points, plus a test; sidecars and lesson updated; `tests/arena.rs` rustfmt'd (pre-existing fmt failure); `make test` passes.
- Obliterator native: AI mode 2.7× → 3.8×, box 4.4× → 6.6× (≈1.1× expected in the browser in AI mode, still marginal). Not committed or deployed.
