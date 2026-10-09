# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

drivesim is a browser driving simulator for NHRL combat robots. A user
signed in to accelerate (the self-hosted site behind authentik) uploads a
robot as a STEP assembly, a Claude conversation works out its wheels,
weapon and materials, and the robot is driven in the browser with a real
EdgeTX radio as a 3D rapier body, with true-RPM weapon precession.
**`docs/drivesim.md` is the source of truth** for decisions (§2, the
user's, dated), architecture and build order (§11). Momentum
(`~/code/momentum`) is the first robot and consumes this repo; nothing
here is Momentum-specific except the optional `momentum-core` control
hook.

## Commands

Everything runs inside the nix shell (`shell.nix`, pinned to the same
nixpkgs as momentum):

```sh
nix-shell --run 'make test'        # importer, crates (test/clippy/fmt), server, broker
nix-shell --run 'make web-dev'     # repo on localhost:8417 (pad test, web builds)
nix-shell --run 'make server-dev'  # server on localhost:8420, dev user, local import
nix-shell --run 'python tools/import/step_import.py robot.step out/robot'
```

`server/` is a uv project (`pyproject.toml` + `uv.lock`); `uv` comes from
the shell.

`out/` is gitignored scratch.

## Conventions

- **Every code file has a sidecar `<file>.md` beside it**, in the
  four-section format (Function / Interface / Implementation /
  Assertions). `.claude/sidecar.json` declares the scope; run the
  `sidecar-check` skill after touching a covered file.
- **One purpose per file**; module files say how the pieces relate.
- **Library crates take bytes, never file paths**, and use no
  native-only dependencies: they run in WASM as well as on the host.
- **Units are SI** everywhere past the importer (OCC reads STEP in mm and
  the importer converts).
- **nix first** for every tool; Cargo owns Rust crates.
- Session logs in `.claude/chat/`, lessons in `.claude/lessons/`.
- Deploying to accelerate changes the user's live site: ask first.
