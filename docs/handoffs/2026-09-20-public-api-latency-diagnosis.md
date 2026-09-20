# Public API Latency Diagnosis

- title: Public API latency diagnosis and safe connection performance optimization
- status: done (production rollout and acceptance completed)
- created_at: 2026-09-20
- updated_at: 2026-09-20
- owner: Codex
- related_files: Cargo.toml; deploy/runtime/Dockerfile; crates/hone-core/src/cloud_runtime.rs; memory/src/web_auth.rs; crates/hone-web-api/src/routes/public.rs; crates/hone-web-api/src/routes/billing.rs; crates/hone-web-api/src/routes/research_overview.rs
- related_docs: docs/deliverables.md; docs/archive/plans/public-api-connection-performance.md; docs/decisions.md#d-2026-09-20-02-exclusive-query-connections-and-optimized-source-runtime; docs/runbooks/source-web-startup.md
- related_prs: none

## Summary

The user's small authenticated JSON requests take about two seconds. Read-only production measurements reproduce substantial origin CPU cost even over loopback. The leading explanation is repeated PostgreSQL connection/authentication on sequential request paths, amplified by an unoptimized production build and concurrent requests. Network transit contributes, but buying Cloudflare acceleration first is not supported by this evidence.

At the initial diagnostic stage, the active production executable was revision `9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d`, from its `-ghcr-runtime` release directory. Code analysis used `git show` at that revision, not the older, dirty local checkout.

## What Changed

At the initial diagnostic stage, only this record and its archive index entry were added. No implementation, deployment, database data, authentication settings, Cloudflare settings, or subscriptions were changed at that stage. The user subsequently authorized implementation and emphasized request/user isolation; the appended implementation stage below records that work without replacing the diagnostic evidence.

## Verification

### Existing browser requests and origin access log

Read the existing authenticated Chrome Network panel, without exporting cookies or replaying valid credentials. Browser response Date, path, body size, ordering, and nearby Cloudflare request identifiers associate the following origin records with the screenshot. The edge and Worker subrequest Ray IDs differ; this is correlation, not a shared end-to-end trace identifier.

| Endpoint | Browser duration | Origin Caddy duration | Response Date UTC |
| --- | ---: | ---: | --- |
| `/api/public/auth/me` | 1.85 s | 1.65283 s | 2026-09-20 06:29:56 |
| `/api/public/research-overview` | 2.03 s | 1.63694 s | 2026-09-20 06:29:57 |
| `/api/public/bootstrap` | 2.29 s | 1.99438 s | 2026-09-20 06:29:59 |
| `/api/public/pushes?limit=1` | 1.92 s | 1.70357 s | 2026-09-20 06:29:59 |

`me` Timing: queue 1.28 ms, stalled 0.42 ms, request sent 0.13 ms, waiting 1.85 s, download 2.33 ms. `pushes`: queue 0.72 ms, stalled 0.28 ms, request sent 0.18 ms, waiting 1.92 s, download 0.43 ms. Browser edge was SJC. These observations do not establish latency for mainland users without a proxy.

Caddy duration includes downstream response writing and is not an exact application-span duration. Browser minus Caddy must not be represented as a precise pure-network breakdown. Nonetheless, corresponding origin durations are large, and independent loopback controls reproduce internal cost.

### Loopback controls on the production host

Used Python urllib with proxies disabled against `http://127.0.0.1:8088/api/public/bootstrap`. The only supplied Cookie was the deliberately invalid, non-secret `hone_web_session=hone-readonly-latency-probe-invalid`; all responses were 401. This takes the missing-session branch before session updates and performs no business writes.

- No Cookie, six sequential requests: 14.42, 7.75, 5.71, 3.84, 3.54, 3.66 ms.
- Invalid Cookie, eight sequential requests: 171.34, 193.58, 183.24, 156.07, 174.43, 199.29, 178.80, 196.53 ms.
- Eight invalid requests: 1,455 ms wall; 1,360 ms console-process user+system CPU, measured from `/proc/<pid>/stat` with `SC_CLK_TCK`.
- Four concurrent invalid requests: 542.04, 536.15, 534.72, 550.81 ms; batch 573 ms wall and 1,010 ms process CPU.
- Background control: 160 ms process CPU during 1,000 ms without probe requests. Process CPU includes other production work; do not attribute every CPU millisecond to the probes.

