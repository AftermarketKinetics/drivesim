# rival_load.rs

## Function

The AI's rival robot (user, 2026-10-10): a public robot the sim page
picks from its dropdown for the AI to drive in place of the robot's own
copy. Its bundle is fetched in the background and kept for the session.

## Interface

- `set_rival(base: String)`: wasm-bindgen export, called by
  `web/index.html`'s dropdown with a bundle base URL ending in '/'
  (`/api/robots/<id>/bundle/`), or "" for the robot's own copy.
- `wanted_rival() -> String`: the latest pick ("" until one is made).
- `Rivals`: resource, inserted by `main.rs`'s `setup`:
  - `Rivals::new(motors_url)`;
  - `get(&mut self, base) -> Option<&Result<Bundle, String>>`: the bundle
    at `base` or why it failed, `None` while it loads. The first call
    for a base starts its fetch.

## Implementation

- **The pick lives in a static `Mutex<String>`**, for the same reason
  the opponent kind lives in an atomic (`opponent_toggle.rs.md`): JS
  calls in outside any Bevy system.
- **Fetched with the driven robot's motor library** (`bundle_load`'s
  `fetch_bundle`, the same loader), so both robots resolve motors from
  one file.
- **Every fetched bundle is kept,** failures included, so switching back
  to a rival rebuilds at once and a broken one isn't refetched every
  frame. Bundles are a few MB of meshes each; the page lists a handful.
- Async results land in `done` (an `Arc<Mutex<Vec>>`) and are moved
  into `fetched` on the next `get`, so no extra system is needed.

## Assertions

- A base is fetched at most once per page load.
- `get` never blocks: it returns `None` until the fetch finishes.
