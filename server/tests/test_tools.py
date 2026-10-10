"""The role conversation's tools on the fixture bundle, with the real checker."""

from __future__ import annotations

import json

import pytest

from conftest import FIXTURE_BUNDLE
from drivesim_server.db import Store
from drivesim_server.prompt import MATERIALS, SYSTEM
from drivesim_server.tools import TOOLS, Ctx, ToolError


@pytest.fixture
def ctx(settings, bundle_dir, tmp_path):
    store = Store(tmp_path / "t.sqlite3")
    rid = store.create_robot("alice", "x")
    events = []

    async def emit(ev):
        events.append(ev)

    c = Ctx(rid, bundle_dir, settings, store, emit)
    c.events = events
    return c


async def test_parts_summary_lists_parts_with_axes(ctx):
    out = await TOOLS["parts_summary"].run(ctx, {})
    assert out["count"] == 14
    assert out["robot_id"] == ctx.robot_id
    wheel = next(line for line in out["parts"] if "wheel_L" in line)
    assert "axis (0.00,1.00,0.00) r 31.8 mm" in wheel
    assert (await TOOLS["parts_summary"].run(ctx, {"contains": "weapon"}))["count"] == 1


async def test_parts_summary_groups_repeats_and_puts_big_parts_first(ctx):
    path = ctx.bundle / "parts.json"
    doc = json.loads(path.read_text())
    screw = dict(doc["parts"][0], product="M3 screw", volume_m3=1e-8)
    doc["parts"] += [dict(screw, id=100 + i, instance=f"M3 screw:{i}") for i in range(5)]
    path.write_text(json.dumps(doc))
    out = await TOOLS["parts_summary"].run(ctx, {})
    assert out["count"] == 19
    screws = [line for line in out["parts"] if "M3 screw" in line]
    assert len(screws) == 1 and screws[0].startswith("5 x product 'M3 screw'")
    # Pairs stay apart; the biggest part (the 300 cm3 base plate) leads,
    # and the five screws, 0.05 cm3 together, come last.
    assert sum("wheel_" in line for line in out["parts"]) == 2
    assert "'base_plate'" in out["parts"][0]
    assert out["parts"][-1] == screws[0]
    assert out["robot_bbox"].endswith(" mm")


async def test_write_checks_and_marks_the_robot(ctx):
    text = (FIXTURE_BUNDLE / "robot.toml").read_text()
    out = await TOOLS["write_robot_toml"].run(ctx, {"robot_toml": text})
    assert out["ok"] is True and abs(out["total_mass_kg"] - 5.282) < 0.01
    assert ctx.store.robot(ctx.robot_id)["checked"] == 1
    assert ctx.events[-1]["type"] == "robot"
    bad = await TOOLS["write_robot_toml"].run(ctx, {"robot_toml": text.replace("mu = 1.4", "mu_typo = 1.4", 1)})
    assert bad["ok"] is False and "mu_typo" in bad["error"]
    assert ctx.store.robot(ctx.robot_id)["checked"] == 0


async def test_show_and_list_motors(ctx):
    assert "simple_bot" in (await TOOLS["show_robot_toml"].run(ctx, {}))["robot_toml"]
    ids = [m["id"] for m in (await TOOLS["list_motors"].run(ctx, {}))["motors"]]
    assert "badass-2820-1820kv" in ids


async def test_write_without_an_import_is_a_tool_error(ctx):
    (ctx.bundle / "parts.json").unlink()
    with pytest.raises(ToolError):
        await TOOLS["write_robot_toml"].run(ctx, {"robot_toml": "x"})


@pytest.mark.parametrize("material", MATERIALS)
async def test_prompt_materials_load(ctx, material):
    """Every material the prompt names is one the Rust loader knows."""
    text = (FIXTURE_BUNDLE / "robot.toml").read_text().replace('default = "al6061"', f'default = "{material}"')
    out = await TOOLS["write_robot_toml"].run(ctx, {"robot_toml": text})
    assert out["ok"] is True, out


def test_the_prompt_points_problems_to_discord_with_the_robot_id():
    assert "arctic256" in SYSTEM and "robot_id" in SYSTEM


def test_the_prompt_ends_with_every_tunable_setting():
    tuning = SYSTEM.split("\n5. ", 1)[1].split("\nThe robot file", 1)[0]
    for key in ("total_mass_kg", "mass_overrides", "mu", "motor_override", "kv_rpm_per_v", "r_ohm",
                "i_max_a", "pole_pairs", "rotor_inertia_kg_m2", "provisional", "efficiency",
                "battery_v", "axis_part", "side", "reverse_turn"):
        assert key in tuning, key