### PostgreSQL control

The active console process connects to `127.0.0.1:5432`; host authentication rules specify `scram-sha-256`. Used the host's existing libpq through Python ctypes, reading credentials only into server memory from the already-running console process. No credentials or query results were exported.

Read-only equivalent absent-session lookup, five fresh native connections:

- Connection: 25.79, 18.22, 17.81, 19.25, 17.89 ms.
- Query on each fresh connection: 2.05, 1.83, 2.02, 1.91, 1.76 ms.
- Same lookup on one reused connection: 0.43, 0.22, 0.28, 0.19, 0.16 ms.

### Deployed source path

At production revision, `connect_client()` calls `connect_new_client()`, which calls `tokio_postgres::connect(..., NoTls)`. A reusable client API exists, but these web-auth methods do not use it.

For a normal domestic-invite authenticated `me` request, the source path contains eight sequential fresh connections: session lookup, user lookup, session last_seen upsert, external profile, billing entitlements, external profile again for access, quota snapshot, admin flag. International-account branches may add work. `bootstrap`, pushes and research overview repeat the same authentication prefix; overview additionally awaits multiple section snapshots sequentially.

The production revision's `deploy/runtime/Dockerfile` builds `--profile source-runtime`. That Cargo profile inherits `dev` with no optimization override in the inspected build configuration. `debug=1` only controls debug information; it does not enable optimization. The deployed lockfile uses postgres-protocol 0.6.11, whose SCRAM implementation performs an iterative HMAC/SHA-256 password derivation. Combined with loopback CPU measurements, unoptimized repeated SCRAM is a strong candidate hotspot. No production CPU-stack profile or production optimized-build A/B was obtained; a later local synthetic A/B is recorded below. `perf` is not installed; none was installed for this investigation.

Follow-up scope check: this is a shared-path problem, not specific to `me`. Deployed handlers for bootstrap, research overview, pushes, quotes, portfolio, subscriptions, daily signals, company ratings, portfolio news, position management, influencer digest, weekly brief and other authenticated routes call `require_public_user` directly or through a helper. For ordinary domestic-invite users, that common prefix performs session lookup, user lookup, last_seen write and access-profile lookup, each with a fresh connection, before endpoint-specific work begins. `me` uses the same session-auth prefix and adds account-summary queries. The earlier loopback probe actually targeted bootstrap. This establishes broad exposure to the same overhead; it does not prove every endpoint has identical costs or exclude additional endpoint-specific bottlenecks.

### Continued investigation: cross-endpoint and executor interference

Same production revision, same loopback origin, one no-cookie and one deliberately invalid-cookie request per endpoint, all 401:

| Endpoint | No Cookie ms | Invalid Cookie ms |
| --- | ---: | ---: |
| auth/me | 12.20 | 149.23 |
| bootstrap | 3.48 | 143.56 |
| research-overview | 7.97 | 157.28 |
| pushes?limit=1 | 7.06 | 144.46 |
| quotes | 6.22 | 143.73 |
| portfolio | 4.75 | 162.74 |
| subscriptions | 6.56 | 148.60 |
| daily-signals/macro | 4.93 | 152.04 |

These probes isolate the common missing-session lookup, not the successful business path. They reproduce nearly identical shared overhead across eight endpoints.

Cross-request interference control: start two invalid-cookie bootstrap requests; 35 ms later submit a no-cookie `auth/me` request (which never queries PostgreSQL). Use three Python HTTP threads and proxies disabled. Three rounds yielded:

| Round | DB request A ms | DB request B ms | No-DB request ms |
| --- | ---: | ---: | ---: |
| 1 | 239.89 | 441.15 | 184.27 |
| 2 | 393.86 | 209.91 | 165.03 |
| 3 | 230.37 | 402.65 | 185.50 |

