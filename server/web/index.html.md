# index.html

## Function
The server's front page:
- the robot list and STEP upload;
- a short guide to using the site (export, upload, answer Claude, drive,
  plug in a radio);
- a radio setup guide for EdgeTX + ELRS, with the download of
  `radio/accelerate-elrs-pocket.yml` and its channel table (user,
  2026-10-10);
- a "drivesim physics/compensations" card: what the sim simulates and
  corrects (user, 2026-10-11).

## Interface
Lists `GET /api/robots` with a state for each robot:
- importing;
- import failed;
- needs setting up;
- ready to drive.

Uploads a `.step`/`.stp` with `POST /api/robots`, then opens
`robot.html?id=…`.

Links to `radio/accelerate-elrs-pocket.yml` with `download`.

Shows today's Claude usage from `GET /api/usage` above the robot list, and
says when it's used up: new robots and replies wait until midnight Pacific,
while set-up robots stay drivable.

When `GET /api/me` says `admin`, shows a link to `admin-dashboard/`.

The guides are `welcome/guide.html`, inserted into `#guide`; every API
call goes through `session.js`'s `apiFetch`.

## Implementation
The three guides are collapsed `<details>` cards above the list, so a
returning user sees their robots first. Robot names are HTML-escaped and
ids URL-encoded before going into the list. A file name is user input.

Plain HTML and JavaScript with no build step. The colours are tokens in
`style.css`, with a dark-mode set.

## Assertions
- Talks only to `/api`; has no state of its own.
- The guide's channel table matches `radio/accelerate-elrs-pocket.yml`
  and the fixture's `[control]` (ch1 turn, ch2 throttle, ch3 weapon, ch5
  arm).
