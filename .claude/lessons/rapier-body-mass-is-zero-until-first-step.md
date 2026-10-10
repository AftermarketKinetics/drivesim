A rapier body's `mass()` reads 0 right after spawning; collider masses are summed into the body only on the first `world.step()`.

2026-10-10: `a_rival_drives_with_its_own_model` asserted the rival's summed body mass straight after `Sim::versus` and got "rival 0 kg". One `control_tick` first fixed it. To check masses before stepping, use the model's (`RobotModel::total_mass`) instead of the world's.
