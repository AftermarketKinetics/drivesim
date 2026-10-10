# usage.py

## Function
Turns token usage into the daily budget's unit, and gives the budget's day
(user, 2026-10-10: invited users only; one budget per user per day,
counted like the bill, reset at midnight Pacific, checked before each new
message, no exemptions).

## Interface
- `WEIGHTS`: input-token equivalents per token, per model, keyed by usage
  field (`input_tokens`, `cache_creation_input_tokens`,
  `cache_read_input_tokens`, `output_tokens`).
- `DEFAULT_MODEL`: whose weights an unknown model gets.
- `weighted(usage, model) -> float`: one turn's usage in input-token
  equivalents.
- `day_bounds(now, tz) -> (start, end)`: the local midnights around `now`.

## Implementation
- **Weights from the price list** (the claude-api reference, read
  2026-10-10). Claude Sonnet 5.5: $2 / MTok input, $10 output, cache
  reads $0.20, 5-minute cache writes 1.25× input. That gives weights of
  1, 5, 0.1 and 1.25. At those prices the 3 M default is about $6 per user
  per day.
- **Why weighted:** most input in a setup conversation is cached history
  resent every step. Counted raw, a budget would stand for several times
  the real cost.
- **Unknown models** (a future `DRIVESIM_MODEL`, or a server-side
  fallback) are weighted like Sonnet 5.5. Add a row when the server
  moves.
- **Day length:** `day_bounds` uses `zoneinfo`, so DST days are 23 or 25
  real hours. `tzdata` is a dependency so the zone resolves in an image
  without system zone files.

## Assertions
- [ ] Weight changes follow Anthropic's published prices, and cite them.
- Tested (tests/test_usage.py): the weighted example, the fallback, the
  Pacific day, and a 25-hour DST day.
