# welcome/index.html

## Function
drivesim's public landing page: what anonymous visitors to
drivesim.combat-robot.systems see (user, 2026-10-10). It says what
drivesim is, has a Sign in button, and shows the two guides.

## Interface
- Served at `/welcome/`. Caddy rewrites `/` to it when there's no
  `authentik_proxy` cookie.
- Sign in links to `/outpost.goauthentik.io/start?rd=/`.
- Points visitors without an account to `https://wiki.combat-robot.systems/`,
  which explains how to get one (user, 2026-10-11).
- Inserts `/welcome/guide.html`.
- Shows `bigwheels.mp4`, a 9 s 1080p H.264 clip of driving in the sim
  (user, 2026-10-11), above Sign in: autoplay, muted, looping, with
  controls. Browsers only autoplay muted video.

## Implementation
Caddy leaves only `/`, `/welcome/*` and `/radio/*` public, so the page
needs nothing else: its styles are inline (the site's `style.css` is
behind login) and it calls no API.

## Assertions
- [ ] Loads nothing outside `/welcome/` and `/radio/`. (Links out, such as
  to the wiki, are fine; it loads no resources from them.)
