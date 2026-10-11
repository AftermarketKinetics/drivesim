# Rapier can't make a contact lossy; blend the velocity change back after the step

Rapier 0.36 has no per-contact impulse cap or absorption: `modify_solver_contacts` exposes only friction, restitution, distance and tangent velocity, and restitution 0 (the default) is already "no bounce". A spinning weapon still hands the struck body the tooth's full speed.

- What works (2026-10-11, `impact_loss.rs`): snapshot velocities before `world.step()`, and after it set `v = before + k·(after − before)` for any body group whose non-floor contact impulse exceeds ~5 g × mass × dt.
- The worry that the tooth just hits again next step and hands the energy back did not happen. With k = 0.5 the weapon lost *less* speed per hit, and the opponent's speed halved.
- Leave each rotor's spin change about its own axis unscaled, and scale every body of a jointed robot by the same k. Otherwise the joints fight the blend.
- A side effect worth knowing: with little recoil, a vertical spinner driven into a wall walks itself up the wall onto its end instead of bouncing off.
