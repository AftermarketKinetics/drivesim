# test_broker.py

## Function
Tests the broker's guarantees without Docker.

## Interface
`python -m unittest test_broker` from `broker/`, or `make test-broker`.
- **Arguments:** `test_sandbox_flags_and_mounts`, `test_ids_must_be_32_hex`,
  `test_timeouts_are_clamped`.
- **HTTP:** `test_run_reaches_the_runner_with_a_clamped_timeout` and
  `test_anything_else_is_404`, against a real local HTTP server with a fake
  runner.

## Implementation
`make_handler` takes the runner as an argument, so the HTTP layer is tested
end to end with nothing actually run.

## Assertions
- A real sandboxed import is checked on accelerate at deployment (gVisor
  isn't installed in WSL), not here.
