# bout.rs

## Function

The AI bout's clock and its end (user, 2026-10-11: "run the vs ai in 3
minute bouts"; at time, freeze, TIME and a Rematch button). It shows an
m:ss clock at the top centre while the opponent is the AI, and links
the page's Rematch button to the round rebuild.

## Interface

- `bout_over() -> bool` (wasm export): true while the bout in play has
  run out. index.html polls it to show its Rematch button.
- `rematch()` (wasm export): asks for the round to be rebuilt as it is.
- `take_rematch() -> bool`: whether a rematch was asked for since the
  last call; `opponent_toggle::apply_opponent_toggle` takes it.
- `BoutClock`: the component marking the clock's text node.
- `spawn_bout_clock(commands)`: the clock, empty at first; `main.rs`
  calls it at startup.
- `clock(left) -> String`: seconds as m:ss.
- `update_bout_clock`: Update system; sets the clock from
  `Sim::bout_left()` and records `bout_over`.

## Implementation

- **The physics owns the bout** (`Sim::bout_left`, `BOUT_S`): the
  clock, the disarm at 0:00 and the AI all read sim time, so they agree
  exactly. This file only shows it.
- **The clock reads 3:00 through the countdown** and starts at FIGHT,
  as NHRL's timer does. Whole seconds round up, so it reads 0:00 only
  once time is up.
- **TIME is in the countdown banner** (`countdown.rs`): it is the same
  big centred message.
- **Statics, not resources,** because the page calls in from JS outside
  any Bevy system. That is the same reason `opponent_toggle` uses one.
- **The Rematch button is HTML,** like the opponent buttons. This build
  has no Bevy UI widgets or picking enabled. The page polls `bout_over`
  rather than Bevy touching the DOM, which would need more web-sys
  features.

## Assertions

- No AI opponent → the clock is empty and `bout_over` is false.
- It never changes the sim; a rematch is a rebuild in `opponent_toggle`.
- `clock` rounds up and pads seconds (3:00, 1:01, 0:01, 0:00). The web
  crate has no test runner (`make test` covers the workspace crates
  only), so this is checked by eye in the browser.
