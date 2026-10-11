# opponent_toggle.rs

## Function

The sim page's opponent switch (user, 2026-10-10): the robot faces an
orange copy of itself by default, an orange box, or the AI driving its
copy or a public rival robot picked from the page's dropdown
(`rival_load.rs`), or AI vs AI, where a second AI drives the robot too
(user, 2026-10-11). The page's controls call in from JS; a Bevy system
rebuilds the round to match.

## Interface

- `set_opponent(kind: u8)`: wasm-bindgen export, called by
  `web/index.html`'s buttons: 0 the copy, 1 the box, 2 the AI, 3 AI vs
  AI; any other value means the copy.
- `Choice { kind, rival }`: an opponent as the page chooses it; `rival`
  is the rival's bundle base URL, "" for the copy, and always "" unless
  `kind` is `Ai` or `AiVsAi`.
- `wanted() -> Choice`: the latest choice (the copy until the page asks
  otherwise). `take_bundle` builds the first round with its kind.
- `apply_opponent_toggle`: Update system; once a bundle is `Loaded`,
  rebuilds the round (`round::build_round`) whenever `wanted()` differs
  from `BuiltWith`, or as it is when `bout::take_rematch()` says the
  page's Rematch button was pressed (2026-10-11). For a rival it asks `Rivals::get`, which starts the
  fetch; it rebuilds once that returns the bundle.

## Implementation

- **The choice lives in a static `AtomicU8`.** JS calls the export
  outside any Bevy system, with no access to the `World`; the system
  polls the static each frame. Wasm is single-threaded here, so
  `Relaxed` is enough.
- A click before the bundle loads is not lost: the first round is built
  with `wanted()`.
- If a rebuild fails (a rival bundle that won't load or spawn, or, which
  shouldn't happen, the same model that just built), `fail` records it
  as built so it isn't retried every frame, and the HUD shows why. The
  previous round keeps running. Picking another choice tries again.
- **While a rival loads, the current round plays on**: nothing is torn
  down until there is something to replace it with.
- The rival is dropped from `wanted()` for the copy and the box, so
  switching AI → Box → AI comes back to the same rival without
  rebuilding the box round when the dropdown changes.

## Assertions

- The default is the copy, matching the page's initially highlighted
  button.
- Nothing rebuilds while `BuiltWith` equals `wanted()`: a round only
  restarts on an actual switch.
- A wanted rival that hasn't loaded yet never tears down the round in
  play.
