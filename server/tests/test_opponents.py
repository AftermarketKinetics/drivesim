"""Public robots as AI opponents: publishing, who may fetch what, the
seeded house robot, and the schema migration."""

from __future__ import annotations

import shutil
import sqlite3

from conftest import FIXTURE_BUNDLE
from drivesim_server.db import Store
from drivesim_server.house import HOUSE_USER, house_id, seed_house

BOB = {"X-authentik-username": "bob"}
SIMPLE = house_id("simple_bot")


def ready_robot(client, user="alice", name="Seraphs V1"):
    """A drivable robot of `user`'s, as a finished setup leaves it."""
    s = client.app.state.s
    rid = s["store"].create_robot(user, name)
    shutil.copytree(FIXTURE_BUNDLE, s["settings"].robots_root / rid / "bundle")
    s["store"].set_status(rid, "ready")
    s["store"].set_checked(rid, True)
    return rid


def test_the_house_robot_is_a_public_12_lb_opponent(client):
    for headers in ({}, BOB):
        opps = client.get("/api/opponents", headers=headers).json()
        assert opps == [{"id": SIMPLE, "name": "simple_bot", "weight_lb": 12, "mine": False}]
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
    assert names == [("Seraphs V1", 12, False), ("simple_bot", 12, False)]
    assert [o["mine"] for o in client.get("/api/opponents").json()] == [True, False]
    # Bob may drive against it: the bundle, nothing else.
    assert client.get(bundle, headers=BOB).status_code == 200
    assert client.get(f"/api/robots/{rid}", headers=BOB).status_code == 404
    assert client.get(f"/api/robots/{rid}/messages", headers=BOB).status_code == 404
    assert client.put(f"/api/robots/{rid}/public", headers=BOB,
                      json={"public": False}).status_code == 404
    # Private again: gone from the list, its class remembered.
    assert client.put(f"/api/robots/{rid}/public", json={"public": False, "weight_lb": 12}).status_code == 200
    assert [o["id"] for o in client.get("/api/opponents").json()] == [SIMPLE]
    assert client.get(bundle, headers=BOB).status_code == 404
    assert client.get(f"/api/robots/{rid}").json()["weight_lb"] == 12


def test_opponents_are_ordered_by_weight_class(client):
    for name, lb in (("heavy", 30), ("ant", 3)):
        rid = ready_robot(client, name=name)
        client.put(f"/api/robots/{rid}/public", json={"public": True, "weight_lb": lb})
    assert [o["weight_lb"] for o in client.get("/api/opponents").json()] == [3, 12, 30]


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
    assert [o["id"] for o in client.get("/api/opponents").json()] == [SIMPLE]


def test_seeding_is_idempotent_and_refreshes_the_bundle(settings):
    store = Store(settings.db_path)
    assert seed_house(settings, store) == [SIMPLE]
    stale = settings.robots_root / SIMPLE / "bundle" / "meshes" / "999.stl"
    stale.write_bytes(b"old")
    store.set_public(SIMPLE, True, 12)
    assert seed_house(settings, store) == [SIMPLE]
    rows = [r for r in store.all_robots() if r["username"] == HOUSE_USER]
    assert len(rows) == 1 and not stale.exists()
    assert not list((settings.robots_root / SIMPLE / "bundle").rglob("*.md"))


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
