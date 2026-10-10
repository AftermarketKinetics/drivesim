# 2026-10-10 motor library

## 00:00 Fill motors.toml
**User:** use the combat-robot-search skill to fill out motors.toml with a wide selection of brushed and brushless motors for 1lb-30lb robots
**Asked:** (1) How should brushed motors be told apart in motors.toml? → Add `kind` field (Recommended). (2) Brushed gearmotors: how to record? → Bare motor, ratio in robot.toml (Recommended). (3) Where should numbers come from? → Discord + vendor pages (Recommended). (4) Roughly how big a library? → ~40 motors (Recommended).
**Did:**
- Plan: add `kind` to MotorSpec/list_motors; research per weight class with crsearch + vendor pages via parallel agents.

## 00:10 Continue after interrupt
**User:**  source /home/lily/code/drivesim/server/.venv/bin/activate
**User:** oops mistake, continue with research
**Did:**
- Relaunched 4 research agents (1/3/12/30 lb); added `MotorKind` (`kind`, default brushless) to `crates/robot/src/motors.rs`, `list_motors.py` + sidecars; physics test constructor updated. Robot/physics crate tests, fmt, server pytest pass.

## 00:40 Library filled
**Did:**
- `data/motors.toml`: 2 → 43 motors (1/3/12/30 lb sections; 13 brushed, 30 brushless), every field sourced, estimates in `provisional`; duplicates merged (Compact 1806, DeWalt, RS-775 — 12 lb agent's 722 kv misread fixed to 1083).
- `motor.rs` doc + sidecar and docs/drivesim.md §5 mention brushed; lesson `brushed-kv-check-the-test-voltage.md`. `make test` passes.
