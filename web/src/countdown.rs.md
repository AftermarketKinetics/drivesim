# countdown.rs

## Function

The AI round's countdown (user, 2026-10-10): "3", "2", "1", then
"FIGHT", big and orange in the middle of the screen, so the driver
knows when the AI opponent starts. Empty for the copy and the box.

## Interface

- `CountdownText`: the component marking the banner's text node.
- `FIGHT_SHOWN_S` 1.0: how long "FIGHT" stays after the AI starts.
- `spawn_countdown(commands)`: the banner, empty at first. `main.rs`
  calls it at startup.
- `banner(sim) -> String`: what it says for a sim.
- `update_countdown`: Update system; sets the banner from `SimRes`.

## Implementation

- **It only reads `Sim`:** `countdown_left()` and `time_s`, both sim
  time (`opponent.rs`), so the banner and the AI agree exactly, and a
  rebuilt round (switching opponent) starts it again.
- Whole seconds are rounded up, so "3" shows from 3.0 s to just over
  2 s left, as a spoken countdown does.
- A file of its own rather than part of `hud.rs`: a separate big
  centred node, not a line of the top-left readout.
- The text is only written when it changes, so Bevy doesn't re-lay it
  out every frame.

## Assertions

- It never changes the sim.
- No AI opponent → always empty.
