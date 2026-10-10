# bundle_load.rs

## Function

Fetches a robot bundle (`parts.json`, `robot.toml`, the meshes) and the
motor library over HTTP. It loads them with `drivesim_robot::load`, the
same loader the server's checker runs, so the browser cannot accept a
robot the checker would reject (docs/drivesim.md §5).

## Interface

- `Bundle { model, meshes, part_colors }`:
  - `meshes` maps each part's `mesh` path to its STL bytes;
  - `part_colors` maps part id to its CAD colour, if the STEP had one.
- `locate() -> Result<(base, motors_url), String>`, from the page URL:
  - `?robot=<32 hex>` gives `/api/robots/<id>/bundle/` and
    `/api/motors.toml` (drivesim-server);
  - `?bundle=<base>&motors=<url>` is a static bundle, such as the fixture
    under `make web-dev`; `/` is appended to `base` if missing.
- `async fetch_bundle(base, motors_url) -> Result<Bundle, String>`.

## Implementation

- **The robot id is checked before it goes into a URL** (32 hex digits,
  as the server's ids are). The server also checks ownership, so this
  only stops a malformed id from fetching some other path.
- Meshes are fetched one at a time after the model loads. The model's
  loader rejects a bad robot before any mesh download starts.
- Errors are strings that carry the URL, because they go straight onto
  the HUD.

## Assertions

- The model is built only by `drivesim_robot::load`, never by a
  browser-side parser.
- `locate` accepts no robot id that isn't 32 hex digits.
- Every mesh named in `parts.json` is in `meshes`, or the load fails.
