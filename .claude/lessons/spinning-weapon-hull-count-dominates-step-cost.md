A spinning weapon's collider count, not robot-vs-robot contact, dominates the physics step: each hull is CCD-swept at weapon speed every step.

Measured 2026-10-10 natively on Obliterator 6 v72 (weapon body = 11 hulls, CCD on), AI round:
- baseline 1.31×; robots ~2 m apart still cost as much as touching, so it isn't the fight.
- Box opponent, own weapon at full: 2.1× (6.6× with the weapon off); same with CCD off: 3.4×.
- Weapon cut to 4 / 2 / 1 hulls: 4.2× / 6.0× / 7.2× (box mode).
- AI round with each weapon merged into one convex hull: 1.8× (103 vertices); capped at 32 vertices: 2.8× (3.1× with CCD off too). Repeats agree within 3%.
- Earlier "3.8× in AI mode" figures were with idle robots and the weapon off; benchmark with the weapon spinning.
The browser ran about half native speed here (0.7× vs 1.39×).

**Weapon CCD can't be dropped to save that time.** Without it (2026-10-10) the hit inverts: at 4 kHz the attacking fixture launched itself 1.06 m and the opponent rose 2 cm; at 2 kHz the spinning robot hopped 8 cm on open floor. The weapon sinks into what it hits before contact pushes back. Chosen instead: 2 kHz with CCD, Obliterator AI round 1.31× → 2.4× native.

**Applied (2026-10-10): `weapon_reach.rs`** drops weapon parts reaching under half the outer radius from collision (Obliterator 11 → 7 hulls): 2.4× → 2.86× native at 2 kHz.
