"""Server settings, read once from the environment.

Every knob the deployment sets lives here so nothing else reads ``os.environ``.
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Settings:
    #: One directory per robot: robot.step, and bundle/ (parts.json,
    #: meshes/, robot.toml). The import sandbox sees a robot's dir as /work.
    robots_root: Path
    #: SQLite file holding robots and their role-conversation history.
    db_path: Path
    #: "broker" (production: gVisor sandbox per import) or "local" (dev:
    #: the importer as a subprocess, no isolation).
    import_mode: str
    #: drivesim's import broker (owns the Docker socket; see broker/).
    broker_url: str
    #: Local mode only: the Python that has pythonocc, and the importer.
    import_python: str
    import_script: Path
    #: The robot checker (crates/cli) and the motor library it reads.
    check_bin: Path
    motors_toml: Path
    model: str
    effort: str
    #: Largest STEP upload accepted, in bytes.
    max_upload_bytes: int
    #: Seconds an import may run before it is killed.
    import_timeout_s: int
    #: Static pages served at / (server/web).
    web_dir: Path
    #: Daily budget per user, in input-token equivalents (usage.py), and the
    #: time zone whose midnight resets it.
    daily_budget: float
    budget_tz: str
    #: Development only: the username used when no authentik header is
    #: present. Never set in a deployed image.
    dev_user: str | None


def load_settings(env: dict[str, str] | None = None) -> Settings:
    e = os.environ if env is None else env
    repo = Path(__file__).resolve().parents[2]
    return Settings(
        robots_root=Path(e.get("DRIVESIM_ROBOTS_ROOT", repo / "server" / "var" / "robots")),
        db_path=Path(e.get("DRIVESIM_DB_PATH", repo / "server" / "var" / "drivesim.sqlite3")),
        import_mode=e.get("DRIVESIM_IMPORT_MODE", "local"),
        broker_url=e.get("DRIVESIM_BROKER_URL", "http://127.0.0.1:8711"),
        import_python=e.get("DRIVESIM_IMPORT_PYTHON", "python3"),
        import_script=Path(e.get("DRIVESIM_IMPORT_SCRIPT", repo / "tools" / "import" / "step_import.py")),
        check_bin=Path(e.get("DRIVESIM_CHECK_BIN", repo / "target" / "release" / "drivesim-check")),
        motors_toml=Path(e.get("DRIVESIM_MOTORS_TOML", repo / "data" / "motors.toml")),
        model=e.get("DRIVESIM_MODEL", "claude-sonnet-5-5"),
        effort=e.get("DRIVESIM_EFFORT", "medium"),
        # accelerate is reached through a Cloudflare tunnel, whose free plan
        # rejects request bodies over 100 MB (the analyzer's finding).
        max_upload_bytes=int(e.get("DRIVESIM_MAX_UPLOAD_BYTES", str(95 * 1024 * 1024))),
        import_timeout_s=int(e.get("DRIVESIM_IMPORT_TIMEOUT_S", "600")),
        web_dir=Path(e.get("DRIVESIM_WEB_DIR", repo / "server" / "web")),
        # 3 M tokens/day, counted like the bill (user, 2026-10-10;
        # provisional until recorded setups show real costs).
        daily_budget=float(e.get("DRIVESIM_DAILY_BUDGET", "3000000")),
        budget_tz=e.get("DRIVESIM_BUDGET_TZ", "America/Los_Angeles"),
        dev_user=e.get("DRIVESIM_DEV_USER") or None,
    )
