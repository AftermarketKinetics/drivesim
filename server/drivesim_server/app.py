"""HTTP surface: robots, STEP upload, the role conversation, live events and
the robot bundle the browser sim loads (docs/drivesim.md §3-4).

Every route is behind authentik (Caddy forward_auth) and scoped to the
signed-in user's own robots (user, 2026-10-09: owner only for v1). Adapted
from the Combat Robot Analyzer's app (2026-10-09). Chat is fire-and-forget:
POST a message, watch ``/events`` (SSE) for streamed text and tool activity.
"""

from __future__ import annotations

import asyncio
import json
import re
import shutil
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any

from fastapi import Depends, FastAPI, HTTPException, Request, UploadFile
from fastapi.responses import FileResponse, StreamingResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, Field

from .auth import current_user
from .config import Settings, load_settings
from .db import Store
from .events import EventBus
from .importer import BUNDLE, STEP_NAME, run_import
from .llm import AnthropicClient, ModelClient
from .loop import Agent

#: Bundle files the browser may fetch: these exact names, and meshes/NNN.stl.
BUNDLE_FILES = {"parts.json", "robot.toml"}
MESH = re.compile(r"^meshes/\d{3,6}\.stl$")


def build_state(settings: Settings | None = None, model: ModelClient | None = None) -> dict[str, Any]:
    settings = settings or load_settings()
    store = Store(settings.db_path)
    bus = EventBus()
    agent = Agent(store, model or AnthropicClient(settings.model, settings.effort), settings, bus)
    return {"settings": settings, "store": store, "bus": bus, "agent": agent, "tasks": set()}


def spawn(s: dict[str, Any], coro: Any) -> None:
    """Run in the background, keeping a reference (the loop only holds weak ones)."""
    task = asyncio.get_running_loop().create_task(coro)
    s["tasks"].add(task)
    task.add_done_callback(s["tasks"].discard)


async def import_and_start(s: dict[str, Any], robot_id: str) -> None:
    """Import the uploaded STEP; on success, start the role conversation."""
    store, bus, agent = s["store"], s["bus"], s["agent"]
    robot_dir = agent.robot_dir(robot_id)
    # The broker mounts bundle/ as the sandbox's only writable path, and the
    # sandbox runs as nobody: it must exist, world-writable, beforehand.
    (robot_dir / BUNDLE).mkdir(exist_ok=True)
    (robot_dir / BUNDLE).chmod(0o777)
    try:
        result = await run_import(s["settings"], robot_id, robot_dir)
    except Exception as exc:  # broker down, timeout: the robot fails, the server lives
        result = None
        error = f"{type(exc).__name__}: {exc}"
    else:
        error = "" if result.ok else (result.stderr.strip()[-2000:] or f"exit {result.code}")
    parts = robot_dir / BUNDLE / "parts.json"
    if result is None or not result.ok or not parts.is_file():
        store.set_status(robot_id, "failed", error or "the importer wrote no parts.json")
        await bus.emit(robot_id, {"type": "status", "status": "failed", "error": error})
        return
    count = len(json.loads(parts.read_text())["parts"])
    store.set_status(robot_id, "ready")
    await bus.emit(robot_id, {"type": "status", "status": "ready", "parts": count})
    name = store.robot(robot_id)["name"]
    await agent.notify(robot_id, f"The STEP import of {name!r} finished: {count} parts. Begin.")


@asynccontextmanager
async def lifespan(app: FastAPI):
    if not hasattr(app.state, "s"):  # tests install their own state first
        app.state.s = build_state()
    yield


app = FastAPI(lifespan=lifespan)


def state(request: Request) -> dict[str, Any]:
    return request.app.state.s


def owned(robot_id: str, request: Request, user: str = Depends(current_user)) -> str:
    """Dependency: the robot id, if it belongs to the caller (404 otherwise)."""
    if len(robot_id) != 32 or state(request)["store"].owner(robot_id) != user:
        raise HTTPException(status_code=404, detail="no such robot")
    return robot_id


@app.get("/api/me")
def me(user: str = Depends(current_user)) -> dict[str, str]:
    return {"username": user}


