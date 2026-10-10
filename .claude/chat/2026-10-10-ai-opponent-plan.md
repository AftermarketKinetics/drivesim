# 2026-10-10 AI opponent plan

## 21:40 plan the AI driver opponent
**User:** plan out a basic ai driver opponent that the user can toggle, so the opponent options are box, copy, and ai
**Did:**
- Read `web/src/opponent_toggle.rs`, `crates/physics/src/sim.rs`, `round.rs`, `mixer.rs`, `index.html` button; noted docs/drivesim.md §1 lists "AI opponent" under Not in v1.
- Drafted plan: `OpponentKind::Ai`, pure `ai_driver.rs` in the physics crate producing `Duties`, three-way page control; asked design questions.
**Asked:** (1) AI weapon: spin up in range / always full / off → **Always full**. (2) Start: when you arm / countdown / immediately → **Countdown**. (3) Behaviour: chase+unstick / chase+aim / chase only → **Chase + unstick**. (4) UI: cycling button / segmented control → **Cycling button**.

## 22:10 build steps 1–3 (physics + tests), checkpoint
**User:** start
**Did:**
- New `crates/physics/src/ai_driver.rs` (pure PD chase + stuck reverse, weapon full), `opponent.rs` (Opponent moved out of sim.rs, 3 s sim-time countdown), `tests/ai.rs`; `OpponentKind::Ai`, `Sim::time_s`, `countdown_left()`; sidecars written/updated. Physics tests, clippy, fmt green.
- Found: against a wall the full-speed vertical disc throws the AI back and often flips it (up.z −0.93) before stuck recovery fires; wall test reworded to "not pinned". AI does not drive correctly when inverted.

## 22:30 inverted driving, web side
**Asked:** "With the disc at full the AI flips itself often (walls and hits), and an inverted robot's wheels drive and turn backwards, so it steers wrong until something rights it. Handle that?" Options: Drive inverted (Recommended) / Leave it / Weapon off when flipped → **Drive inverted**.
**User:** make the toggle more obvious, make it three buttons where the chosen one is highlighted
**Did:**
- `opponent::pose_of` takes heading/speed from chassis −x when inverted (unit-tested; fixture can't drive inverted, it lands on its disc). Web: `set_opponent(u8)`, new `web/src/countdown.rs` banner, `index.html` three highlighted buttons (replacing the cycling button).
- Docs: §1/§2/§6 of docs/drivesim.md, guide.html, sidecars for main/index/opponent_toggle/countdown.

## 23:00 verified, ready for browser check
**Did:**
- `make web-build` (exports `set_opponent`) and full `make test` green; web clippy/fmt clean.
- Not yet seen in a browser; nothing committed or deployed.

## 23:15 arena roof
**User:** add a roof so its impossible for bots to launch themselves out of the box
**Did:**
- `arena.rs`: 10 cm roof slab on the walls' tops (2.4 m), overlapping them; `arena_render.rs` draws it in the wall polycarbonate; sidecars + docs/drivesim.md §6.
- New `crates/physics/tests/arena.rs` (+ .md): 30 m/s straight up and at a top corner stay inside; both fail with the roof removed (45.9 m, 6.8 m).
- Noted a concurrent session editing `sim.rs`/`shapes.rs` (contact margin, hull cap); left untouched.

## 23:30 "fly through the upper walls"
**User:** i believe the upper part of the walls is both transparent and non-colliding under some circumstances, i can fly through it sometimes
**Asked:** "Was it the deployed site (drivesim.combat-robot.systems) or local make web-dev where you flew through?…" Deployed site / Local web-dev / Not sure → **Local web-dev**. "Make the wall tops visible, so you can see where the cage ends?…" Top rail (Recommended) / Tint roof darker / Leave as is → **Top rail**.
**Did:**
- Probe: 0/120 throws (5–80 m/s, 1.0–2.3 m high, spinning) got through the walls; cause was the local build predating the roof (robots went over the invisible 2.4 m top). Rebuilt web.
- `arena_render.rs`: opaque 4 cm dark rail along the wall tops (`RAIL_M`); sidecar updated.

## 23:55 review and commit
**User:** all good, read and commit the files before deploying
**Did:**
- Read every change from this session, "slow on mint" (contact margin, hull cap) and "public opponents" (Rival, access/opponents/house.py, DB migration, robot.html publish card); they fit together. `make test` 61 passed + all crate suites green.
- One combined commit, staged by name (files overlap across the three sessions).
