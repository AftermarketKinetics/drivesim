# index.html

## Function
The robot list and STEP upload: the server's front page.

## Interface
Lists `GET /api/robots` with a state for each robot:
- importing;
- import failed;
- needs setting up;
- ready to drive.

Uploads a `.step`/`.stp` with `POST /api/robots`, then opens
`robot.html?id=…`.

## Implementation
Plain HTML and JavaScript with no build step. The colours are tokens in
`style.css`, with a dark-mode set.

## Assertions
- Talks only to `/api`; has no state of its own.
