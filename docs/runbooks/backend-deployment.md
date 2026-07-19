# Runbook: Backend Deployment

Last updated: 2026-07-19

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

1. Run the public build locally. A generic `bun run build:web` only updates `packages/app/dist` and does not validate the public deployment artifact.
2. Confirm `packages/app/dist-public/index.html` has a new modification time and references the expected hashed entry asset.
3. Merge or push the frontend change to the production branch.
4. Wait for Cloudflare Pages to finish the deployment.
5. Verify:

```bash
curl -fsS https://hone-claw.com/ >/dev/null
curl -fsS https://hone-claw.com/chat >/dev/null
curl -fsS https://hone-claw.com/roadmap >/dev/null
```

The deployment is not complete merely because the source tree changed, the backend restarted, or a Vite development server shows the new behavior. Before reporting a public Web fix as live, compare all three served layers:

```bash
rg -o '/assets/[^" ]+\.js' packages/app/dist-public/index.html
curl -fsS http://127.0.0.1:8088/chat | rg -o '/assets/[^" ]+\.js'
curl -fsS 'https://hone-claw.com/chat?asset-check=1' | rg -o '/assets/[^" ]+\.js'
```

The local public build and port `8088` must reference the newly built entry. The Cloudflare Pages entry must change from the pre-deploy hash. For a protocol-sensitive frontend/backend change, also inspect the deployed lazy chunk for the new protocol markers; a `200` status alone is insufficient. For example, active-chat recovery requires `active_run`, `started_at_ms`, `run_progress`, and `interrupted_run`, and must not retain the old `in_flight + Date.now()` recovery branch. Record the final production entry/chunk hashes in the task handoff.

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
- `/api/public/community*`

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

The CLI loads an ignored `.env` relative to its startup working directory. A
supervisor must therefore start `hone-cli` with the repository root as its
working directory, or explicitly export the complete reviewed runtime
environment before launch. Starting an immutable binary from a temporary build
worktree without setting the working directory can silently omit cloud
credentials and fall back to local authority. The child processes may use an
immutable runtime-root directory; the CLI supervisor working directory is the
important load boundary.

When the intended production authority is cloud, restart is not complete until
the live `/api/meta` response confirms all of the following:

```text
cloud_mode=cloud
cloud_storage_authoritative=true
cloud_postgres_health.ok=true
cloud_oss_health.ok=true
local_durable_dependency_count=0
```

Also compare the supervisor's actual working directory with the intended
repository root and fail the deployment if they differ. Do not infer authority
from a separate `cloud doctor` command launched in a different working
directory; that command may have loaded a different `.env` from the live
process.

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
hone-cli cloud migrate --from-data-dir ./data --session-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --web-auth-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --quota-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --skill-registry-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --notification-prefs-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --portfolio-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --llm-audit-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --company-profiles-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --upload-oss --apply --concurrency 12 --json
hone-cli cloud migrate --from-data-dir ./data --upload-oss --apply --reuse-existing --concurrency 4 --json
```

### Community archive reconciliation and asset backfill

Community repair is intentionally split into two explicit, dry-run-first operations:

```bash
hone-cli cloud community-contents --manifest /path/to/complete-topic-manifest.json
hone-cli cloud community-contents --manifest /path/to/complete-topic-manifest.json --apply

