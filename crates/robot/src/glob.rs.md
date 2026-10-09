# glob.rs

## Function

Matches robot.toml's part-name patterns against part names.

## Interface

`glob_match(pattern, name) -> bool`: `*` any run, `?` one character,
everything else literal, case-sensitive.

## Implementation

Iterative wildcard match with one backtrack point, no allocation beyond
the two char vectors. Case-sensitive because CAD names are, and `Wheel`
(product) vs `wheel_L` (instance) is a distinction users rely on.

## Assertions

- Tested: literals, `*`, `?`, empty names, backtracking.
