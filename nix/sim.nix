# The browser sim (web/) as a store path: what `make web-build` writes to
# server/web/sim/, built here so the server image builds from a clean
# checkout (accelerate's clover migration takes this repo as a flake input).
# Same three steps as the Makefile: cargo for wasm32, wasm-bindgen (the CLI in
# this pin matches web/Cargo.toml's `=0.2.127`), wasm-opt; plus index.html.
{ pkgs ? import ./pkgs.nix }:

let
  inherit (pkgs) lib;
  noDocs = dir: lib.fileset.fileFilter (f: !(lib.hasSuffix ".md" f.name)) dir;
in
pkgs.rustPlatform.buildRustPackage {
  pname = "drivesim-sim";
  version = "0.1.0";
  # web/ path-depends on ../crates, whose manifests inherit `edition` from
  # the root workspace, so the root Cargo.toml comes along too.
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      (noDocs ../crates)
      ../web/Cargo.toml
      ../web/Cargo.lock
      ../web/index.html
      (noDocs ../web/src)
    ];
  };
  cargoRoot = "web";
  cargoLock.lockFile = ../web/Cargo.lock;

  nativeBuildInputs = [ pkgs.lld pkgs.wasm-bindgen-cli pkgs.binaryen ];

  # buildRustPackage's own build/install phases target the host; this is a
  # wasm32 build with no binary to install, so both are replaced.
  buildPhase = ''
    runHook preBuild
    cargo build --offline --release --target wasm32-unknown-unknown \
      --manifest-path web/Cargo.toml --target-dir target-wasm
    runHook postBuild
  '';
  installPhase = ''
    runHook preInstall
    mkdir -p $out
    wasm-bindgen --target web --no-typescript --out-name drivesim_web --out-dir $out \
      target-wasm/wasm32-unknown-unknown/release/drivesim-web.wasm
    wasm-opt -Os --strip-debug --strip-producers --enable-bulk-memory --enable-nontrapping-float-to-int \
      $out/drivesim_web_bg.wasm -o $out/drivesim_web_bg.wasm
    cp web/index.html $out/index.html
    runHook postInstall
  '';
  doCheck = false;
}