hone-cli cloud community-assets --manifest /path/to/verified-assets.json
hone-cli cloud community-assets --manifest /path/to/verified-assets.json --apply
```

`community-contents` is a bootstrap/recovery command that requires the complete source timeline: `source_topic_index` and source file positions must each be contiguous from zero. It first reconciles existing file-backed rows by source file position, then uses `candidate_fingerprint + occurrence` as the stable identity for missing or non-file posts. Apply mode locks the community space and inserts every missing post and its ordered resources in one PostgreSQL transaction. A second dry-run must report `would_insert=0` before the migration is considered complete. **Do not use this command for the weekly append:** inserting newer topics at the front shifts source file positions and can match new rows to old content. Until a dedicated idempotent `community-append` workflow is implemented and reviewed, stop rather than improvising an incremental `community-contents --apply`.

`community-assets` accepts only ordinary non-symlink files with an allowlisted MIME/magic signature, exact manifest byte size, and exact SHA-256. It verifies or creates a full-SHA immutable object key, reads the object back from R2, and only then promotes the PostgreSQL resource row through an optimistic lock. Never put source cookies, signed download URLs, or authorization headers in either manifest. Source-protected resources stay metadata-only unless the authorized source UI legitimately exposes their bytes.

Every promoted resource keeps the previous SHA, size, object URI, and access state under `raw_metadata.community_asset_backfill`. Immutable R2 objects are retained for rollback. If a backfill must be reverted, restore the previous row values from that audit metadata or a PG snapshot; do not delete the old or new object until the restored application path has been verified.

The migrator uploads recognized durable files and indexes them in PG `cloud_documents`. It also imports legacy `sessions/*.json` into PG `cloud_sessions`, web invite users / auth sessions from the configured SQLite DB into PG, `conversation_quota/*.json` into PG, `runtime/skill_registry.json` into PG, `notif_prefs/*.json` into PG, `portfolio/*.json` into PG, `llm_audit.sqlite3` rows into PG, and actor-scoped `company_profiles/**/*.md` into PG `cloud_company_profile_files`; use `--session-only --apply`, `--web-auth-only --apply`, `--quota-only --apply`, `--skill-registry-only --apply`, `--notification-prefs-only --apply`, `--portfolio-only --apply`, `--llm-audit-only --apply`, or `--company-profiles-only --apply` for fast idempotent passes before the larger object migration. Use the lower-concurrency `--reuse-existing` retry when proxy or OSS connections drop during a large upload. Historical SQLite files outside sessions / web auth / LLM audit are still counted by the broad migrator but are not current runtime hot-path dependencies. Sessions, web auth, quota, cron, skill registry, notification prefs, portfolio, LLM audit, and company profiles are PG-backed in `cloud.mode=cloud`; generated images, uploads, and attachment/document surfaces are OSS-backed where the runtime has actor/file context.

## Public Community Private-R2 Edge Rollout

This is an operator-run rollout. At implementation close on 2026-07-19, the work had published and idempotently rechecked the initial **private** R2 derived snapshot (`662` contents, `833` resources, `719` edge descriptors, `34` feed pages, `754` publication objects; final dry-run `no_op=true`, `would_write=0`, `conflicts=[]`) without deploying a Worker or Pages bundle, changing a production variable or secret, switching traffic, or restarting the backend.

Later on 2026-07-19, the brand-new `hone-public-community-edge` Worker completed disabled provisioning as version `e01c1603-7c34-476a-b63b-33ac74244108`. It has only the exact `hone-claw.com/_community/v1/*` route, binds `COMMUNITY_BUCKET` to the existing private `honeclaw` bucket, and keeps `workers_dev=false` and `preview_urls=false`. The first deployment had no remote predecessor, omitted `EDGE_DISABLED`, installed no secret, and returned Worker-owned `503 {"error":"community_edge_unavailable"}`; the two legacy anonymous community probes remained `401`.

Implementation commits `385e35b0` and `100f5608` are now on `main`; follow-up `cb796cce` changes docs only. Their automatic Pages deployments completed, but the production entry and community chunks contain no `_community`, `edge-session`, or `community_edge` marker, so discovery remains compiled out. An exact `100f5608` immutable backend build and hash manifest are staged under `target/deploy-100f5608`, while the running backend remains the prior build with healthy cloud authority and zero active chats. `POST /api/public/community/edge-session` still returns `404`, proving no backend restart or traffic cutover occurred. The external supervisor must perform the Step 1 restart and pass the `mode=off` probe before Step 4 or any activation work. Keep each remaining gate closed until the preceding verification passes. Backend restarts below belong to the external process supervisor; do not restart it from an ad-hoc shell.

```text
authenticated browser
  -> POST /api/public/community/edge-session (short-lived HttpOnly grant)
  -> /_community/v1/* Cloudflare Worker
  -> auth before Cache API
  -> private R2 binding
  -> fixed legacy origin only on an eligible feed/resource GET miss/error
```

Resource HEAD failures are intentionally returned as non-2xx by the Worker so the existing client can choose its legacy URL; the Worker does not perform a second internal resource download for HEAD. A missing/invalid active resource index, an inactive resource version, or an invalid descriptor fails closed and never reaches the legacy origin.

PostgreSQL remains the canonical archive. R2 feed pages and descriptors are derived snapshots. Redis is not required and should not sit in front of image/PDF/attachment bodies: it would add another service while leaving durable binary delivery and origin bandwidth unsolved. Consider Redis later only if measurements show independent metadata or personalized-state contention.

### Step 1: preserve the current user path

1. Keep backend config at the safe default:

   ```yaml
   cloud:
     community_delivery:
       mode: "off"
       token_ttl_secs: 900
       secret_env: "HONE_COMMUNITY_EDGE_HMAC_SECRET"
   ```

2. Keep the Cloudflare Pages production build variable absent or set to `HONE_APP_COMMUNITY_EDGE_DISCOVERY=0`.
3. Confirm the legacy surface still answers. Anonymous `401` is the expected auth boundary:

   ```bash
   curl -i https://hone-claw.com/api/public/community
   curl -i https://hone-claw.com/api/public/community/resources/1
   ```

4. Deploy the reviewed backend build with `mode=off` through the normal supervisor workflow, then let the external service perform its controlled restart. Re-run `/api/meta` cloud-authority checks from the earlier backend section. Do not proceed if the legacy community page regresses.

With `mode=off`, this endpoint is safe to probe without a login and must return `200` JSON containing `enabled=false`, `mode="off"`, and no token or user identifier:

```bash
curl -i -X POST https://hone-claw.com/api/public/community/edge-session
```

### Step 2: bind the existing private R2 bucket

Use the same R2 bucket already selected by the backend's active `HONE_OSS_*` settings. Do not create a public duplicate bucket and do not give the browser a bucket URL.

1. In Cloudflare R2, verify that the `r2.dev` development URL is disabled, no custom domain exposes the bucket, existing `community/zsxq/51115212285814/resources/` objects are private, and the backend publisher's S3-compatible credentials can read/write the bucket.
2. Confirm `workers/public-community-edge/wrangler.jsonc` still binds `COMMUNITY_BUCKET` to `bucket_name = honeclaw`, which is the active 2026-07-19 `HONE_OSS_BUCKET`. If the backend bucket changes later, stop and update this reviewed binding before deploying; do not silently create or bind a duplicate bucket.
3. Keep these fixed boundaries unchanged unless a new delivery version is deliberately designed:

   ```text
   route: hone-claw.com/_community/v1/*
   feed prefix: community/zsxq/51115212285814/delivery/v1
   descriptor prefix: community/zsxq/51115212285814/delivery/v1/resources
   asset prefix: community/zsxq/51115212285814/resources
   legacy origin: https://origin.hone-claw.com
   workers_dev: false
   preview_urls: false
   ```

4. Confirm `hone-claw.com` is an orange-cloud/proxied hostname in the Cloudflare zone.
5. Confirm the fixed origin is independent of the Worker route and healthy before enabling fallback:

   ```bash
   curl -i https://origin.hone-claw.com/api/public/auth/me
   ```

   Require valid DNS and TLS plus a backend JSON `401`. A redirect/loop through `hone-claw.com`, Pages HTML, Cloudflare-branded error, or certificate failure is a stop condition.

The Worker uses an R2 binding; it does not need the backend's R2 access key or secret. Keep those S3-compatible credentials only on the backend/publisher host.

### Step 3: verify and deploy the Worker while disabled

From the repository root:

```bash
cd workers/public-community-edge
bun install --frozen-lockfile
bun run typecheck
bun run test
bun run deploy:dry-run
```

Stop on any failure. For a brand-new Worker, absence is safe only after confirming that no remote `EDGE_DISABLED` value exists. For any existing, restored, or previously deployed Worker, first set `EDGE_DISABLED=true` in the Cloudflare dashboard and deploy that variable change; `keep_vars=true` can otherwise preserve a remote `false` even though the variable is absent from this file. Only then deploy the exact route:

```bash
bunx wrangler deploy
```

The negatively named switch is fail-closed at runtime: missing, empty, unknown, or true values disable the Worker; production activation later should use exactly `EDGE_DISABLED=false`. Operationally, never infer the remote value from local Wrangler config—verify it in Cloudflare before every deploy.

```bash
curl -i https://hone-claw.com/_community/v1/feed/latest.json
```

Expected disabled result: Worker-owned `503` JSON with `community_edge_unavailable`. A `200`, Pages HTML, R2 body, or Cloudflare branded error means the route/switch is not in the reviewed state.

### Step 4: install one shared signing secret without opening traffic

Generate one high-entropy value in the approved secret manager. After trimming surrounding whitespace it must be **32..1024 UTF-8 bytes**. Do not paste it into chat, a shell transcript, `config.yaml`, Wrangler config, Pages variables, R2, logs, or a commit. Store the exact same value in the backend process environment under `HONE_COMMUNITY_EDGE_HMAC_SECRET` (or the exact env selected by `secret_env`) and in the Worker secret `COMMUNITY_EDGE_HMAC_SECRET`. An invalid backend value returns `enabled=false` and clears the scoped cookie; an invalid Worker value returns fail-closed `503`.

From `workers/public-community-edge`, the interactive Worker command is:

```bash
bunx wrangler secret put COMMUNITY_EDGE_HMAC_SECRET
```

**Cloudflare deployment warning:** `wrangler secret put` creates a new Worker version and immediately deploys it. Before running it, re-check that `EDGE_DISABLED` is still absent or true. If an immediate deployment is not acceptable, use Cloudflare's versions workflow (`wrangler versions secret put ...`, followed later by an explicit version deployment). Secret rotation after activation needs the same caution.

Repeat the anonymous disabled-route probe and require the same `503`. Do not set `EDGE_DISABLED=false` yet.

### Step 5: publish and verify the private R2 snapshot

Initial 2026-07-19 status: completed for the current `662`-content archive. Do not repeat this step before the first activation unless PostgreSQL community data or eligible resource metadata changes. After any later archive change, repeat the dry-run/apply/final-dry-run sequence exactly as written below.

Confirm that new community rows were inserted through the separately reviewed append/import workflow and promote only legitimately captured resources through `community-assets`. `community-contents` is bootstrap-only and must not be used as the weekly incremental entry point. The edge publisher reads PostgreSQL; it does not scrape the source and must never receive source cookies or signed source URLs.

From the backend host's reviewed repository working directory, with cloud-authoritative PG and the active provider specifically set to R2:

```bash
hone-cli cloud doctor --ensure-schema --json
hone-cli cloud community-publish
```

The dry-run must report `ok=true`, `resource_verification="head_exists_only"`, a nonzero `content_count`, `conflicts=[]`, and a plausible split between `edge_resource_count` and `legacy_resource_count`. A legacy resource remains on the compatibility path; a conflict stops the rollout. Do not use `--apply` to work around a conflict. Dry-run promises no PostgreSQL or R2 **business-data writes**; normal config loading may still create local runtime directories or tighten local file permissions.

Dry-run performs exact-key validation and an R2 HEAD/existence check only; it does **not** claim to verify object metadata or bytes. Apply uses bounded concurrency of two to GET every edge-eligible archived resource and verify byte size, SHA-256, and normalized content type against PostgreSQL **before any publication object is written**. The required key is exactly `{resource_id}-{full_sha256}.<safe ext>` directly under `asset-prefix`. The historical apply therefore reads several GiB and may take time. A mismatch/read failure is a blocking conflict, not a reason to bypass verification or raise concurrency casually. Resources outside `1B..=128MiB` remain legacy and receive no `delivery_path`; feed pages are capped at 8MiB, descriptors at 64KiB, the active index at 1MiB, and `display_name` at 1024 UTF-8 bytes.

```bash
hone-cli cloud community-publish --apply
hone-cli cloud community-publish
```

The final dry-run must report `ok=true`, `no_op=true`, `would_write=0`, and `conflicts=[]`. Apply loads all pages/resources through one `REPEATABLE READ READ ONLY` snapshot on the dedicated PostgreSQL advisory-lock session, verifies that session again before the first R2 write and before each mutable write, and treats explicit unlock failure as a failed command. Publication order is immutable descriptors, mutable `resources/active.json`, immutable cursor pages, then mutable `feed/latest.json`; both mutable objects are read back after writing and `latest.json` remains last. `active.json` is the authoritative resource-id/version allowlist checked by every resource request before shared byte cache, so an omitted old version is immediately inactive even if immutable bytes remain cached. Never delete prior immutable R2 objects during retry or rollback. Keep the Worker disabled while inspecting that feed JSON, descriptors, and `active.json` contain no secret, source authorization material, actor identity, or public session token.

### Step 6: issue grants in backend shadow mode

Change only the backend config to:

```yaml
cloud:
  community_delivery:
    mode: "shadow"
    token_ttl_secs: 900
    secret_env: "HONE_COMMUNITY_EDGE_HMAC_SECRET"
```

Keep `HONE_APP_COMMUNITY_EDGE_DISCOVERY=0` and keep the Worker disabled. Let the normal external supervisor restart the backend, then verify `/api/meta`, legacy community feed/resources, login, and logout.

In a logged-in `https://hone-claw.com` browser console, request a grant without inspecting or copying cookies:

```javascript
await fetch("/api/public/community/edge-session", {
  method: "POST",
  credentials: "include",
}).then(async (response) => ({
  status: response.status,
  body: await response.json(),
}));
```

Expected body: `enabled=true`, `mode="shadow"`, `base_path="/_community/v1"`, and a near-term `expires_at`. It must not contain a token, secret, phone number, or actor ID. An anonymous request must return `401` and clear any scoped edge cookie. Logout must clear both `hone_web_session` and `hone_community_edge`.

### Step 7: activate and canary the Worker

In Cloudflare Worker Settings, add the plain-text variable `EDGE_DISABLED=false` and deploy that variable change while Pages remains at zero and backend mode remains `shadow`. Do not use `EDGE_DISABLED=true` to activate; true disables the Worker.

Anonymous access must now stop at edge auth:

```bash
curl -i https://hone-claw.com/_community/v1/feed/latest.json
```

Expected result: `401` JSON with `invalid_edge_session`, never a feed or R2 redirect.

In a logged-in same-origin browser console, issue the shadow grant and canary the feed without reading/copying the HttpOnly cookie:

```javascript
await fetch("/api/public/community/edge-session", {
  method: "POST",
  credentials: "include",
}).then(async (response) => ({
  status: response.status,
  body: await response.json(),
}));

await fetch("/_community/v1/feed/latest.json", {
  credentials: "include",
}).then(async (response) => ({
  status: response.status,
  contentType: response.headers.get("content-type"),
  body: await response.json(),
}));
```

Require `200`, JSON, the expected newest content, and a valid `next_before`. Canary one returned `delivery_path` with `HEAD` and `GET`; verify image display, PDF preview, and attachment download. A protected/ineligible resource may intentionally have no `delivery_path` and must stay on the legacy API.

Also prove the compatibility fallback with an authenticated page key that is deliberately absent from R2:

```javascript
await fetch("/_community/v1/feed/pages/9007199254740991.json", {
  credentials: "include",
}).then(async (response) => ({
  status: response.status,
  contentType: response.headers.get("content-type"),
  body: await response.json(),
}));
```

Require backend-shaped JSON `200` with an empty `items` list. Before using this canary, confirm that exact R2 page key is absent. Any Pages HTML, redirect loop, or 5xx means the fixed origin/DNS/TLS/fallback prerequisite is not satisfied.

Also require unsupported methods to return `405`, malformed/version-mismatched paths not to expose objects, a bad/expired edge cookie to return `401` before any R2/cache response, the R2 bucket to remain private, and Worker 5xx/fallback volume to stay low.

### Step 8: move backend to prefer without moving users

After the shadow canary passes, change backend `cloud.community_delivery.mode` to `prefer` and let the normal external supervisor restart it. Keep the Pages build variable at zero. Repeat the logged-in grant/feed/resource canary and require `mode="prefer"`. Normal users still use legacy because the shipped frontend discovery flag is off.

### Step 9: enable the Pages client last

In the Cloudflare Pages **production** build environment, set:

```text
HONE_APP_COMMUNITY_EDGE_DISCOVERY=1
```

Build/deploy the reviewed public artifact with the normal Frontend Update flow. This is a Vite compile-time flag, so changing the dashboard variable without a new Pages build does not activate discovery.

After deployment, verify an existing logged-in user can:

1. open the first community page and an older cursor page;
2. see images, including an edge failure falling back to legacy;
3. preview a PDF (the client HEAD-preflights edge before choosing the iframe source);
4. download an attachment (edge once, then legacy);
5. mark the latest post seen and observe correct personal unread state;
6. log out and log in again without a stale edge grant.

The frontend must not loop/retry the edge hot path. A discovery/feed/resource failure clears the active edge choice for a short backoff and returns to the existing API.

### Step 10: observe before expanding scope

Monitor Cloudflare Worker count/latency/401/5xx/exceptions, R2 operations and bytes, backend `/api/public/community*` count/latency and fallback rate, browser image/PDF/download failures, content/version mismatches, and publisher conflict/idempotence reports.

Do not add Redis or Cloudflare Images during the initial canary. First measure whether remaining latency comes from personalized PG state, uncached legacy resources, very large originals, or image format/dimensions. A later Images binding may be justified for thumbnails/format conversion, but it is a separately billed transform/cache design and must not make private originals public.

### Immediate rollback

Rollback in this order:

1. In the Cloudflare Worker dashboard set `EDGE_DISABLED=true` **and deploy that variable change**. Do not rely on removing the variable: `keep_vars=true` can preserve the last deployed value. Edge calls must return Worker-owned `503`; the compatible client immediately uses legacy. This is the fastest kill switch.
2. Restore Pages production to `HONE_APP_COMMUNITY_EDGE_DISCOVERY=0` and redeploy the public bundle.
3. Return backend `cloud.community_delivery.mode` to `shadow` or `off` and let the external supervisor perform the controlled restart. `off` clears the scoped cookie on the next grant request; existing grants live at most 3600 seconds, while the disabled Worker blocks them immediately.
4. Retain PG rows and R2 feed/descriptors/assets for diagnosis and idempotent retry. Do not delete immutable objects in an emergency rollback.

After rollback, re-run the anonymous legacy `401`, one real logged-in feed/resource browser check, and `/api/meta` authority checks. Rollback is not complete merely because the Worker dashboard shows disabled.

### Immediate resource revocation

For a single resource/version that must stop being served, use this order:

1. In the Cloudflare Worker dashboard set `EDGE_DISABLED=true` and deploy; verify the edge route returns `503`.
2. Use Cloudflare's **global** cache purge. The current Worker does not emit `Cache-Tag`, so use **Purge Everything** for an emergency; do not call Worker `cache.delete`, which removes only the cache in the data center handling that request. See Cloudflare's [Workers cache behavior](https://developers.cloudflare.com/workers/reference/how-the-cache-works/) and [global purge options](https://developers.cloudflare.com/cache/how-to/purge-cache/).
3. Revoke the canonical PostgreSQL resource row through the reviewed data workflow so it is no longer edge-eligible, then run `community-publish` dry-run and apply. Confirm the new mutable `resources/active.json` omits that resource/version and the apply read-back succeeds.
4. If traffic should resume, set `EDGE_DISABLED=false` and deploy. With an authenticated grant, require the old direct `/_community/v1/resources/<id>/<version>` path to return `404 resource_not_active`, never legacy bytes. Repeat from more than one geography if available, and keep the Worker disabled if any location still serves the object.

Immutable R2 bytes/descriptors may remain for forensics and rollback; the per-request active-index gate, not object deletion, is the revocation authority. Shared resource cache entries expire within one hour, but revocation never waits for that TTL because the active index is checked before cache lookup.

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

## Drain Active Chats Before A Controlled Restart

The admin backend exposes the current process's real chat-run count separately from conversation quota:

```bash
curl -fsS http://127.0.0.1:8077/api/runtime/active-chat-runs
```

Expected idle response:

```json
{"count":0}
```

`hone-cli start` polls this endpoint after a normal Ctrl-C and waits for active turns to finish before terminating child processes. Runtime children use separate Unix process groups so the terminal interrupt reaches the CLI supervisor first instead of stopping the Web child before it can be queried. The wait is bounded by the configured agent overall timeout plus a short grace period, capped at six minutes; repeated endpoint failures or the cap allow shutdown to continue with an explicit warning. Prefer sending SIGINT to the supervisor process so this drain path runs. Do not broadcast a signal directly to child PIDs, use `kill -9`, replace the backend process directly, or treat quota `in_flight` as a drain signal.

If a background supervisor launches the prebuilt `target/debug/hone-cli start --build` under a minimal environment, its `PATH` must still include the Cargo binary directory (normally `$HOME/.cargo/bin`). Otherwise the CLI exits before writing `data/runtime/current.pid` and the backend ports remain down. Either include Cargo in the supervisor `PATH`, or finish the required build first and launch without `--build`; never treat a missing PID file as a successful restart.

After restart, verify both the new process and the drain endpoint:

```bash
curl -fsS http://127.0.0.1:8077/api/meta
curl -fsS http://127.0.0.1:8077/api/runtime/active-chat-runs
```

An unexpected process death cannot finish the old turn. Public bootstrap must report that persisted unanswered turn as interrupted; it must not recreate a local “thinking” timer.

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
