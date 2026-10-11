"""House robots: bundles shipped with the server, seeded into the store as
public robots so the AI opponent has choices before any owner publishes
one (user, 2026-10-10: start with simple_bot, as a database robot;
2026-10-11: plus its 1, 3 and 30 lb versions).

Each entry in HOUSE is a bundle directory under ``settings.house_root``.
Its robot gets a fixed id, so the row is created once and every start
only refreshes its bundle files from the image.
"""

from __future__ import annotations

import hashlib
import os
import shutil
from pathlib import Path

from .config import Settings
from .db import Store
from .importer import BUNDLE

#: The owner house robots are listed under. Not a valid authentik login
#: on accelerate, so no one can open or delete them as owner.
HOUSE_USER = "drivesim-house"

#: Bundle directory name -> (display name, weight class lb): simple_bot
#: and its versions scaled to the other classes (user, 2026-10-11).
HOUSE = {
    "simple_bot_1lb": ("simple_bot 1 lb", 1),
    "simple_bot_3lb": ("simple_bot 3 lb", 3),
    "simple_bot": ("simple_bot", 12),
    "simple_bot_30lb": ("simple_bot 30 lb", 30),
}


def house_id(name: str) -> str:
    """The fixed robot id of house robot `name`: 32 hex chars, like a uuid4."""
    return hashlib.sha256(f"house:{name}".encode()).hexdigest()[:32]


def _writable(tree: Path) -> None:
    """Gives the owner write access throughout `tree`. Bundles come from
    the Nix store read-only, and copytree copies modes, so without this the
    next start's rmtree can't empty them and the copy fails (2026-10-11:
    the server crash-looped on its first restart after a deploy)."""
    for root, dirs, files in os.walk(tree):
        os.chmod(root, 0o755)
        for f in files:
            os.chmod(os.path.join(root, f), 0o644)


def seed_house(settings: Settings, store: Store) -> list[str]:
    """Copy each house bundle into robots_root and make sure its row
    exists; returns the ids seeded. A missing bundle is skipped."""
    seeded = []
    for dirname, (name, weight_lb) in HOUSE.items():
        src = settings.house_root / dirname
        if not (src / "parts.json").is_file():
            continue
        rid = house_id(dirname)
        dst = settings.robots_root / rid / BUNDLE
        if dst.exists():
            _writable(dst)
            shutil.rmtree(dst)
        shutil.copytree(src, dst, ignore=shutil.ignore_patterns("*.md"))
        _writable(dst)
        store.seed_robot(rid, HOUSE_USER, name, weight_lb)
        seeded.append(rid)
    return seeded
