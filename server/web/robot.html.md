# robot.html

## Function
One robot's page: import status, the role conversation, and the latest
check result.

## Interface
- Reads `?id=` from the URL.
- Loads `GET /api/robots/{id}` and `/messages`.
- Listens to `/events` over SSE and handles these event types:
  - `text` (streamed into the current reply);
  - `turn_end`, `tool_start`, `error`, `idle`;
  - `status` (refreshes the status line);
  - `robot` (shows the check report).
- Sends replies with `POST /messages`.
- **Delete this robot** (user, 2026-10-11): after a `confirm()` naming
  the robot, `DELETE /api/robots/{id}`, then back to the list. The
  upload, robot file and conversation go; the day's usage stays counted.
- Loads `session.js`; every API call goes through `apiFetch`, so an
  expired login reloads the page instead of failing.
- Loads `markdown.js` and renders assistant replies as markdown,
  re-rendering the streaming reply as each chunk arrives. Its raw text is
  kept on the element as `raw`.
- **Admin view** (user, 2026-10-10): when `GET /api/robots/{id}` says
  `mine: false` (an admin viewing someone else's robot), the reply box
  and Delete are hidden, and a line names the owner. The server refuses
  both anyway.
- **Publicly accessible** (user, 2026-10-10): the `#share` card, shown
  only to the owner of a ready, checked robot. A checkbox plus a weight
  class select (1/3/12/30 lb, the owner's choice) call
  `PUT /api/robots/{id}/public` with `{public, weight_lb}`. Ticking it
  with no class chosen asks for one and sends nothing. Changing the class
  of a public robot saves immediately. A public robot is listed as an AI
  opponent on everyone's sim page.
- Once the robot is checked, links to the browser sim at
  `sim/?robot={id}` (`make web-build` puts the app in `web/sim/`).

## Implementation
- Automated notices (`[Automated notice …]`) are hidden from the replayed
  history; the user didn't write them.
- Tool calls show as `→ name` lines.
- All text is HTML-escaped before display. Assistant text goes through
  `renderMarkdown`, which escapes first; user text is escaped and shown
  with its line breaks (`white-space: pre-wrap`). Before 2026-10-10,
  replies were escaped into one block, so newlines collapsed and
  `**bold**` showed raw (user report).

## Assertions
- Never inserts model or user text as HTML unescaped.
- The publish control never sends `public: true` without a weight class.
