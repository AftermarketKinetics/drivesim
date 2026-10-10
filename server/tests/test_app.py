"""The HTTP surface end to end: upload -> local import -> scripted role
conversation -> checked robot -> bundle served; ownership and limits."""

from __future__ import annotations

from dataclasses import replace

from conftest import FIXTURE_BUNDLE, FIXTURE_STEP, wait_for

BOB = {"X-authentik-username": "bob"}


def upload(client, data=None, name="simple_bot.step"):
    data = data if data is not None else FIXTURE_STEP.read_bytes()
    return client.post("/api/robots", files={"file": (name, data)})


def test_upload_imports_then_the_conversation_writes_a_checked_robot(client, model):
    model.queue(
        [{"type": "tool_use", "id": "t1", "name": "parts_summary", "input": {}}],
        [{"type": "tool_use", "id": "t2", "name": "write_robot_toml",
          "input": {"robot_toml": (FIXTURE_BUNDLE / "robot.toml").read_text()}}],
        [{"type": "text", "text": "Ready to drive."}],
    )
    r = upload(client)
    assert r.status_code == 201
    rid = r.json()["id"]
    assert wait_for(lambda: client.get(f"/api/robots/{rid}").json()["checked"] == 1), \
        client.get(f"/api/robots/{rid}").json()
    info = client.get(f"/api/robots/{rid}").json()
    assert info["status"] == "ready" and info["name"] == "simple_bot"
    # The model saw the import notice, then each tool result.
    first = model.calls[0][-1]["content"][0]["text"]
    assert "simple_bot" in first and "14 parts" in first
    summary = model.calls[1][-1]["content"][0]["content"]
    assert "wheel_L" in summary
    assert '"ok": true' in model.calls[2][-1]["content"][0]["content"]
    # The bundle the sim loads.
    assert client.get(f"/api/robots/{rid}/bundle/parts.json").status_code == 200
    assert client.get(f"/api/robots/{rid}/bundle/meshes/000.stl").status_code == 200
    assert "simple_bot" in client.get(f"/api/robots/{rid}/bundle/robot.toml").text
    assert len(client.get(f"/api/robots/{rid}/messages").json()) == 6


def test_robots_are_the_owners_only(client):
    rid = upload(client).json()["id"]
    assert client.get(f"/api/robots/{rid}", headers=BOB).status_code == 404
    assert client.get("/api/robots", headers=BOB).json() == []
    assert client.get(f"/api/robots/{rid}/bundle/parts.json", headers=BOB).status_code == 404


def test_only_bundle_files_are_served(client):
    rid = upload(client).json()["id"]
    wait_for(lambda: client.get(f"/api/robots/{rid}").json()["status"] != "importing")
    # Percent-encoded so the client does not normalise the dots away.
    for rel in ("..%2Frobot.step", "robot.step", "meshes%2F..%2F..%2Frobot.step", "meshes/x.stl"):
        assert client.get(f"/api/robots/{rid}/bundle/{rel}").status_code == 404, rel


def test_a_broken_step_fails_the_robot_not_the_server(client):
    rid = upload(client, b"ISO-10303-21; this is not a model", "junk.step").json()["id"]
    assert wait_for(lambda: client.get(f"/api/robots/{rid}").json()["status"] == "failed")
    assert client.post(f"/api/robots/{rid}/messages", json={"text": "hi"}).status_code == 409


def test_oversized_uploads_are_refused(client):
    s = client.app.state.s
    s["settings"] = replace(s["settings"], max_upload_bytes=1000)
    assert upload(client).status_code == 413
    assert client.get("/api/robots").json() == []


def test_no_header_and_no_dev_user_is_401(client):
    s = client.app.state.s
    s["settings"] = replace(s["settings"], dev_user=None)
    assert client.get("/api/me").status_code == 401
    assert client.get("/api/me", headers=BOB).json() == {"username": "bob"}


def test_the_radio_model_is_downloadable_from_the_front_page(client):
    # The guides are one fragment that the front and landing pages load.
    for page in ("/", "/welcome/"):
        assert '<div id="guide">' in client.get(page).text and "/welcome/guide.html" in client.get(page).text
    guide = client.get("/welcome/guide.html").text
    assert 'href="/radio/accelerate-elrs-pocket.yml" download' in guide
    model = client.get("/radio/accelerate-elrs-pocket.yml")
    assert model.status_code == 200
    assert 'name: "Accelerate"' in model.text and "srcRaw: Ail" in model.text


def test_pages_are_revalidated_by_content_not_mtime(client):
    # A nix image gives every file a 1970 mtime: without these headers pages
    # were cached for years and same-size rebuilds came back 304 (the
    # analyzer's lesson). Every page and the sim must revalidate by content.
    first = client.get("/")
    assert first.headers["cache-control"] == "no-cache"
    etag = first.headers["etag"]
    assert client.get("/", headers={"If-None-Match": etag}).status_code == 304
    model = client.get("/radio/accelerate-elrs-pocket.yml")
    assert model.headers["cache-control"] == "no-cache" and model.headers["etag"] != etag


def test_the_robot_page_offers_delete_and_delete_removes_everything(client, settings):
    assert 'id="delete"' in client.get("/robot.html").text
    rid = upload(client).json()["id"]
    assert wait_for(lambda: client.get(f"/api/robots/{rid}").json()["status"] != "importing")
    assert client.delete(f"/api/robots/{rid}").status_code == 204
    assert client.get(f"/api/robots/{rid}").status_code == 404
    assert not (settings.robots_root / rid).exists()
    assert rid not in [r["id"] for r in client.get("/api/robots").json()]


def test_the_landing_page_shows_the_demo_video(client):
    assert 'src="/welcome/bigwheels.mp4"' in client.get("/welcome/").text
    video = client.get("/welcome/bigwheels.mp4")
    assert video.status_code == 200 and video.headers["content-type"] == "video/mp4"


def test_the_landing_page_uses_only_absolute_public_paths():
    # Caddy serves the landing page at / by an internal rewrite, so the
    # browser resolves relative paths against /, outside the public
    # /welcome/* and /radio/*: a relative bigwheels.mp4 hit the login.
    import re
    from conftest import REPO
    for name in ("index.html", "guide.html"):
        page = (REPO / "server/web/welcome" / name).read_text()
        for attr, url in re.findall(r'\b(src|href)="([^"]+)"', page):
            if url.startswith(("https://", "#")):
                continue
            assert url.startswith(("/welcome/", "/radio/", "/outpost.goauthentik.io/")), (name, url)
        for url in re.findall(r'fetch\("([^"]+)"', page):
            assert url.startswith("/welcome/"), (name, url)
