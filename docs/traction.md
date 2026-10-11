# Traction: cleats and treads on the arena floor

Status: **plan, 2026-10-11; nothing built.** The three architecture
choices in §2 are the user's. Everything else is a *proposal* and open
to review. This extends `drivesim.md` §6 (physics) and answers its §10
open question "Wheel friction feel".

## 1. Why

Today every wheel is a rapier cylinder at its outermost radius with one
Coulomb friction coefficient, `mu` from robot.toml, multiplied by the
floor's 1.0. That coefficient cannot depend on slip speed, on the load
the wheel carries, or on what the wheel is made of. Momentum's wheel is
not a tyre, though. Since 2026-10-11 it is a urethane tread (Ø46) between
two 4130 rings of six raked spikes whose tips stand 1.5 mm proud of it
(Ø49). Today's `mu = 1.8` is a guess standing in for "the spikes dig in".

Whether the spikes dig in at all is a question the physics can answer.
Momentum's spikes are true points that the laser burns back to roughly a
0.8 × 1.6 mm flat. Painted birch plywood resists indentation at about
26 MPa (Brinell, provisional). At 26 N on one tip, that flat bears
20 MPa: below the floor's hardness, so the tip barely sinks, and a tip
that does not sink cannot plough. On a softer floor (pine, ~16 MPa) the
same tip sinks ~0.3 mm. So the grip the cleats give is very sensitive
to floor hardness and tip geometry. That sensitivity is what a design
choice like "six spikes, not twelve" needs from the sim, and no single
`mu` can show it.

The investigation that led here (2026-10-11, lesson
`chassis-flush-with-wheel-contact-steals-traction.md`) found a different
first-order problem: Momentum's chassis sat in the wheels' contact plane.
Momentum fixed that the same day (commit `1e92e34`: 3 mm ground
clearance, wheel Ø43 → Ø49). `out/momentum` predates that commit; rebuild
it with `make drivesim` before testing anything here.

## 2. Decisions (user, 2026-10-11)

- **A contact-modification hook sets μ; rapier still solves friction.**
  Each physics step, every wheel–floor contact manifold gets its friction
  coefficient from this model. Stiction, stability and the coupling with
  joints stay rapier's implicit solve, which works on any wheel mass.
  Rejected: our own forward/sideways force model with rapier's wheel
  friction zeroed. It would give true anisotropy, but it puts an explicit
  force on light wheels at 2 kHz. That is the same class of divergence as
  the gear-coupling term (`explicit-gear-coupling-diverges-on-light-robots.md`).
  It would also need either a large regularising slip speed (~0.5 m/s of
  creep on a 50 g low-ratio wheel) or effective masses through the joints.
- **Floor contacts only.** Wheels against opponents, walls and the box
  keep the plain Coulomb `mu`. The floor is the only surface whose
  material is modelled. Every other pair would need numbers that do not
  exist.
- **A full model, not a re-tuned `mu`:** the grip comes from the tooth
  geometry, the floor's material and the slip.

## 3. The model

Inputs, per wheel, each physics step:
- the wheel's normal load `N` on the floor, from the last solve (the sum
  of the manifold points' normal impulses ÷ dt);
- the slip velocity at the contact, the wheel body's velocity at the
  contact point (spin included) projected onto the floor;
- the wheel's spin angle relative to the chassis, which says which spikes
  are down.

Output: one μ for the manifold.

**Which spikes are down.** A ring of `n` spikes with tips at `R_tip` over
a tread of `R_tread` rolls on a spike only while that spike is lower
than the tread: within `φ_e = acos(R_tread / R_tip)` of straight down.
For Momentum that is acos(23/24.5) = 20°, so a spike is down for 40° of
every 60°. A third of the time the wheel stands on urethane alone. The
model resolves this from the wheel angle rather than averaging it. The
grip ripples at `n` × the wheel's rotation frequency (6 per turn), and
2 kHz resolves it at Momentum's top wheel speed (about 2,300 rpm, so a
spike passes 230 times a second). Both rings are counted. They are in
phase on Momentum (`spike_deg0` is shared); the phase is a parameter.
The collider stays a cylinder at `R_tip`: the polygonal roll of a
spiked wheel (a 1.5 mm bump per spike) is left out.

**How far a spike sinks.** Each engaged tip carries its share of the
load not on the tread. A tip is a flat `f` (along the circumference) by
the plate thickness `w`, widening with flank half-angle `θ`. At depth `δ`
it bears on `w · (f + 2δ tanθ)`, so

```text
δ = max(0, (N_tip / (H w) − f) / (2 tanθ)),   capped at the protrusion p
```

Once `δ` reaches `p` (1.5 mm on Momentum), the tread touches and takes
the rest of the load. Tread compliance is ignored: until then it carries
nothing, after that everything beyond the tips' full-depth load.

