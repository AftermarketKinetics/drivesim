# mass.rs

## Function

Rigid-body mass properties, and combining parts into one body.

## Interface

`MassProps { mass, com, inertia }` (kg, m, kg m² about `com`);
`MassProps::NONE`; `combine(&[MassProps]) -> MassProps`.

## Implementation

Mass-weighted CoM; each part's own inertia plus the parallel-axis term to
the combined CoM. An empty or massless set gives `NONE` rather than
dividing by zero.

## Assertions

- Tested: a dumbbell's inertia, off-diagonal terms for a skewed layout,
  the empty case.
