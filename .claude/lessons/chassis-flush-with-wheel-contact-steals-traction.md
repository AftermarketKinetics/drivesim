# A chassis bottom flush with the tyre contact plane takes the robot's weight off its wheels

When a robot's chassis underside sits in the same plane as its wheels' contact patches, rapier's rigid contacts split the load between the plate and the wheels however a fraction of a degree of pitch decides, and they chatter between the two. The wheels then barely grip, so the robot is slow and turns badly.

- Found 2026-10-11 on Momentum (`out/momentum`, from `make drivesim`): floor plate, rails, wedge, forks and tail boom all at z = 0, wheel (cleat tip Ø43) bottoms at z = 0. While driving, the chassis carried ~46 of 52 N; each wheel had zero load on 33–60% of control ticks. Full throttle: wheels at no-load surface speed (5.15 m/s) within 0.1 s, chassis ~1.1 m/s.
- Turning came out non-monotonic: about 13 rad/s at half turn stick but 3.4 rad/s at full. More wheel speed meant more chatter and less wheel load.
- Growing the wheel radius by 3 mm (chassis 3 mm off the floor) fixed both: 5.9 m/s in 0.45 s, and turn rate rising steadily with the stick to ~35 rad/s.
- How to apply: when a robot "feels sluggish", first measure per-body floor load while driving (sum `contact_pairs_with(floor)` impulses / dt per parent body). Check the parts' min z against the wheel bottoms before tuning motors or mu.
