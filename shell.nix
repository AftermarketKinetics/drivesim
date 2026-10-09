{ pkgs ? import (fetchTarball {
    # nixos-unstable rev of 2026-09-10, the same pin as ~/code/momentum, so
    # both repos share one store and one rustc (1.97.1). Everything below is
    # on cache.nixos.org at this rev (pythonocc-core checked 2026-10-09);
    # bump the two repos together.
    url = "https://github.com/NixOS/nixpkgs/archive/8ce4ef6cb6f871616146b9fe26d2a5ae594e94fe.tar.gz";
    sha256 = "1df6x8chkipc0j0yawmzr8430zpab5i2xk9hplyq1m5iqcl2c7y4";
  }) {}
}:

# The whole toolchain: Rust for the sim crates and the browser (WASM) app,
# Python with OpenCASCADE for the STEP importer (tools/import). Rust *crate*
# dependencies are managed by Cargo (Cargo.lock), not this file; Python
# packages are listed here because the importer and server need so few.
pkgs.mkShell {
  packages = [
    # Host Rust; rustc comes from this pin (no rust-toolchain.toml). Its
    # sysroot already carries the wasm32-unknown-unknown std, so the web
    # build needs no other rustc (checked 2026-10-09).
    pkgs.rustc                  # 1.97.1
    pkgs.cargo
    pkgs.clippy
    pkgs.rustfmt
    pkgs.rust-analyzer

    # Web build: wasm-bindgen's CLI must match the `wasm-bindgen` crate
    # version exactly (web/Cargo.toml pins it to this); wasm-opt shrinks
    # the output.
    pkgs.lld                    # rustc's wasm32 target links with `lld`
    pkgs.wasm-bindgen-cli       # 0.2.127
    pkgs.binaryen               # 132 (wasm-opt)

    # Server (server/): its Python *packages* (FastAPI, anthropic 1.x, ...)
    # are uv's, in server/pyproject.toml + uv.lock; nix provides uv and the
    # interpreter (UV_PYTHON below). The docker CLI is for testing the
    # import broker locally.
    pkgs.uv
    pkgs.docker-client

    # STEP importer: exact B-rep tree, mass properties and tessellation.
    (pkgs.python3.withPackages (ps: [
      ps.pythonocc-core         # 7.9.0 (OCCT 7.9)
      ps.numpy
    ]))
  ];

  # The server's venv must not pick up the importer's OCC Python above.
  UV_PYTHON = "${pkgs.python3}/bin/python3";

  shellHook = ''
    echo "drivesim shell — docs/drivesim.md is the design"
  '';
}
