# Cloud Runtime Impact Report 2026-05-28

- title: Cloud Runtime Impact Report 2026-05-28
- status: done
- created_at: 2026-05-28
- updated_at: 2026-05-28
- owner: Codex
- related_files:
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-web-api/src/lib.rs`
  - `crates/hone-web-api/src/cloud_oss.rs`
  - `crates/hone-web-api/src/routes/meta.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `memory/src/session.rs`
  - `memory/src/quota.rs`
  - `bins/hone-cli/src/start.rs`
  - `config.example.yaml`
  - `.env` (local only, ignored; keys not copied into this report)
- related_docs:
  - `docs/current-plan.md`
  - `docs/current-plans/cloud-pg-oss-runtime-migration.md`
  - `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`
- related_prs:
  - none

## Summary

当前 `main` 的云化状态是“配置与部分 OSS 文件路径已接入”，不是“业务状态已完全迁到 PG / OSS”。本机 `/api/meta` 返回 `desktop-v1`，并暴露 `cloud_runtime`、`cloud_postgres`、`cloud_oss`、`oss_file_proxy` capabilities；但该接口当前没有 `runtime_role`、`cloud_storage_authoritative`、`local_durable_dependency_count` 字段，也没有执行 PG schema、PG query、OSS authenticated object operation 的 live health check。

如果现在在多台机器上各部署一份 app，不能安全得到分布式扩展效果。多副本会共享一部分远端配置和 public OSS 文件路径，但 sessions、web auth、quota、portfolio、cron、audit、notification prefs、skill registry/company profile 等业务持久状态仍主要在本机 JSON / SQLite / 目录里。scheduler、event engine、channel listener、iMessage 等也没有 runtime role 或 PG lease 保护，直接多开会产生重复执行和状态分叉。

## Verification

测试与探针在 2026-05-28 于 `/Users/ecohnoch/Desktop/honeclaw` 执行，起始 HEAD 为 `09b7d8bb`，分支 `main`。

| Check | Result | Time | Notes |
| --- | --- | ---: | --- |
| `cargo check --workspace --all-targets --exclude hone-desktop` | pass | 160.02s initial, 5.13s incremental after fmt | workspace 编译检查通过 |
| `cargo test --workspace --all-targets --exclude hone-desktop` | fail | 283.02s | `hone-integrations` 的 6 个 `feishu_facade` mock HTTP tests 失败 |
| `cargo test -p hone-integrations feishu_facade -- --nocapture --test-threads=1` | fail | 28.88s | 串行复现同一类 `error sending request for url (http://127.0.0.1:.../)` transport failure |
| `bun run test:web` | pass | 0.84s | 185 pass, 0 fail |
| `bash tests/regression/run_ci.sh` | pass | 191.09s | CI-safe regression scripts passed |
| `cargo fmt --all -- --check && git diff --check` | pass | 0.94s | 本轮修复了一个 `rustfmt` 机械差异 |
| `curl http://127.0.0.1:8077/api/meta` | pass | under 5s | 返回 `api_version=desktop-v1` 和 cloud capabilities；缺少 distributed/runtime-authoritative fields |
| PG direct TCP | fail | 8665.8ms | direct connect to configured RDS endpoint timed out |
| PG via SOCKS5 proxy | pass | 156.3ms | SOCKS5 CONNECT to configured RDS endpoint succeeded |
| OSS bucket direct HTTPS HEAD | fail | 10041.9ms | direct bucket endpoint timed out |
| OSS bucket via SOCKS5 HTTPS HEAD | pass | 1765.3ms | reached bucket and got `HTTP/1.1 403 Forbidden`, which proves network reachability without exposing credentials |

The Feishu facade failures are currently a test/runtime transport blocker, not evidence from this run that cloud storage paths are broken. The failing assertions never reached the intended JSON / HTTP error parsing path because the local mock request failed at transport send time.

## Current Cloud Surface

`cloud.postgres` and `cloud.oss` are first-class config sections with env fallback. `.env` contains PG / OSS variable names, including proxy variables, but the code currently consumes only the base PG / OSS fields. `HONE_POSTGRES_PROXY` and `HONE_OSS_PROXY` are present in the local manifest but are not implemented in the runtime clients.

OSS is partially active for public uploads and file/image proxy reads:

- `crates/hone-web-api/src/routes/public.rs` can store public uploads in OSS when OSS config is present.
- `crates/hone-web-api/src/routes/files.rs` can resolve managed `oss://...` URIs.
- `crates/hone-web-api/src/cloud_oss.rs` creates a plain `reqwest::Client::new()` and does not apply proxy, pooling, timeout, retry, or signed URL policy beyond the current direct client behavior.

PG is currently configuration-only for the main business state. There is no PG-backed repository layer, no schema migration table, no runtime lease table, and no automatic `CREATE TABLE IF NOT EXISTS` path for sessions/auth/quota/cron/audit/portfolio/prefs/company profile metadata.

## Local Durable Dependencies

The remaining durable local dependencies visible in code are:

