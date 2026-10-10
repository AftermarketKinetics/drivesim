"""House robots: bundles shipped with the server, seeded into the store as
public robots so the AI opponent has choices before any owner publishes
one (user, 2026-10-10: start with simple_bot, as a database robot).

Each entry in HOUSE is a bundle directory under ``settings.house_root``.
Its robot gets a fixed id, so the row is created once and every start
only refreshes its bundle files from the image.
"""

from __future__ import annotations

import hashlib
import shutil

from .config import Settings
from .db import Store
from .importer import BUNDLE

#: The owner house robots are listed under. Not a valid authentik login
#: on accelerate, so no one can open or delete them as owner.
HOUSE_USER = "drivesim-house"

#: Bundle directory name -> (display name, weight class lb).
HOUSE = {"simple_bot": ("simple_bot", 12)}


def house_id(name: str) -> str:
    """The fixed robot id of house robot `name`: 32 hex chars, like a uuid4."""
    return hashlib.sha256(f"house:{name}".encode()).hexdigest()[:32]


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
        shutil.rmtree(dst, ignore_errors=True)
        shutil.copytree(src, dst, ignore=shutil.ignore_patterns("*.md"))
        store.seed_robot(rid, HOUSE_USER, name, weight_lb)
        seeded.append(rid)
    return seeded
