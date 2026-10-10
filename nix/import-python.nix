# The importer's Python: OpenCASCADE bindings and numpy. Used by shell.nix
# (tests, local imports) and by the sandbox image, so both run one build.
{ pkgs }:
pkgs.python3.withPackages (ps: [
  ps.pythonocc-core # 7.9.0 (OCCT 7.9)
  ps.numpy
])
