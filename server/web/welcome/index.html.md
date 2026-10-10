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
**It must never make the browser fetch a protected path.** Any anonymous
request outside the public paths (`/favicon.ico` included, which browsers
fetch by themselves) gets authentik's outpost to set an `authentik_proxy`
cookie on its way to the login redirect. Caddy's landing matcher then
sees a cookie and sends `/` to login for the cookie's hour. That was the
"everything 302s" report of 2026-10-11, seen even in incognito. So the
page declares its icon explicitly: the shared site icon at
`https://combat-robot.systems/favicon.svg`, on the public apex host, where
no outpost cookie can be set (changed 2026-10-10 from an inline `data:`
icon). Caddy also serves `/favicon.*` publicly on this host now, but the
explicit link is what the test checks.

**Every path is absolute** (`/welcome/...`): Caddy serves this page at `/`
by an internal rewrite, so the browser's base is `/` and a relative
`bigwheels.mp4` resolved to `/bigwheels.mp4`, behind login (user report,
2026-10-11).

Caddy leaves only `/`, `/welcome/*` and `/radio/*` public, so the page
needs nothing else: its styles are inline (the site's `style.css` is
behind login) and it calls no API.

## Assertions
- [ ] Every `src`/`href` it loads is absolute and under `/welcome/` or
  `/radio/` (tested).
- [ ] Loads nothing outside `/welcome/` and `/radio/`. (Links out, such as
  to the wiki, are fine; it loads no resources from them.)
