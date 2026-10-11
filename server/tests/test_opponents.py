"""Public robots as AI opponents: publishing, who may fetch what, the
seeded house robot, and the schema migration."""

from __future__ import annotations

import shutil
import sqlite3

from conftest import FIXTURE_BUNDLE
from drivesim_server.db import Store
from drivesim_server.house import HOUSE, HOUSE_USER, house_id, seed_house

BOB = {"X-authentik-username": "bob"}
SIMPLE = house_id("simple_bot")
#: Every house robot, in class order (1, 3, 12, 30 lb).
HOUSE_IDS = [house_id(d) for d in HOUSE]
HOUSE_ROWS = [(name, lb, False) for name, lb in HOUSE.values()]


def ready_robot(client, user="alice", name="Seraphs V1"):
    """A drivable robot of `user`'s, as a finished setup leaves it."""
    s = client.app.state.s
    rid = s["store"].create_robot(user, name)
    shutil.copytree(FIXTURE_BUNDLE, s["settings"].robots_root / rid / "bundle")
    s["store"].set_status(rid, "ready")
    s["store"].set_checked(rid, True)
    return rid


def test_the_house_robots_are_public_opponents_one_per_class(client):
    for headers in ({}, BOB):
        opps = client.get("/api/opponents", headers=headers).json()
        assert [(o["id"], o["name"], o["weight_lb"], o["mine"]) for o in opps] == [
            (house_id("simple_bot_1lb"), "simple_bot 1 lb", 1, False),
            (house_id("simple_bot_3lb"), "simple_bot 3 lb", 3, False),
            (SIMPLE, "simple_bot", 12, False),
            (house_id("simple_bot_30lb"), "simple_bot 30 lb", 30, False),
        ]
        assert client.get(f"/api/robots/{SIMPLE}/bundle/parts.json", headers=headers).status_code == 200
        assert client.get(f"/api/robots/{SIMPLE}/bundle/meshes/000.stl", headers=headers).status_code == 200
    # Drivable, but not anyone's to read or change.
    assert client.get(f"/api/robots/{SIMPLE}").status_code == 404
    assert client.get(f"/api/robots/{SIMPLE}/messages").status_code == 404
    assert client.delete(f"/api/robots/{SIMPLE}").status_code == 404
    assert client.get("/api/robots").json() == []


def test_an_owner_publishes_and_unpublishes_their_robot(client):
    rid = ready_robot(client)
    bundle = f"/api/robots/{rid}/bundle/robot.toml"
    assert client.get(bundle, headers=BOB).status_code == 404
    r = client.put(f"/api/robots/{rid}/public", json={"public": True, "weight_lb": 12})
    assert r.status_code == 200
    assert client.get(f"/api/robots/{rid}").json()["public"] == 1
    names = [(o["name"], o["weight_lb"], o["mine"]) for o in client.get("/api/opponents", headers=BOB).json()]
    assert names == HOUSE_ROWS[:2] + [("Seraphs V1", 12, False)] + HOUSE_ROWS[2:]
    assert [o["mine"] for o in client.get("/api/opponents").json()] == [False, False, True, False, False]
    # Bob may drive against it: the bundle, nothing else.
    assert client.get(bundle, headers=BOB).status_code == 200
    assert client.get(f"/api/robots/{rid}", headers=BOB).status_code == 404
    assert client.get(f"/api/robots/{rid}/messages", headers=BOB).status_code == 404
    assert client.put(f"/api/robots/{rid}/public", headers=BOB,
                      json={"public": False}).status_code == 404
    # Private again: gone from the list, its class remembered.
    assert client.put(f"/api/robots/{rid}/public", json={"public": False, "weight_lb": 12}).status_code == 200
    assert [o["id"] for o in client.get("/api/opponents").json()] == HOUSE_IDS
    assert client.get(bundle, headers=BOB).status_code == 404
    assert client.get(f"/api/robots/{rid}").json()["weight_lb"] == 12