No-DB baseline before: 11.61, 8.15, 5.48 ms; after: 6.56, 8.31, 5.14 ms. This directly demonstrates cross-request delay even for a path that does not touch the database. Only six missing-session DB probes were added for this experiment; no valid session was replayed.

Production has two logical CPUs and two `tokio-runtime-w` threads, plus main and tracing threads. In the locked tokio-postgres 0.7.17 source, `connect_raw.rs` invokes `scram.update(body.data())` synchronously inside its async authentication flow. The SCRAM iteration loop contains no async yield. This gives a concrete mechanism for connection authentication to occupy request-executor threads and delay unrelated ready requests. Exact production stack-time attribution still requires profiling.

Other checks: sampled PostgreSQL activity had zero lock waiters and zero blocked backends; application database deadlock count was zero. Host IO and memory pressure were near zero, and service memory events showed no OOMs. These are point-in-time checks, not proof that contention never occurs. Cron storage initialization already uses `ensure_cloud_schema_once` with a ready flag; normal calls do not rerun its DDL.

The `pushes` successful domestic-user path has at least seven fresh connections: four in auth/access, then legacy-presence check, list and unread count. If there is no legacy-prefixed row, its legacy backfill also loads session history before attempting an upsert. An empty backfill writes no completion marker, so users with no legacy items can repeat that history scan. This is an additional endpoint-specific cost, not the shared cause of all slow APIs. Bootstrap likewise loads all session messages before selecting the response history page.

### Local synthetic SCRAM optimization A/B

Created a standalone temporary benchmark outside the repository, using no network and only a synthetic password/salt. Seeded its Cargo.lock from deployed revision `9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d`, built offline, and confirmed every resolved dependency name/version appeared in the production lockfile. Actual production role authentication uses 4096 SCRAM iterations; the benchmark uses the same count.

Rust 1.97.1 on the local macOS host; three alternating debug/release runs, each with two warmups and 20 retained samples:

| Profile | Samples | Median ms | Range ms |
| --- | ---: | ---: | ---: |
| dev, opt-level 0, debug=1 | 60 | 22.19596 | 21.33304–23.57508 |
| release, optimized, debug=1 | 60 | 0.42688 | 0.41088–0.69150 |

Median ratio: 52.0x for this isolated computation on this host. This is **not** a production API speedup measurement; CPU architecture, runtime workload, full connection setup and SQL are different. It strengthens the optimization mechanism rather than predicting end-to-end gains.

Reproduction: create a standalone Cargo package depending on `postgres-protocol = "=0.6.11"`; set debug=1 and incremental=false for dev and release; seed Cargo.lock with `git show <production-revision>:Cargo.lock`; build both profiles with `cargo build --offline`; alternate runs of the two binaries. Core synthetic benchmark:

```rust
use postgres_protocol::authentication::sasl::{ChannelBinding, ScramSha256};
use std::{hint::black_box, time::Instant};

fn main() {
    let mut times = Vec::new();
    for _ in 0..22 {
        let mut auth = ScramSha256::new(
            b"synthetic-diagnostic-password", ChannelBinding::unrequested());
        let first = std::str::from_utf8(auth.message()).unwrap();
        let nonce = first.split_once("r=").unwrap().1;
        let msg = format!("r={}servernonce,s=c3ludGhldGljc2FsdA==,i=4096", nonce);
        let start = Instant::now();
        auth.update(black_box(msg.as_bytes())).unwrap();
        black_box(auth.message());
        times.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("{:?}", &times[2..]);
}
```

## Diagnostic-stage Risks / Follow-ups (rollout results appended below)

- Do not call the whole two seconds network latency or claim an exact backend/network percentage from Caddy logs.
- The full authenticated path was observed in browser and logs, not directly benchmarked with copied credentials on loopback.
- No optimized binary or connection-reuse change was deployed; expected improvement remains unmeasured.
- Do not globally replace fresh connections in transaction/advisory-lock/test-isolation paths. Review ownership and concurrency semantics when introducing pooling or reuse.
- Preserve SCRAM authentication; reducing password-derivation strength is not the proposed remedy.
- Production and local working-tree revisions differ. Any remediation must deliberately select its deployment revision and preserve other users' uncommitted changes.

