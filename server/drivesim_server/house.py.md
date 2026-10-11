# house.py

## Function
House robots: bundles shipped with the server and seeded into the store
as public robots, so the AI opponent has choices before any owner
publishes one (user, 2026-10-10: start with simple_bot, as a database
robot rather than a separate code path).

## Interface
- `HOUSE_USER` `"drivesim-house"`: their owner.
- `HOUSE`: bundle directory name → (display name, weight class lb):
  `simple_bot_1lb` ("simple_bot 1 lb", 1), `simple_bot_3lb` (3),
  `simple_bot` ("simple_bot", 12) and `simple_bot_30lb` (30), in class
  order (user, 2026-10-11: simple_bot scaled to each weight class).
- `house_id(name) -> str`: the fixed 32-hex-character robot id.
- `seed_house(settings, store) -> [id]`: for each bundle present under
  `settings.house_root`, replaces `robots_root/<id>/bundle/` with a copy
  (sidecar `.md` files left out) and inserts its row if absent. Returns
  the ids seeded. `app.build_state` calls it at every start.

## Implementation
- **A fixed id per house robot** (sha256 of `house:<name>`, cut to 32
  hex characters, the shape every route accepts) makes seeding
  idempotent: the row is created once (`INSERT OR IGNORE`), while the
  files are refreshed from the image every start, so a fixture change
  ships with the next deploy.
- **The row is left alone after it is created**: a later change to its
  name or class in `HOUSE` needs the row updated by hand.
- `drivesim-house` is not an authentik login, so no one can open, chat
  with or delete a house robot; admins see it on the dashboard.
- **Seeded copies are made writable** (`_writable`: dirs 755, files
  644), before the old copy is removed and after the new one is made.
  The image's bundles are read-only Nix store files and copytree copies
  modes; the old `rmtree(ignore_errors=True)` then silently left a
  read-only tree, and the copy over it failed. The server crash-looped on
  its first restart after the 2026-10-11 deploy, until the bundles on
  mint were made writable by hand.
- A missing bundle (e.g. a test with another `house_root`) is skipped,
  not an error.

## Assertions
- [ ] Seeding twice leaves one row per house robot.
- [ ] A house robot is public, ready and checked from the moment it is
  seeded.
