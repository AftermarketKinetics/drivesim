# opponent_toggle.rs

## Function

The sim page's opponent switch (user, 2026-10-10): the robot faces an
orange copy of itself by default, or an orange box. The page's button
calls in from JS; a Bevy system rebuilds the round to match.

## Interface

- `set_opponent_box(on: bool)`: wasm-bindgen export, called by
  `web/index.html`'s button. `true` asks for the box, `false` the copy.
- `wanted() -> OpponentKind`: the latest choice (the copy until the page
  asks otherwise). `take_bundle` builds the first round with it.
- `apply_opponent_toggle`: Update system; once a bundle is `Loaded`,
  rebuilds the round (`round::build_round`) whenever `wanted()` differs
  from `BuiltWith`.

## Implementation

- **The choice lives in a static `AtomicBool`.** JS calls the export
  outside any Bevy system, with no access to the `World`; the system
  polls the static each frame. Wasm is single-threaded here, so
  `Relaxed` is enough.
- A click before the bundle loads is not lost: the first round is built
  with `wanted()`.
- If a rebuild fails (it shouldn't: the same model just built), the
  failure is recorded as built so it isn't retried every frame, and the
  HUD shows why. The previous round keeps running.

## Assertions

- The default is the copy, matching the page's initial button label.
- Nothing rebuilds while `BuiltWith` equals `wanted()`: a round only
  restarts on an actual switch.
