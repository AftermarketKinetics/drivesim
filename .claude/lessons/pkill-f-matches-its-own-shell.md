# `pkill -f <pattern>` kills the shell that runs it when the pattern appears in that shell's own command line

The Bash tool runs each command as `bash -c '<the whole command>'`, so
any `pkill -f "foo"` inside it matches its own parent shell, which
contains the text "foo". That kills the shell (exit 144) and loses the
rest of the command. It happened twice in this project (2026-10-09 and
2026-10-10).

How to apply: use a pattern that cannot match itself, `[f]oo`, which
matches "foo" but not the literal text `[f]oo`. Better still, stop
background jobs with TaskStop by their id.
