# pkgs.nix

## Function
The repo's one nixpkgs pin, imported by `shell.nix` and the deploy images,
so the dev shell and the sandbox build from the same packages.

## Interface
- `import ./nix/pkgs.nix` gives the package set.

## Implementation
- nixos-unstable rev 8ce4ef6c (2026-09-10), the same pin as
  ~/code/momentum, so the two repos share one store and one rustc.
- Moved out of `shell.nix` on 2026-10-10, when the import image needed the
  same pin.

## Assertions
- [ ] The rev and sha256 change together, and together with Momentum's
  pin.
