# padtest/index.html

## Function

Shows exactly what the browser's Gamepad API reports for the radio, so
the web input mapping is checked against the radio's channel monitor
(docs/drivesim.md §7), as `paddump.exe` did for Windows on 2026-10-09.

## Interface

Served by `make web-dev` at `/tools/padtest/`. Shows each pad's `id`,
its `mapping` ("standard" or empty), and every axis with its value and a
bar centred on 0; pressed buttons are listed.

## Implementation

Plain JavaScript polling `navigator.getGamepads()` every animation
frame; Chrome only exposes a pad after it has seen input from it. No
build step, so it works before any Rust exists.

## Assertions

- Displays raw API values: no deadzone, scaling or reordering.