def test_opponents_are_ordered_by_weight_class(client):
    for name, lb in (("heavy", 30), ("ant", 3)):
        rid = ready_robot(client, name=name)
        client.put(f"/api/robots/{rid}/public", json={"public": True, "weight_lb": lb})
    assert [o["weight_lb"] for o in client.get("/api/opponents").json()] == [1, 3, 3, 12, 30, 30]


def test_publishing_needs_a_weight_class_and_a_drivable_robot(client):
    rid = ready_robot(client)
    for body in ({"public": True}, {"public": True, "weight_lb": 5}, {"public": False, "weight_lb": 7}):
        assert client.put(f"/api/robots/{rid}/public", json=body).status_code == 422, body
    s = client.app.state.s
    s["store"].set_checked(rid, False)
    r = client.put(f"/api/robots/{rid}/public", json={"public": True, "weight_lb": 12})
    assert r.status_code == 409
    # A public robot whose check later fails drops out of the list.
    s["store"].set_checked(rid, True)
    client.put(f"/api/robots/{rid}/public", json={"public": True, "weight_lb": 12})
    s["store"].set_checked(rid, False)
    assert [o["id"] for o in client.get("/api/opponents").json()] == HOUSE_IDS


def test_seeding_is_idempotent_and_refreshes_the_bundle(settings):
    store = Store(settings.db_path)
    assert seed_house(settings, store) == HOUSE_IDS
    stale = settings.robots_root / SIMPLE / "bundle" / "meshes" / "999.stl"
    stale.write_bytes(b"old")
    store.set_public(SIMPLE, True, 12)
    assert seed_house(settings, store) == HOUSE_IDS
    rows = [r for r in store.all_robots() if r["username"] == HOUSE_USER]
    assert len(rows) == len(HOUSE) and not stale.exists()
    assert not list((settings.robots_root / SIMPLE / "bundle").rglob("*.md"))


def test_seeding_twice_from_a_read_only_image_works(settings, tmp_path):
    """The image's bundles are read-only Nix store files, and copytree
    copies modes: a second start used to fail to replace them."""
    import os
    import stat
    from dataclasses import replace
    root = tmp_path / "house"
    shutil.copytree(settings.house_root / "simple_bot", root / "simple_bot",
                    ignore=shutil.ignore_patterns("*.md"))
    for d, _, files in os.walk(root, topdown=False):
        for f in files:
            os.chmod(os.path.join(d, f), stat.S_IRUSR | stat.S_IRGRP | stat.S_IROTH)
        os.chmod(d, stat.S_IRUSR | stat.S_IXUSR | stat.S_IRGRP | stat.S_IXGRP | stat.S_IROTH | stat.S_IXOTH)
    s = replace(settings, house_root=root)
    store = Store(s.db_path)
    try:
        assert seed_house(s, store) == [SIMPLE]
        assert seed_house(s, store) == [SIMPLE]
        assert (s.robots_root / SIMPLE / "bundle" / "robot.toml").is_file()
    finally:
        for d, _, _ in os.walk(root):
            os.chmod(d, 0o755)


def test_a_missing_house_root_seeds_nothing(settings, tmp_path):
    from dataclasses import replace
    s = replace(settings, house_root=tmp_path / "nowhere")
    assert seed_house(s, Store(s.db_path)) == []


def test_an_older_database_gains_the_new_columns(tmp_path):
    db = tmp_path / "old.sqlite3"
    with sqlite3.connect(db) as c:
        c.execute("CREATE TABLE robots (id TEXT PRIMARY KEY, username TEXT NOT NULL, "
                  "name TEXT NOT NULL DEFAULT '', status TEXT NOT NULL, error TEXT NOT NULL DEFAULT '', "
                  "checked INTEGER NOT NULL DEFAULT 0, created REAL NOT NULL)")
        c.execute("INSERT INTO robots VALUES ('a' , 'alice', 'old', 'ready', '', 1, 0)")
    store = Store(db)
    assert store.robot("a")["public"] == 0 and store.robot("a")["weight_lb"] == 0
    Store(db)  # a second start must not try to add them again
