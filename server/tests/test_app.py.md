# test_app.py

## Function
The HTTP surface end to end: upload, then a real local import, then the
scripted conversation, then the real check, then the bundle served.

## Interface
Tests:
- `test_upload_imports_then_the_conversation_writes_a_checked_robot`
- `test_robots_are_the_owners_only`
- `test_only_bundle_files_are_served`
- `test_a_broken_step_fails_the_robot_not_the_server`
- `test_oversized_uploads_are_refused`
- `test_no_header_and_no_dev_user_is_401`

## Implementation
**Traversal cases are percent-encoded**: the HTTP client normalises a
literal `meshes/../parts.json` to `parts.json` before sending, which made a
first version of the test "fail" on an allowed file (2026-10-09).

The import notice must name the fixture's 14 parts.

## Assertions
- Checks what the model was sent (the import notice, the tool results) as
  well as what the API returns.
