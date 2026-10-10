A full-speed vertical spinner AI never triggers its stuck-reverse against a wall: the disc throws it off first.

Measured 2026-10-10 (fixture simple_bot, `tests/ai.rs`): driving into the −x wall with the disc at full, the AI was thrown back 1.6 m within 4 s and flipped (up.z −0.93) repeatedly, while its 1 s stuck timer never filled. A "backs off the wall" test passed, but for that reason, not because of the reverse logic. When a physics test checks an outcome, trace *why* it happened before trusting it; test the controller logic separately (ai_driver.rs unit tests).
