# drivesim build and test entry points; run inside nix-shell.
#   make test     importer tests + cargo test, clippy and fmt over the crates
#   make web-dev  serve the repo on localhost:8417 (pad test, web builds)
#   make fixtures re-import fixtures/simple_bot from the fixture STEP
#   make server-dev  the server on localhost:8420 (dev user, local import)
#   make web-build   the browser app into server/web/sim/

.PHONY: test test-import test-rust

test: test-import test-rust

test-rust:
	cargo test -q
	cargo clippy -q --all-targets -- -D warnings
	cargo fmt --check

test-import:
	python -m unittest discover -s tools/import/tests

# Serve the repo on localhost with cross-origin isolation (tools/dev_server.py).
.PHONY: web-dev
web-dev:
	python tools/dev_server.py

# Rebuild the fixture bundle's geometry from the fixture STEP (robot.toml
# is hand-written and kept).
.PHONY: fixtures
fixtures:
	rm -rf fixtures/simple_bot/meshes fixtures/simple_bot/parts.json
	python tools/import/step_import.py tools/import/tests/fixtures/simple_bot.step fixtures/simple_bot

# Server and broker (server/ is a uv project; uv comes from shell.nix).
.PHONY: test-server test-broker server-dev
test: test-server test-broker
test-server:
	cargo build -q --release -p drivesim-cli
	cd server && uv run pytest -q
test-broker:
	cd broker && python -m unittest -q test_broker
# The server on http://localhost:8420 as user "dev", importing locally (no
# sandbox, no authentik). Needs ANTHROPIC_API_KEY or an `ant auth login`.
server-dev:
	cargo build -q --release -p drivesim-cli
	cd server && DRIVESIM_DEV_USER=dev DRIVESIM_IMPORT_PYTHON=$$(command -v python3) \
	  uv run uvicorn drivesim_server.app:app --host 127.0.0.1 --port 8420

# The browser app -> server/web/sim/ (served by the server at /sim/, and by
# web-dev at /server/web/sim/). The test robot without a server:
#   http://localhost:8417/server/web/sim/?bundle=/fixtures/simple_bot/&motors=/data/motors.toml
.PHONY: web-build
web-build:
	cd web && cargo build --release --target wasm32-unknown-unknown
	rm -rf server/web/sim && mkdir -p server/web/sim
	wasm-bindgen --target web --no-typescript --out-name drivesim_web --out-dir server/web/sim \
	  web/target/wasm32-unknown-unknown/release/drivesim-web.wasm
	wasm-opt -Os --strip-debug --strip-producers --enable-bulk-memory --enable-nontrapping-float-to-int \
	  server/web/sim/drivesim_web_bg.wasm -o server/web/sim/drivesim_web_bg.wasm
	cp web/index.html server/web/sim/index.html
