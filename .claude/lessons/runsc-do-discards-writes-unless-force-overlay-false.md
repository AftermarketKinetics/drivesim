# `runsc do` throws away everything the command writes unless you pass `do --force-overlay=false`, and it also gives the sandbox its own `/tmp`

Used to check the STEP importer under gVisor without registering runsc
with Docker (the analyzer's `runsc --rootless --network=none
--ignore-cgroups do <cmd>`, 2026-10-10):
- The importer ran to completion and printed every part, but no files
  appeared on the host.
- `runsc do` lays a writable tmpfs overlay over the host root and
  discards it on exit.
- The global `--overlay2=none` does not change that. The `do`
  subcommand's own `-force-overlay` (default true) does.
- Even with writes on, an output dir under `/tmp` came out empty: the
  sandbox mounts its own `/tmp`.

How to apply: write output outside `/tmp`, and run
`runsc --rootless --network=none --ignore-cgroups do --force-overlay=false <cmd>`.
Then compare the files with a runc or host run. The importer's output was
byte-identical.
