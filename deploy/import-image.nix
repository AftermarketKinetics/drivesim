# The STEP import sandbox image: `nix-build deploy/import-image.nix &&
# docker load < result`.
#
# Holds only what one import runs: the importer (tools/import) and its
# Python with OpenCASCADE, plus `sh` and `timeout` for the broker's fixed
# command (broker/broker.py import_argv). No network tools and no package
# manager. The broker starts one container per upload (runsc, --network=none,
# --read-only) and it exits when the import does. Adapted from the Combat
# Robot Analyzer's sandbox/image.nix (2026-10-10).
{ pkgs ? import ../nix/pkgs.nix { } }:

let
  python = import ../nix/import-python.nix { inherit pkgs; };

  # Only the importer's own modules: tests, fixtures, sidecar docs and
  # bytecode are for development, not the sandbox.
  importer = pkgs.runCommand "drivesim-importer" { } ''
    mkdir -p $out/opt/import
    cp ${../tools/import}/*.py $out/opt/import/
  '';
in
pkgs.dockerTools.buildLayeredImage {
  name = "drivesim-import";
  tag = "latest";

  contents = [
    python
    importer
    pkgs.dash      # `sh`, for the broker's `sh -c 'umask 0 && exec timeout ...'`
    pkgs.coreutils # `timeout`
    pkgs.dockerTools.fakeNss # /etc/passwd with `nobody`
  ];

  # /work holds the bind mounts (robot.step read-only, bundle/ writable);
  # /tmp must exist for the tmpfs the broker mounts over it.
  extraCommands = ''
    mkdir -p work tmp
    chmod 1777 tmp
    ln -s ${pkgs.dash}/bin/dash bin/sh
  '';

  config = {
    User = "65534:65534";
    WorkingDir = "/work";
    Env = [ "PATH=/bin" "HOME=/tmp" ];
  };
}
