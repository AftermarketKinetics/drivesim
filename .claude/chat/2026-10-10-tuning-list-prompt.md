# 2026-10-10 tuning list in the setup prompt

## 21:45 tuning list at end of setup
**User:** make the system prompt present to the user all of the possible tweaks and modifications that could be made to more properly simulate the robot (things like mu for tyre traction) at the end of setup
**Did:**
- Added step 5 to `server/drivesim_server/prompt.py`: once ready, list every tunable robot.toml setting with value + measured/given/guessed, top 2–3 first.
- Test `test_the_prompt_ends_with_every_tunable_setting` in `server/tests/test_tools.py`; sidecars updated; 35 tests pass.
