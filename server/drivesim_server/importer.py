"""Runs the STEP importer on one robot: in a gVisor sandbox through the
broker (production), or as a local subprocess (development).

The importer reads ``<robot>/robot.step`` and writes ``<robot>/bundle/``
(parts.json + meshes/). The server never parses STEP itself: a STEP upload is
untrusted input to OpenCASCADE (docs/drivesim.md §2).
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from pathlib import Path

import httpx

from .config import Settings

STEP_NAME = "robot.step"
BUNDLE = "bundle"


@dataclass(frozen=True)
class ImportResult:
    code: int
    stdout: str
    stderr: str

    @property
    def ok(self) -> bool:
        return self.code == 0


async def run_import(settings: Settings, robot_id: str, robot_dir: Path) -> ImportResult:
    if settings.import_mode == "broker":
        return await _via_broker(settings, robot_id)
    if settings.import_mode == "local":
        return await _local(settings, robot_dir)
    raise ValueError(f"unknown import mode {settings.import_mode!r}")


async def _via_broker(settings: Settings, robot_id: str) -> ImportResult:
    t = settings.import_timeout_s
    async with httpx.AsyncClient(timeout=t + 60) as client:
        resp = await client.post(f"{settings.broker_url.rstrip('/')}/imports/{robot_id}/run",
                                 json={"timeout": t})
    try:
        data = resp.json()
    except ValueError:
        raise RuntimeError(f"broker: HTTP {resp.status_code}, non-JSON reply") from None
    if resp.status_code != 200:
        raise RuntimeError(f"broker: {data.get('error', resp.status_code)}")
    return ImportResult(int(data["code"]), data["stdout"], data["stderr"])


async def _local(settings: Settings, robot_dir: Path) -> ImportResult:
    proc = await asyncio.create_subprocess_exec(
        settings.import_python, str(settings.import_script),
        str(robot_dir / STEP_NAME), str(robot_dir / BUNDLE),
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    try:
        out, err = await asyncio.wait_for(proc.communicate(), settings.import_timeout_s)
    except TimeoutError:
        proc.kill()
        await proc.wait()
        return ImportResult(137, "", f"import timed out after {settings.import_timeout_s} s")
    return ImportResult(proc.returncode or 0, out.decode(errors="replace")[-(1 << 20):],
                        err.decode(errors="replace")[-8192:])
