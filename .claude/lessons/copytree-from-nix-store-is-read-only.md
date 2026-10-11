# Files copied out of the Nix store stay read-only

`shutil.copytree` (and `cp -a`) copy permissions, so anything copied from a Nix store path into a writable location arrives read-only (dirs `r-x`, files `r--`). Replacing it later with `rmtree(..., ignore_errors=True)` then silently fails, and the next copy hits `FileExistsError`.

- Found 2026-10-11: house.py re-seeds its bundles from `/app/house` (the image's Nix store) on every start. The first start after any deploy worked; the next restart crash-looped the live server. Fixed by chmod-ing the tree writable after copying and before removing.
- Tests passed throughout because their source (the repo's fixtures) was writable. A test of anything that copies from the image should make its source read-only first.
