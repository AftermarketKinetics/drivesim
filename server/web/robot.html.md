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

## Implementation
- Automated notices (`[Automated notice …]`) are hidden from the replayed
  history; the user didn't write them.
- Tool calls show as `→ name` lines.
- All text is HTML-escaped before display.

## Assertions
- Never inserts model or user text as HTML unescaped.
