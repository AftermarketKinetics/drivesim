# test_usage.py

## Function
The daily budget end to end: the weights, the Pacific day, recording each
turn's usage, refusing new work once it is spent, and keeping usage when
a robot is deleted.

## Interface
Tests:
- `test_turns_are_weighted_like_the_bill`
- `test_the_budget_day_runs_midnight_to_midnight_pacific`
- `test_usage_is_recorded_and_a_spent_budget_refuses_new_work`

## Implementation
The end-to-end test shrinks the budget to 10,000 with
`dataclasses.replace` and queues one scripted turn reporting 16,000
equivalents of usage. A real fixture upload's import notice runs that
turn. Then:
- a reply and a second upload both get 429, naming midnight Pacific;
- deleting the robot leaves the usage counted.

DST: the day's real length is measured with timestamps. Subtracting
aware datetimes in one zone compares wall clocks and always gives 24 h.

## Assertions
- No network: the model is the scripted one.
