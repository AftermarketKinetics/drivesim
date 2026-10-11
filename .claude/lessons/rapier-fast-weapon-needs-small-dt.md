# A full-RPM weapon on a rapier joint needs a sub-millisecond step

A 10,000 RPM weapon body on a rapier3d 0.36 revolute impulse joint throws the whole robot across the arena at a 1 ms step, whatever the solver substep count. At 2 kHz (30° of weapon rotation per step) or faster it is stable and holds 9,998 RPM.

- Found 2026-10-09 in the timing spike (Momentum-sized robot, 1 kg r100 mm disc). Unstable at 1 ms with the joint motor off and the gyroscopic term off too, so it is the joint's linearised constraint at 60°/step, not precession or the motor. Stable at 0 and 100 rad/s at 1 ms.
- Raising `num_solver_iterations` (substeps) does not fix it; shrinking `dt` does. Rapier's multibody (reduced-coordinate) joint was worse, not better.
- Cost in WASM (Node 24 / V8, one thread, whole scene): 2 kHz 11.4x real time, 4 kHz 6.6x, 8 kHz 4.0x. Native is ~3x faster.
- How to apply: size the physics step from the fastest rotor (aim ≤ ~15–30° per step) rather than from the control rate, and keep the control tick (1 kHz for Momentum's core) as a multiple of it.
- 2026-10-10: the sim moved from 4 kHz to 2 kHz for speed (user's choice). Every drive/weapon test held; the chaotic single-hit test's margin shrank (opponent 0.56 m vs robot 0.39 m, was 1.88 vs 0.32). 2 kHz is the floor: don't go lower without re-measuring a 10k RPM weapon.
