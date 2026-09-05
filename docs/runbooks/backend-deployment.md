# Runbook: Backend Deployment

Last updated: 2026-09-05

## When to Use

- Updating the public web frontend served from Cloudflare Pages
- Updating the backend origin service used by the public API
- Verifying the Cloudflare Worker route between the public site and backend origin
- Moving the backend origin to a different managed host
- Building or staging the managed Linux runtime through GitHub Actions and GHCR

## Public Topology

The public entrypoint is split into two layers:

- `hone-claw.com`: Cloudflare Pages serves the static public web bundle
- `hone-claw.com/api/public/*`: Cloudflare Worker proxies public API requests to the backend origin
- `origin.hone-claw.com`: backend origin hostname used by Cloudflare, not a user-facing entrypoint
- `hone-claw.com/_media/v1/*`: Cloudflare Worker reads and writes chat image objects directly in R2; these bytes never reach the backend origin

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
curl -fsSI https://hone-claw.com/ | \
  rg -i 'strict-transport-security|content-security-policy|x-frame-options|x-content-type-options|referrer-policy'
```

For the HTML response, require `Strict-Transport-Security: max-age=31536000`,
`Content-Security-Policy: frame-ancestors 'none'`, and
`X-Frame-Options: DENY`. All public responses must also include
`X-Content-Type-Options: nosniff` and
`Referrer-Policy: strict-origin-when-cross-origin`. `_worker.js` is part of the
public security boundary; do not mark a frontend deployment healthy when these
headers are missing or duplicated with a weaker value.

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

### Runtime timezone is mandatory in production

Set the top-level `timezone` in the deployed `config.yaml` to an explicit IANA
name, for example:

```yaml
timezone: "Asia/Shanghai"
```

The resolution order is top-level `timezone`, `HONE_TIMEZONE`, the host IANA
timezone, the host's current UTC offset, then UTC. The environment variable is
therefore an override for host detection when the config field is absent; it
does not override an explicit config value.

Managed Linux containers commonly report UTC regardless of the operator's
workstation timezone. Never rely on host detection for production scheduling.
Before a restart, verify the effective config contains the intended IANA name
without printing unrelated secrets. After restart, check runtime logs/report
metadata and one scheduled date/time projection. A timezone change affects new
timestamps, cron/date keys, and rendering only; do not rewrite historical rows.

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

### Linux runtime image through GHCR

Do not compile a production runtime on the managed backend host and do not copy
macOS binaries to Linux. `.github/workflows/runtime-image.yml` builds the six
managed binaries inside digest-pinned Debian Bookworm `linux/amd64`, writes the exact
Git SHA into the binaries and bundle metadata, and publishes a `scratch`
artifact image to:

```text
ghcr.io/b-m-capital-research/honeclaw-runtime:<40-character-git-sha>
```

The workflow uses `packages: write` only for its job-scoped `GITHUB_TOKEN`,
links the image to the public source repository before first publication, and
reuses the scoped BuildKit GHA cache. Treat the manifest digest reported by the
workflow as the deployment identity; the mutable `main` tag is only a discovery
alias and must never be a production input.

The managed host does not need Docker or another container daemon. Install the
pinned `crane` release once after verifying the upstream checksum:

```bash
crane_version=0.20.6
crane_archive=go-containerregistry_Linux_x86_64.tar.gz
crane_sha256=c1d593d01551f2c9a3df5ca0a0be4385a839bd9b86d4a76e18d7b17d16559127
crane_tmp_dir="$(mktemp -d)"
curl -fsSL \
  "https://github.com/google/go-containerregistry/releases/download/v${crane_version}/${crane_archive}" \
  -o "${crane_tmp_dir}/${crane_archive}"
printf '%s  %s\n' "$crane_sha256" "${crane_tmp_dir}/${crane_archive}" | sha256sum -c -
tar -xzf "${crane_tmp_dir}/${crane_archive}" -C "$crane_tmp_dir" crane
sudo install -o root -g root -m 0755 "$crane_tmp_dir/crane" /usr/local/bin/crane
rm -rf -- "$crane_tmp_dir"
```

Stage by immutable digest and expected source revision. The staging script
exports `/release`, rejects symlinks, requires the exact six binaries and
metadata fields, checks every payload against `SHA256SUMS`, and refuses an
embedded revision mismatch. It does **not** switch traffic or restart anything:

Before staging, check the filesystem that owns `/opt/hone/releases`. Require at
least 2 GiB available for the current bundle shape, in addition to the current
and rollback releases; stop before export when that floor is not met. A full
filesystem can let staging finish and then make the restarted process fail while
atomically writing its effective config, so an idle-chat check alone is not a
sufficient preflight.

```bash
available_kib="$(df --output=avail /opt/hone/releases | tail -n 1 | tr -d ' ')"
test "$available_kib" -ge 2097152
```

```bash
revision=<40-character-git-sha>
image_digest=sha256:<workflow-reported-digest>
sudo bash scripts/stage_ghcr_runtime.sh \
  --image "ghcr.io/b-m-capital-research/honeclaw-runtime@${image_digest}" \
  --revision "$revision"