Cloudflare official documentation checked on 2026-09-20:

- https://developers.cloudflare.com/smart-shield/configuration/argo/ — Argo optimizes origin/network transit, not application CPU or SQL execution.
- https://developers.cloudflare.com/smart-shield/get-started/ — Smart Shield + Argo is available for purchase on Free/Pro/Business; upgrading the site plan alone is not equivalent to enabling this capability. Traffic must use the applicable proxied path.

Do not buy a larger plan as the first remediation. After reducing origin latency, evaluate Argo with p50/p95 before/after data and confirm it covers the actual Worker's origin subrequest. Existing earlier observations indicated an sslip.io origin target, so main-zone enablement alone must not be assumed sufficient; current Worker configuration and account entitlements were not re-read here.

## Next Entry Point

Production is now running the isolated optimization revision, with rollout evidence appended below. Preserve the change in future main integration, observe real-load queueing, and measure residual browser/network latency before changing Cloudflare products. The earlier diagnostic and local implementation stages remain historical evidence.

## Implementation Stage — 2026-09-20

User authorization: “基于这些问题优化看看”; additional acceptance requirement: “重点观察不要导致线程串掉、数据错乱这些问题，这些是底线”. Local code and tests are complete; no commit, push, release, deployment or Cloudflare purchase was performed.

### Changes and ownership boundaries

- Private ordinary `CloudPgRuntime::connect_client` calls now borrow exclusive connections from a bounded pool of four, keyed by the existing resolved database configuration and test namespace. Call-site audit found no nested acquisition while holding a lease. Actor predicates, SQL parameters, authentication checks and session semantics remain unchanged.
- Transactions, migrations, session advisory locks and existing cached EventStore connections retain their dedicated ownership. The pool never borrows these clients. Connection-local `pg_temp` fixtures remain pinned; named-schema memory tests exercise the production pooling path.
- Every ordinary query tracks completion. SQL/transport errors, dropped query futures and panic discard the connection and abort its driver instead of returning it to the pool. Failed operations are never replayed: an autocommit write may already have succeeded before an error or cancellation becomes visible. Closing transport is not a guarantee that a previously submitted write was rolled back.
- `source-runtime` adds `opt-level=3`, retaining dev's overflow checks/debug assertions, `debug=1`, `incremental=false` and the existing target/provenance path. No external dependencies were added. Existing unrelated worktree changes, including the OSS body-limit change in the same source file, were preserved.

### Safety and regression verification

Development PG settings: `HONE_POSTGRES_HOST=127.0.0.1`, `HONE_POSTGRES_PORT=5433`, `HONE_POSTGRES_USER=honeclaw`, `HONE_POSTGRES_PASSWORD=honeclaw_dev`, `HONE_POSTGRES_DATABASE=honeclaw`. Tests used the existing local PostgreSQL with isolated schemas; no production database was used.

| Verification | Result |
| --- | --- |
| `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app --offline` | Passed |
| `cargo test -p hone-core --lib --profile source-runtime cloud_runtime::query_pool --offline` | 8 passed, manual benchmark ignored; optimized executable |
| Explicit memory PostgreSQL tests (`cargo test -p hone-memory --lib -- --ignored`; executed the already-built test binary) | 93 passed, including login/token revocation, permissions, session storage, billing and cron |
| Full workspace all-target tests, `--no-fail-fast` | 2842 passed, 3 existing failures, 114 ignored; subsequent optimized run additionally covers the new panic regression |
| `bun run test:web` | 552 passed |
| Edge Worker `bun run typecheck && bun run test` | Typecheck passed; 45 tests passed; locked development dependencies installed with scripts disabled |
| CI-safe scripts, continuing after finance-static failure | 21/22 scripts passed, including modified runtime image contract and source deployment contracts |
| Changed-file format check, explicit rustfmt on all three pool/core files, `git diff --check` | Passed |