@app.get("/api/robots")
def list_robots(request: Request, user: str = Depends(current_user)) -> list[dict[str, Any]]:
    return state(request)["store"].robots_for(user)


@app.post("/api/robots", status_code=201)
async def upload(file: UploadFile, request: Request, user: str = Depends(current_user)) -> dict[str, Any]:
    """A new robot from a STEP upload; the import starts at once."""
    s = state(request)
    name = Path(file.filename or "robot").stem
    rid = s["store"].create_robot(user, name)
    target = s["agent"].robot_dir(rid) / STEP_NAME
    size, cap = 0, s["settings"].max_upload_bytes
    with target.open("wb") as out:
        while chunk := await file.read(1 << 20):
            size += len(chunk)
            if size > cap:
                out.close()
                shutil.rmtree(target.parent, ignore_errors=True)
                s["store"].delete_robot(rid)
                raise HTTPException(status_code=413, detail=f"file larger than {cap >> 20} MB")
            out.write(chunk)
    target.chmod(0o644)
    spawn(s, import_and_start(s, rid))
    return {"id": rid, "bytes": size}


@app.get("/api/robots/{robot_id}")
def robot(request: Request, robot_id: str = Depends(owned)) -> dict[str, Any]:
    r = state(request)["store"].robot(robot_id)
    return {k: r[k] for k in ("id", "name", "status", "error", "checked", "created")}


@app.delete("/api/robots/{robot_id}", status_code=204)
def delete(request: Request, robot_id: str = Depends(owned)) -> None:
    s = state(request)
    shutil.rmtree(s["settings"].robots_root / robot_id, ignore_errors=True)
    s["store"].delete_robot(robot_id)


@app.get("/api/robots/{robot_id}/messages")
def history(request: Request, robot_id: str = Depends(owned)) -> list[dict[str, Any]]:
    return state(request)["store"].messages(robot_id)


class Message(BaseModel):
    text: str = Field(min_length=1, max_length=8000)


@app.post("/api/robots/{robot_id}/messages", status_code=202)
async def post_message(msg: Message, request: Request, robot_id: str = Depends(owned)) -> dict[str, bool]:
    if state(request)["store"].robot(robot_id)["status"] != "ready":
        raise HTTPException(status_code=409, detail="the robot is not imported yet")
    spawn(state(request), state(request)["agent"].run(robot_id, msg.text))
    return {"accepted": True}


@app.get("/api/robots/{robot_id}/events")
async def events(request: Request, robot_id: str = Depends(owned)) -> StreamingResponse:
    bus: EventBus = state(request)["bus"]
    q = bus.subscribe(robot_id)

    async def stream():
        try:
            yield ": connected\n\n"
            while not await request.is_disconnected():
                try:
                    ev = await asyncio.wait_for(q.get(), timeout=15)
                    yield f"data: {json.dumps(ev, default=str)}\n\n"
                except TimeoutError:
                    yield ": keepalive\n\n"
        finally:
            bus.unsubscribe(robot_id, q)

    return StreamingResponse(stream(), media_type="text/event-stream",
                             # no-transform: Cloudflare must not compress (and so
                             # buffer) the stream on its way through the tunnel.
                             headers={"Cache-Control": "no-cache, no-transform",
                                      "X-Accel-Buffering": "no"})


@app.get("/api/robots/{robot_id}/bundle/{rel:path}")
def bundle_file(rel: str, request: Request, robot_id: str = Depends(owned)) -> FileResponse:
    """What the browser sim fetches: parts.json, robot.toml and the meshes."""
    if rel not in BUNDLE_FILES and not MESH.match(rel):
        raise HTTPException(status_code=404, detail="no such file")
    path = state(request)["settings"].robots_root / robot_id / BUNDLE / rel
    if not path.is_file():
        raise HTTPException(status_code=404, detail="no such file")
    return FileResponse(path, headers={"Cache-Control": "private, no-cache"})


@app.get("/api/motors.toml")
def motors(request: Request, user: str = Depends(current_user)) -> FileResponse:
    return FileResponse(state(request)["settings"].motors_toml, media_type="text/plain")


# The pages, last so every /api route above takes precedence.
_web = load_settings().web_dir
if _web.is_dir():
    app.mount("/", StaticFiles(directory=_web, html=True), name="web")
