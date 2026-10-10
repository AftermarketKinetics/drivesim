# 2026-10-10 discord help in the role prompt

## 00:00 point problems to arctic256
**User:** add to the system prompt that any errors or issues should be reported to arctic256 on discord, much like the analyzer system prompt
**Did:**
- `server/drivesim_server/prompt.py`: broken tools/reports/imports or unresolved problems → message arctic256 on Discord, quoting the full robot ID.
- `tools/parts_summary.py` now returns `robot_id` (kept out of the prompt for caching, like the analyzer's `show_setup` `session_id`); sidecars + 2 test asserts; 50 server tests pass.

## 00:10 admin dashboard
**User:** allow the admin user (arctic/lily) to access all bots under /admin-dashboard
**Asked:** Admin list: env var vs hardcoded → "Env var (Recommended)". Access: read+drive / full / full admin-billed → "Read + drive (Recommended)". Dashboard: table+usage / plain table → "Table + usage (Recommended)".

## 00:12 opponent toggle (mid-turn)
**User:** add a toggle option in the simulator to change the opponent from an orange version of your bot to an orange box
**Asked:** Toggle UI: HTML button / keyboard key / URL param → "HTML button (Recommended)". On switch: reset round / swap in place → "Reset the round (Recommended)". Box spec: old box orange / old box default → "Old box, orange (Recommended)". Admins: arctic,lily / arctic / lily → "check the authentik dashboard for which one is the authorotative name, i believe it's arctic"
**Did:**
- authentik username lookup on mint was blocked by the permission classifier; DRIVESIM_ADMINS value left for the user.
- Admin: `DRIVESIM_ADMINS` in config, `is_admin`/`admin_user` in auth, `viewable` dep for read routes (post/delete stay `owned`), new `admin.py` (`GET /api/admin/robots`), `web/admin-dashboard/index.html`, robot.html read-only view, index link; sidecars + docs §2 + `make server-dev` admin; 53 tests pass.
- Opponent toggle delegated to a fork (physics box + HTML button, reset on switch).

## 00:40 authentik username
**User:** (pasted authentik user row: username `arctic`, name Lily, Internal)
**Did:**
- `~/code/accelerate/docker-compose.yml`: drivesim service gets `DRIVESIM_ADMINS=arctic` (local edit, not deployed).

## 00:50 commit/push/deploy (interrupted)
**User:** commit and push everything, then deploy
**Did:**
- Waited for the toggle fork, read push.sh / accelerate rsync deploy history; user interrupted before any commit.
- Fork finished: `OpponentKind` + `target.rs` box, web `round.rs`/`opponent_toggle.rs`/`target_render.rs`, sim page button; make test + web-build pass, not browser-tested.

## 01:10 go
**User:** go
**Did:**
- sidecar-check on changed files: fixed `test_tools.py.md`, rest OK. Committed + pushed drivesim 240bf98 (incl. welcome favicon, verified public); accelerate d80cf4c committed (no remote).
- `deploy/push.sh` denied by the auto-mode classifier; deploy left for the user.