The eight pool safety regressions cover: exclusive lease bounds; 32 concurrent cold requests preserving response IDs; 48 actor/namespace combinations with repeated writes and reads; SQL errors and query cancellation; connection failure and terminated idle backend recovery; transaction cancellation/rollback without absorbing unrelated writes; advisory-lock exclusivity and `pg_temp` locality; panic after a committed write without replay or connection reuse.

Unrelated failures are **not** reported as green: full workspace failures are `config::tests::soul_prompt_keeps_the_full_investment_contract` and the two Agent tests `deferred_prefix_ignores_structurally_invalid_datafetch_activation` / `first_batch_identity_route_limit_executes_only_six_valid_routes`. Their HEAD reproductions are recorded in the same-day research-report handoff. Finance-static currently fails 10 assertions: the 9 HEAD failures documented there, plus assertion 41, whose expected literal `if finance_round_is_read_only && !finance_round_is_known_read_only` was changed in the pre-existing Agent worktree diff. This task did not edit Agent/finance files or relax those checks.

### Actual connection/query benchmark

Independent temporary PostgreSQL 16 on loopback port 55439, initialized with host `scram-sha-256` authentication (4096 iterations), synthetic benchmark password, no business data. Server automatically stopped after the run. On this macOS ARM64 host, each strategy ran 12 batches of 8 identical sequential parameterized SELECTs; pooled samples are warm. Both paths include the test namespace preparation appropriate to fresh connections. No timing threshold is a CI gate.

| Build | Strategy | Median per 8 queries | Min–max | Distinct backends across 96 queries |
| --- | --- | ---: | ---: | ---: |
| dev, opt-level 0 | Fresh per query | 314.097 ms | 281.044–361.442 ms | 96 |
| dev, opt-level 0 | Pooled | 2.224 ms | 1.650–3.202 ms | 1 |
| source-runtime, opt-level 3 | Fresh per query | 39.217 ms | 37.819–42.111 ms | 96 |
| source-runtime, opt-level 3 | Pooled | 2.545 ms | 1.403–3.504 ms | 1 |

Reproduce against a local test database configured for SCRAM, after exporting its `HONE_POSTGRES_*` settings:

```bash
cargo test -p hone-core --lib pool_benchmark_eight_sequential_queries --offline -- --ignored --nocapture
cargo test -p hone-core --lib --profile source-runtime pool_benchmark_eight_sequential_queries --offline -- --ignored --nocapture
```

This measures connection/query cost, **not** authenticated HTTP latency or production p95. The warm pool removes authentication from repeated ordinary queries; optimized code also reduces fresh-connection CPU work. The sub-millisecond differences between warm-pool profiles are measurement noise, not evidence that optimized queries regress. Cold-start and larger-load production gains remain unmeasured.

### Rollout / rollback follow-up

- Apply only the reviewed optimization to an isolated revision compatible with production `9a2f27c…`; do not deploy the current mixed worktree. Build through the existing revision-bound source-runtime process and retain the previous artifact for rollback.
- On the candidate, exercise multiple authenticated accounts/conversations concurrently and verify returned actor/session IDs and saved records, plus logout/revocation and transaction/cancellation behavior. Then compare `/me`, `/research-overview`, `/bootstrap`, `/pushes?limit=1` p50/p95 and CPU with matched origin/browser measurements.
- Four ordinary leases are a conservative starting bound, not a total-process PostgreSQL connection limit: dedicated/cached connections are additional. Observe queueing and long-running queries before tuning capacity. No automatic write retry may be added as a performance shortcut.
- No schema/data migration is introduced. Rollback selects the prior revision artifact; do not weaken authentication or actor filtering. Cloudflare acceleration remains a subsequent decision based on residual network delay and verified route coverage.


## Production Rollout and Acceptance — 2026-09-20

