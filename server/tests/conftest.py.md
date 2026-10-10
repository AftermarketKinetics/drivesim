# conftest.py

## Function
Shared server test setup:
- the server on temporary directories;
- the local importer;
- the real `drivesim-check` binary;
- a scripted model in place of the API.

## Interface
- **Fixtures:** `settings`, `model` (a `ScriptedModel`), `client` (a
  FastAPI `TestClient` with that state), `bundle_dir` (the fixture bundle
  copied into a robot directory).
- **Helpers:** `wait_for(pred, timeout)`, `occ_python()`.
- **Constants:** `FIXTURE_STEP`, `FIXTURE_BUNDLE`.
- **`ScriptedModel`:**
  - `queue(*content_lists, usage=None)` adds turns: one containing
    `tool_use` blocks stops for `tool_use`, otherwise `end_turn`. `usage`
    is reported for each queued turn (default `{}`);
  - `calls` records the messages each turn was sent.

## Implementation
`occ_python()` finds the shell's `python3`, the one with pythonocc, by
skipping `.venv` entries on `PATH`, because `uv run` puts the venv first.
The app's lifespan keeps the state the fixture installs.

## Assertions
- No test calls the Anthropic API.