**Grip of a spike.** A sunk tip pushed along the floor ploughs. Its
frontal face is `w · δ` against the floor's resistance, and its bearing
face carries the vertical load. For a 2-D wedge pushed square to its
edge, that ratio is `cot θ` (Bowden–Tabor; `(2/π) cot θ` for a cone).
The frontal pressure is capped by the wood shearing out a chip ahead of
the tooth, `τ / sin ψ` with shear-plane angle `ψ`, whichever is lower
than `H`. Steel-on-paint adhesion `μ_a` acts on the whole bearing load,
so

```text
F_tip = μ_a · N_tip + min(H, τ / sin ψ) · w · δ
```

**Rake.** Momentum's spikes alternate 6° of rake either way, so whichever
way the wheel turns, half hook into the floor and half trail. A hooking
spike's leading flank is steeper (`θ − rake`), and a trailing one's
shallower (`θ + rake`). Each spike uses its own angle for the current
slip direction.

**Slip direction.** The hook gives one isotropic μ per manifold, but it
knows which way the contact is slipping. A spike ploughs hard along the
circumference and weakly sideways, where it presents only its 1.6 mm
plate edge. So the frontal term is weighted by the slip direction: full
along the wheel, the plate-edge value across it, interpolated on an
ellipse. That is anisotropy in magnitude only, recomputed each step from
the slip direction, not a true anisotropic cone. It is the most the hook
can do (§2).

**Slip speed.** While the spikes roll in and out without sliding, a tip
only has to break the wood it presses into. Once the wheel spins up past
`v_tear`, each tip tears a fresh groove. Grip moves from the engaged
value to a lower sliding one:

```text
μ_spike(v) = μ_slide + (μ_engaged − μ_slide) · exp(−|v| / v_tear),   μ_slide = k_slide · μ_engaged
```

The tread has its own static and sliding μ on the same curve.

**The manifold's μ.** The load-weighted mix of the two:

```text
μ = (Σ F_tip(v) + μ_tread(v) · N_tread) / N
```

with `N = Σ N_tip + N_tread`. Below about 0.5 N of load, the hook uses the
tread's μ: the spike share is noise there, and nothing depends on it.

**Feedback.** μ depends on slip, and slip depends on μ: a wheel spun past
`v_tear` loses grip and spins faster. That is real (wheel-spin is
self-sustaining), but per-step it can flicker. The slip speed the model
reads is low-pass filtered (τ ≈ 2 ms, proposal) before it enters the curve.

## 4. Robot file

A wheel keeps `mu` as the shorthand: a plain tyre with that static and
sliding μ, exactly today's behaviour. A wheel with a `[wheel.tyre]`
table gets the model; giving both is an error.

```toml
[[wheel]]
name = "wheel_left"
# ... as now, without mu
[wheel.tyre]
tread = "urethane"            # material key: static/sliding mu on the floor
tread_r_m = 0.023
[wheel.tyre.cleat]            # absent: tread only
rings = 2
spikes = 6                    # per ring
phase_deg = 0.0               # ring-to-ring offset
tip_r_m = 0.0245
tip_flat_m = 0.0008           # laser-burnt point
plate_t_m = 0.0016
flank_half_angle_deg = 23     # Momentum: 16° base span at r 18.5, 6 mm tall
rake_deg = 6
rake = "alternating"          # or "forward", "none"
```

The wheel's collider radius stays the outermost radius (`tip_r_m` when
cleated), found by the importer as now. The loader checks
`tread_r_m < tip_r_m` and that the importer's radius agrees with
`tip_r_m` within 0.5 mm. Momentum's `scripts/drivesim-bundle.py` emits
this table from `_wheel_stack.py`, so the datums cannot drift.

Floor and tread materials (`H`, `τ`, `ψ`, `μ_a`, tread μ, `v_tear`,
`k_slide`) are constants in the physics crate, not in robot files. They
describe the arena, which is the same for every robot.

## 5. Constants (all PROVISIONAL)

| Symbol | Proposal | What it is | Where it should come from |
|---|---|---|---|
| `H` | 26 MPa | plywood face indentation hardness | Brinell, birch; Wood Handbook (FPL-GTR-282) |
| `τ` | 10 MPa | wood shear strength | shear parallel to grain, same source |
| `ψ` | 30° | chip shear-plane angle | wood-cutting literature |
| `μ_a` | 0.3 | steel on dead-flat paint | unmeasured |
| tread μ static / sliding | 1.0 / 0.8 | cast urethane on paint | unmeasured |
| `v_tear` | 0.2 m/s | engaged → sliding | unmeasured |
| `k_slide` | 0.7 | sliding / engaged spike grip | unmeasured |

The numbers that matter most are `H` and `f`. §1's arithmetic shows the
answer flipping between "barely sinks" and "sinks 0.3 mm" across plausible
values. A cheap measurement would settle `H` and the spikes' real grip:
a cleated wheel loaded with a known weight on a painted plywood offcut,
pulled with a luggage scale. Until then the HUD should say these are
unmeasured.

