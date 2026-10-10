# drivesim-server's Python environment, built by uv2nix from server/uv.lock,
# so production runs exactly the versions the lockfile pins (anthropic 1.x
# among them). Used by deploy/server-image.nix. Adapted from the Combat Robot
# Analyzer's nix/analyzer-env.nix (2026-10-10), with the same pyproject-nix
# revisions; bump the three together and re-run
#   nix-prefetch-url --unpack https://github.com/pyproject-nix/<repo>/archive/<rev>.tar.gz
{ pkgs ? import ./pkgs.nix }:

let
  inherit (pkgs) lib;
  fetch = repo: rev: sha256: builtins.fetchTarball {
    url = "https://github.com/pyproject-nix/${repo}/archive/${rev}.tar.gz";
    inherit sha256;
  };
  pyproject-nix = import (fetch "pyproject.nix" "6a8a7881d75b6f98967e7b8069f4ead331384301"
    "1grz8w8x6pznvlnvn292d8a9rb0ii7kwgid1a53fbmd6gk8qz0y5") { inherit lib; };
  uv2nix = import (fetch "uv2nix" "a24323e9e6ecbbf305c238845ff6c612d50467c0"
    "025iy1jam1jva3iiz09z3m9wph1zawjifijs1sdygq1zzadw6nf0") { inherit lib pyproject-nix; };
  build-systems = import (fetch "build-system-pkgs" "953d35d03a51dbecaf3feb3fb679eb63e90ebeb4"
    "100k300759g60zw99ff2l57zi4hy4yny4pa0mfr1hpbfrbsfwam7") { inherit lib pyproject-nix uv2nix; };

  workspace = uv2nix.lib.workspace.loadWorkspace { workspaceRoot = ../server; };

  # Only the package's own sources reach the store: server/ also holds
  # .venv/, var/ (robots and the database) and tests/.
  onlySources = final: prev: {
    drivesim-server = prev.drivesim-server.overrideAttrs (_: {
      src = lib.fileset.toSource {
        root = ../server;
        fileset = lib.fileset.unions [
          ../server/pyproject.toml
          (lib.fileset.fileFilter (f: f.hasExt "py") ../server/drivesim_server)
        ];
      };
    });
  };

  pythonSet = (pkgs.callPackage pyproject-nix.build.packages { python = pkgs.python3; }).overrideScope
    (lib.composeManyExtensions [
      build-systems.overlays.wheel
      (workspace.mkPyprojectOverlay { sourcePreference = "wheel"; })
      onlySources
    ]);
in
pythonSet.mkVirtualEnv "drivesim-server-env" workspace.deps.default
