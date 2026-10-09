# common/mod.rs

## Function

Shared test setup: the fixture robot from `fixtures/simple_bot`, and
radio frames built the way the app builds them.

## Interface

`fixture_dir()`, `fixture_model()`, `fixture_mesh(path)`,
`radio(turn, throttle, weapon, armed) -> Frame`, using the fixture
mixer's channels: ch1 turn, ch2 throttle, ch3 weapon, ch5 arm.

## Implementation

Meshes are read from disk here; the app fetches them over HTTP. Both go
through the same `mesh` closure signature.

## Assertions

- Builds frames with `drivesim_input::frame_from_axes`, never by hand.
