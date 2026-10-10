# 2026-10-10 crop bigwheels video

## 20:20 crop browser/taskbar out of demo clip
**User:** crop the bigwheel.mp4 video to crop out the chrome and windows bars
**Did:**
- `server/web/welcome/bigwheels.mp4` cropped 1920×1080 → 1920×944 (rows 87–1030; Chrome UI ended at row 86, taskbar began at 1032), re-encoded x264 crf 23 + faststart, audio copied.
- `index.html.md` sidecar updated to the new dimensions.

## 20:30 deploy
**User:** deploy
**Did:**
- Planned a server-image-only deploy (only the video changed since the last deploy; the local sim build is unchanged). The auto-mode classifier denied building and loading the image on mint, so the deploy is left for the user to run.

## 20:45 deploy attempt (run by the user)
**User:** (ran `t=$(nix-shell --run 'nix-build ... server-image.nix') && ssh mint docker load < "$t" && ... up -d drivesim` via `!`)
**Did:**
- Image built (/nix/store/y6342csi…-drivesim.tar.gz) but load failed: the shell.nix banner went into `$t`. Gave the corrected load + up command; lesson `nix-shell-banner-pollutes-command-substitution.md`.

## 20:55 deploy (run by the user)
**User:** (ran the corrected `docker load` + `docker compose up -d drivesim` via `!`)
**Did:**
- Image loaded, drivesim recreated; a pasted line break made it `up -d` with no service name (others stayed Running, no change). Live check: /welcome/bigwheels.mp4 200 at 465335 bytes (the cropped file), landing 200.
