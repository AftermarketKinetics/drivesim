# sim.nix

## Function
Builds the browser sim (`web/`, Bevy + drivesim-physics) into a store path
with the same files `make web-build` writes to `server/web/sim/`. The server
image takes the sim from here, so it builds from a clean git checkout, which
is how accelerate's clover deploy consumes this repo (a flake input).

## Interface
- `import ./nix/sim.nix { inherit pkgs; }` gives a directory with
  `drivesim_web_bg.wasm`, `drivesim_web.js` and `index.html`, served at
  `/sim/`.

## Implementation
- `rustPlatform.buildRustPackage` only for vendoring: `cargoRoot = "web"`
  and `cargoLock.lockFile = web/Cargo.lock` give an offline cargo; the build
  and install phases are replaced because the target is wasm32 and nothing
  is a host binary.
- The source fileset is the root `Cargo.toml` (the `crates/*` manifests
  inherit `edition` from it), `crates/`, and `web/`'s manifest, lock,
  `index.html` and `src/`, sidecars excluded. `web/target/` and the local
  `server/web/sim/` never reach the store.
- `--target-dir target-wasm` keeps the wasm build out of whatever target
  directory the cargo setup hook configures for the host.
- wasm-bindgen and wasm-opt flags are copied from the Makefile's
  `web-build`; the CLI in `nix/pkgs.nix` is 0.2.127, which
  `web/Cargo.toml` pins the crate to.
- The rustc in this pin already ships the wasm32 std (shell.nix); `lld` is
  its linker.

## Assertions
- [ ] The wasm-bindgen/wasm-opt flags and `index.html` copy match the
  Makefile's `web-build`; change both together.
- [ ] `web/Cargo.toml`'s `wasm-bindgen = "=x"` equals
  `pkgs.wasm-bindgen-cli.version`.
- [ ] Output file names stay `drivesim_web*` + `index.html` (the pages load
  `/sim/drivesim_web.js`).
