# Runbook: Backend Deployment

Last updated: 2026-05-29

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

Keep `packages/app/public/asset-recovery-sw.js` and `packages/app/public/_worker.js` in the public build too. They prevent stale JavaScript chunk requests after a frontend deploy from staying on a `text/html` asset response; the app also auto-reloads once when it detects this stale-asset condition.

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

## Public Auth Runtime Env

Public SMS login and optional captcha are runtime env configuration, not `config.yaml` fields. Keep real values in the backend host environment or supervisor, never in committed files. The active admin-created Web invite users remain the public-login invite-list admission source before any SMS send/check.

Required for SMS send/check:

```text
ALIBABA_CLOUD_ACCESS_KEY_ID
ALIBABA_CLOUD_ACCESS_KEY_SECRET
```

The backend also accepts the compatibility aliases `ALIYUN_ACCESS_KEY_ID` / `ALIYUN_ACCESS_KEY_SECRET` and `HONE_ALIYUN_ACCESS_KEY_ID` / `HONE_ALIYUN_ACCESS_KEY_SECRET`. Prefer the `ALIBABA_CLOUD_*` names for new deployments.

Optional SMS overrides:

```text
HONE_ALIYUN_SMS_ENDPOINT=dypnsapi.aliyuncs.com
HONE_ALIYUN_SMS_COUNTRY_CODE=86
HONE_ALIYUN_SMS_SIGN_NAME=速通互联验证码
HONE_ALIYUN_SMS_TEMPLATE_CODE=100001
HONE_ALIYUN_SMS_TEMPLATE_PARAM={"code":"##code##","min":"5"}
```

Optional Aliyun Captcha 2.0 guard for public SMS sends:

```text
HONE_ALIYUN_CAPTCHA_PREFIX=<captcha-prefix>
HONE_ALIYUN_CAPTCHA_SCENE_ID=<scene-id>
HONE_ALIYUN_CAPTCHA_REGION=cn
HONE_ALIYUN_CAPTCHA_ENDPOINT=<optional-endpoint-override>
HONE_ALIYUN_CAPTCHA_ENABLED=false
```

When `HONE_ALIYUN_CAPTCHA_PREFIX` and `HONE_ALIYUN_CAPTCHA_SCENE_ID` are both set, public SMS sends must pass server-side Aliyun captcha verification before the SMS provider is called. Captcha verification uses the same Aliyun AccessKey env variables as SMS.

Optional cookie override:

```text
HONE_PUBLIC_SECURE_COOKIE=true
```

Use `HONE_PUBLIC_SECURE_COOKIE=true`, `1`, or `yes` when the backend origin cannot reliably infer HTTPS from proxy headers. Use `false`, `0`, or `no` only for local HTTP diagnostics. Invalid non-empty values intentionally keep `Secure=true`.

## Cloud Storage Runtime Env

Managed PG / OSS settings are runtime env configuration. Keep real values in the backend host environment, local ignored `.env`, or process supervisor, never in committed config or docs. `config.example.yaml` documents the env var names under `cloud.*` with empty credential fields.

Storage authority mode:

```text
HONE_CLOUD_MODE=local|cloud|auto
HONE_RUNTIME_ROLE=web|worker|all
```

Use `HONE_CLOUD_MODE=local` for local fallback. Use `cloud` only when PG and OSS are both configured and intended to be authoritative. Use `auto` only for development compatibility with older env-presence behavior.

Postgres migration target:

```text
HONE_CLOUD_MODE=cloud
DATABASE_URL=<postgres-url>
HONE_POSTGRES_PROXY=socks5://127.0.0.1:1082
```

Compatibility pieces accepted when `DATABASE_URL` is not set:

```text
HONE_POSTGRES_HOST=<host>
HONE_POSTGRES_PORT=5432
HONE_POSTGRES_USER=<user>
HONE_POSTGRES_PASSWORD=<password>
HONE_POSTGRES_DATABASE=<database>
```

Object storage for public uploads and durable cloud files:

```text
HONE_OSS_PROVIDER=aliyun_oss|r2|s3
HONE_OSS_ACCESS_KEY_ID=<access-key-id>
HONE_OSS_ACCESS_KEY_SECRET=<access-key-secret>
HONE_OSS_BUCKET=<bucket>
HONE_OSS_ENDPOINT=https://oss-cn-beijing.aliyuncs.com
HONE_OSS_REGION=oss-cn-beijing
HONE_OSS_PROXY=socks5://127.0.0.1:1082
```

For Cloudflare R2, use the S3-compatible endpoint and region:

```text
HONE_OSS_PROVIDER=r2
HONE_OSS_ENDPOINT=https://<account-id>.r2.cloudflarestorage.com
HONE_OSS_REGION=auto
```

To compare Aliyun OSS and R2 without losing rollback settings, keep runtime `HONE_OSS_*` pointed at the active provider and store the alternate Aliyun settings under:

```text
HONE_ALIYUN_OSS_PROVIDER=aliyun_oss
HONE_ALIYUN_OSS_ACCESS_KEY_ID=<access-key-id>
HONE_ALIYUN_OSS_ACCESS_KEY_SECRET=<access-key-secret>
HONE_ALIYUN_OSS_BUCKET=<bucket>
HONE_ALIYUN_OSS_ENDPOINT=https://oss-cn-beijing.aliyuncs.com
HONE_ALIYUN_OSS_REGION=oss-cn-beijing
HONE_ALIYUN_OSS_PROXY=socks5://127.0.0.1:1082
```

And R2 comparison settings under:

```text
HONE_R2_PROVIDER=r2
HONE_R2_ACCESS_KEY_ID=<access-key-id>
HONE_R2_ACCESS_KEY_SECRET=<access-key-secret>
HONE_R2_BUCKET=<bucket>
HONE_R2_ENDPOINT=https://<account-id>.r2.cloudflarestorage.com
HONE_R2_REGION=auto
HONE_R2_PROXY=socks5://127.0.0.1:1082
```

When OSS is configured, `/api/public/upload` writes objects under `public-uploads/<user>/<date>/...` and returns `oss://bucket/key`. Actor durable files use `users/{actor_storage_key}/...` namespaces. `/api/public/image` and `/api/public/file` can proxy managed OSS paths back through the backend.

Runtime checks:

```bash
hone-cli cloud doctor --ensure-schema --json
hone-cli cloud object-bench --size-kib 256 --iterations 3 --json
hone-cli cloud migrate --from-data-dir ./data --json
hone-cli cloud migrate --from-data-dir ./data --quota-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --upload-oss --apply --concurrency 12 --json
hone-cli cloud migrate --from-data-dir ./data --upload-oss --apply --reuse-existing --concurrency 4 --json
```

The migrator uploads recognized durable files and indexes them in PG `cloud_documents`. It also imports legacy `conversation_quota/*.json` into PG; use `--quota-only --apply` for a fast idempotent quota pass before the larger object migration. Use the lower-concurrency `--reuse-existing` retry when proxy or OSS connections drop during a large upload. SQLite files are currently counted but skipped because they need structured row-wise import into PG. Session, auth, audit, portfolio, cron, notification preference, KB, and company-profile hot-path repositories are still local until their dedicated PG-backed adapters are completed; quota is PG-backed in `cloud.mode=cloud`.

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
