After changing physics the user will try in the browser, run `make web-build` in the same change: the page runs whatever is in server/web/sim/.

2026-10-10: the roof landed in crates/physics with tests, but server/web/sim/ was still the earlier build, so the user drove without it, saw robots "fly through the upper walls", and reported a collision bug. The walls were fine (0/120 probe throws escaped); robots went over the unmarked 2.4 m top. Check the build timestamp against the change before debugging a browser report.
