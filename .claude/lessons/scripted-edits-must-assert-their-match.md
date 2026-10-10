# A scripted text replacement must assert that it matched; otherwise a no-op passes silently and gets reported as done

On 2026-10-10 the wheel markings in `web/src/robot_render.rs` were
written with a Python `str.replace` whose search text no longer existed.
`cargo fmt` had already reflowed the function. The replace changed
nothing; the build, clippy and the sidecar update all went through. The
user drove a build described as having marked wheels that had none, and
the stale sidecar was committed before a `git status` showed only the
`.md` changed.

How to apply:
- Every scripted edit goes through a helper that asserts
  `old in text` before replacing (or uses the Edit tool, which fails
  loudly).
- Before saying a change is in, check its distinctive name is in the
  file (`grep -c`), or check the file appears in `git status`.
