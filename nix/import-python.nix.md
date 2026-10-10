# import-python.nix

## Function
The STEP importer's Python, with pythonocc-core (OCCT 7.9) and numpy,
shared by `shell.nix` and `deploy/import-image.nix`.

## Interface
- `import ./nix/import-python.nix { inherit pkgs; }` gives the
  interpreter, with the packages installed.

## Implementation
One definition means the importer tested in the dev shell is the build
that runs in the sandbox.

## Assertions
- [ ] Holds only what `tools/import` imports.
