# 2026-10-10 public robots as AI opponents

## 23:50 plan public opponents
**User:** add the ability for bot owners to tag their bot as "publically accessible" and then set all bots that are publically accessible as options for the ai opponent, in a dropdown menu once opponent (ai) is selected. start with seraphs and the simple-bot as the publically accessible set. tag them by weightclass
**Did:**
- Read db.py (no public/class columns), app.py `viewable` (owner/admin only), `Sim::new` (opponent is always a copy of the driven model), web bundle_load/round/opponent_toggle, index.html buttons.
- Found: Seraphs V1 is a DB robot (local id 03164b…, owner `dev`); simple_bot is a repo fixture, not in the DB.
- Drafted plan; asked design questions.
**Asked:** (1) Weight class: owner picks / derived from mass / derived+override → **Owner picks**. (2) Dropdown: all grouped by class / only your class / flat → **All, grouped by class**. (3) simple_bot: house bot from image / seed as DB robot → **Seed as a DB robot**. (4) Seraphs on prod: owner ticks box / admins can toggle / not on prod → **Owner ticks the box**.

## 00:40 built public opponents
**Did:**
- Physics `Sim::versus` + `Rival` (opponent owns its model); server `access.py`/`opponents.py`/`house.py`, `public`/`weight_lb` columns with migration, simple_bot seeded as `drivesim-house` robot; robot.html publish card; web `rival_load.rs`, AI dropdown in index.html; docs §1/§2/§6, guide, image ships `/app/house`.
- `make test` + `make web-build` green; live API check on a server at :8421 (the dev DB was migrated, backup in scratchpad; local Seraphs set public 12 lb). Not seen in a browser; nothing committed or deployed.
- Lesson `rapier-body-mass-is-zero-until-first-step.md`.
