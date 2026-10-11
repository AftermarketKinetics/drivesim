# test_opponents.py

## Function
Public robots as AI opponents, through the HTTP surface and the store:
publishing, who may fetch what, the seeded house robots, and the schema
migration.

## Interface
Tests:
- `test_the_house_robots_are_public_opponents_one_per_class`: simple_bot
  1 lb, 3 lb, 12 lb and 30 lb are listed in that order for alice and bob
  alike; the 12 lb one's bundle is fetchable by both, but its page,
  history and delete are 404 and it is in no one's robot list.
- `test_an_owner_publishes_and_unpublishes_their_robot`: bob sees and
  fetches the bundle only while it is public, never its page or
  history, and can't change its public state; unpublished, the class is
  remembered.
- `test_opponents_are_ordered_by_weight_class`.
- `test_publishing_needs_a_weight_class_and_a_drivable_robot`: 422 for
  no or a wrong class, 409 for an unchecked robot, and a public robot
  that later fails its check drops out of the list.
- `test_seeding_is_idempotent_and_refreshes_the_bundle`: one row per
  house robot after two seeds, stale files gone, no sidecar `.md` copied.
- `test_a_missing_house_root_seeds_nothing`.
- `test_an_older_database_gains_the_new_columns`: a pre-2026-10-10
  `robots` table gets `public` and `weight_lb`, and a second start
  doesn't try again.

## Implementation
`ready_robot` makes a drivable robot straight in the store with the
fixture bundle, skipping the import and conversation that `test_app.py`
already covers end to end.

## Assertions
- Every test that lists opponents accounts for the four seeded house
  robots (`HOUSE_IDS`, `HOUSE_ROWS`), in class order.
