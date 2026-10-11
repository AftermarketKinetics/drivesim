# drivesim: the server image. The FastAPI service (uv2nix env from
# server/uv.lock), the robot checker, the motor library, and the pages with
# the browser sim.
#   nix-build deploy/server-image.nix && docker load < result
#
# The house robots (fixtures/simple_bot and its _1lb, _3lb, _30lb
# versions; user 2026-10-10, 2026-10-11) ship in
# /app/house and are seeded into the store as public AI opponents at start.
#
# The sim (server/web/sim/) is the local `make web-build` output (user,
# 2026-10-10: ship the local build); deploy/push.sh rebuilds it before every
# push, and this build fails if it is missing. Runs as 1000:100, so robot dirs
# and the database on the mint bind mounts are the operator's. Compose
# publishes no port: only Caddy reaches it, which is what makes the
# X-authentik-* identity headers trustworthy. Adapted from the analyzer's
# deploy/analyzer-image.nix (2026-10-10).
{ pkgs ? import ../nix/pkgs.nix }:

let
  inherit (pkgs) lib;
  env = import ../nix/server-env.nix { inherit pkgs; };
  check = import ../nix/drivesim-check.nix { inherit pkgs; };
  # Pages and the sim, without sidecar docs.
  web = lib.fileset.toSource {
    root = ../server/web;
    fileset = lib.fileset.fileFilter (f: !(lib.hasSuffix ".md" f.name)) ../server/web;
  };
  # House robots' bundles, without sidecar docs (house.py).
  house = lib.fileset.toSource {
    root = ../fixtures;
    fileset = lib.fileset.fileFilter (f: !(lib.hasSuffix ".md" f.name)) (lib.fileset.unions [
      ../fixtures/simple_bot ../fixtures/simple_bot_1lb ../fixtures/simple_bot_3lb ../fixtures/simple_bot_30lb
    ]);
  };
  app = pkgs.runCommand "drivesim-app-files" { } ''
    test -f ${web}/sim/drivesim_web_bg.wasm \
      || { echo "server/web/sim is missing: run make web-build first" >&2; exit 1; }
    mkdir -p $out/app
    ln -s ${web} $out/app/web
    cp ${../data/motors.toml} $out/app/motors.toml
    ln -s ${house} $out/app/house
  '';
in
pkgs.dockerTools.buildLayeredImage {
  name = "drivesim";
  tag = "latest";
  contents = [ env app pkgs.cacert pkgs.dockerTools.fakeNss ];
  extraCommands = "mkdir -p tmp && chmod 1777 tmp";
  config = {
    User = "1000:100";
    WorkingDir = "/app";
    Entrypoint = [ "${env}/bin/uvicorn" "drivesim_server.app:app" "--host" "0.0.0.0" "--port" "8000"
                   "--proxy-headers" "--forwarded-allow-ips" "*" ];
    Env = [
      "DRIVESIM_ROBOTS_ROOT=/robots"
      "DRIVESIM_DB_PATH=/data/drivesim.sqlite3"
      "DRIVESIM_IMPORT_MODE=broker"
      "DRIVESIM_BROKER_URL=http://drivesim-broker:8711"
      "DRIVESIM_CHECK_BIN=${check}/bin/drivesim-check"
      "DRIVESIM_MOTORS_TOML=/app/motors.toml"
      "DRIVESIM_WEB_DIR=/app/web"
      "DRIVESIM_HOUSE_ROOT=/app/house"
      "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
      "HOME=/tmp"
    ];
  };
}
