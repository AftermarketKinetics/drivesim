# The one nixpkgs pin, shared by shell.nix and the deploy images so the
# importer in the sandbox is the same build as the one tests run.
# nixos-unstable rev of 2026-09-10, the same pin as ~/code/momentum, so both
# repos share one store and one rustc (1.97.1). Everything used is on
# cache.nixos.org at this rev (pythonocc-core checked 2026-10-09, gvisor
# 2026-10-10); bump the two repos together.
# `system` defaults to the current one for the dev shell and nix-build; flake
# consumers (accelerate's images.nix) pass it, since pure evaluation has no
# builtins.currentSystem.
{ system ? builtins.currentSystem }:
import (fetchTarball {
  url = "https://github.com/NixOS/nixpkgs/archive/8ce4ef6cb6f871616146b9fe26d2a5ae594e94fe.tar.gz";
  sha256 = "1df6x8chkipc0j0yawmzr8430zpab5i2xk9hplyq1m5iqcl2c7y4";
}) { inherit system; }
