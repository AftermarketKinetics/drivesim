# pkgs.nix

## Function
The repo's one nixpkgs pin, imported by `shell.nix` and the deploy images,
so the dev shell and the sandbox build from the same packages.

## Interface
- `import ./nix/pkgs.nix { }` gives the package set for the current system;
  `{ system = "x86_64-linux"; }` gives it for a named one.
- Every file's default is `pkgs ? import …/pkgs.nix { }`.

## Implementation
- nixos-unstable rev 8ce4ef6c (2026-09-10), the same pin as
  ~/code/momentum, so the two repos share one store and one rustc.
- Moved out of `shell.nix` on 2026-10-10, when the import image needed the
  same pin.

- **`system` is an argument** (2026-10-20): accelerate's flake builds the
  deploy images, and pure flake evaluation has no `builtins.currentSystem`,
  which `import nixpkgs { }` falls back to. The default keeps the dev shell
  and plain `nix-build` working as before.

## Assertions
- [ ] Nixpkgs is instantiated with `{ inherit system; }`, never `{ }`.
- [ ] The rev and sha256 change together, and together with Momentum's
  pin.
