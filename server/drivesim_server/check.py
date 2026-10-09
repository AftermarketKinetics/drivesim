"""Runs ``drivesim-check`` (crates/cli) on a robot's bundle: the same Rust
loader and physics the sim uses, so the role conversation checks its
robot.toml against the code that will drive it."""

from __future__ import annotations

import asyncio
import json
from pathlib import Path
from typing import Any

from .config import Settings

TIMEOUT_S = 120


async def check_bundle(settings: Settings, bundle: Path) -> dict[str, Any]:
    """The checker's JSON report; ``{"ok": False, "error": ...}`` on any
    failure, including the checker itself crashing or timing out."""
    proc = await asyncio.create_subprocess_exec(
        str(settings.check_bin), str(bundle), str(settings.motors_toml),
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    try:
        out, err = await asyncio.wait_for(proc.communicate(), TIMEOUT_S)
    except TimeoutError:
        proc.kill()
        await proc.wait()
        return {"ok": False, "error": f"the check took longer than {TIMEOUT_S} s"}
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        tail = err.decode(errors="replace")[-2000:]
        return {"ok": False, "error": f"the checker failed (exit {proc.returncode}): {tail}"}
