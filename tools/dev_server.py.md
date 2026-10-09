# dev_server.py

## Function

Serves the repo over localhost for browser development (docs/drivesim.md
§2: local first, nothing deployed until the last step).

## Interface

```
python tools/dev_server.py [PORT]    # or: make web-dev
```

Default port 8417; serves the repo root, so the pad test is at
`http://localhost:8417/tools/padtest/`.

## Implementation

- Adds `Cross-Origin-Opener-Policy: same-origin` and
  `Cross-Origin-Embedder-Policy: require-corp`. Without them the browser
  refuses `SharedArrayBuffer` (WASM threads) and coarsens timers, which
  would skew the physics timing spike.
- `.wasm` is served as `application/wasm`, which streaming compilation
  requires.
- `no-store` so a rebuilt WASM file is never served stale.
- Binds to 127.0.0.1; WSL forwards localhost to the Windows browser.

## Assertions

- Never binds a non-loopback address: this is not the deployment server.
