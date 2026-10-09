#!/usr/bin/env python3
"""drivesim's import broker: the only process that holds the Docker socket.

The server asks it to run the STEP importer on one robot, and it can ask for
nothing else: there is no command to choose, only a robot id and a timeout.
Each import runs in a fresh gVisor sandbox with no network, the STEP mounted
read-only and only the robot's bundle/ writable (docs/drivesim.md §4).
Adapted from the Combat Robot Analyzer's cra-broker (2026-10-09), which
exec'd chosen commands into long-lived sessions; drivesim needs one fixed
command per upload. Standard library only.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROUTE = re.compile(r"^/imports/([0-9a-f]{32})/run$")
TIMEOUT_MIN, TIMEOUT_MAX, TIMEOUT_DEFAULT = 10, 1800, 600
STDOUT_KEEP, STDERR_KEEP = 1 << 20, 8192


def config(env=os.environ) -> dict[str, str]:
    return {
        # Robot dirs *on the Docker host*: bind-mount sources are resolved by
        # the daemon, not inside this container. Same directory as the
        # server's DRIVESIM_ROBOTS_ROOT.
        "host_root": env["DRIVESIM_ROBOTS_HOST_ROOT"],
        "image": env.get("DRIVESIM_IMPORT_IMAGE", "drivesim-import:latest"),
        "runtime": env.get("DRIVESIM_SANDBOX_RUNTIME", "runsc"),
        "cpus": env.get("DRIVESIM_SANDBOX_CPUS", "2"),
        "memory": env.get("DRIVESIM_SANDBOX_MEMORY", "4g"),
        "bind": env.get("DRIVESIM_BROKER_BIND", "127.0.0.1:8711"),
        "parallel": env.get("DRIVESIM_BROKER_PARALLEL", "2"),
    }


def clamp_timeout(t) -> int:
    try:
        t = int(t)
    except (TypeError, ValueError):
        t = TIMEOUT_DEFAULT
    return max(TIMEOUT_MIN, min(TIMEOUT_MAX, t))


def import_argv(robot_id: str, timeout: int, cfg: dict[str, str]) -> list[str]:
    """The one docker command this broker ever runs."""
    assert re.fullmatch(r"[0-9a-f]{32}", robot_id), robot_id
    d = f"{cfg['host_root']}/{robot_id}"
    return [
        "docker", "run", "--rm", "--name", f"drivesim-import-{robot_id}",
        "--runtime", cfg["runtime"], "--network=none", "--read-only",
        "--tmpfs", "/tmp:rw,size=1g", "--cap-drop=ALL",
        "--security-opt", "no-new-privileges", "--pids-limit=256",
        "--cpus", cfg["cpus"], "--memory", cfg["memory"],
        "-v", f"{d}/robot.step:/work/robot.step:ro",
        "-v", f"{d}/bundle:/work/bundle:rw",
        cfg["image"],
        # umask 0: under gVisor the default 022 leaves sandbox-made files
        # unwritable by the server (the analyzer's finding, 2026-10-08).
        # timeout inside the sandbox: killing the docker CLI out here would
        # leave the import running in there.
        "sh", "-c", f"umask 0 && exec timeout -s KILL {timeout} "
                    "python3 /opt/import/step_import.py /work/robot.step /work/bundle",
    ]


def run_import(robot_id: str, timeout: int, cfg: dict[str, str]) -> dict:
    p = subprocess.run(import_argv(robot_id, timeout, cfg), capture_output=True, timeout=timeout + 60)
    return {"code": p.returncode,
            "stdout": p.stdout.decode(errors="replace")[-STDOUT_KEEP:],
            "stderr": p.stderr.decode(errors="replace")[-STDERR_KEEP:]}


def make_handler(cfg: dict[str, str], gate: threading.Semaphore, runner=run_import):
    class Handler(BaseHTTPRequestHandler):
        def _reply(self, code: int, body: dict) -> None:
            data = json.dumps(body).encode()
            self.send_response(code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_POST(self):  # noqa: N802 (http.server's name)
            m = ROUTE.match(self.path)
            if not m:
                return self._reply(404, {"error": "not found"})
            try:
                n = int(self.headers.get("Content-Length") or 0)
                body = json.loads(self.rfile.read(n) or b"{}")
                with gate:
                    out = runner(m.group(1), clamp_timeout(body.get("timeout")), cfg)
                self._reply(200, out)
            except Exception as exc:
                self._reply(500, {"error": f"{type(exc).__name__}: {exc}"})

        def do_GET(self):  # noqa: N802
            self._reply(404, {"error": "not found"})

        def log_message(self, fmt, *args):  # requests to stderr, never stdout
            sys.stderr.write("broker: " + fmt % args + "\n")

    return Handler


def main() -> None:
    cfg = config()
    host, port = cfg["bind"].rsplit(":", 1)
    gate = threading.Semaphore(int(cfg["parallel"]))
    ThreadingHTTPServer((host, int(port)), make_handler(cfg, gate)).serve_forever()


if __name__ == "__main__":
    main()
