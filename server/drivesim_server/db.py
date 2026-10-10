"""SQLite store: robots and their append-only role-conversation history.

Adapted from the Combat Robot Analyzer's session store (2026-10-09). Messages
are stored exactly as the API returned them and replayed unchanged, because
thinking blocks are only valid in the conversation that produced them. No
message is ever updated or deleted; a robot is only deleted whole.
"""

from __future__ import annotations

import json
import sqlite3
import time
import uuid
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

#: importing -> ready | failed. A robot is drivable once it is ready and its
#: robot.toml passes the check (``checked``).
STATUSES = ("importing", "ready", "failed")

#: NHRL weight classes, lb, an owner tags a public robot with (user,
#: 2026-10-10: the owner picks; 0 = untagged).
WEIGHT_CLASSES = (3, 12, 30)

SCHEMA = """
CREATE TABLE IF NOT EXISTS robots (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL,
    error TEXT NOT NULL DEFAULT '',
    checked INTEGER NOT NULL DEFAULT 0,
    created REAL NOT NULL,
    public INTEGER NOT NULL DEFAULT 0,
    weight_lb INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS robots_by_user ON robots(username, created);
CREATE TABLE IF NOT EXISTS messages (
    robot_id TEXT NOT NULL REFERENCES robots(id),
    seq INTEGER NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created REAL NOT NULL,
    PRIMARY KEY (robot_id, seq)
);
-- (public, weight_lb) added 2026-10-10; MIGRATIONS adds them to older files.
-- One row per model turn, for the daily budget. No reference to robots:
-- deleting a robot must not hand its usage back.
CREATE TABLE IF NOT EXISTS usage (
    username TEXT NOT NULL,
    robot_id TEXT NOT NULL,
    at REAL NOT NULL,
    model TEXT NOT NULL,
    usage TEXT NOT NULL,
    weighted REAL NOT NULL
);
CREATE INDEX IF NOT EXISTS usage_by_user ON usage(username, at);
"""

#: Columns added after the first deploy: (table, column, definition). The
#: CREATE above has them; an older file gets them by ALTER at startup.
MIGRATIONS = (
    ("robots", "public", "INTEGER NOT NULL DEFAULT 0"),
    ("robots", "weight_lb", "INTEGER NOT NULL DEFAULT 0"),
)


