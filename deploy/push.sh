#!/usr/bin/env bash
# Build drivesim's three images here and load them into Docker on mint.
#
#   deploy/push.sh [--dry-run]        # from the repo root, inside nix-shell
#
# Rebuilds the browser sim first (make web-build, ~13 min: the server image
# ships the local build, user 2026-10-10), so the image can never carry a
# stale sim. Only builds, copies and `docker load`s: it does not start or
# restart anything. Bringing the services up is accelerate's job:
#   ssh mint 'cd ~/accelerate && docker compose up -d drivesim-broker drivesim'
# Adapted from the analyzer's deploy/push.sh (2026-10-10).
set -euo pipefail

MINT=${MINT_HOST:-mint}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DRY=0
[ "${1:-}" = "--dry-run" ] && DRY=1
cd "$ROOT"

make web-build

declare -A IMAGES=(
  [drivesim-import]="deploy/import-image.nix"
  [drivesim-broker]="deploy/broker-image.nix"
  [drivesim]="deploy/server-image.nix"
)

for name in "${!IMAGES[@]}"; do
  tarball=$(nix-build --no-out-link "${IMAGES[$name]}")
  size=$(du -h "$tarball" | cut -f1)
  echo "==> $name: $tarball ($size)" >&2
  if [ "$DRY" -eq 0 ]; then
    # Stream rather than copy, so mint only ever holds the loaded image.
    ssh "$MINT" docker load < "$tarball"
  fi
done

if [ "$DRY" -eq 0 ]; then
  # Robot dirs are bind-mounted into every import sandbox, which runs as
  # nobody; the database is the server's alone.
  ssh "$MINT" 'mkdir -p ~/drivesim/robots ~/drivesim/data && chmod 777 ~/drivesim/robots && chmod 700 ~/drivesim/data'
fi
