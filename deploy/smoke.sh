#!/usr/bin/env bash
# End-to-end check of the import sandbox: one STEP through the broker's exact
# docker command (broker/broker.py import_argv), as the service runs it.
#
#   deploy/smoke.sh [runtime] [step]   # runtime: runsc (default) or runc
#
# Run from the repo root inside nix-shell. Builds and loads the image, imports
# the STEP (default: the importer fixture) into a scratch robot dir, and
# exits non-zero unless the bundle has parts.json and every mesh, written so
# the host user can delete them.
set -euo pipefail

RUNTIME=${1:-runsc}
STEP=${2:-tools/import/tests/fixtures/simple_bot.step}
ROOT=$(mktemp -d "$PWD/out/smoke.XXXXXX")
ID=$(head -c16 /dev/urandom | od -An -tx1 | tr -d ' \n')
trap 'rm -rf "$ROOT"' EXIT

docker load < "$(nix-build --no-out-link deploy/import-image.nix)" >/dev/null

# As the server leaves it: robot.step readable, bundle/ world-writable (the
# sandbox runs as nobody).
mkdir -p "$ROOT/$ID/bundle"
cp "$STEP" "$ROOT/$ID/robot.step"
chmod 0644 "$ROOT/$ID/robot.step"
chmod 0777 "$ROOT/$ID/bundle"

DRIVESIM_ROBOTS_HOST_ROOT="$ROOT" DRIVESIM_SANDBOX_RUNTIME="$RUNTIME" python3 - "$ID" <<'PY'
import json, os, sys, time
sys.path.insert(0, "broker")
from broker import config, run_import
rid = sys.argv[1]
t = time.time()
out = run_import(rid, 600, config())
assert out["code"] == 0, out["stderr"][-2000:]
bundle = os.path.join(os.environ["DRIVESIM_ROBOTS_HOST_ROOT"], rid, "bundle")
doc = json.load(open(os.path.join(bundle, "parts.json")))
missing = [p["mesh"] for p in doc["parts"] if not os.path.isfile(os.path.join(bundle, p["mesh"]))]
assert not missing, f"missing meshes: {missing[:5]}"
# The host user must be able to delete what the sandbox wrote (the umask 0
# in the broker's command; gVisor defaults to 022).
probe = os.path.join(bundle, doc["parts"][0]["mesh"])
assert os.stat(probe).st_mode & 0o002, f"{probe} is not world-writable"
print(f"ok: {len(doc['parts'])} parts in {time.time() - t:.1f} s")
PY
