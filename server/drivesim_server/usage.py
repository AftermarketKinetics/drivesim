"""Token usage against the daily budget (user, 2026-10-10: invited users,
one budget per user per day, counted like the bill, reset at midnight
Pacific, checked before each new message).

A turn's usage is weighted into input-token equivalents, so the budget
tracks cost rather than raw tokens: most input in a setup conversation is
cached history, which is billed at a tenth of normal input.
"""

from __future__ import annotations

from datetime import datetime, time, timedelta
from typing import Any
from zoneinfo import ZoneInfo

#: Input-token equivalents per token, from Anthropic's price list for the
#: model (the claude-api reference, read 2026-10-10): Claude Sonnet 5.5 is
#: $2 / MTok input, $10 output, cache reads $0.20, 5-minute cache writes
#: 1.25x input. A model missing here is weighted like Sonnet 5.5; add its
#: row when the server moves to it.
WEIGHTS: dict[str, dict[str, float]] = {
    "claude-sonnet-5-5": {
        "input_tokens": 1.0,
        "cache_creation_input_tokens": 1.25,
        "cache_read_input_tokens": 0.1,
        "output_tokens": 5.0,
    },
}
DEFAULT_MODEL = "claude-sonnet-5-5"


def weighted(usage: dict[str, Any], model: str) -> float:
    """One turn's usage in input-token equivalents."""
    w = WEIGHTS.get(model, WEIGHTS[DEFAULT_MODEL])
    return sum(float(usage.get(k) or 0) * f for k, f in w.items())


def day_bounds(now: datetime, tz: str) -> tuple[datetime, datetime]:
    """The budget day containing `now`: (last midnight, next midnight) in
    `tz`, as aware datetimes."""
    zone = ZoneInfo(tz)
    local = now.astimezone(zone)
    start = datetime.combine(local.date(), time(0), tzinfo=zone)
    end = datetime.combine(local.date() + timedelta(days=1), time(0), tzinfo=zone)
    return start, end
