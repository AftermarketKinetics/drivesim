# `lib.fileset.fileFilter` takes a path, not a file set

`lib.fileset.fileFilter pred (lib.fileset.unions [ ... ])` fails at build time with "Second argument is a file set, but it should be a path instead". To filter several directories, intersect: `lib.fileset.intersection (lib.fileset.fileFilter pred ../dir) (lib.fileset.unions [ ../dir/a ../dir/b ])`.

- Found 2026-10-11 when deploy/server-image.nix went from one house bundle to four. `nix-instantiate --parse` passed; only the build caught it, after push.sh had already spent ~15 min on the web build and loaded the other two images. Next time, `nix-build deploy/server-image.nix` before running push.sh after editing an image.
- To list what a file set will contain: `nix-instantiate --eval --read-write-mode -E '"${lib.fileset.toSource {...}}"'` and `ls` the path (plain `nix-build -E` on `toSource` printed nothing here).