- status: done; deployed and measured
- User authorization: “发上线测一下看看”. No formal version or release tag was requested or created.
- Cutover: **2026-09-20 08:27:16 UTC / 16:27:16 Asia/Shanghai**. Accepted exact runtime revision: `3e26eb4fe574ff9aae94ddb2b21732c9f8ede416`, based on previous live revision `9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d`.
- Source: isolated branch `codex/api-query-pool-performance-20260920`. The shared dirty checkout was not deployed; unrelated edits, including the OSS body-limit patch in the same source file, were excluded. Remote main was not advanced by this rollout. Future main-based deployment must include this optimization to preserve the gains.
- [Immutable Linux build](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/35498221222); [candidate bundle export](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/35499109600). Image digest: `sha256:fd4c2eaa6267e26e3a8a9d2488accfe070aa649411cacac406e201b839de8bbe`; archive SHA-256: `f9314169d0077640be70cd684880beea62ce86f2f19fc1450dcde40f763a3963`; live CLI SHA-256: `c5dfba90fb7eabbfa3f0515af32faf26d8a3db9a0d39eb29d20887e715818d46`.
- Runtime Image export support is in `6801d5d3`, with per-revision export concurrency in `516c0543`. These operations-only commits did not change the deployed runtime code. The export job uses job-scoped `packages: read`, checks an exact revision and immutable digest, verifies the bundle, and produces a checksummed artifact. No operator-token scope was expanded and no broad token was installed on production.
- `source-runtime` is compiled with `opt-level=3`, confirmed by the build log. `/api/meta.build.profile` still says `debug` because that field derives from retained debug assertions; it does not mean this executable is unoptimized.

### Candidate verification

The isolated candidate passed its eight optimized PostgreSQL pool safety regressions and 93 explicit PostgreSQL memory regressions. Full workspace tests produced **2874 passes, 3 failures, 115 ignored**. The three failures are the pre-existing Agent mocks `deferred_prefix_ignores_structurally_invalid_datafetch_activation` and `first_batch_identity_route_limit_executes_only_six_valid_routes`, plus `config::tests::config_example_avoids_stale_config_knobs` (the unchanged production README_EN is missing literal `llm.providers.openrouter.api_key/api_keys`). This differs from the older dirty-checkout failure list above; neither run is represented as wholly green. Runtime image and source deployment shell contracts passed. No frontend or Worker product code was changed by the candidate.

### Successful authenticated origin requests

These are **Caddy origin durations for HTTP 200**, observed while the existing signed-in browser loaded chat/research pages and polled pushes. No valid Cookie was exported or replayed. Pre-rollout sampling ended 08:18:27 UTC; post-rollout samples were strictly filtered to timestamps at or after cutover and ended 08:36:20 UTC. Small samples and changing background work limit statistical claims; this is not a sustained load test or browser end-to-end p95.

| Endpoint | Before: median ms (n) | After: median ms (n) | After min–max ms |
| --- | ---: | ---: | ---: |
| `/api/public/auth/me` | 939.667 (1) | 12.777 (3) | 12.711–13.599 |
| `/api/public/research-overview` | 1636.940 (1; earlier 06:29:57 diagnostic sample) | 39.676 (3) | 26.170–57.588 |
| `/api/public/bootstrap` | 2993.413 (3) | 36.285 (4) | 23.686–112.295 |
| `/api/public/pushes` | 1461.364 (7) | 15.571 (11) | 9.741–51.196 |

The before overview sample is explicitly earlier because no successful overview request occurred in the immediate pre-rollout window. Do not turn this table into a uniform paired A/B percentage or a pure network subtraction. The broad origin reduction confirms that repeated connection/authentication and unoptimized CPU work were major shared contributors. Cloudflare configuration and subscription were unchanged.

### Controlled common-auth-path check

Python urllib on production loopback, same five paths, ten sequential invalid-cookie requests per path; all returned JSON HTTP 401. Fake session values are deliberately absent and non-secret. This exercises only the common absent-session lookup, not successful business handlers. Ten no-cookie checks per path and forty requests at four-way concurrency also returned 401 (140 checks total per deployment).

| Endpoint | Before median ms | After median ms |
| --- | ---: | ---: |
| auth/me | 153.731 | 2.785 |
| research-overview | 153.874 | 3.061 |
| bootstrap | 157.538 | 2.602 |
| pushes?limit=1 | 154.674 | 3.435 |
| history | 139.746 | 2.535 |

