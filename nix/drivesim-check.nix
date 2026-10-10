# drivesim-check (crates/cli), the robot checker the server runs on every
# robot.toml the role conversation writes. Built from the Cargo workspace at
# the repo root with its Cargo.lock. Only the crates and the workspace
# manifests reach the store; target/, web/ and the rest do not.
{ pkgs ? import ./pkgs.nix }:

let
  inherit (pkgs) lib;
in
pkgs.rustPlatform.buildRustPackage {
  pname = "drivesim-check";
  version = "0.1.0";
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      (lib.fileset.fileFilter (f: !(lib.hasSuffix ".md" f.name)) ../crates)
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;
  cargoBuildFlags = [ "-p" "drivesim-cli" ];
  # The workspace's tests need the fixtures (fixtures/, data/); they run in
  # `make test`, not here.
  doCheck = false;
}
