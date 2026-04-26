# Runbook: Backend Deployment

Last updated: 2026-04-26

## When to Use

- Updating the public web frontend served from Cloudflare Pages
- Updating the backend origin service used by the public API
- Verifying the Cloudflare Worker route between the public site and backend origin
- Moving the backend origin to a different managed host

## Public Topology

The public entrypoint is split into two layers:

- `hone-claw.com`: Cloudflare Pages serves the static public web bundle
- `hone-claw.com/api/public/*`: Cloudflare Worker proxies public API requests to the backend origin
- `origin.hone-claw.com`: backend origin hostname used by Cloudflare, not a user-facing entrypoint

Do not document private host location, workstation names, tunnel provider internals, credentials, or concrete process owner details in public files. Use “backend origin” or “managed backend host” in public-facing documentation.

## Frontend Update

Cloudflare Pages is connected to the GitHub repository. The Pages build should use:

```bash
bun install --frozen-lockfile && bun run build:web:public
```

Build output directory:

```text
packages/app/dist-public
```

Normal update flow:

1. Merge or push the frontend change to the production branch.
2. Wait for Cloudflare Pages to finish the deployment.
3. Verify:

```bash
curl -fsS https://hone-claw.com/ >/dev/null
curl -fsS https://hone-claw.com/chat >/dev/null
curl -fsS https://hone-claw.com/roadmap >/dev/null
```

For SPA routes, keep `packages/app/public/_redirects` in the public build:

```text
/* /index.html 200
```

## Backend Origin Update

The backend origin runs the public API surface used by the Pages frontend:

- `/api/public/auth/*`
- `/api/public/history`
- `/api/public/chat`
- `/api/public/upload`
- `/api/public/image`
- `/api/public/file`
- `/api/public/events`
- `/api/public/digest-context`
- `/api/public/company-profile`

Before updating the backend origin:

1. Confirm the current production branch and release target.
2. Ensure the backend config has no real secrets committed to the repository.
3. Build the frontend public bundle if the backend will serve local public assets as a fallback:

```bash
bun install --frozen-lockfile
bun run build:web:public
```

4. Restart the backend service using the host-specific process supervisor.
5. Verify the origin health through the origin hostname:

```bash
curl -i https://origin.hone-claw.com/api/public/auth/me
```

Expected unauthenticated result is `401` with an application JSON error. A Cloudflare error page, HTML SPA response, or connection failure means the origin path is not healthy.

## Worker Route

The Cloudflare Worker must route:

```text
hone-claw.com/api/public/* -> origin.hone-claw.com/api/public/*
```

Recommended fallback behavior:

- Return upstream responses unchanged when the backend origin is healthy.
- Return `503` JSON for API origin failures.
- Do not cache public API responses.

Post-change verification:

```bash
curl -i https://hone-claw.com/api/public/auth/me
```

Expected unauthenticated result is `401` with an application JSON error. When the backend origin is intentionally unavailable, the expected result is a Worker-owned `503` JSON maintenance response rather than a Cloudflare branded error page.

## Cookie And SSE Checks

Public login uses an HttpOnly cookie scoped to `/` on `hone-claw.com`. Keep public API traffic same-origin from the browser perspective:

```text
browser -> https://hone-claw.com/api/public/*
Worker  -> https://origin.hone-claw.com/api/public/*
```

Do not point browser code directly at `origin.hone-claw.com` unless CORS, cookie domain, SameSite, and SSE behavior are deliberately redesigned.

Verify after auth-related changes:

```bash
curl -i https://hone-claw.com/api/public/auth/me
curl -i https://hone-claw.com/api/public/events
```

`/api/public/events` requires an authenticated cookie in real use. For unauthenticated requests, `401` is expected.

## Security Notes

- Do not expose the admin web surface through the public domain.
- Keep admin APIs behind separate authentication and non-public routing.
- Do not commit API tokens, tunnel tokens, runtime databases, or exported production config.
- Prefer documenting stable host roles over physical or personal infrastructure details.
- If the backend origin moves, update the Worker origin hostname or DNS target first, then rerun the verification commands above.

## Rollback

Frontend rollback:

1. In Cloudflare Pages, promote the previous successful deployment.
2. Re-check `/`, `/chat`, and `/roadmap`.

Backend rollback:

1. Revert the backend origin to the previous known-good release or process configuration.
2. Restart through the host-specific supervisor.
3. Verify both direct origin and public Worker path:

```bash
curl -i https://origin.hone-claw.com/api/public/auth/me
curl -i https://hone-claw.com/api/public/auth/me
```