class Store:
    def __init__(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        self.path = path
        with self._conn() as c:
            c.executescript(SCHEMA)
            for table, col, definition in MIGRATIONS:
                have = {r["name"] for r in c.execute(f"PRAGMA table_info({table})")}
                if col not in have:
                    c.execute(f"ALTER TABLE {table} ADD COLUMN {col} {definition}")

    @contextmanager
    def _conn(self) -> Iterator[sqlite3.Connection]:
        conn = sqlite3.connect(self.path)
        conn.row_factory = sqlite3.Row
        try:
            yield conn
            conn.commit()
        finally:
            conn.close()

    def create_robot(self, username: str, name: str) -> str:
        rid = uuid.uuid4().hex
        with self._conn() as c:
            c.execute(
                "INSERT INTO robots (id, username, name, status, created) VALUES (?, ?, ?, 'importing', ?)",
                (rid, username, name[:120], time.time()),
            )
        return rid

    def robots_for(self, username: str) -> list[dict[str, Any]]:
        with self._conn() as c:
            rows = c.execute(
                "SELECT id, name, status, error, checked, created, public, weight_lb FROM robots "
                "WHERE username = ? ORDER BY created DESC",
                (username,),
            ).fetchall()
        return [dict(r) for r in rows]

    def all_robots(self) -> list[dict[str, Any]]:
        """Every robot, newest first, with its owner and its total weighted
        usage (for the admin dashboard)."""
        with self._conn() as c:
            rows = c.execute(
                "SELECT r.id, r.username, r.name, r.status, r.error, r.checked, r.created, "
                "COALESCE((SELECT SUM(u.weighted) FROM usage u WHERE u.robot_id = r.id), 0) AS used "
                "FROM robots r ORDER BY r.created DESC"
            ).fetchall()
        return [dict(r) for r in rows]

    def public_robots(self) -> list[dict[str, Any]]:
        """Robots their owners made public that are drivable (ready and
        checked), by weight class then name: the AI opponent choices."""
        with self._conn() as c:
            rows = c.execute(
                "SELECT id, name, username, weight_lb FROM robots "
                "WHERE public = 1 AND status = 'ready' AND checked = 1 "
                "ORDER BY weight_lb, name COLLATE NOCASE"
            ).fetchall()
        return [dict(r) for r in rows]

    def set_public(self, robot_id: str, public: bool, weight_lb: int) -> None:
        """Public needs a weight class; a private robot keeps its tag (or 0),
        so publishing it again remembers the class."""
        assert weight_lb in WEIGHT_CLASSES or (not public and weight_lb == 0), weight_lb
        with self._conn() as c:
            c.execute("UPDATE robots SET public = ?, weight_lb = ? WHERE id = ?",
                      (int(public), weight_lb, robot_id))

    def seed_robot(self, robot_id: str, username: str, name: str, weight_lb: int) -> None:
        """A drivable, public robot with a fixed id (house.py), created
        once and left alone after; no conversation."""
        with self._conn() as c:
            c.execute(
                "INSERT OR IGNORE INTO robots (id, username, name, status, checked, created, "
                "public, weight_lb) VALUES (?, ?, ?, 'ready', 1, ?, 1, ?)",
                (robot_id, username, name, time.time(), weight_lb),
            )

    def used_by_user_since(self, since: float) -> dict[str, float]:
        """Weighted usage per username since `since` (epoch s)."""
        with self._conn() as c:
            rows = c.execute("SELECT username, SUM(weighted) FROM usage WHERE at >= ? "
                             "GROUP BY username", (since,)).fetchall()
        return {r[0]: float(r[1]) for r in rows}

    def robot(self, robot_id: str) -> dict[str, Any] | None:
        with self._conn() as c:
            row = c.execute("SELECT * FROM robots WHERE id = ?", (robot_id,)).fetchone()
        return dict(row) if row else None

    def owner(self, robot_id: str) -> str | None:
        r = self.robot(robot_id)
        return r["username"] if r else None

    def set_status(self, robot_id: str, status: str, error: str = "") -> None:
        assert status in STATUSES, status
        with self._conn() as c:
            c.execute("UPDATE robots SET status = ?, error = ? WHERE id = ?", (status, error[:4000], robot_id))

    def set_checked(self, robot_id: str, checked: bool, name: str | None = None) -> None:
        with self._conn() as c:
            c.execute("UPDATE robots SET checked = ? WHERE id = ?", (int(checked), robot_id))
            if name:
                c.execute("UPDATE robots SET name = ? WHERE id = ?", (name[:120], robot_id))

    def append(self, robot_id: str, role: str, content: Any) -> None:
        with self._conn() as c:
            (n,) = c.execute(
                "SELECT COALESCE(MAX(seq), -1) + 1 FROM messages WHERE robot_id = ?", (robot_id,)
            ).fetchone()
            c.execute(
                "INSERT INTO messages (robot_id, seq, role, content, created) VALUES (?, ?, ?, ?, ?)",
                (robot_id, n, role, json.dumps(content), time.time()),
            )

    def messages(self, robot_id: str) -> list[dict[str, Any]]:
        with self._conn() as c:
            rows = c.execute(
                "SELECT role, content FROM messages WHERE robot_id = ? ORDER BY seq", (robot_id,)
            ).fetchall()
        return [{"role": r["role"], "content": json.loads(r["content"])} for r in rows]

    def record_usage(self, username: str, robot_id: str, model: str,
                     usage: dict[str, Any], weighted: float) -> None:
        with self._conn() as c:
            c.execute("INSERT INTO usage (username, robot_id, at, model, usage, weighted) "
                      "VALUES (?, ?, ?, ?, ?, ?)",
                      (username, robot_id, time.time(), model, json.dumps(usage), weighted))

    def used_since(self, username: str, since: float) -> float:
        """Weighted usage (input-token equivalents) since `since` (epoch s)."""
        with self._conn() as c:
            row = c.execute("SELECT COALESCE(SUM(weighted), 0) FROM usage "
                            "WHERE username = ? AND at >= ?", (username, since)).fetchone()
        return float(row[0])

    def delete_robot(self, robot_id: str) -> None:
        with self._conn() as c:
            c.execute("DELETE FROM messages WHERE robot_id = ?", (robot_id,))
            c.execute("DELETE FROM robots WHERE id = ?", (robot_id,))
