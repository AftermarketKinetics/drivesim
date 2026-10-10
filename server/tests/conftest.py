"""Shared server test setup: a server on temp dirs with the local importer,
the real drivesim-check binary, and a scripted model instead of the API."""

from __future__ import annotations

import os
import shutil
import time
from dataclasses import replace
from pathlib import Path
from typing import Any

import pytest
from fastapi.testclient import TestClient

from drivesim_server.app import app, build_state
from drivesim_server.config import load_settings
from drivesim_server.llm import Turn

REPO = Path(__file__).resolve().parents[2]
FIXTURE_STEP = REPO / "tools" / "import" / "tests" / "fixtures" / "simple_bot.step"
FIXTURE_BUNDLE = REPO / "fixtures" / "simple_bot"


def occ_python() -> str:
    """The shell's python3 with pythonocc, not the uv venv's (first on PATH)."""
    for d in os.environ.get("PATH", "").split(os.pathsep):
        p = Path(d) / "python3"
        if ".venv" not in d and p.is_file():
            return str(p)
    raise RuntimeError("no python3 outside the venv on PATH")


class ScriptedModel:
    """Returns the queued turns in order; records what it was sent."""

    def __init__(self) -> None:
        self.turns: list[Turn] = []
        self.calls: list[list[dict[str, Any]]] = []

    def queue(self, *content_lists: list[dict[str, Any]], usage: dict[str, Any] | None = None) -> None:
        for content in content_lists:
            uses_tool = any(b["type"] == "tool_use" for b in content)
            self.turns.append(Turn(content=content, stop_reason="tool_use" if uses_tool else "end_turn",
                                   usage=usage or {}))

    async def turn(self, *, system, tools, messages, on_text) -> Turn:
        self.calls.append([dict(m) for m in messages])
        if not self.turns:
            return Turn(content=[{"type": "text", "text": "(no script)"}], stop_reason="end_turn", usage={})
        t = self.turns.pop(0)
        for b in t.content:
            if b["type"] == "text":
                await on_text(b["text"])
        return t


@pytest.fixture
def settings(tmp_path):
    s = load_settings({})
    return replace(s, robots_root=tmp_path / "robots", db_path=tmp_path / "db.sqlite3",
                   import_mode="local", import_python=occ_python(), dev_user="alice")


@pytest.fixture
def model():
    return ScriptedModel()


@pytest.fixture
def client(settings, model):
    app.state.s = build_state(settings, model)
    with TestClient(app) as c:
        yield c
    del app.state.s


def wait_for(pred, timeout=60.0):
    end = time.time() + timeout
    while time.time() < end:
        if pred():
            return True
        time.sleep(0.1)
    return False


@pytest.fixture
def bundle_dir(tmp_path):
    """A robot dir holding the fixture bundle, as an import would leave it."""
    d = tmp_path / "robot"
    shutil.copytree(FIXTURE_BUNDLE, d / "bundle")
    return d
