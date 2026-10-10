---
name: spin_axis
---
# spin.rs

Which way a weapon spins, from robot.toml's `spin` (docs/drivesim.md §5).
Positive weapon duty spins a rotor positively about its `axis_dir`, so
choosing the direction means choosing that vector's sign. The importer's
sign is arbitrary: the largest component comes out positive. That left
the fixture's vertical disc spinning front edge down. Every hit then
threw the robot itself into the air, and under full throttle it
launched itself (2026-10-10). Real verticals spin the front edge up,
which throws the opponent up and drives the attacker down.

```rust-sig
pub const VERTICAL_DEFAULT: &str = "front-up";
pub const HORIZONTAL_DEFAULT: &str = "ccw";
pub fn spin_axis(dir: [f64; 3], spin: Option<&str>) -> Result<[f64; 3], String>
```

```requires
dir is a unit vector in the robot frame (x forward, y left, z up)
```

```ensures
result is dir or -dir
upright axis (|dir.z| ≥ 0.5): "ccw" (default) gives result.z > 0, "cw" result.z < 0
cross axis (|dir.y| ≥ 0.5): "front-up" (default) gives result.y < 0, "front-down" result.y > 0
an axis along the robot (neither) is an error, whatever spin says
a spin word that does not fit the axis is an error naming the words that do
```

```test vertical_default
([0, 1, 0], None) => Ok([0, -1, 0])
([0, -1, 0], Some("front-down")) => Ok([0, 1, 0])
```

```test horizontal_default
([0, 0, 1], None) => Ok([0, 0, 1])
([0, 0, 1], Some("cw")) => Ok([0, 0, -1])
```

```test errors
([0, 1, 0], Some("cw")) => Err
([0, 0, 1], Some("front-up")) => Err
([1, 0, 0], None) => Err
```

## Implementation

- **Front-up needs dir.y < 0:** a point at the rotor's front, +x of its
  axis, moves along dir × x̂, whose z component is −dir.y.
- The unit tests in the file are these cases; there is no sidecar test
  runner in this repo.
- Wheels never come here. Their direction comes from the drive
  (`forward_sign`), and a wheel with `spin` is a load error.