```

The expected result is
`/opt/hone/releases/<revision>-ghcr-runtime`. Keep the current release intact,
then continue with environment validation, two idle reads, atomic symlink
replacement, systemd restart, exact `/api/meta` verification, and rollback
retention below.

After the new release and one same-revision restart pass acceptance, retain the
current release, the immediate previous release, and one known-good secondary
rollback. Superseded GHCR releases may be removed only after resolving each
explicit target under `/opt/hone/releases`, rejecting symlinks, proving it is
neither `/opt/hone/current` nor a retained rollback, and recording that its
immutable artifact can be rebuilt. Never use a wildcard or prune user data,
Codex state, skill rollbacks, database backups, or session-binding backups to
make room for a runtime.

Prefer anonymous export when package visibility and organization policy allow
it. If the repository-linked package is private, use only a short-lived or
operator-provided credential scoped to `read:packages`. Transfer it through the
approved secret channel, pass it to `crane auth login` over standard input, and
set `DOCKER_CONFIG` to a newly created mode-`0700` temporary directory. Stage
and verify the exact digest, then delete that directory immediately. Do not put
the token in command arguments/history, reuse a broad personal token, copy the
temporary config into `/root/.docker`, or leave any registry credential on the
host. A failed minimal-auth export stops before staging or cutover.

The current GHCR runtime bundle is executable-only: it contains the six managed
binaries, release metadata, the soul asset, and verification tooling, but it
does not contain the repository `skills/` tree or public share images. A
revision that adds or changes a runtime skill is therefore not deployable merely
because the new binary image is active. Before cutover:

1. Read the live process's `HONE_SKILLS_DIR` without printing the rest of its
   environment. Confirm that it is absolute and readable by the service user.
2. Stage only the changed skill directory from the exact target revision,
   reject symlinks, compare every file to a recorded SHA-256 manifest, and move
   the verified directory into place atomically. Never run `git pull` over a
   dirty host checkout or overwrite an existing modified skill directory.
   Preserve the revision's file modes as part of that manifest. Before cutover,
   require every frontmatter-declared script to match the Git executable bit and
   verify it with `test -x` as the service user; a byte-identical renderer with
   mode `0644` is not a valid skill deployment.
3. Query loopback `GET /api/skills` and require the target skill to be present,
   enabled, and loaded from the system root. This readback is the runtime proof;
   finding a `SKILL.md` on disk is insufficient.
4. When a renderer resolves a repository-relative public asset, verify that
   asset separately. The earnings renderer accepts an explicit
   `HONE_ZSXQ_SHARE_IMAGE`; otherwise it expects
   `packages/app/public/membership_zsxq.jpg` relative to its installed skill.

### Earnings workflow OpenRouter route

The administrator-only earnings preview and earnings analysis turns have a
dedicated runner/model route. They do not inherit the global chat model:

```yaml
agent:
  earnings_workflow:
    runner: "opencode_acp"
    model: "google/gemini-3.1-pro-preview"
