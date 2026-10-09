"""Per-robot event fan-out to whoever is watching (SSE connections).

Copied from the Combat Robot Analyzer (2026-10-09). Events are best-effort and not stored: the message history in SQLite is the
record, and a client that reconnects reloads it.
"""

from __future__ import annotations

import asyncio
from collections import defaultdict
from typing import Any


class EventBus:
    def __init__(self) -> None:
        self._subs: dict[str, set[asyncio.Queue[dict[str, Any]]]] = defaultdict(set)

    def subscribe(self, robot_id: str) -> asyncio.Queue[dict[str, Any]]:
        q: asyncio.Queue[dict[str, Any]] = asyncio.Queue(maxsize=1000)
        self._subs[robot_id].add(q)
        return q

    def unsubscribe(self, robot_id: str, q: asyncio.Queue[dict[str, Any]]) -> None:
        subs = self._subs.get(robot_id)
        if subs is not None:
            subs.discard(q)
            if not subs:
                del self._subs[robot_id]

    async def emit(self, robot_id: str, event: dict[str, Any]) -> None:
        for q in list(self._subs.get(robot_id, ())):
            try:
                q.put_nowait(event)
            except asyncio.QueueFull:
                pass  # a stalled client misses events rather than stalling the agent
