# __init__.py (drivesim_server)

## Function
Marks the server package: uploads, sandboxed import, the role conversation
and robot bundles (docs/drivesim.md §3–4).

## Interface
None beyond the package; the app is `drivesim_server.app:app`.

## Implementation
Module map: `config` (settings), `auth`, `db` (store), `events`, `llm` and
`loop` (the agent), `prompt`, `tools/`, `importer` (broker or local),
`check` (drivesim-check), `app` (routes).

## Assertions
- Only `config.py` reads the environment.
