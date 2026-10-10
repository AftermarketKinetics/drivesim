// fetch() for the site's own API, aware of the login in front of it.
// Behind authentik's forward_auth, an expired session turns every API call
// into a redirect to the login host, which fetch cannot follow across sites:
// the page would just show "Failed to fetch" (the analyzer's lesson,
// 2026-10-08). So redirects are not followed, and one coming back reloads
// the page, which goes through the login and returns here.
(function () {
  async function apiFetch(url, options) {
    const resp = await fetch(url, Object.assign({}, options, { redirect: "manual" }));
    if (resp.type === "opaqueredirect" || resp.status === 401) {
      location.reload();
      throw new Error("signed out: reloading to sign in again");
    }
    return resp;
  }
  window.apiFetch = apiFetch;
})();
