# Stage files by name, never `git add -A`: another session may have uncommitted work in the same repo

On 2026-10-10 a `git add -A` commit swept in another session's in-progress
motor library: `data/motors.toml` (+438 lines), `motors.rs`, `motor.rs`,
`list_motors.py`, their sidecars, its chat log and a lesson. The global
rule already says to leave other sessions' uncommitted files alone; `-A`
breaks it silently. Fixed with `git reset --soft HEAD~1`, then
`git restore --staged <their files>`, then `git commit -C ORIG_HEAD`, after
checking with md5sums that their files on disk did not change.

How to apply:
- Before committing, read `git status` and stage only the files this
  session touched, by name.
- A modified file you don't remember editing, or an untracked chat log
  that isn't yours, belongs to someone else: leave it unstaged and
  mention it.
