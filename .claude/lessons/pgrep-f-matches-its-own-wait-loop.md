# `pgrep -f <pattern>` inside a wait loop matches the loop itself

`while pgrep -f 'make web-build'; do sleep 10; done; deploy...` never ends: the loop's own `bash -c` command line contains the pattern, so pgrep always finds a match. It happened twice on 2026-10-10/11 (a stuck monitor, then a deploy that silently never ran after the user said "commit and deploy").

- How to apply: wait on a PID (`wait $pid`, or `while kill -0 $pid`), chain the commands with `&&` in one background job, or anchor the pattern so it can't match itself (`pgrep -f '^make web-build'` matches only processes whose command line starts with it).
- After queuing anything behind a wait, check that it actually started before telling the user it is running.
