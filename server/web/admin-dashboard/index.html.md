# admin-dashboard/index.html

## Function
The admin dashboard at `/admin-dashboard/` (user, 2026-10-10): every
robot on the site in one table, for the deployment's admins.

## Interface
- Loads `GET /api/admin/robots`. On 403 it says the page is for admins
  and shows nothing else.
- A card with each user's Claude usage today against the daily budget
  (over budget in red).
- One row per robot: name (links to `../robot.html?id=…`, which admins
  can open read and drive only), owner, state (with a `drive` link to
  `../sim/?robot=…` once checked; an import error shows on hover), upload
  time, lifetime usage, and the full robot ID.
- A filter box matching owner, name or robot ID. The ID is what users
  quote on Discord, from `parts_summary`'s `robot_id`.
- Loads `../style.css` and `../session.js`; every API call goes through
  `apiFetch`.

## Implementation
- It's a directory with an `index.html`, so `/admin-dashboard` works with
  the existing static mount (Starlette redirects it to the trailing
  slash) without a dedicated route. That's why every path is `../`.
- The page itself is public behind authentik like every other page. The
  API is what's restricted: a non-admin gets the "for admins" line.
- Filtering is client-side over the one response; there are few robots.
- Wider than other pages (1100 px) because of the table, which scrolls
  horizontally inside its wrapper on a phone.

## Assertions
- [ ] Every user-supplied string (names, usernames, errors, IDs) is
  HTML-escaped before insertion; IDs are URL-encoded in links.
- [ ] Talks only to `/api`; has no state of its own.
