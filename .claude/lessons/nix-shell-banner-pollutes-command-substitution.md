# nix-shell's banner pollutes `$(nix-shell --run ...)`

`shell.nix`'s shellHook echoes "drivesim shell — docs/drivesim.md is the
design" to stdout, so `t=$(nix-shell --run 'nix-build ...')` captures the
banner plus the store path, and `docker load < "$t"` fails with "No such
file or directory" (2026-10-10, server-image deploy).

- Call `nix-build` directly (it needs no dev shell), or `| tail -1`.
- `deploy/push.sh` is safe: it runs inside the shell and calls nix-build itself.