## 6. Code layout

One purpose per file; each gets its sidecar in the same change.

- `crates/robot/src/tyre.rs`: the `Tyre` and `Cleat` types, parsed
  from `[wheel.tyre]`. `mu` becomes `Tyre::Plain`. Loader checks live
  here. `Rotor.mu` is replaced by `Rotor.tyre`, and every user of `mu`
  (`spawn.rs`, `check` report, server tools) follows.
- `crates/physics/src/floor_material.rs`: the §5 constants, with sources,
  and the tread material table.
- `crates/physics/src/cleat_contact.rs`: pure. Which spikes are down for a
  wheel angle; how far each sinks for a load; how the load splits between
  spikes and tread.
- `crates/physics/src/traction.rs`: pure. μ for (slip velocity in the
  wheel frame, load, wheel angle, `Tyre`, floor). It calls
  `cleat_contact`. All of §3's formulas live in these two files and
  nowhere else.
- `crates/physics/src/traction_hook.rs`: the `PhysicsHooks`
  implementation. Stateless: it looks up a μ that `Traction` prepared,
  keyed by the wheel collider's `user_data`, and only for manifolds
  against the floor collider.
- `crates/physics/src/tyre_state.rs`: `Traction`, the per-robot state,
  like `Drivetrain`. Before the step it reads last step's floor load per
  wheel, filters the slip speed and computes μ per wheel. `Sim` keeps one
  per robot.
- `spawn.rs`: wheel colliders get `ActiveHooks::MODIFY_SOLVER_CONTACTS`
  and their `user_data`. `sim.rs` and `rest.rs` step through one helper
  that calls `step_with_events(&hook, &())` in place of `world.step()`,
  so a settled robot and a driven one see the same grip.
- `crates/physics/tests/traction.rs`: §7.
- `server/drivesim_server/prompt.py`: the role conversation learns the
  tyre table. It asks about cleats only when the STEP shows spiked parts
  on a wheel; otherwise it keeps writing `mu`.
- `docs/drivesim.md`: §6 points here; §10's "Wheel friction feel" and
  "Floor grip" entries are updated to say what this does and does not
  settle.

Keeping the μ preparation outside the hook keeps the hook `Sync` without
locks (rapier's `PhysicsHooks: MaybeSync`), and keeps every formula
testable without a world.

## 7. Tests

Pure (sidecar `test` blocks, `cleat_contact` and `traction`):
- a spike within `φ_e` of straight down is engaged, one just outside is
  not; a tread-only tyre never has engaged spikes;
- the sink depth is 0 below `H·w·f` per tip, grows linearly with load
  after that, and stops at the protrusion, with the rest of the load on
  the tread;
- the frontal term is capped by `τ / sin ψ` when that is below `H`;
- a hooking spike grips more than a trailing one at the same load;
- slip along the wheel grips more than slip across it, for a cleat; a
  plain tyre is the same in every direction;
- μ falls monotonically from engaged to sliding as slip speed grows;
- `Tyre::Plain(mu)` returns `mu` for every input.

World (`tests/traction.rs`, the fixture plus a cleated variant of it):
- a plain-`mu` robot drives exactly as before: the existing `drive.rs`
  tests pass unchanged;
- straight-line acceleration is grip-limited: within 10% of μ·g for the
  μ the model reports at that slip;
- a parked robot with the motors braking does not creep when pushed
  below its grip, and slides once pushed past it;
- a robot held up with its wheels spinning gets no floor force;
- `control_tick(None)` for 5 s gains no energy (the
  explicit-gear-coupling lesson's check);
- turn rate rises monotonically with the turn stick on a robot whose
  chassis clears the floor (the regression that started this);
- physics cost per step rises by under 5% (two wheel manifolds per robot
  per step).

## 8. Left out

- Lasting floor damage: grooves the next pass falls into, paint wear,
  impact marks for spikes. The floor is new every step.
- Tyre and spike compliance, and the 1.5 mm polygonal bump a spiked wheel
  rolls over. The collider stays a cylinder at the tip radius.
- True anisotropic friction. §3's slip-direction weighting is the hook's
  approximation.
- Cleats on anything but the floor (§2).
- Rolling resistance, heat and wear.

## 9. Build order

1. `tyre.rs` and its loader checks; `mu` keeps working; existing tests pass.
2. `floor_material.rs`, `cleat_contact.rs`, `traction.rs` with their pure
   tests. **Check in:** a table of μ against load and slip for Momentum's
   cleats at the §5 constants. That is where the constants get argued
   about.
3. `tyre_state.rs`, `traction_hook.rs`, the stepping helper, the world
   tests.
4. Momentum's `drivesim-bundle.py` emits the tyre table; drive the
   rebuilt bundle in the browser.
5. Server prompt, `drivesim.md` §6 and §10, deploy (ask first).
