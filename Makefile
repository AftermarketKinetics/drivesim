# drivesim build and test entry points; run inside nix-shell.
#   make test     importer tests + cargo test, clippy and fmt over the crates
#   make web-dev  serve the repo on localhost:8417 (pad test, web builds)
#   make fixtures re-import fixtures/simple_bot from the fixture STEP

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