Four-way concurrency, forty invalid-session requests: median **445.024 → 8.044 ms**, empirical p95 **680.820 → 14.664 ms**, max **720.014 → 64.859 ms**. The p95 uses the sorted element at `floor((n-1)*0.95)`. No-cookie medians after rollout were 1.373–1.858 ms. Timing is diagnostic evidence, not a CI gate.

### Residual public-path latency

A separate anonymous `auth/me` experiment used six sequential requests in one curl process over HTTP/2, without cookies. All returned the expected 401. First request (new connection): total **1143.358 ms**, cumulative TLS completion 625.201 ms. Five connection-reusing requests: total **200.662–202.645 ms**, median **201.088 ms** (`num_connects=0`). The same no-cookie origin path was approximately 1–2 ms locally. This demonstrates substantial residual public-path/client-connection overhead after the backend fix; it does not isolate Cloudflare, each network hop, or mainland no-proxy user experience. It is an anonymous control, not the authenticated browser's end-to-end measurement. The public chat page also returned 200. Do not claim that users' whole requests now take only the origin table's 13–40 ms.

### Correctness, health and cutover impact

- Before switching, three reads reported zero active chats. The independent cutover task verified all bundle checksums and environment presence, switched the release symlink atomically and restarted the managed web service. Failed revision/storage acceptance would have restored the prior symlinks and restarted the previous artifact.
- After rollout, the live revision and executable path matched the intended bundle. PostgreSQL/object storage remained authoritative and healthy, with zero local durable dependencies. Final 08:40:27 UTC inspection: service active/running, `NRestarts=0`, exit status 0, active chats 0.
- The existing browser's **13 message IDs, order and rendered contents matched exactly** before/after restart and again after visiting research and returning to chat. All 13 IDs were distinct; captured browser errors after cutover: 0. This is one live account/read-path check. Cross-actor/session writes, cancellation, rollback, advisory locks and connection failure behavior are covered by the isolated PostgreSQL regressions, not claimed as a live multi-account canary.
- PostgreSQL sample: eight idle clients, one active inspection, five background processes, **no idle-in-transaction**. The pool's four-lease bound applies to ordinary clients, not the additional dedicated/cached connections.
- The origin recorded **two HTTP 502 responses at 08:27:17 UTC**, both during the service restart. Through final health inspection there were no subsequent public API 5xx responses; 59 successful public API responses were recorded. This was a brief restart interruption, not zero-downtime deployment.
- Nine sampled application ERROR entries after rollout were all scheduler-class messages with normalized shapes already present during the prior 24 hours. No sampled PostgreSQL/auth errors or panic. Classification and historical comparison ran on the server and exported counts only; no raw error rows were retained in this handoff. Existing scheduler issues remain outside this rollout.
- There were no enabled/active standalone channel worker units before rollout; none was added or enabled. Configuration, database schema, user data and authentication strength were unchanged by the deployment; browser reads retain their normal application last-seen behavior.

### Retention and rollback

`/opt/hone/previous` points to the verified `9a2f27c7c37f694ae3ac63ca1c5f7db71f0cd62d-ghcr-runtime` artifact. Two earlier bundles (`17a25de6…`, `161fe4b3…`) were also retained. Rollback requires the normal active-chat check, atomically selecting the previous release, restarting the web service, then verifying `/api/meta`, storage health and the executable path. No database migration or rollback is needed.

Two extra inactive artifacts (`a50f9186…`, `93818046…`) were pruned only after archive checksums and bundle manifests matched downloaded recoverable copies, current/previous guards passed, and no process executable used either directory. [First backup export](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/35498891694), [second backup export](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/35499819831). Actions exports have seven-day retention; the pinned registry images remain the rebuild-free recovery source. Root disk free space finished at **2.9 GiB**; temporary uploaded archives and the completed cutover task were cleaned. Disk capacity remains modest and should be checked before another artifact is staged.

Follow-up: include `3e26eb4f` when integrating the published branch into main; evaluate residual browser/network delay with comparable authenticated samples before purchasing Cloudflare acceleration. A short low-volume acceptance run cannot establish long-term high-load queue behavior or prove the absence of every possible isolation bug.
