# welcome/guide.html

## Function
The two guides, "How it works" and "Set up your radio (EdgeTX + ELRS)",
as one HTML fragment shared by the signed-in front page and the public
landing page.

## Interface
- Fetched and inserted by `index.html` and `welcome/index.html`.
- Links the radio model as `/radio/accelerate-elrs-pocket.yml` (absolute,
  so it works from both pages).

## Implementation
One file, so the two pages can't drift apart. It sits under `/welcome/`
because that path is public.

## Assertions
- [ ] Its channel table matches the radio model and the `[control]`
  defaults (ch1 turn, ch2 throttle, ch3 weapon, ch5 arm).
- [ ] Its description of the sim's opponent (an orange copy by default,
  the "Opponent" buttons for a box or the AI, the AI's countdown and
  full weapon, the AI's robot dropdown of public robots by weight
  class, each switch restarting the round)
  matches `web/index.html` and `web/src/opponent_toggle.rs`.
- [ ] Its "Share your robot" step matches `server/web/robot.html`'s
  publish card and what `access.py` shares (bundle only, never the
  conversation).
