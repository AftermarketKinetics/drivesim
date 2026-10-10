"""The daily budget: weights, the Pacific day, recording and enforcement."""

from __future__ import annotations

import dataclasses
from datetime import UTC, datetime

from conftest import FIXTURE_STEP, wait_for
from drivesim_server.app import app, build_state
from drivesim_server.usage import day_bounds, weighted
from fastapi.testclient import TestClient


def test_turns_are_weighted_like_the_bill():
    u = {"input_tokens": 1000, "cache_read_input_tokens": 100_000,
         "cache_creation_input_tokens": 2000, "output_tokens": 500}
    # 1000 + 0.1 * 100000 + 1.25 * 2000 + 5 * 500
    assert weighted(u, "claude-sonnet-5-5") == 16_000
    assert weighted(u, "some-future-model") == 16_000  # falls back to Sonnet 5.5
    assert weighted({}, "claude-sonnet-5-5") == 0


def test_the_budget_day_runs_midnight_to_midnight_pacific():
    start, end = day_bounds(datetime(2026, 10, 10, 6, 30, tzinfo=UTC), "America/Los_Angeles")
    assert (start.date().isoformat(), start.hour, end.date().isoformat()) == ("2026-10-09", 0, "2026-10-10")
    # Across the November DST change the day is 25 real hours long.
    # (Subtracting datetimes in one zone compares wall clocks: always 24 h.)
    start, end = day_bounds(datetime(2026, 11, 1, 12, 0, tzinfo=UTC), "America/Los_Angeles")
    assert end.timestamp() - start.timestamp() == 25 * 3600


def test_usage_is_recorded_and_a_spent_budget_refuses_new_work(settings, model):
    settings = dataclasses.replace(settings, daily_budget=10_000)
    app.state.s = build_state(settings, model)
    try:
        with TestClient(app) as client:
            assert client.get("/api/usage").json()["used"] == 0
            # The import notice starts the first turn, which spends 16 000.
            model.queue([{"type": "text", "text": "hello"}],
                        usage={"input_tokens": 1000, "cache_read_input_tokens": 100_000,
                               "cache_creation_input_tokens": 2000, "output_tokens": 500})
            rid = client.post("/api/robots", files={"file": ("simple_bot.step", FIXTURE_STEP.read_bytes())}).json()["id"]
            assert wait_for(lambda: client.get("/api/usage").json()["used"] == 16_000)
            u = client.get("/api/usage").json()
            assert u["budget"] == 10_000 and u["tz"] == "America/Los_Angeles"
            # Spent: replies and uploads are refused, with the reset time.
            r = client.post(f"/api/robots/{rid}/messages", json={"text": "more"})
            assert r.status_code == 429 and "midnight Pacific" in r.json()["detail"]
            r = client.post("/api/robots", files={"file": ("again.step", FIXTURE_STEP.read_bytes())})
            assert r.status_code == 429
            # Deleting the robot does not give the usage back.
            client.delete(f"/api/robots/{rid}")
            assert client.get("/api/usage").json()["used"] == 16_000
    finally:
        del app.state.s
