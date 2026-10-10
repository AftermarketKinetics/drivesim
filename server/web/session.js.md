# session.js

## Function
`fetch()` for the site's own API that copes with the login in front of it.
When the authentik session expires, the page reloads to sign in again,
instead of showing "Failed to fetch".

## Interface
- `window.apiFetch(url, options) -> Promise<Response>`, loaded by
  `index.html` and `robot.html` before their scripts.

## Implementation
Behind Caddy's `forward_auth`, an expired session turns API calls into a
redirect to the auth host, which fetch can't follow across sites (the
analyzer's lesson, 2026-10-08). With `redirect: "manual"`, such a reply is
an opaque redirect. That, or a 401, reloads the page, which goes through
the login and back. Locally there is no login, so it never fires.

## Assertions
- Every `/api` call on the pages goes through `apiFetch`.