```

The OpenRouter credential remains in canonical config under
`llm.providers.openrouter.api_key/api_keys`; it is injected into the OpenCode
child and must not be copied into `agent.opencode`, an environment file, the
release directory, or a command argument. On a managed host, keep the canonical
config owner-only, use the interactive provider configurator or an approved
stdin-only secret update path, write through a mode-`0600` staging file, and
atomically replace the exact file the service loads. Never print the old/new
key or the whole config while checking the change.

Before restart, validate only non-secret fields and credential presence:

```text
agent.earnings_workflow.runner = opencode_acp
agent.earnings_workflow.model = google/gemini-3.1-pro-preview
llm.providers.openrouter.kind = openrouter
llm.providers.openrouter.base_url = https://openrouter.ai/api/v1
llm.providers.openrouter.api_key/api_keys has at least one non-placeholder value
```

Also require a real authenticated probe to the exact model and a complete
OpenCode ACP `initialize -> session/new -> session/prompt` probe. HTTP `200`
from the models endpoint or ACP `initialize` alone is insufficient. Runtime
acceptance must show `runner=opencode_acp` and transport model
`openrouter/google/gemini-3.1-pro-preview` without logging the credential, then
complete the forced `earnings-research` skill and persist/download its PDF.
Missing credentials, an unsupported workflow runner, a different response
model, or fallback to the global model is a stop condition.

The earnings PDF renderer also needs a Linux Chromium executable and a CJK font
available to the service account. On a Debian managed host, install and verify
the minimal runtime dependencies before enabling the entry:

```bash
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  chromium fonts-noto-cjk
chromium --version
fc-match "Noto Sans CJK SC"
```

Run the repository-owned renderer as the actual service user, then render every
page of the resulting PDF to PNG for visual acceptance. Require A4 pages,
legible Chinese text, the exact `知识星球：巴芒科技` watermark, the Knowledge
Planet share page, and no tofu-square glyphs, clipping, or overlap. A successful
renderer exit or `%PDF` header alone does not prove that the host has usable CJK
fonts.

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

`hone-cli start` treats the local
`/api/runtime/active-chat-runs` response as process readiness. Do not replace
that startup probe with `/api/meta`: metadata intentionally performs live
PostgreSQL and object-storage checks, so a slow external dependency can exceed
the short process-readiness timeout and make the supervisor terminate an
otherwise listening backend. This lightweight startup boundary does not relax
deployment acceptance; operators must still verify the full `/api/meta` cloud
authority fields below after the process is ready.

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

### Managed-host operator access and origin health

Keep the exact cloud project, instance, zone, address, and connection command in
an ignored operator note, never in the repository. Prefer the managed access
proxy path over depending on a public SSH firewall rule, and keep identity
permissions least-privileged and auditable. Instance-scoped authentication
metadata does not survive replacement by definition: after a host is rebuilt
or replaced, re-check the intended login policy before attempting a deployment.

Host login and a listening `8077` / `8088` do not prove that public users can
reach the service. If the Pages homepage loads but `/api/public/*` or
`origin.hone-claw.com/api/public/auth/me` fails:

1. Confirm loopback `8088` returns the expected unauthenticated JSON `401`.
2. Confirm the managed service is active, its executable resolves under
   `/opt/hone/current`, and its working directory and runtime environment match
   the reviewed production configuration.
3. Confirm the Cloudflare public API route still points `/api/public/*` to the
   backend origin and that the origin hostname returns the same JSON `401`.
4. Inspect only the failed network or service lane. Never paste complete proxy
   diagnostics, request headers, session cookies, environment files, or cloud
   identity output into logs or tickets.
5. Require repeated `401` responses from both the origin and public hostname,
   plus one real public-client bootstrap, before declaring recovery.

### Origin access log

The reverse proxy in front of `8088` writes a per-request JSON access log to
`/var/log/caddy/origin-access.log`, rolling at 20MiB with five files kept for
14 days. Use it to answer "did this request reach the origin at all", which the
process journal cannot: a rejected or unauthenticated request produces no
application log line, so journal silence is not evidence of non-delivery. Each
entry carries `Cf-Ray`, so an upstream delivery dispute can be reconciled by ray
ID against the provider's own logs.

Both this log and the default error logger delete `X-Hone-Origin-Token`,
`Cookie`, `Authorization` and `Stripe-Signature` before writing. Keep that
filter when editing the proxy config: the shared origin token is a credential,
and it was previously written verbatim into every proxy error entry.

Validate proxy config as root with care. Running the validator under `sudo`
creates the log file owned by `root`, and the service account then cannot open
it, which fails the reload and leaves the unit stuck in `reloading`. Ensure the
log file is owned by the proxy service account, then reload through the proxy's
admin API rather than restarting, so the listener keeps its connections.

## Public Auth Runtime Env

### Scheduled-push unsubscribe and Email Sending

Scheduled pushes delivered to Feishu, Discord, or Telegram append a signed,
login-free unsubscribe link when `HONE_UNSUBSCRIBE_SECRET` is present. Generate
at least 32 random bytes, keep the same value in every backend/channel process,
and store it only in the production secret environment. Rotating it immediately
invalidates every previously delivered unsubscribe link. Missing configuration
fails closed: no link is emitted and no unsigned link is accepted.

The public capability URL is
`/api/public/unsubscribe/{job_id}.{signature}`. Production Pages only proxies
`/api/public/*`, so the handler must remain on the public API router. `GET`
renders a confirmation page and must never mutate state; only the form `POST`
disables the job.

Cloudflare Email Sending uses the existing runtime-only
`HONE_CLOUDFLARE_ACCOUNT_ID`, `HONE_CLOUDFLARE_EMAIL_API_TOKEN`, and
`HONE_EMAIL_FROM` inputs. The `email.api_token_env` config defaults to the same
token variable so operators do not duplicate the credential. The current
`hone_core::email::EmailSender` is provider plumbing and is not yet called by a
scheduler delivery path; configuring these values enables the existing email
verification sender and prepares the push sender, but does not by itself make
scheduled pushes arrive by email.

Public SMS login and optional captcha are runtime env configuration, not
`config.yaml` fields. Keep real values in the backend host environment or
supervisor, never in committed files. The active, non-revoked admin-created Web
invite user list remains the public-login invite-list admission source and the
final admission decision, but public responses must not disclose membership.
Provider delivery may run only after all server-side
guards pass; the HTTP response path must remain generic and independent of
provider latency. Code verification precedes the invite lookup during login,
and non-members still fail closed without a session.

The application-enforced abuse limits are:

- at most one successful send per phone per 60 seconds;
- at most 10 successful sends per phone per rolling day;
- at most 60 send attempts per source IP per rolling hour;
- at most 16,384 tracked limiter identities process-wide, with unseen
  identities rejected while full instead of growing the map.

These are security invariants. Browser cooldowns, Aliyun quotas, captcha, and
Cloudflare WAF rules are additional layers and must not replace them. A
production canary must use a designated test invite/number and must never send
bulk SMS. Verify that repeated sends return the same generic public response,
that the second accepted delivery is not possible inside 60 seconds, and that
uninvited or invalid-code login attempts return the same generic `401` shape
without revealing membership.

Required for SMS send/check:

```text
ALIBABA_CLOUD_ACCESS_KEY_ID
ALIBABA_CLOUD_ACCESS_KEY_SECRET
```

The backend also accepts the compatibility aliases `ALIYUN_ACCESS_KEY_ID` / `ALIYUN_ACCESS_KEY_SECRET` and `HONE_ALIYUN_ACCESS_KEY_ID` / `HONE_ALIYUN_ACCESS_KEY_SECRET`. Prefer the `ALIBABA_CLOUD_*` names for new deployments.

Before every managed backend start or restart, validate the exact persistent
environment file that the supervisor will load. The validator prints only the
matched variable names and never credential values:

```bash
sudo bash scripts/check_backend_runtime_env.sh /etc/hone/runtime.env
```

On a systemd host, install the validator outside the immutable release tree and
make it a persistent start gate rather than relying on an operator remembering
the check:

```ini
[Service]
ExecStartPre=+/usr/local/sbin/hone-check-web-env /etc/hone/runtime.env
```

Keep `/etc/hone/runtime.env` owned by `root:root` with mode `0600`. Update it
through a `0600` staging file plus an atomic install, validate the staged and
installed files, then restart. Never pass credential values in command-line
arguments or print the environment file. A missing or placeholder credential
must block a future start while leaving the currently running process intact.

The SMS send endpoint intentionally returns the same generic acceptance body
for eligible and ineligible phones before the detached provider call finishes.
Therefore an HTTP `200` alone is not a delivery canary. After restart, perform
one designated-number send and require both provider acceptance and absence of
`SMS verification send failed after generic acceptance` in the service journal.

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

PostgreSQL runtime and deployment role:

```text
HONE_RUNTIME_ROLE=web|worker|all
DATABASE_URL=<postgres-url>
HONE_POSTGRES_PROXY=socks5://127.0.0.1:1082
```

PostgreSQL is mandatory and authoritative in every deployment mode, including local development. `HONE_DEPLOYMENT_MODE=local` controls local-only product behavior but does not select a storage backend. Object storage is optional; omit `HONE_OSS_*` locally when only Docker PostgreSQL is available.

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
hone-cli cloud migrate --from-data-dir ./data --quota-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --skill-registry-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --notification-prefs-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --portfolio-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --company-profiles-only --apply --json
hone-cli cloud migrate --from-data-dir ./data --event-store-only --apply --json
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

`community-contents` is a bootstrap/recovery command that requires the complete source timeline: `source_topic_index` and source file positions must each be contiguous from zero. It first reconciles existing file-backed rows by source file position, then uses `candidate_fingerprint + occurrence` as the stable identity for missing or non-file posts. Apply mode locks the community space and inserts every missing post and its ordered resources in one PostgreSQL transaction. A second dry-run must report `would_insert=0` before the migration is considered complete. **Do not use this command for the weekly append:** inserting newer topics at the front shifts source file positions and can match new rows to old content. Use the dedicated `community-append` workflow below for incremental updates; never substitute positional bootstrap reconciliation.

Production community commands must go through `scripts/community_production.py`, not a bare CLI in the repository directory. The wrapper reads the managed service environment over IAP, checks a previously reviewed PostgreSQL identity, creates a loopback-only tunnel and runs the CLI from an isolated temporary directory. Credentials remain in memory. The ignored `data/community-imports/production-operator.json` contains only `project`, `instance`, `zone` and `expected_pg_identity_sha256`; review those values against the managed service before provisioning or changing them. A fingerprint mismatch requires investigation, not automatic repinning. Repository `.env` is not production authority.

```bash
cargo build -p hone-cli
python3 scripts/community_production.py cloud community-inspect --anchor-only
python3 scripts/community_production.py cloud community-append --manifest /path/to/append.json
python3 scripts/community_production.py cloud community-append --manifest /path/to/append.json --apply
python3 scripts/community_production.py cloud community-append --manifest /path/to/append.json
python3 scripts/community_production.py cloud community-assets --manifest /path/to/verified-assets.json
```

`community-inspect --anchor-only` emits the exact anchor object without model-derived hashes. `community-append` accepts continuous newest-first source items with stable identities and verifies the original anchor even on replay. New inserts also require the current head to match, and are written oldest-first so the database's timestamp/ID ordering preserves the source's same-minute order. Existing source identities must have matching author, time, body and ordered resources; a previously unknown file identity may be legitimately completed by asset backfill. The operation is atomic and a replay must show zero inserts. Use the production wrapper for asset apply and publisher operations too. See [Community insights sync](community-insights-daily-sync.md) for the capture and scheduling workflow.

`community-assets` accepts only ordinary non-symlink files with an allowlisted MIME/magic signature, exact manifest byte size, and exact SHA-256. It verifies or creates a full-SHA immutable object key, reads the object back from R2, and only then promotes the PostgreSQL resource row through an optimistic lock. Never put source cookies, signed download URLs, or authorization headers in either manifest. Source-protected resources stay metadata-only unless the authorized source UI legitimately exposes their bytes.

Every promoted resource keeps the previous SHA, size, object URI, and access state under `raw_metadata.community_asset_backfill`. Immutable R2 objects are retained for rollback. If a backfill must be reverted, restore the previous row values from that audit metadata or a PG snapshot; do not delete the old or new object until the restored application path has been verified.

The migrator uploads recognized durable files and indexes them in PG `cloud_documents`. It also imports legacy `sessions/*.json` into PG `cloud_sessions`, `conversation_quota/*.json`, `runtime/skill_registry.json`, `notif_prefs/*.json`, `portfolio/*.json`, and actor-scoped `company_profiles/**/*.md`; use the matching narrow `--*-only --apply` modes for fast idempotent passes before the larger object migration. The completed Web-auth and LLM-audit import flags intentionally reject further use. The retained `--event-store-only` channel reads `events.sqlite3` only as historical input and imports the five event tables into PG. Use the lower-concurrency `--reuse-existing` retry when proxy or OSS connections drop during a large upload. All runtime stores are PostgreSQL-backed; generated images, uploads, and attachment/document surfaces use OSS where configured and otherwise retain their explicit local file fallback.

## Public Community Private-R2 Edge Rollout

### Current recovery and historical rollout evidence

The July rollout narrative below is historical. A September 5 audit found that the Worker had since been enabled and configured with a managed HTTPS fallback, while the backend signing secret was absent from the managed process environment and a local automation had published a different database into the same R2 prefix. Do not redeploy the July Worker configuration over the live version. Inspect actual deployed code, bindings and variables first; preserve the reviewed origin fallback and signing secret. The current recovery and exact acceptance results are recorded in [the September 5 handoff](../handoffs/2026-09-05-community-freshness-assets-latency.md).

Publish from the same production PostgreSQL authority used by the API before enabling edge grants. An immutable key conflict must never be overwritten: use a new delivery prefix with matching Worker feed/resource configuration and cache isolation if a projection migration requires it. `latest.json` is published last, after resources and older pages. Bounded parallel metadata preflight reduces publication time while preserving conflict checks and full-byte verification during apply.

The backend grant secret must be in the managed supervisor environment file, not only a checkout `.env`. Apply environment updates atomically with restricted permissions and use the normal managed restart. Resource HEAD and conditional GET responses inspect object metadata and size without downloading the body; ordinary GET still verifies full SHA-256. A metadata response is not evidence of full-byte integrity.

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

## Public Media Edge Rollout

Chat image attachments move browser <-> nearest Cloudflare PoP <-> R2. The
backend origin in us-central1 only mints short-lived signed capabilities; it
never carries image bytes on either leg. Before this path existed, a pasted
screenshot crossed the Pacific to be stored and crossed back in full to render a
72px thumbnail.

Components:

- `workers/public-media-edge` — Worker on `hone-claw.com/_media/v1/*`, R2 binding
  `MEDIA_BUCKET` -> the existing private `honeclaw` bucket.
- `POST /api/public/media/session` — issues the `hone_media_edge` read cookie.
- `POST /api/public/media/upload-grant` — issues one single-use upload
  capability per file and refreshes the read cookie in the same response.
- `cloud.media_delivery` in backend config; secret in
  `HONE_MEDIA_EDGE_HMAC_SECRET`.

### Authorization model

Reads and writes are deliberately authorized differently.

| | Read | Write |
| --- | --- | --- |
| Carrier | `hone_media_edge` cookie, `HttpOnly; Secure; SameSite=Strict; Path=/_media/v1/` | `X-Hone-Media-Token` request header |
| Scope | the caller's own `public-uploads/<user>/` prefix | one exact object key |
| Also bound to | — | content type, byte ceiling |
| Lifetime | `read_ttl_secs`, clamped 60..3600 | `write_ttl_secs`, clamped 30..300 |
| Reuse | until expiry | single use: the edge refuses to overwrite an existing object |

Why a cookie for reads rather than a signed URL: a capability in a query string
leaks through `Referer`, browser history, and any intermediary's access log.
Why a header for writes: the client must not choose the key, and a custom header
cannot be set by a cross-site form, so it doubles as the CSRF guard. The Worker
answers no CORS preflight and rejects any request whose `Origin` is not
`https://hone-claw.com`.

What the Worker enforces independently of the origin's signature:

- Key shape: exactly `public-uploads/<owner>/<day>/<stored-name>`, every segment
  matching `[A-Za-z0-9._-]+` and never `.` or `..`; `%` anywhere is rejected, so
  there is no second decoding pass to disagree about.
- Ownership: the requested key must start with the signed `pfx`, and `pfx` must
  itself be a two-segment owner root under the configured upload prefix. A grant
  that authenticates one tenant while minting a key for another fails here even
  though its signature is valid.
- Content type: `image/png`, `image/jpeg`, `image/webp`, `image/gif` only.
  **`image/svg+xml` is excluded and must stay excluded** — an SVG served from
  `hone-claw.com` is same-origin script, so accepting one would turn the upload
  box into stored XSS.
- Magic bytes: the leading bytes must match the signed content type, so a token
  signed for `image/png` cannot be used to store HTML or SVG.
- Size: `Content-Length` is required and checked against the token's ceiling, and
  the body is read through a cap so a lying `Content-Length` cannot exhaust
  memory. The declared and actual lengths must agree.
- Responses carry `Content-Security-Policy: default-src 'none'; sandbox`,
  `X-Content-Type-Options: nosniff`, `Cross-Origin-Resource-Policy: same-origin`,
  `Referrer-Policy: no-referrer`, and `Content-Disposition: inline`.

The token wire format is pinned by matching golden vectors on both sides
(`routes::public_media::tests` and `workers/public-media-edge/test`). A change to
claim order or encoding on either side fails a test instead of silently breaking
uploads in production.

### Fail-closed defaults

- `MEDIA_EDGE_DISABLED` absent disables the Worker. Activation is an explicit
  `MEDIA_EDGE_DISABLED=false`, preserved across deploys by `keep_vars`.
- A missing, short, or oversized secret returns `503`; so does an unbound bucket.
- Backend `mode: "off"` issues no capabilities at all.
- The client falls back to the origin proxy (`/api/public/upload`,
  `/api/public/image`) whenever the edge is off, a grant is refused, or a `PUT`
  is not acknowledged. A `PUT` counts as acknowledged only on `201` with a JSON
  `{"ok":true}` body, because a missing Worker route makes Pages answer
  `/_media/v1/*` with the SPA shell and a `200`.

### Deployment record

2026-08-30: `hone-public-media-edge` deployed and activated on
`hone-claw.com/_media/v1/*` (version `8ddb8edc-2f31-4d5e-b1d3-d405ee8ecf92`),
bound to R2 bucket `honeclaw`, `MEDIA_EDGE_DISABLED=false`, secret
`MEDIA_EDGE_HMAC_SECRET` installed.

Deployed with a scoped API token named `hone-media-edge-deploy` rather than
`wrangler login`: three permissions only — account `Workers Scripts:Edit` and
`Workers R2 Storage:Edit`, zone `Workers Routes:Edit` limited to hone-claw.com.
The OAuth flow was rejected for this purpose because it grants 29 scopes
including Pages:Write, Email Sending, Connectivity Directory Admin, and a
persistent refresh token. Note that a runtime variable set in the dashboard does
not reach the running Worker until the next deploy; `keep_vars: true` preserves
it, so `wrangler deploy` is the way to activate it.

Verified live against the deployed Worker (self-test objects removed afterwards):

| Case | Result |
| --- | --- |
| GET, no cookie | `401 missing_media_session` |
| GET, valid cookie, own prefix, absent object | `404 not_found` |
| GET, valid cookie, another tenant's key | `403 object_outside_session_scope` |
| GET, expired / wrong-audience / over-broad `pfx` / tampered signature | `401` |
| GET, duplicated cookie header | `401` |
| Read cookie presented as an upload token | `401 invalid_upload_token` |
| PUT, no token | `401 missing_upload_token` |
| PUT, SVG bytes under an `image/png` token | `415 unsupported_image_format` |
| PUT, body over the token's ceiling | `413 upload_too_large` |
| PUT, token key ≠ request path | `403 upload_token_key_mismatch` |
| PUT, valid | `201 {"ok":true}` |
| PUT, replayed token | `409 object_already_exists` |
| Read back | `200`, `image/png`, `inline`, `nosniff`, CSP, bytes identical |

**The backend half is not deployed yet**, so no capability is minted in
production and the edge 401s every request. The client keeps using the origin
proxy until `cloud.media_delivery.mode` is set. `HONE_MEDIA_EDGE_HMAC_SECRET`
must be installed on the origin with the exact bytes already held by the Worker
before switching the mode; a mismatch fails closed.

2026-08-30（后端半边）：`37d8fbaa` 上线到 GCE `instance-20260731-081043`。
`cloud.media_delivery` 写进 `/srv/honeclaw/config.yaml`（先落 `off`，再翻到 `shadow`），
`HONE_MEDIA_EDGE_HMAC_SECRET` 装进 `/etc/hone/runtime.env`（600 root:root，`openssl rand -base64 48`，
64 字节）。两个后端端点已验活：`POST /api/public/media/session` → `401`、
`/api/public/media/upload-grant` → `415`（都不是 404，说明路由挂上了）。
Worker 侧外部复验：`GET /_media/v1/o/...` 无 cookie → `401` 且带 CSP / CORP / nosniff / Referrer-Policy，
`OPTIONS` → `405 Allow: GET, HEAD, PUT`（无 CORS 预检），路由绑定正常。

密钥对齐走的是**反方向**：不动 Worker，把 origin 改成 Worker 已经持有的那份。
部署 Worker 的那次会话把密钥留在了工作目录里（`media_edge_secret`，64 字节，
时间戳与 `wrangler secret put` 同一分钟），指纹核对一致后用管道写进 origin 的 runtime.env，
值不落盘、不回显。**这条路径不需要任何 Cloudflare 凭据**——Cloudflare 的 secret 写入后读不回来，
但要对齐并不一定要往那边写。

`mode: "prefer"` 已生效。对线上 Worker 的三条端到端验证（用同一份密钥按契约自签读 cookie）：

| 用例 | 结果 | 说明 |
| --- | --- | --- |
| 自己 prefix 下不存在的对象 | `404 not_found` | Worker **接受了 origin 的签名**，这就是两边密钥一致的证明 |
| 同一 cookie 够别人的 prefix | `403 object_outside_session_scope` | 归属隔离生效 |
| 签名改一个字节 | `401 invalid_media_session` | 确实在验签，不是放行一切 |

剩下的只有浏览器侧那一条（runbook 第 6 步）：粘一张图，devtools 里确认 `PUT` 打到
`/_media/v1/o/...` 拿 `201`，且后面没有 `/api/public/upload`。

### Rollout order

1. Deploy the Worker with `MEDIA_EDGE_DISABLED` absent and confirm every
   `/_media/v1/*` request returns the Worker-owned `503`.
2. Install the same secret in both places — Worker secret
   `MEDIA_EDGE_HMAC_SECRET` and backend `HONE_MEDIA_EDGE_HMAC_SECRET`. Generate
   with `openssl rand -base64 48`. It is a secret, never a Worker var.
3. Confirm `workers/public-media-edge/wrangler.jsonc` still binds `MEDIA_BUCKET`
   to `bucket_name = honeclaw`, the active `HONE_OSS_BUCKET`. Do not create a
   public duplicate bucket and do not hand the browser a bucket URL.
4. Set backend `cloud.media_delivery.mode: "shadow"` and restart. Capabilities
   are minted; clients still use the origin proxy.
5. Set `MEDIA_EDGE_DISABLED=false` on the Worker and verify by hand:

```bash
curl -i https://hone-claw.com/_media/v1/o/public-uploads/x/y/z.png
curl -i -X OPTIONS https://hone-claw.com/_media/v1/o/public-uploads/x/y/z.png
```

   Expect `401 {"error":"missing_media_session"}` for the first and
   `405` with `Allow: GET, HEAD, PUT` for the second. A Pages HTML body or a
   Cloudflare-branded error is a stop condition: the route is not bound.

6. Move backend to `mode: "prefer"` and restart. Paste an image in a logged-in
   browser and confirm in devtools that the `PUT` goes to `/_media/v1/o/...`,
   returns `201`, and that no `/api/public/upload` request follows.
7. Confirm the send path still works end to end: the chat turn must accept the
   `oss://` path and the model must receive the image.

### Rollback

Fastest first:

1. Backend `cloud.media_delivery.mode: "off"` and restart — clients return to the
   origin proxy on their next call; already-stored objects keep rendering through
   `/api/public/image`.
2. Or remove `MEDIA_EDGE_DISABLED` from the Worker — every `/_media/v1/*` request
   becomes `503` and the client falls back on error.
3. Rotating `HONE_MEDIA_EDGE_HMAC_SECRET` invalidates every outstanding cookie
   and upload capability immediately. Rotate the Worker secret and the backend
   env together; a mismatch is fail-closed, not fail-open.

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

## Preserve Enabled Channel Workers Across Web Restarts

Managed channel workers such as `hone-channel@feishu.service` are separate
processes from `hone-web.service`. A template instance may use
`PartOf=hone-web.service` so a Web stop or restart also stops the worker, but
`PartOf` does not start that worker again. `systemctl enable` only joins the
worker to its normal boot target and is not, by itself, a reverse dependency
from Web.

For every channel worker that production is expected to keep online, install an
explicit reverse dependency once on the host:

```bash
sudo systemctl add-wants hone-web.service hone-channel@feishu.service
sudo systemctl daemon-reload
```

Substitute only channel instances that are enabled in the reviewed production
configuration; do not turn this example into an unconditional list of every
compiled channel. After every controlled Web cutover, verify the expected
workers independently of Web health:

```bash
sudo systemctl is-enabled hone-channel@feishu.service
sudo systemctl is-active hone-channel@feishu.service
sudo systemctl show hone-channel@feishu.service -p NRestarts --value
sudo journalctl -u hone-channel@feishu.service --since "10 minutes ago" --no-pager
```

Acceptance requires the expected worker to be active, its transport to have
reconnected, and a recent real message to complete the receive/send path without
channel errors. A healthy `/api/meta` response proves only the Web process; it
does not prove that any sidecar channel is receiving events.

## Audit Codex ACP Bindings After Rollout-State Changes

HONE stores each logical session's `codex_acp_session_id` in authoritative session metadata, while Codex stores the corresponding rollout in its configured persistent `CODEX_HOME`. Replacing, clearing, restoring, or changing ownership of that Codex home without reconciling the bindings can leave a valid HONE history pointing to a rollout the adapter can no longer resume.

After any deliberate Codex state cleanup or restore:

1. Drain active chats to zero before inspecting or repairing bindings.
2. Inventory native thread IDs from the live service user's Codex state and compare them with nonempty `codex_acp_session_id` values in authoritative session storage. Compare IDs only; never export prompts, titles, credentials, or message bodies.
3. Before an operator repair, save an owner-only backup of the affected session ID plus the three bounded fields `codex_acp_session_id`, `codex_acp_session_mode`, and `codex_acp_instruction_fingerprint`, with a checksum. Do not copy complete session content when these fields are sufficient.
4. Remove only those three fields for IDs proven absent; do not delete the HONE session, user/assistant history, uploads, or actor identity. The next turn checkpoints a new native ID before its first prompt.
5. Verify a real authenticated native turn, then recount bindings and require service health, cloud-authoritative storage and active-chat count to remain healthy.

The runtime also recognizes the validated Codex ACP `1.1.7` structured response `error.data.details = "no rollout found for thread id <same persisted id>"` before any prompt and replaces that unusable binding in place. This is not a generic retry: `Internal error` without that exact structured proof, a different ID, timeout, auth/permission failure, process exit, or matching stderr text must still fail closed. Never bulk-clear bindings merely because the number of HONE sessions differs from the current Codex task count; retained, archived and deliberately external state may require separate operator judgment.

## Security Notes

- Do not expose the admin web surface through the public domain.
- Keep admin APIs behind separate authentication and non-public routing.
- Do not commit API tokens, tunnel tokens, runtime databases, or exported production config.
- Prefer documenting stable host roles over physical or personal infrastructure details.
- If the backend origin moves, update the Worker origin hostname or DNS target first, then rerun the verification commands above.
- The media edge secret (`HONE_MEDIA_EDGE_HMAC_SECRET` / Worker `MEDIA_EDGE_HMAC_SECRET`) is the whole authorization boundary for chat image objects. Keep it out of YAML and out of Worker vars, and rotate both sides together.
- Never add `image/svg+xml` to the media edge content-type allowlist. Objects are served from `hone-claw.com`, so an SVG there is same-origin script.
- Do not add CORS headers to `/_media/v1/*`. Same-origin-only is what keeps a leaked upload capability unusable from another site.

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
