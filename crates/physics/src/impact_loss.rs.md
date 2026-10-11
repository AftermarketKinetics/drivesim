# impact_loss.rs

## Function

Makes impacts lossy (user, 2026-10-11: "more energy lost to
destruction/heat/sound … especially when hitting the walls or the
opponent's weapon"). A real hit spends much of its energy deforming
armour, walls and teeth. A rigid restitution-0 contact does not: it gives
the struck body all the velocity needed to stop the tooth's approach. So
after each physics step, any body group that took an impact keeps only
`KEEP` of the velocity change the step gave it.

## Interface

- `KEEP` 0.5: the fraction of an impact's velocity change kept (user,
  2026-10-11: "everything 50%, floor exempt").
- `IMPACT_ACCEL` 5 g: a step's contact impulse on a group counts as an
  impact when, divided by the group's mass, it is above this times `dt`.
- `Group { bodies, rotors }`: bodies that share one verdict (a robot:
  chassis first, then wheels and weapons; or the box). `rotors` are
  `(body, axis)`, with the axis a unit vector in the first body's frame.
- `Velocities`, plus `snapshot(world, groups)`: each group's bodies'
  `(linvel, angvel)`, taken before `world.step()`.
- `absorb(world, groups, before, floor)`: after the step, for each group
  whose contacts with anything but `floor` and itself total an impact,
  sets every body's velocity to `before + KEEP × (after − before)`.
  `Sim::physics_step` calls the pair whenever `Sim::floor` is set.

## Implementation

- **Why after the step:** rapier 0.36 can't cap or absorb a contact
  impulse. `modify_solver_contacts` exposes only friction, restitution,
  distance and tangent velocity, and restitution is already 0
  everywhere.
- **No re-collision feedback**, which was the worry: measured on the
  fixture (2026-10-11), the weapon lost less speed per hit with the loss
  than without (1048 → 642 rad/s rather than → 590). Scaling back the
  victim's velocity change does not make the tooth simply hit again and
  hand the energy back.
- **A rotor keeps all of its spin change about its axis.** The weapon
  still loses every bit of speed the hit took from it, so the energy
  really leaves the system. Everything else is scaled by the same factor
  across the robot's bodies. The joints stay consistent: the joint
  constraint is linear in the velocities, and spin about the joint axis
  is a free DOF.
- **One verdict per group**, from the total of all its contact pairs'
  impulse magnitudes. Scaling only the chassis would leave the wheels and
  weapon fighting their joints.
- **The floor is exempt** (the user's choice): landings and wheel grip
  stay exact. The roof and walls are not.
- **5 g threshold:** driving, braking and shoving matches stay under
  ~1 g (tyre mu ≤ 1.4), so they are never touched;
  `driving_into_the_opponent_shoves_it` still moves it 0.3 m+.
- **Measured effect** (fixture, 2026-10-11):
  - a spun-up hit throws the opponent at 3.7 m/s instead of 6.7, up
    0.08 m instead of 0.56;
  - weapon into the wall recoils at 0.5 m/s instead of 5.8, and the
    weapon walks the robot up the wall onto its end instead;
  - Obliterator 6 v72 against its AI copy runs faster with it on (3.6×
    vs 3.1× native), since violent contacts settle sooner.
- **Momentum isn't conserved in an impact.** That is the point: the
  missing momentum went into the wall or the armour.

## Assertions

- The floor collider never triggers the loss.
- Rotor spin change about the rotor's axis is never scaled.
- Every body of a group is scaled by the same `KEEP`.
- Tested in tests/impact.rs: a hit throws the opponent below 0.7× its
  lossless speed; the wall recoil is below 0.3× its lossless speed.