- Session JSON / SQLite storage via `memory/src/session.rs`.
- Web invite users and auth sessions via `WebAuthStorage::new(&core.config.storage.session_sqlite_db_path)`.
- Conversation quota JSON files via `memory/src/quota.rs`.
- Portfolio files under configured portfolio dir.
- Cron job JSON plus local execution history.
- LLM audit SQLite.
- Notification preference JSON files.
- Runtime skill registry override JSON.
- Generated images and long-lived local file proxy sources outside the partial OSS public upload path.
- Company profile documents / event docs that still rely on local actor sandbox or local managed files.
- Runtime logs, locks, cache, and sandbox directories. These are acceptable local runtime artifacts, but should not be counted as business truth once strict cloud mode is real.

`cloud.strict_no_local_storage=true` is correctly dangerous today: startup code is designed to fail while local dependency reporting still finds durable local stores. That guard should remain off until PG / OSS authoritative repositories are actually implemented.

## Distributed Impact

Current multi-machine deployment is not safe for active-active business state:

| Area | Current behavior | Multi-machine risk |
| --- | --- | --- |
| Web/API replicas | Every instance builds a full `HoneBotCore` with local stores | User/session/history state can diverge per host |
| Web auth | SQLite path from local storage config | Login sessions and invite state are host-local |
| Chat sessions | JSON / SQLite local storage plus process-local locks | Concurrent runs can fork or overwrite session state |
| Quota | JSON files plus process-local locks | Multiple hosts can reserve quota independently and overrun limits |
| Cron | Local cron stores and local scheduler startup | Duplicate job execution and inconsistent run history |
| Event engine | Starts from web-api runtime when enabled | Duplicate notifications and independent delivery state |
| Channel listeners | CLI start enables sidecars by config, not role | Duplicate ingress/outbound handling |
| iMessage | Local privileged integration | Must remain single-host leader until explicitly redesigned |
| Portfolio / audit / prefs | Local JSON / SQLite | User-visible state and compliance/audit trail split by host |
| OSS public uploads | Partially remote | Durable only for code paths that already return `oss://...`; proxy env is not honored |

The intended topology should still be `web` replicas plus a single `worker` leader, but the code does not yet expose `HONE_RUNTIME_ROLE=web|worker|all` or PG-backed `distributed_leases`. Without those, every app process is effectively `all`.

## Latency Analysis

The largest current latency risk is network path mismatch. The local environment can reach PG and OSS through SOCKS5, but direct PG and direct OSS probes time out. Since the runtime clients do not consume proxy env variables, any future cloud code path that connects directly from this machine will likely pay 8-10 second timeout costs or fail outright.

Approximate measured baselines:

- PG direct TCP: unavailable within 8.7s.
- PG SOCKS5 CONNECT: 156ms to establish a tunnel to RDS.
- OSS direct HTTPS HEAD: unavailable within 10.0s.
- OSS SOCKS5 HTTPS HEAD: 1.77s to TLS + HEAD, returning unauthenticated `403`.
- `cargo check` full workspace: 160s cold-ish, 5s incremental.
- CI-safe regression suite: 191s.
- Frontend tests: under 1s.

Expected application-level latency after a real cloud cutover:

- PG reads/writes should use pooled connections; opening a new tunneled connection per request would add noticeable tail latency.
- Quota/session/cron locks should use short PG transactions with row locks or leases. This adds network round trips but removes local split-brain risk.
- OSS upload/download should reuse a configured reqwest client with proxy, timeouts, and retry/backoff. Creating a new client per request increases TLS/proxy setup cost.
- `/api/meta` should stay fast, but it should separate static capabilities from live health. Live PG/OSS checks should be bounded by tight timeouts and cached status.

## Required Changes Before Multi-Machine Scale-out

1. Add `HONE_RUNTIME_ROLE=web|worker|all` and gate scheduler, event engine, channel listener, and iMessage startup by role.
2. Add PG schema migrations and repository backends for sessions, web auth, quota, portfolio, cron, audit, notification prefs, skill registry overrides, and company profile metadata.
3. Add `distributed_leases` / runtime locks for worker leadership, session run ownership, channel listener ownership, and cron job claiming.
4. Implement PG proxy support through a local tunnel strategy and OSS `reqwest::Proxy` support for `HONE_OSS_PROXY`.
5. Move durable files, generated artifacts, attachments, and company profile documents to OSS; keep only temp sandbox/cache/log/lock locally.
6. Add `hone-cli cloud doctor` and make `/api/meta` report `runtime_role`, `cloud_storage_authoritative`, `local_durable_dependency_count`, and bounded live health.
7. Add idempotent `hone-cli cloud migrate --from-data-dir ./data --upload-oss` before enabling strict mode.
8. Keep iMessage as a single-worker integration with explicit leader ownership.

## Risks / Follow-ups

- The workspace Rust test gate is still red because of `hone-integrations` Feishu facade mock transport failures. Fix or quarantine that test issue before treating a full Rust test run as a green release signal.
- Cloud capabilities in `/api/meta` can currently be misread as proof of authoritative PG/OSS ownership. They should be renamed or supplemented with live health and dependency counts.
- The local `.env` contains proxy variables that the runtime does not yet honor. This is the main reason cloud runtime latency can look acceptable in manual proxy probes but fail in app code.
- `docs/current-plans/cloud-pg-oss-runtime-migration.md` remains active; this report is an impact snapshot, not completion of the migration.

## Next Entry Point

Start with runtime role and lease boundaries before moving more stores: role gating prevents duplicate side effects, and PG leases give a safe foundation for session, cron, and channel ownership. Then implement PG repositories and OSS proxy support behind the existing config contract, followed by a live `cloud doctor` regression.
