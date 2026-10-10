# test_app.py

## Function
The HTTP surface end to end: upload, then a real local import, then the
scripted conversation, then the real check, then the bundle served.

## Interface
Tests:
- `test_upload_imports_then_the_conversation_writes_a_checked_robot`
- `test_robots_are_the_owners_only`
- `test_admins_read_and_drive_every_robot_but_cannot_change_it`
- `test_the_admin_dashboard_lists_every_robot_for_admins_only`
- `test_admins_come_from_a_comma_separated_env_var`
- `test_only_bundle_files_are_served`
- `test_a_broken_step_fails_the_robot_not_the_server`
- `test_oversized_uploads_are_refused`
- `test_no_header_and_no_dev_user_is_401`
- `test_the_radio_model_is_downloadable_from_the_front_page` (via the shared guide)
- `test_pages_are_revalidated_by_content_not_mtime`
- `test_the_robot_page_offers_delete_and_delete_removes_everything`
- `test_the_landing_page_shows_the_demo_video`
- `test_the_landing_page_uses_only_absolute_public_paths`

## Implementation
**Traversal cases are percent-encoded**: the HTTP client normalises a
literal `meshes/../parts.json` to `parts.json` before sending, which made a
first version of the test "fail" on an allowed file (2026-10-09).

The import notice must name the fixture's 14 parts.

The admin dashboard lists the seeded house robot (`house.py`) after
alice's, being older.

## Assertions
- Checks what the model was sent (the import notice, the tool results) as
  well as what the API returns.
