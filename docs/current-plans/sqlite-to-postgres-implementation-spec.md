# honeclaw SQLite → PostgreSQL — Definitive Implementation Spec

**Written against `114a6c8d` (2026-08-16), verified read-only. Every instruction cites `file:line` at that revision.**
Codex CLI: re-run `git rev-parse HEAD` before starting. If it is not `114a6c8d`, re-verify §1 before trusting anything below.

---

## 1. Corrected facts

These override the task prompt, the six discovery reports, `docs/current-plans/sqlite-to-postgres-migration-2026-08-16.md`, and any verifier claim they contradict. **C-1 through C-4 are the ones that change what work remains.**

### C-1 (BLOCKING) — The tree moved. Phases 0 and 1 are already shipped.

The discovery reports were produced against `59c38cec`. HEAD is now `114a6c8d`. Five commits landed in between:

| Commit | Effect on this plan |
|---|---|
| `62d0c889` | Fixed the per-call `Runtime::new()` CPU burn **for cron only** (see C-4) |
| `340e51b9` | Pinned the cron duration timestamp parameter to text |
| `cf713e74` | Added the zero-residue requirement to the plan doc |
| `9ba2f7d7` | **Phase 0 DONE** — `docker-compose.dev.yml`, `scripts/dev_pg.sh`, `.env.example`, `docs/runbooks/local-postgres-development.md` all exist and were verified live |
| `18ff42c2` | **Phase 1 DONE** — `cloud_web_user_external_state` table, Cloud read/write path, SQLite importer, ignored live-PG regression test |
| `114a6c8d` | Local data migration executed and reconciled (`docs/handoffs/2026-08-16-local-sqlite-to-pg-data-migration.md`) |

Phase 0 artifacts verified present on disk: `docker-compose.dev.yml`, `scripts/dev_pg.sh`, `docs/runbooks/local-postgres-development.md`.
The plan doc's "25 tables" is wrong; `9ba2f7d7`'s commit message records the measured value as **26**, and the doctor field is `postgres_health.ok`, not `pg.ok`.

**Consequence: do not implement §3 of the plan doc. Section 2 below is a residue-closure task, not an implementation task.**

### C-2 — "`web_user_external_state` has no PG counterpart" was never true, and is now doubly false.

Before `18ff42c2`, the Cloud backend already persisted external state as JSON embedded in `cloud_web_invite_users.record` via `CloudWebInviteRecord.external_state` (`memory/src/web_auth.rs:157-158`). The `LEFT JOIN` the plan doc cites as evidence of a gap (`memory/src/web_auth.rs:2091`) is the **SQLite→Cloud exporter** (`export_cloud_records`, `memory/src/web_auth.rs:2070`), not a cloud read path.

The real defects were (a) no uniqueness enforcement on email in cloud mode, and (b) an O(n) full-table scan on every email login. Both are now fixed by the dedicated table at `crates/hone-core/src/cloud_runtime.rs:1432-1450` with `idx_cloud_web_user_external_email`.

### C-3 — GCE "zero sqlite files on disk" cannot be true for the event store under `role=all`.

`crates/hone-channels/src/core/bot_core.rs:113-125` opens `EventStore::open(configured_event_store_path(&config))` **unconditionally** — there is no `is_cloud_authoritative()` branch around it. `EventStore::open` does `create_dir_all` + `Connection::open` + `CREATE TABLE IF NOT EXISTS` (`crates/hone-event-engine/src/store.rs:106-192`), i.e. it always materialises the file. The path resolves to `$HONE_DATA_DIR/events.sqlite3` or `<sessions_dir>/../events.sqlite3` (`crates/hone-channels/src/core/bot_core.rs:889-897`). `crates/hone-web-api/src/lib.rs:947-950` deliberately opens the same path, with a comment saying all consumers must share one file.

The plan doc's "zero sqlite3 files under `/srv/honeclaw/data/`" was measured when GCE ran `role=web`. Since 2026-08-15 GCE runs `role=all`. **This must be re-measured on the host before Phase 2 cutover** (see §7, OQ-1). It does not block Phase 2 implementation.

### C-4 (NEW — missed by every discovery pass and every verifier) — the CPU-burn bridge bug is live in **seven** modules, not one.

`62d0c889` fixed only `memory/src/cron_job/mod.rs`. Every other sync→async bridge still does per-call `tokio::runtime::Runtime::new()`:

| Module | Lines |
|---|---|
| `memory/src/web_auth.rs` | `2198`, `2205` |
| `memory/src/session.rs` | `242`, `249` |
| `memory/src/billing.rs` | `1034`, `1041` |
| `memory/src/llm_audit.rs` | `634`, `641` |
| `memory/src/quota.rs` | `315`, `321` |
| `memory/src/portfolio.rs` | `506`, `513` |
| `memory/src/company_profile/storage.rs` | `1278`, `1284` |

`memory/src/web_auth.rs:2190-2207` is the exact anti-pattern `62d0c889`'s comment (`memory/src/cron_job/mod.rs:47-58`) documents as having burned 47 CPU-minutes in 26 wall-minutes on a 2-vCPU GCE box. Phase 1 made `web_auth` hotter, not cooler: every `load_external_state` / `save_external_state` now round-trips through it (`memory/src/web_auth.rs:400`, `:430`).

Today these are cloud-mode-only. **After Phase 3 they are the only path, on every box including the developer Mac.** Extracting the shared bridge is therefore not an optional tidy-up — it is a Phase-3 prerequisite. See §4.0.

### C-5 — `memory/` modules are not uniformly "dual-backend".

- `memory/src/session.rs:99` — `SessionRuntimeBackend { Json, Sqlite, CloudPg }` — **three** arms.
- `memory/src/cron_job/mod.rs:78`, `:88`, `:101` — `new()` (JSON dir), `with_sqlite()`, `new_cloud()` — **three** constructors. `bins/hone-cli/src/main.rs:246-248` still reaches the JSON-only arm when `session_sqlite_db_path` is empty.
- `memory/src/web_auth.rs:145` and `memory/src/billing.rs:114` are genuinely two-arm.
- `memory/src/llm_audit.rs` is two-arm via `Option<CloudPgRuntime>`.

Deleting "the Sqlite variant" leaves session and cron with a **JSON** arm that also has to go, or the fallback survives silently.

### C-6 — Verifier corrections to the store-schema proposal that WIN (carried into §3)

All independently re-verified at `114a6c8d`:

| Claim | Correction | Evidence |
|---|---|---|
| json_extract sites are `:665, :820, :1632` | Six sites: `:665, :820, :828, :1141, :1225, :1632`. `:1141` (per-firm analyst cooldown) and `:1225` (TheFly fan-out guard) were omitted; PG has no `json_extract()` so both fail at PREPARE, not at runtime | `crates/hone-event-engine/src/store.rs:1141`, `:1225` |
| `SELECT EXISTS` not mentioned | Two sites bound to `i64`: `:789` (`contains_event`) and `:812` (`actor_has_delivered_earnings_for_document`). PG returns BOOLEAN; `row.get::<_,i64>` errors on **every** call. The repo's own PG code reads it as `bool` at `crates/hone-core/src/cloud_runtime.rs:4298-4304` | `crates/hone-event-engine/src/store.rs:789`, `:812` |
| `attempts: u32` needs fixing at `:400`, `:435` | **Four** sites. `:460` and `:487` were omitted — those run *after* the LLM spend; leaving them `u32` means the job is never marked completed/retry and is re-claimed and re-researched forever. Public field `EarningsContinuityJob.attempts: u32` at `:102` changes too | `crates/hone-event-engine/src/store.rs:460`, `:487`, `:102` |
| `TransactionBehavior::Immediate` appears once (`:380`) | **Three** sites: `:380`, `:615`, `:1855` | verified by grep |
| LIKE→ILIKE list omits `:1013`, `:2040`, `:2044` | Full inventory: `819, 860, 892, 893, 936, 1012, 1013, 1039, 1069, 1138, 1153, 1189, 1222, 1223, 1260, 1261, 1627, 1629, 1630, 1631, 2040, 2044` | verified by grep |
| Index `(actor, channel, status, sent_at_ts)` covers all hot reads | It cannot: `list_actors_with_quiet_held_since` (`:1421-1427`) and `broadcasted_event_ids_since` (`:1704-1710`) have **no** `actor` predicate; `delivered_event_ids_since` (`:1397-1401`) has no `channel`. Needs two more indexes | see §3.2 DDL |
| Three statements build dynamic `?` SQL | Correct and load-bearing: `:1069-1089`, `:1138-1168`, `:2014-2068`. All three need `$N` renumbering with a running counter, not find-and-replace | verified |
| `?4` reuse | Confirmed at `:1225` and `:1226` — one value, two placeholders. Sequential renumbering produces 5 placeholders for 4 params | `crates/hone-event-engine/src/store.rs:1225-1226` |
| SQLite non-INTEGER PRIMARY KEY does not imply NOT NULL | Correct. `events.id` (`:117`), `engine_meta.key` (`:135`), `earnings_continuity_jobs.job_key` (`:173`) are today NULL-able. Pre-flight null checks required (§3.5 step 2) | verified |
| `$.`-prefix bug at `:828` makes the guard "always false" | Overstated — `:828` is one arm of an OR whose other arm (`:822-827`, URL canonicalisation) still works. Degrades to URL-match-only. Also: `key_path` is a Rust-side bind computed at `:809`, so the fix is a Rust edit | `crates/hone-event-engine/src/store.rs:809`, `:821-829` |
| `ON CONFLICT (actor, source_id)` risks aborting the delivery_log row | Recommendation (bare `ON CONFLICT DO NOTHING`) is right, justification is wrong — under `RETURNING id` the `delivery_log_id` is unique by construction. Real reason: SQLite `INSERT OR IGNORE` swallows NOT NULL/CHECK/datatype failures too; a targeted clause narrows it for no benefit | `crates/hone-event-engine/src/store.rs:1772-1788` |

### C-7 — Verifier corrections to the deletion-safety proposal that WIN

Re-verified at `114a6c8d`:

- **`is_cloud_authoritative()` has exactly 22 call sites + 1 definition** (`crates/hone-core/src/config/server.rs:624`). Definitive list in §5.3. The plan doc's "22" is right; the discovery pass's enumeration was not.
- **Five further cloud-mode branches do not call it**: `crates/hone-web-api/src/routes/meta.rs:146`, `crates/hone-web-api/src/routes/public.rs:166`, `crates/hone-web-api/src/routes/company_ratings.rs:442` (string `as_str()` form); `crates/hone-web-api/src/lib.rs:1137`, `crates/hone-web-api/src/routes/meta.rs:295` (`effective_enabled()`).
- **`_ => Self::Local` (`crates/hone-core/src/config/server.rs:617`) is a test affordance, not a typo guard.** `CloudConfig` derives `Default` (`server.rs:510`) so `mode == ""`, and `HoneConfig.cloud` is bare `#[serde(default)]` (`crates/hone-core/src/config/mod.rs:128`), so a YAML with no `cloud:` block never reaches `default_cloud_mode`. This repo's live `config.yaml` has no `cloud:` block. Every `HoneConfig::default()` test lands on Local through this arm; unconditional cloud sends them into `.expect(...)` panics at `crates/hone-channels/src/core/bot_core.rs:98` and `:106`.
- **Two error-swallowing fallbacks the discovery pass missed**: `crates/hone-channels/src/core/bot_core.rs:611-622` and `bins/hone-cli/src/main.rs:242-251` both use `&& let Ok(storage) = CronJobStorage::new_cloud(postgres)` and drop to `with_sqlite` on a live PG error, with no logging. `bot_core.rs:611` feeds the production cron scheduler.
- **`CronJobStorage::new_cloud` cannot fail after the first success** — `CLOUD_CRON_SCHEMA_READY` (`memory/src/cron_job/mod.rs:41-43`, gate at `:108`) short-circuits `ensure_schema`. The silent-degradation window is bounded to before the first success.
- **`packages/app/src/pages/public-dev-login-contract.test.ts:16` asserts the literal Rust source text** `cloud_mode.eq_ignore_ascii_case("local")` inside `crates/hone-web-api/src/routes/public.rs`. Editing `public.rs:177` fails a `bun test` that no Rust tooling flags.
- **`tests/regression/ci/test_billing_http_e2e.sh:58` boots the real server with `HONE_CLOUD_MODE=local` and no Postgres**, then seeds `$TMP_ROOT/data/sessions.sqlite3` with python3 (`:93-97`, `:145-149`, `:736`). It runs on every push via `tests/regression/run_ci.sh` (globbed at `run_ci.sh:9`, invoked at `.github/workflows/ci.yml:72`).
- **`handle_portfolio` is cloud-aware through a hidden global**, not a `CloudMode` branch: `PortfolioStorage::new` reads the global set by `configure_cloud_portfolio_storage`, populated only inside `HoneBotCore::new` (`crates/hone-channels/src/core/bot_core.rs:128`). Grepping `is_cloud_authoritative` will never surface it.

### C-8 (NEW) — Two features use `session_sqlite_db_path` purely as a **directory anchor**, not as a database.

- `crates/hone-web-api/src/routes/research_store.rs:19` — `data_root()` for durable research snapshots
- `crates/hone-web-api/src/routes/community_forum.rs:841` — `forum_root()`

`StorageConfig` (`crates/hone-core/src/config/server.rs:395-427`) has **no `data_dir` field**. Deleting `session_sqlite_db_path` breaks two unrelated features unless a replacement anchor lands first. Prescribed fix in §5.2.

### C-9 (NEW) — `bins/hone-feishu` depends on a type that lives in `session_sqlite.rs`.

`hone_memory::session_sqlite::InterruptedSessionInfo` is used at `bins/hone-feishu/src/handler.rs:261`, `:263`, `:303`, `:2205`, `:2210`, `:2215`, `:2228`. The JSON backend also produces it (`memory/src/session.rs:170-200`), so the type is backend-neutral and must be **relocated**, not deleted, when `memory/src/session_sqlite.rs` goes. It is re-exported at `memory/src/lib.rs:64`.

### C-10 — `bins/hone-cli/Cargo.toml:21 rusqlite` is entirely unused. **Confirmed at HEAD**: `grep -rn 'rusqlite' bins/hone-cli/src/` returns nothing. The `cloud migrate` importer goes through `hone_memory::WebAuthStorage::new` (`bins/hone-cli/src/cloud.rs:2851`), not rusqlite. This line is deletable today, standalone, zero risk.

### C-11 (NEW) — `cron_job_runs` has 17 unmigrated rows and no import channel.

`docs/handoffs/2026-08-16-local-sqlite-to-pg-data-migration.md` (Risks §2): PG has `cloud_cron_job_runs`, but `cloud migrate` imports only cron **definitions**, never history. 17 rows sit in the cold backup with no path forward. Not event-engine work; must be resolved before the cold backup is deleted.

---

## 2. Phase 1 — `web_user_external_state`: DONE, three residue items remain

**Do not re-implement.** What shipped in `18ff42c2`:

| Deliverable | Location |
|---|---|
| DDL + evolution columns + unique partial index | `crates/hone-core/src/cloud_runtime.rs:1432-1450` |
| Row→record decoder | `crates/hone-core/src/cloud_runtime.rs:806` |
| Read (with legacy-JSON fallback) | `crates/hone-core/src/cloud_runtime.rs:2875-2907` |
| Email lookup (index-first, legacy second) | `crates/hone-core/src/cloud_runtime.rs:2909-2957` |
| Upsert | `crates/hone-core/src/cloud_runtime.rs:3800-3820` |
| SQLite importer | `crates/hone-core/src/cloud_runtime.rs:3800`, exporter at `memory/src/web_auth.rs:2070-2172` |
| Cloud write path | `memory/src/web_auth.rs:429-435` |
| Cloud read path | `memory/src/web_auth.rs:399-404` |
| Cloud email lookup | `memory/src/web_auth.rs:474-483` |
| Live-PG regression test | `memory/src/web_auth.rs:3592` (`#[ignore]`) |

Shipped DDL, for reference when writing §3's DDL in the same style:

```sql
CREATE TABLE IF NOT EXISTS cloud_web_user_external_state (
  user_id              TEXT PRIMARY KEY REFERENCES cloud_web_invite_users(user_id) ON DELETE CASCADE,
  email_address        TEXT,
  email_verified_at    TEXT,
  identity_kind        TEXT DEFAULT 'domestic_invite',
  email_challenge_json TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_cloud_web_user_external_email
  ON cloud_web_user_external_state(email_address) WHERE email_address IS NOT NULL;
```

### 2.1 Residue P1-A — the legacy-JSON compatibility branch violates the no-compat-layer decision

Two queries carry a second arm that reads `record #>> '{external_state,...}'`:

- `crates/hone-core/src/cloud_runtime.rs:2882-2896` — the `CASE WHEN s.user_id IS NULL THEN ... ELSE ...` arms in `find_web_user_external_state_record`
- `crates/hone-core/src/cloud_runtime.rs:2929-2953` — the `UNION ALL` second branch + `source_priority` ordering in `find_web_invite_user_record_by_email`

**Action, in this order:**

1. Add a one-shot backfill (a `hone-cli cloud` subcommand or an idempotent `ensure_schema` statement — **not** on any hot path):
   ```sql
   INSERT INTO cloud_web_user_external_state
     (user_id, email_address, email_verified_at, identity_kind, email_challenge_json)
   SELECT u.user_id,
          NULLIF(u.record #>> '{external_state,email_address}',    ''),
          NULLIF(u.record #>> '{external_state,email_verified_at}', ''),
          NULLIF(u.record #>> '{external_state,identity_kind}',     ''),
          NULLIF(u.record #>> '{external_state,email_challenge}',   '')
   FROM cloud_web_invite_users u
   LEFT JOIN cloud_web_user_external_state s ON s.user_id = u.user_id
   WHERE s.user_id IS NULL AND u.record ? 'external_state'
   ON CONFLICT (user_id) DO NOTHING;
   ```
   Run it against **GCE** first and record the row count in the handoff. Local source count is 0 (`docs/handoffs/2026-08-16-local-sqlite-to-pg-data-migration.md`, Risks §4), so local success proves nothing.
2. Strip `record - 'external_state'` from `cloud_web_invite_users.record` in the same transaction.
3. Delete `crates/hone-core/src/cloud_runtime.rs:2882-2896` (collapse to plain `s.<col>`) and `:2929-2953` (collapse to the first branch only, drop `source_priority`).
4. Delete the compat shim `cloud_record_from_value` external-state patching at `memory/src/web_auth.rs:2485-2492` and the field `CloudWebInviteRecord.external_state` at `memory/src/web_auth.rs:157-158`.
5. `memory/src/web_auth.rs:344` currently writes `external_state: WebUserExternalState::default()` into the record; with the field gone this line goes too.

**Gate:** step 3 must not merge until step 1 has been applied to production and the backfill count is written into the handoff.

### 2.2 Residue P1-B — `run_cloud_web_auth` is the C-4 anti-pattern

`memory/src/web_auth.rs:2190-2207`. Fix as part of §4.0, not here.

### 2.3 Residue P1-C — `find_external_user_by_email` SQLite arm

`memory/src/web_auth.rs:485-503` still exists; deleted in Phase 3 (§4.2).

---

## 3. Phase 2 — event-engine store → PostgreSQL

**Scope: `crates/hone-event-engine/src/store.rs` (3547 lines) plus every caller. This is ~80% of the remaining work.**

### 3.0 Delete before porting — 6 methods, zero behaviour change

Verified repo-wide (all file types, excluding `target/` and `node_modules/`): these have **no production call site**. Delete them rather than port them.

| Method | Line | Why safe |
|---|---|---|
| `count_events` | `store.rs:776` | tests only: `tests.rs:51`, `tests.rs:1142`, `pipeline.rs:364`, `store.rs:2317/2613/3494` |
| `today_signal_kinds` | `store.rs:909` | self-documented "历史兼容 shim"; only `store.rs:3432` |
| `count_high_sent_since` | `store.rs:1050` | tests only. The name survives in `warn!` **strings** at `router/dispatch.rs:221`, `router/sink.rs:21`, `router/policy.rs:249` and in `docs/releases/v0.4.1.md:38,130` — update those strings in the same commit |
| `last_high_sink_send_for_symbol` | `store.rs:1113` | tests only; name appears in a `warn!` string at `router/dispatch.rs:371` |
| `earnings_continuity_job_status` | `store.rs:494` | `#[cfg(test)]` helper |
| `event_research_object_key` | `store.rs:715` | `#[cfg(test)]` helper |

Tests that exercise these four public methods must be **rewritten to call the `_for_category` / `_category` variants** — not deleted. The two `#[cfg(test)]` helpers become inline PG queries in the ported test module.

### 3.1 Sync/async bridge design (decide this before writing any SQL)

**The blocker is not `EventStore` — it is fifteen synchronous callers.** `store.rs` holds `Mutex<Connection>` (`store.rs:45`, `std::sync::Mutex` imported at `store.rs:26`). `std::sync::MutexGuard` is `!Send`; four methods hold it across long multi-statement transactions. Making the store `async` makes those futures `!Send` and breaks `tokio::spawn` at `router/dispatch.rs:920`, `:929`, `engine.rs:287`, `spawner.rs:104`.

Sync callers that would each have to become `async`, cascading through `hone-event-engine`'s **public, re-exported** API:

`router/classify.rs:36`, `router/dispatch.rs:505`, `:546`, `:845`, `:899`, `digest/curation.rs:131`, `unified_digest/scheduler.rs:1174`, `unified_digest/sources/synth.rs:35` (→ `unified_digest/collector.rs:59`), `unified_digest/sources/global.rs:31` and `global_digest/collector.rs:50` (→ `unified_digest/collector.rs:82`), `weekly_report.rs:193`, `:234`, `:318` (→ `:167` → `:84`), `pollers/earnings_surprise.rs:157`, `crates/hone-channels/src/scheduler.rs:1914`, `crates/hone-channels/src/agent_session/core.rs:1836`/`:1892`/`:1907`, `crates/hone-web-api/src/routes/notifications.rs:186`.

**Prescribed design: keep every `EventStore` method signature synchronous. Bridge internally.**

Replace `conn: Mutex<Connection>` (`store.rs:45`) with `postgres: CloudPgRuntime`. Inside each method, call the shared blocking bridge from §4.0. Rationale, in order of weight:

1. It converts a 15-function async cascade across a public API boundary into a mechanical body rewrite.
2. It preserves the `Send`-ness of `router/dispatch.rs:920`, `:929`, `engine.rs:287`, `spawner.rs:104`.
3. It is the pattern already proven in this repo (`memory/src/cron_job/mod.rs:130-186`) and already load-bearing in `memory/src/web_auth.rs`, `billing.rs`, `llm_audit.rs`.
4. It contains the risk: the *only* new failure mode is blocking a tokio worker, which the shared bridge's `spawn` + `recv_timeout` already handles.

Connection acquisition: use `connect_cached_client()` throughout (never `connect_client` / `connect_new_client`). `connect_new_client` spawns the connection driver onto the **current** runtime; with a long-lived bridge runtime the cached connection is actually reusable — that is the second reason the bridge runtime must be `LazyLock`, not per-call.

`ensure_schema` guard: one process-global `AtomicBool`, same shape as `CLOUD_CRON_SCHEMA_READY` (`memory/src/cron_job/mod.rs:41-43`, gate at `:108`). `ensure_schema` is ~430 lines of DDL; `EventStore::open` is called per-HTTP-request at `crates/hone-web-api/src/routes/notifications.rs:196` and per-tool-invocation at `crates/hone-tools/src/missed_events_tool.rs:113`.

**Constructor change.** `EventStore::open(path)` (`store.rs:106`) has no PG analogue. Replace with `EventStore::new_cloud(postgres: CloudPgRuntime)`. Four production sites must be handed a runtime instead of a path:

| Site | Change |
|---|---|
| `crates/hone-event-engine/src/engine.rs:276` | takes `CloudPgRuntime` from config |
| `crates/hone-channels/src/core/bot_core.rs:115` | already constructs `cloud_pg_runtime` at `:78`; reuse it |
| `crates/hone-web-api/src/routes/notifications.rs:196` | **stop opening per request** — thread an `Arc<EventStore>` through `AppState` |
| `crates/hone-tools/src/missed_events_tool.rs:113` | **stop opening per `execute`** — `missed_events_tool.rs:10-13` documents the premise "open is idempotent + fast (<1ms)"; that premise is false for a network DB. Hold a pooled handle on `BotCore` |

Reentrancy traps that must survive verbatim — hoisting the lock/handle in any of these three deadlocks or double-acquires on the dispatch hot path:
- `insert_event` (`store.rs:234`) scopes its guard to `{ }` at `:235-253` and releases it **before** `backfill_earnings_research_materials` (which re-locks at `:580`) and before `append_jsonl_mirror`.
- `count_high_sent_since_for_category` (`store.rs:1057`) early-returns at `:1064`/`:1067` **before** taking the lock at `:1088`; the `_all` sibling locks itself at `:1094`.
- `last_high_sink_send_for_symbol_category` (`store.rs:1125`) early-returns at `:1133`/`:1136` before the lock at `:1164`.

`append_jsonl_mirror` (`store.rs:734`) does a blocking `std::fs` write on the per-event path, already outside the lock. Keep it synchronous and outside the bridge. The JSONL mirror's stated rationale (`store.rs:20-21`: "for when SQLite is corrupt") no longer holds under PG — **keep it anyway this round**; it is the independent reconciliation baseline for §3.5.

### 3.2 Full DDL for all 5 tables

Add to `ensure_schema` in `crates/hone-core/src/cloud_runtime.rs`, immediately after the existing table block (insert after `crates/hone-core/src/cloud_runtime.rs:1450`). Source of truth for the SQLite shape: `crates/hone-event-engine/src/store.rs:114-187`.

**Table names stay unprefixed** (`events`, not `cloud_events`) — the ported SQL is large, dynamically built in three places, and every rename is an opportunity for a silent typo in a string-concatenated fragment. `billing_entitlements` (`cloud_runtime.rs:1413`) already sets the precedent for an unprefixed table in this schema.

```sql
-- ========== events ==========
-- All *_json columns stay TEXT. See risk note E-2 below.
CREATE TABLE IF NOT EXISTS events (
  id             TEXT PRIMARY KEY,
  kind_json      TEXT   NOT NULL,
  severity       TEXT   NOT NULL,
  symbols_json   TEXT   NOT NULL,
  occurred_at_ts BIGINT NOT NULL,
  title          TEXT   NOT NULL,
  summary        TEXT   NOT NULL,
  url            TEXT,
  source         TEXT   NOT NULL,
  payload_json   TEXT   NOT NULL,
  created_at_ts  BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_events_occurred_at ON events(occurred_at_ts);
CREATE INDEX IF NOT EXISTS idx_events_source      ON events(source);
-- NEW (absent in SQLite): purge_events_older_than (store.rs:754) and
-- event_breakdown_by_source (store.rs:2118) both filter created_at_ts and seq-scan today.
CREATE INDEX IF NOT EXISTS idx_events_created_at  ON events(created_at_ts);
-- NEW: list_earnings_research_materials (store.rs:665) is a full scan today.
-- text->jsonb is IMMUTABLE so this is legal, but the build HARD-FAILS if any row
-- holds non-JSON text. Create it AFTER the data copy, never inside ensure_schema.
-- CREATE INDEX IF NOT EXISTS idx_events_research_object_key
--   ON events ((payload_json::jsonb ->> 'hone_earnings_research_object_key'));

-- ========== engine_meta ==========
-- `key` and `value` are both NON-RESERVED in PostgreSQL; no quoting needed.
CREATE TABLE IF NOT EXISTS engine_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- ========== delivery_log ==========
-- BIGSERIAL deliberately, NOT `GENERATED ALWAYS AS IDENTITY`: the migration in
-- §3.5 inserts explicit ids, which BIGSERIAL permits and GENERATED ALWAYS rejects
-- without OVERRIDING SYSTEM VALUE.
CREATE TABLE IF NOT EXISTS delivery_log (
  id         BIGSERIAL PRIMARY KEY,
  event_id   TEXT   NOT NULL,
  actor      TEXT   NOT NULL,
  channel    TEXT   NOT NULL,
  severity   TEXT   NOT NULL,
  sent_at_ts BIGINT NOT NULL,
  status     TEXT   NOT NULL,
  body       TEXT
);
CREATE INDEX IF NOT EXISTS idx_delivery_event_actor ON delivery_log(event_id, actor, sent_at_ts);
CREATE INDEX IF NOT EXISTS idx_delivery_sent_at     ON delivery_log(sent_at_ts);
-- NEW: hot reads with an actor predicate —
-- store.rs:1074-1078, 1098-1102, 1149-1152, 1217-1221, 1282-1284, 1314-1317, 1453-1456, 1535-1538
CREATE INDEX IF NOT EXISTS idx_delivery_actor_channel_status_sent
  ON delivery_log(actor, channel, status, sent_at_ts DESC);
-- NEW: hot reads with NO actor predicate — store.rs:1421-1427, 1704-1710 (C-6)
CREATE INDEX IF NOT EXISTS idx_delivery_channel_status_sent
  ON delivery_log(channel, status, sent_at_ts DESC);
-- NEW: delivered_event_ids_since (store.rs:1397-1401) has no channel predicate (C-6)
CREATE INDEX IF NOT EXISTS idx_delivery_actor_status_sent
  ON delivery_log(actor, status, sent_at_ts DESC);

-- ========== delivered_push_context ==========
-- NO FOREIGN KEY to delivery_log. SQLite declares none, and store.rs:2939-2983
-- ('opening_an_old_delivery_log_does_not_backfill_historical_context') depends on
-- delivery_log rows existing with no context row. An FK also breaks
-- purge_delivery_log_older_than (store.rs:762-773), which deletes the two tables
-- in separate statements, context first.
CREATE TABLE IF NOT EXISTS delivered_push_context (
  delivery_log_id            BIGINT PRIMARY KEY,   -- supplied explicitly, no sequence
  actor                      TEXT   NOT NULL,
  source_id                  TEXT   NOT NULL,
  delivered_at_ms            BIGINT NOT NULL,      -- MILLISECONDS. Everything else is seconds.
  body                       TEXT   NOT NULL,
  observed_native_session_id TEXT,
  claimed_turn_id            TEXT,
  claim_expires_at_ms        BIGINT,
  consumed_turn_id           TEXT,
  consumed_at_ms             BIGINT,
  CONSTRAINT delivered_push_context_actor_source_key UNIQUE (actor, source_id)
);
CREATE INDEX IF NOT EXISTS idx_delivered_push_context_pending
  ON delivered_push_context(actor, consumed_at_ms, delivered_at_ms, delivery_log_id);
-- NEW, semantically identical, much better for the hot claim path (store.rs:1917-1921):
CREATE INDEX IF NOT EXISTS idx_delivered_push_context_unconsumed
  ON delivered_push_context(actor, delivered_at_ms, delivery_log_id)
  WHERE consumed_at_ms IS NULL;

-- ========== earnings_continuity_jobs ==========
CREATE TABLE IF NOT EXISTS earnings_continuity_jobs (
  job_key         TEXT    PRIMARY KEY,
  actor_json      TEXT    NOT NULL,
  event_json      TEXT    NOT NULL,
  status          TEXT    NOT NULL,
  attempts        INTEGER NOT NULL DEFAULT 0,  -- int4; bind/read as i32, never u32 (C-6)
  next_attempt_ts BIGINT  NOT NULL,
  lease_until_ts  BIGINT,
  last_error      TEXT,
  created_at_ts   BIGINT  NOT NULL,
  updated_at_ts   BIGINT  NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_earnings_continuity_jobs_due
  ON earnings_continuity_jobs(status, next_attempt_ts, lease_until_ts);
```

**Risk notes bound to this DDL**

- **E-1 — every timestamp stays epoch `BIGINT`, never `TIMESTAMPTZ`.** `SELECT MAX(d.sent_at_ts)` at `store.rs:1147`, `:1183`, `:1215`, `:1281` is read as `Option<i64>`; `MAX(int4)` returns int4 and would fail. `engine_meta` read at `store.rs:220` must be `CAST(value AS BIGINT)`, not `INTEGER` — PG `INTEGER` is int4 and `store.rs:218-223` reads `Option<i64>`, so a literal `AS INTEGER` port errors on **every startup**.
- **E-2 — never promote `*_json` to `jsonb`.** jsonb re-serialises on input (key reorder, whitespace normalisation, duplicate-key drop, numeric normalisation). That (a) breaks every `LIKE '%"earnings_released"%'` substring predicate, (b) changes the bytes round-tripped back into `MarketEvent.payload` at `store.rs:701/977/1361/1500/1578/1675`, and (c) makes `UPDATE events SET payload_json = $2` (`store.rs:647`) infer `$2` as jsonb, which `tokio_postgres` cannot satisfy from a `String`.
- **E-3 — `engine_meta.value` is TEXT but `store.rs:209-212` binds an `i64`.** SQLite TEXT affinity coerces silently; PG rejects. Bind `now.timestamp().to_string()`.
- **E-4 — `INSERT OR IGNORE` → `ON CONFLICT DO NOTHING` with a BARE target everywhere** (`store.rs:210`, `:239`, `:353`, `:1794`). Never `DO UPDATE`. On `store.rs:239` the rows-affected return **is** the dedup gate: `pipeline.rs:51-58` only dispatches when `Ok(true)`. Any `RETURNING`/`DO UPDATE` rewrite re-pushes every duplicate event to every user. On `store.rs:210`, a `DO UPDATE` would rewrite `baseline_at_ts` on every process start, collapsing the entire below-baseline suppression contract (`store.rs:17-18`) with nothing in Rust noticing (the rowcount is discarded).
- **E-5 — sequence reseat is mandatory.** `setval(pg_get_serial_sequence('delivery_log','id'), COALESCE(max(id),0)+1, false)`. `setval(seq, max(id), true)` is wrong (next id collides); `setval(seq, (SELECT max(id)), false)` is wrong the same way. Skipping it hard-errors on the first insert — which is *loud and good* — but becomes **silent** the moment anyone adds `ON CONFLICT DO NOTHING` to `store.rs:1732` or `:1774` "to smooth the migration": the row is dropped, `RETURNING` yields nothing, and every confirmed push vanishes from the audit trail.
- **E-6 — `COALESCE(lease_until_ts, 0)` at `store.rs:389` ports verbatim. Do NOT rewrite it as `GREATEST`.** PG `GREATEST` ignores NULL and returns 0; SQLite scalar `MAX(0, NULL)` returns NULL — opposite semantics. This exact trap already wrote silent "0 ms" values into `cloud_cron_job_runs.duration_ms` on 2026-08-15. On any nullable column, write `CASE WHEN x IS NULL THEN NULL ELSE ... END` explicitly.
- **E-7 — NULL sort order.** PG sorts NULLs LAST for ASC / FIRST for DESC; SQLite always FIRST. Every current `ORDER BY` key (`store.rs:520, 592, 666, 391, 1318, 1457, 1634, 1921, 2064, 2119, 2199`) is NOT NULL, so there is no live divergence — it goes live the instant `claim_expires_at_ms`, `consumed_at_ms` or `lease_until_ts` enters an `ORDER BY`.
- **E-8 — `delivered_at_ms` is milliseconds, `sent_at_ts` is seconds**, both written from one `now` inside one transaction (`store.rs:1783` vs `:1803`). `purge_delivery_log_older_than` encodes the skew as a literal `cutoff.saturating_mul(1000)` at `store.rs:766`. Getting it wrong by 1000× either purges nothing or purges the entire table on every sweep.

### 3.3 Method-by-method port table

Legend for **Freq**: HOT = per-event or per-turn; WARM = per-tick or per-actor-slot; COLD = daily/HTTP-driven.

| # | Method (`store.rs:`) | Freq | Required SQL change | Callers that must change |
|---|---|---|---|---|
| 1 | `open` `:106` | — | Replace with `new_cloud(CloudPgRuntime)`; DDL moves to `ensure_schema`; drop `busy_timeout` (`:112`) entirely — do not translate it | `engine.rs:276`; `bot_core.rs:115`; `notifications.rs:196` (→ pooled `AppState` handle); `missed_events_tool.rs:113` (→ pooled `BotCore` handle); `examples/per_actor_override_e2e.rs:58` |
| 2 | `with_jsonl_path` `:198` | — | none (no SQL) | `engine.rs:278` |
| 3 | `ensure_baseline` `:207` | COLD | `INSERT ... VALUES ('baseline_at_ts', $1) ON CONFLICT DO NOTHING`; **bind a String** (E-3) | private, from `:194` |
| 4 | `baseline_at` `:216` | COLD | `CAST(value AS BIGINT)` (E-1). PG raises on non-numeric text where SQLite returned 0 — that is safer but converts a silent data bug into a hard startup failure. Expect it | `engine.rs:282` |
| 5 | `insert_event` `:234` | HOT | `ON CONFLICT DO NOTHING`, bare target. Preserve `Ok(affected>0)` exactly (E-4). Keep the guard scoping at `:235-253` | `pipeline.rs:51`; ~60 test sites |
| 6 | `link_earnings_research_object` `:288` | HOT | none directly; delegates to #12. `&mut MarketEvent` mutation must land **before** #5 serialises the payload | `pipeline.rs:44` |
| 7 | `enqueue_earnings_continuity_job` `:327` | COLD | `ON CONFLICT DO NOTHING`. `$4` reused 3× as bigint — type-consistent, no cast | `router/dispatch.rs:909` (sync fn) |
| 8 | `claim_due_earnings_continuity_jobs` `:369` | WARM | **Highest-risk rewrite.** `TransactionBehavior::Immediate` (`:380`) has no PG equivalent. Add `FOR UPDATE SKIP LOCKED` to the SELECT **and** an `AND attempts=$2-1` guard to the UPDATE at `:428-436`, which today keys on `job_key` only. Without both, two workers claim the same job, duplicate LLM spend, duplicate push. Read `attempts` as **i32** at `:400` (C-6) | `router/dispatch.rs:956` |
| 9 | `complete_earnings_continuity_job` `:448` | COLD | Bind `attempts` as **i32** at `:460` (C-6). Rows-affected `bool` return is a lease-ownership check consumed at `dispatch.rs:961-968`, `:979-986` — never return `Ok(())` | `router/dispatch.rs:960`, `:978` |
| 10 | `retry_earnings_continuity_job` `:465` | COLD | Bind `attempts` as **i32** at `:487` (C-6). Backoff arithmetic (`:472-473`, `:484`) is pure Rust — ports as-is, valid only while `next_attempt_ts` is epoch BIGINT | `router/dispatch.rs:1002` |
| 11 | *(delete)* `earnings_continuity_job_status` `:494` | — | §3.0 | — |
| 12 | `nearest_earnings_research_object_key` `:505` | HOT | `json_each(events.symbols_json)` (`:517`) → `EXISTS (SELECT 1 FROM jsonb_array_elements_text(events.symbols_json::jsonb) AS s(value) WHERE lower(s.value)=lower($4))`. `ORDER BY ABS(occurred_at_ts-$3)` works only because the column stays BIGINT. `LIMIT 200` truncation is a silent correctness knob — a plan-driven tie reorder picks a different quarter as the research anchor | private, from #6 |
| 13 | `backfill_earnings_research_materials` `:566` | HOT-adjacent | Same `json_each` rewrite at `:589`. `Immediate` at `:615` — note the candidate SELECT at `:582-613` runs **outside** the transaction, so the read-then-write is a lost-update window in *both* engines (pre-existing, not a regression). Under a real pool it becomes reachable: use `jsonb_set` or `SELECT ... FOR UPDATE` | private, from `:262` |
| 14 | `list_earnings_research_materials` `:655` | COLD | `json_extract(payload_json,'$.hone_earnings_research_object_key')` → `(payload_json::jsonb ->> 'hone_earnings_research_object_key')` — **drop the `$.` prefix**. The path is hardcoded here while everywhere else it goes through `EARNINGS_RESEARCH_OBJECT_KEY`; a constant rename breaks this with no compile error | `router/dispatch.rs:862` (sync fn) |
| 15 | *(delete)* `event_research_object_key` `:715` | — | §3.0 | — |
| 16 | `append_jsonl_mirror` `:734` | HOT | no SQL. Keep sync, keep outside the bridge | private, from `:272` |
| 17 | `purge_events_older_than` `:750` | COLD | Add batching (`DELETE ... WHERE ctid IN (SELECT ctid ... LIMIT 5000)` in a loop). Unbounded single DELETE becomes a long-running txn competing with live writers | `engine.rs:293` |
| 18 | `purge_delivery_log_older_than` `:760` | COLD | Two unbounded DELETEs in one txn (`:763`). Preserve `cutoff.saturating_mul(1000)` at `:766` exactly (E-8) and the **context-before-log** ordering. Return the SECOND delete's count | `engine.rs:298` |
| 19 | *(delete)* `count_events` `:776` | — | §3.0 | — |
| 20 | `contains_event` `:786` | HOT | **`SELECT EXISTS` returns BOOLEAN in PG.** Read as `bool` (`cloud_runtime.rs:4298-4304` shows the shape) or wrap `EXISTS(...)::int`. The `i64` bind at `:788` fails on every call. A false `false` re-runs a paid LLM quality review and can re-push | `pollers/earnings_surprise.rs:169` (sync `was_reviewed`, called in a loop from async `fetch` at `:142`) |
| 21 | `actor_has_delivered_earnings_for_document` `:800` | HOT | **Most dialect-bound query in the file.** (a) `SELECT EXISTS` → bool (`:812`). (b) `json_extract(...) = 1` at `:820` → `(e.payload_json::jsonb ->> 'earnings_quality_review_applied') = 'true'` — `= '1'` never matches (silently disables SEC-document suppression → duplicate 8-Ks to every user); `::int = 1` is a runtime cast error. (c) `instr()` at `:824` does not exist in PG → `strpos()`; rewrite the whole `substr`/`CASE` canonicalisation at `:822-827`. (d) `key_path` is a **Rust-side** bind at `:809` built as `format!("$.{EARNINGS_DOCUMENT_KEY}")` — strip the `$.` in Rust, not SQL (`EARNINGS_DOCUMENT_KEY` = `"hone_earnings_release_document_key"`, `earnings_document.rs:10`). Leaving it degrades the guard to URL-match-only, not always-false | `router/dispatch.rs:157` |
| 22 | `symbol_signal_kinds_in_window` `:848` | HOT (2× per symbol per news event) | `LIKE` → `ILIKE` at `:860` (C-6 / E-9) | `router/classify.rs:49`, `:69` (sync `maybe_upgrade_news`) |
| 23 | `list_analyst_grade_payloads_in_window` `:880` | HOT | `ILIKE` at `:892`, `:893` | `router/dispatch.rs:516` (sync); `weekly_report.rs:205` (sync) |
| 24 | *(delete)* `today_signal_kinds` `:909` | — | §3.0 | — |
| 25 | `list_upcoming_earnings` `:922` | WARM (per-minute) | `ILIKE` at `:936` | `unified_digest/scheduler.rs:219`; `unified_digest/sources/synth.rs:40` (sync); `weekly_report.rs:323` (sync) |
| 26 | `next_upcoming_earnings_for_symbol` `:999` | HOT | `ILIKE` at `:1012`, **and `:1013`** — `:1013` is bound with the `to_uppercase()` needle from `:1005` and was omitted from the discovery list (C-6). `.optional()` → `Option<i64>` maps fine; do not confuse with the `MAX()` methods, which return a row containing NULL | `router/dispatch.rs:561` (sync) |
| 27 | `count_event_ids_in_window` `:1027` | COLD | `ESCAPE '\'` at `:1039` is portable, but PG's `standard_conforming_strings` handling of the escape literal differs; a naive copy yields patterns that match nothing, turning the weekly report's "defense intercepted N" into a permanent 0 with no error | `weekly_report.rs:244`, `:248` (sync) |
| 28 | *(delete)* `count_high_sent_since` `:1050` | — | §3.0 + update the three `warn!` strings | — |
| 29 | `count_high_sent_since_for_category` `:1057` | HOT | **Dynamic SQL, variable param count** (2 + N tags). Rebuild `:1069-1081` with a running `$N` counter; `params_from_iter` ordering rebuilt too. `ILIKE` on the generated predicates. This is `high_severity_daily_cap`: wrong 0 → unlimited High pushes; wrong large → every High demoted to digest and immediate pushes stop. Preserve the deliberate `status='sent'` exclusion of `'dryrun'` | `router/dispatch.rs:195` |
| 30 | `count_high_sent_since_all` `:1093` | HOT | mechanical | private (reentrancy trap — §3.1) |
| 31 | *(delete)* `last_high_sink_send_for_symbol` `:1113` | — | §3.0 + update the `warn!` string at `dispatch.rs:371` | — |
| 32 | `last_high_sink_send_for_symbol_category` `:1125` | HOT | Dynamic `$N` rebuild of `:1138-1157`; conditional `firm_clause` at `:1138-1142` changes the param count. **`json_extract(e.payload_json,'$.gradingCompany')` at `:1141` was omitted from the discovery inventory** (C-6) → `(e.payload_json::jsonb ->> 'gradingCompany')`. `SELECT MAX(x)` over zero rows returns **one row containing NULL** — the binding must stay `row.get::<_, Option<i64>>(0)`, NOT `.optional()`; a non-Option bind panics on the common new-user case. `ILIKE` at `:1153` | `router/dispatch.rs:348` |
| 33 | `last_high_sink_send_for_symbol_all` `:1174` | HOT (fallback) | `ILIKE` at `:1189`. Reachable in production only when `category_kind_tags` (`:2240`) returns None — adding an EventKind category without adding it to that map silently cools down across ALL kinds | private |
| 34 | `last_high_sink_send_for_analyst_news_url` `:1200` | HOT | **`?4` is reused at `:1225` and `:1226`.** Sequential renumbering produces 5 placeholders for 4 values and fails at runtime, not compile time — use `$4` twice. **`json_extract` at `:1225` was omitted** (C-6) → `->>` (not `->`: `->` yields quoted jsonb and never matches, disabling the TheFly batch guard so one aggregated article fans out as 5-10 pushes). `ILIKE` at `:1222` | `router/dispatch.rs:302` |
| 35 | `last_price_band_max_bps_for_symbol_direction` `:1239` | HOT | `ILIKE` at `:1260`, `:1261`. The max is computed **in Rust** over all returned ids (`parse_bps_from_band_id`, `:2257`) — do not add a `LIMIT` for performance, it changes the answer. Note `:1261` embeds `%` without `ESCAPE`, unlike #27 | `router/dispatch.rs:249` |
| 36 | `last_digest_success_at` `:1277` | WARM | `MAX()`-returns-NULL-row binding (see #32). Unbounded MAX over the actor's whole delivery_log | `unified_digest/scheduler.rs:337` |
| 37 | `list_missed_digest_items_since` `:1302` | COLD | No SQL LIMIT; truncation happens in Rust at `missed_events_tool.rs:126` after full deserialisation. The `NOT IN` status filter is a negative list — any new status string surfaces in user-visible `/missed` output | `missed_events_tool.rs:122` |
| 38 | `delivered_event_ids_since` `:1389` | WARM | **Do NOT add `JOIN events`.** The no-JOIN is deliberate and documented at `:1383-1388`: synthetic ids (`digest.synth.earnings_countdown`) never hit `events`. Adding the join re-pushes the same countdown every slot. `rows.flatten()` at `:1409` swallows per-row decode errors into a smaller set — a partial failure looks like "nothing delivered" → duplicate pushes | `unified_digest/scheduler.rs:374` |
| 39 | `list_actors_with_quiet_held_since` `:1416` | WARM | Must keep emitting the flattened `channel::scope::user_id` string from `delivery_actor_key` (`:2178`) — it is re-parsed at `unified_digest/scheduler.rs:1053` and `notifications.rs:343`. Normalising actor into columns silently empties the quiet-flush audience | `unified_digest/scheduler.rs:261` |
| 40 | `list_quiet_held_since` `:1441` | COLD | The doc comment at `:1438` says "LEFT JOIN 风格"; **the SQL is an INNER JOIN and must stay one.** A literal reading of the comment produces NULL event columns and panics the row mapper at `:1456-1470` (non-Option `row.get`) | `unified_digest/scheduler.rs:851` |
| 41 | `list_recent_digest_item_events` `:1523` | WARM | mechanical. Note the caller's error path `let Ok(recent) = ... else { return DigestCuration::kept(events) }` at `curation.rs:138` silently disables 24h topic-dedup on any DB failure, with no log line — add one | `digest/curation.rs:138` (sync) |
| 42 | `list_global_digest_news_candidates` `:1614` | WARM | **Largest unbounded result set in the file.** `json_extract(payload_json,'$.source_class')` at `:1632` → `->>`. `ILIKE` at `:1627`, `:1629-1631`. The three-branch predicate encodes a deliberately ASYMMETRIC severity gate documented at `:1596-1613` (RSS: any severity; FMP trusted: Low allowed; FMP non-trusted: high/medium only). The two `source LIKE 'fmp.stock_news:%'` branches overlap **on purpose** — "simplifying" them drops mainline hard news from Pass-1 | `global_digest/collector.rs:62` (sync) → `unified_digest/sources/global.rs:33` (sync) → `scheduler.rs:648` |
| 43 | `broadcasted_event_ids_since` `:1698` | WARM | `rows.flatten()` at `:1719` swallows decode errors → same re-broadcast hazard as #38 | `global_digest/collector.rs:65` (sync) |
| 44 | `log_delivery` `:1722` | **HOTTEST WRITE** | Mechanical INSERT, but **24 production call sites, 15 of which discard the Result with `let _ =`**: `dispatch.rs:63,127,160,392,457,487,659,711,810`; `scheduler.rs:463,570,584,864,970,1176`. Under SQLite a local write failure was near-impossible; under PG each becomes a silent swallow of a **network** failure. `delivery_log` is the audit trail that #29/#32/#38/#43 all read back — missing rows mean cap/cooldown/dedup silently under-count and users get flooded. **Every one of those 15 must at minimum `warn!` on error.** `log_omitted_digest_items` (`scheduler.rs:1174`) is a sync free fn looping N inserts — make it a batch INSERT | 24 sites (above) |
| 45 | `log_confirmed_delivery` `:1758` | HOT | **Three breakages.** (a) `tx.last_insert_rowid()` at `:1788` has no PG equivalent and is per-connection, i.e. meaningless with a pool → change the INSERT at `:1774-1777` to `... RETURNING id` and read it as i64 in the same transaction; never `currval()`/`lastval()`. (b) `INSERT OR IGNORE` at `:1794` → **bare** `ON CONFLICT DO NOTHING` (C-6); this is the transport-retry idempotency key documented at `:1795-1797` — a retried send writes a second delivery_log row but must NOT create a second context row. (c) Both statements stay in one transaction | `dispatch.rs:672`, `:772`; `scheduler.rs:547`, `:947`; `crates/hone-channels/src/scheduler.rs:1933` (sync `record_confirmed_scheduled_delivery`) |
| 46 | `claim_delivered_push_context` `:1818` | — | pure delegation | tests only; production reaches #47 |
| 47 | `claim_delivered_push_context_with_native_observation` `:1838` | HOT (per turn) | **Longest and most concurrency-sensitive method.** `Immediate` at `:1855` exists specifically so two RUNTIMES (event engine + channel runtime, separate processes) cannot both select the same push in the deferred read→write upgrade window (`:1849-1851`). PG default READ COMMITTED does not reproduce that. It happens to survive because its UPDATE is guarded (`WHERE delivery_log_id=$1 AND consumed_at_ms IS NULL AND claimed_turn_id IS NULL`, `:1946-1955`) — **that guard is now the entire safety property; state it in a comment and never widen it.** Add `FOR UPDATE SKIP LOCKED` to the candidate SELECT at `:1917-1921` for defence in depth. Both `load_claimed_push_context` calls (`:1855`, `:1957`) and both `count_pending_push_context` calls (`:1859`, `:1959`) must stay inside the SAME transaction | `crates/hone-channels/src/agent_session/core.rs:1857` (sync) ← `core.rs:2413` |
| 48 | `complete_delivered_push_context` `:1968` | HOT | mechanical. Error is only logged at `core.rs:1897-1904`, never propagated — a PG network failure strands the push until lease expiry, then re-injects it into a later turn (user sees the same push twice) | `core.rs:1896` (sync) ← `core.rs:2923` |
| 49 | `release_delivered_push_context` `:1990` | HOT | mechanical. **Do not widen the WHERE clause** — dropping `consumed_at_ms IS NULL` un-consumes an already-delivered push | `core.rs:1911` (sync) ← `core.rs:2538`, `:3000` |
| 50 | `list_recent_delivery_logs` `:2009` | COLD | **Biggest mechanical rewrite**: up to nine optional `?` fragments appended at `:2014-2068`, needs an explicit running `$N` counter. **`d.actor LIKE ?` at `:2040` and `:2044` were omitted from the discovery ILIKE list** (C-6). This is the file's **only** `LEFT JOIN` — rows whose event was purged appear with NULL event columns and the mapper at `:2069-2101` correctly binds `Option<String>`; tightening to INNER JOIN silently loses all older rows from the web notifications page. `ORDER BY d.sent_at_ts DESC, d.id DESC` at `:2064` is the one per-**second** tie-break (vs per-millisecond at `:1921`/`:2199`), so it is ~1000× more exposed to BIGSERIAL's weaker commit-order monotonicity | `notifications.rs:217` (sync) ← `notifications.rs:107`, `:158` |
| 51 | `event_breakdown_by_source` (free fn) `:2109` | COLD | Reaches into the **private field** `store.conn.lock()` at `:2114` — easy to miss when auditing `impl EventStore`. Filters `created_at_ts` (ingest time) while nearly everything else filters `occurred_at_ts` (event time); swapping them silently changes what the daily report counts. `ORDER BY 2` is valid in both | `daily_report.rs:63` |
| 52 | `delivery_breakdown_per_actor` (free fn) `:2131` | COLD | Same private-field access at `:2136`. `weekly_report.rs:237` uses `.ok()?` — a DB failure silently omits the entire push-quality section with no log line | `daily_report.rs:64`; `weekly_report.rs:237` (sync) |
| 53 | `load_claimed_push_context` `:2187` | — | Signature is typed on `rusqlite::Transaction<'_>`; retype on the PG transaction. Must stay in #47's transaction | private |
| 54 | `count_pending_push_context` `:2214` | — | **`(?4 IS NULL OR ...)` at `:2233` is the trickiest predicate in the file.** PG needs a known type for the placeholder: write `($4::text IS NULL OR claimed_turn_id IS NULL OR claimed_turn_id <> $4)`. Bare `$4 IS NULL` errors with "could not determine data type". Getting it wrong makes `remaining_count` wrong, which is what the UI tells the user about queued pushes | private |
| 55 | helpers `:2155-2270` | — | No SQL. But `severity_tag` (`:2170`) and `delivery_actor_key` (`:2178`) define the exact string encodings every WHERE clause compares against, and `delivery_actor_key`'s output is re-parsed **outside** store.rs (`scheduler.rs:1053`, `notifications.rs:343`). `truncate_store_error` (`:2166`) caps `last_error` at 300 chars in Rust — the PG column must not be tighter | — |

**E-9 — the `LIKE` → `ILIKE` sweep is not cosmetic.** Verified empirically: `sqlite3 "SELECT 'aapl' LIKE '%AAPL%'"` → `1`. SQLite's default LIKE is ASCII-case-insensitive; PG's is case-**sensitive**. Every needle built from `symbol.to_uppercase()` (`store.rs:854, 886, 1005, 1139, 1179, 1211, 1249`) silently stops matching any row whose stored casing differs. Visible effect: **fewer** cooldown/dedup hits → duplicate pushes, with no error anywhere. Full site list in C-6. Either use `ILIKE` on all 22, or normalise casing at write time and prove it with a backfill query — do not do half of each.

### 3.4 The `Immediate` → PG mapping table (state this explicitly, do not leave it implicit)

| Site | SQLite behaviour | PG replacement |
|---|---|---|
| `store.rs:380` (`claim_due_earnings_continuity_jobs`) | write lock before SELECT makes read→UPDATE atomic across processes | `FOR UPDATE SKIP LOCKED` on the SELECT **+** `AND attempts=$2-1` on the UPDATE at `:428-436` |
| `store.rs:615` (`backfill_earnings_research_materials`) | no real protection — candidate SELECT at `:582-613` runs before the txn opens | pre-existing lost-update window; add `SELECT ... FOR UPDATE` or `jsonb_set`. Not a regression |
| `store.rs:1855` (`claim_delivered_push_context_with_native_observation`) | prevents two runtimes claiming the same push | guarded UPDATE at `:1946-1955` already provides it; add `FOR UPDATE SKIP LOCKED` at `:1917-1921` for depth. **Document that the guard is now the safety property** |

### 3.5 Data-migration procedure (preserves `delivery_log` ids)

Source: `data/backups/pre-pg-migration/events.sqlite3` (161,390,592 bytes, cold, SHA-256 pinned in `docs/handoffs/2026-08-16-local-sqlite-to-pg-data-migration.md`). Never read the live `data/events.sqlite3`.

Expected source counts, already measured and recorded in that handoff:

```
events                   76147
engine_meta                  1
delivery_log             83928
delivered_push_context     154
earnings_continuity_jobs     0
```

Deliver as `hone-cli cloud migrate --event-store-only --from-sqlite <path>`, default dry-run, `--apply` to write, matching the existing migrate contract (`bins/hone-cli/src/cloud.rs:70-111` for flag style, `:215-249` for report shape). Report `changed`/`skipped` per table.

**Procedure — order is load-bearing:**

1. `PRAGMA wal_checkpoint(TRUNCATE)` — **only** if you ever migrate from a live file. The cold backup already has no `-wal`/`-shm` sidecar (verified in the handoff); do not touch it.
2. **Pre-flight null checks** (C-6 — SQLite non-INTEGER PKs are NULL-able):
   ```sql
   SELECT count(*) FROM events                   WHERE id      IS NULL;  -- must be 0
   SELECT count(*) FROM engine_meta              WHERE key     IS NULL;  -- must be 0
   SELECT count(*) FROM earnings_continuity_jobs WHERE job_key IS NULL;  -- must be 0
   ```
   Any non-zero aborts before the copy; otherwise the copy dies mid-transaction with an undiagnosable not-null violation.
3. **Pre-flight orphan check**: `SELECT count(*) FROM delivered_push_context c LEFT JOIN delivery_log d ON d.id=c.delivery_log_id WHERE d.id IS NULL` — record the number; do not fail on it (there is no FK today and the DDL keeps it that way).
4. Copy `events`, `engine_meta`, `earnings_continuity_jobs` in 5000-row batches. Bind every `*_json` column as `String` (E-2).
5. Copy `delivery_log` **naming `id` explicitly in the column list** so `nextval` is not consumed. 5000-row batches.
6. Copy `delivered_push_context`, `delivery_log_id` supplied explicitly. **Must follow step 5.**
7. **Reseat the sequence** (E-5):
   ```sql
   SELECT setval(pg_get_serial_sequence('delivery_log','id'), COALESCE(max(id),0)+1, false) FROM delivery_log;
   ```
8. Build the optional expression index on `events` (§3.2). It will hard-fail if any `payload_json` row is not valid JSON — that failure is a real data finding, not an index problem.
9. **Reconciliation, all four must pass and all four go in the handoff:**
   - per-table `count(*)` equality against step 0's numbers;
   - `SELECT count(*) FROM delivered_push_context c JOIN delivery_log d ON d.id=c.delivery_log_id` equals 154 minus the step-3 orphan count;
   - `min(id)`/`max(id)` on `delivery_log` identical between SQLite and PG;
   - content hash of `payload_json` on a random 1000-row sample of `events`, plus a full `md5(string_agg(...))` if runtime allows.
10. **Independent baseline**: `data/events.jsonl` (278 MB) is untouched by every migration so far (handoff, Risks §7). Use it to cross-check the `events` row count.

**Why the id preservation matters and why nothing will tell you if it breaks**: `delivered_push_context` is **never JOINed** to `delivery_log` anywhere in the repo (grepped all `*.rs`) — `claim_delivered_push_context` reads `body` from `delivered_push_context` itself at `:1915`/`:2194`. So a reassignment produces no error, no user-visible symptom, and destroys the append-only audit trail promised at `store.rs:7-8`. The only forensic handle: `DeliveredPushContextRecord.delivery_log_id` (`store.rs:85`) is copied into agent-visible turn metadata as the `delivery_log_ids` array at `crates/hone-channels/src/runners/types.rs:350`.

### 3.6 Test porting for Phase 2 — 155 tests, **zero deletions permitted**

`store.rs:2277-3547` (1270 lines, 36% of the file) is the **only executable specification** of the concurrency contracts: IMMEDIATE-transaction claim ordering, lease expiry, append-only audit, INSERT-OR-IGNORE idempotency. It is built entirely on `tempfile::tempdir()` + local SQLite.

| File | Tests | `EventStore::open` sites |
|---|---|---|
| `crates/hone-event-engine/src/store.rs` | 29 | 33 |
| `crates/hone-event-engine/src/router/tests.rs` | 66 (`#[tokio::test]`) | 52 |
| `crates/hone-event-engine/src/pollers/earnings_surprise.rs` | 17 | 2 |
| `crates/hone-event-engine/src/global_digest/collector.rs` | 13 | 1 |
| `crates/hone-event-engine/src/tests.rs` | 11 | 4 |
| `crates/hone-event-engine/src/unified_digest/collector.rs` | 5 | 1 |
| `crates/hone-event-engine/src/weekly_report.rs` | 4 | 2 |
| `crates/hone-event-engine/src/daily_report.rs` | 3 | 1 |
| `crates/hone-event-engine/src/unified_digest/sources/synth.rs` | 3 | 1 |
| `crates/hone-event-engine/src/pipeline.rs` | 2 | 2 |
| `crates/hone-event-engine/src/unified_digest/sources/global.rs` | 2 | 1 |
| **Total** | **155** | **100 test-side** (102 in `src/`+`examples/` minus `engine.rs:276` prod and `examples/per_actor_override_e2e.rs:58`) |

Porting rules:

- Replace the `tempdir()` fixture with a per-test schema/namespace against the local Docker PG from `docker-compose.dev.yml`. Mark them `#[ignore]` with the repo's existing reason string (`"requires HONE_POSTGRES_* and a running local PostgreSQL"`, `memory/src/web_auth.rs:3592`) and add a `tests/regression/manual/` connectivity script — that is the established convention.
- Three tests reach into the private `store.conn` field (`store.rs:2656`, `:2831`, `:3483`) — they need a store-internal test accessor, not deletion.
- `store.rs:2853` `delivered_push_context_crosses_store_connections_and_respects_body_budget` opens a **writer and a reader on the same path**. Its PG equivalent needs two independent `CloudPgRuntime` handles. It has no cheap substitute; it is the only proof that the two-runtime claim protocol works.
- `store.rs:2939-2983` `opening_an_old_delivery_log_does_not_backfill_historical_context` is why `delivered_push_context` has no FK — it must keep passing.
- `store.rs:2412` `continuity_job_survives_restart_retries_and_recovers_an_expired_lease` is the only coverage of the lease/retry state machine; port it before touching #8.

**Phase 2 acceptance gate (from the plan doc §7.3, still correct):** replay `crates/hone-event-engine/src/tests.rs:1189` `replay_push_quality_audit` against the migrated PG and diff the push results **line by line** against the SQLite run. That is the only evidence that dedup, delivery_log correlation and push-context claim semantics survived.

---

## 4. Phase 3 — delete the SQLite backends from `memory/`

### 4.0 PREREQUISITE — extract the shared bridge first (C-4)

**This is a separate, mergeable commit that must land before any deletion.**

Add to `hone-core` (suggested `crates/hone-core/src/cloud_bridge.rs`) a generic copy of the proven implementation at `memory/src/cron_job/mod.rs:59-186`:

- `LazyLock<std::io::Result<tokio::runtime::Runtime>>`, `new_multi_thread().worker_threads(2).thread_name(...)` — fixed at 2, not core-count, because this path is all PG round-trips and must not fight web/agent for cores on a 2-vCPU box;
- not in a tokio context → `runtime.block_on` (`mod.rs:169`);
- in a tokio context → `runtime.spawn` + `sync_channel(1)` + `recv_timeout(timeout + 5s)` (`mod.rs:175-186`). **`Handle::current().block_on` panics on a runtime thread — never use it.**
- per-operation timeout, env-overridable, defaulting to 15s (`mod.rs:39`, `:138-145`).

Then replace all seven call sites from C-4 with it. Do **not** refactor `CronJobStorage`'s behaviour while doing so — `memory/src/cron_job/mod.rs` keeps its own module-level wrapper delegating to the shared fn.

The two ignored bridge tests at `memory/src/cron_job/mod.rs:218` and `:247` (`cloud_cron_timeout_returns_storage_error_instead_of_blocking`, `cloud_cron_bridge_works_from_inside_a_tokio_context`) must be generalised to the shared fn, not left pinned to cron.

### 4.1 `memory/src/session_sqlite.rs` — whole-file delete (832 lines), with one relocation

**Before deleting**, move `InterruptedSessionInfo` (`memory/src/session_sqlite.rs:449`) into `memory/src/session.rs` (C-9). Keep the re-export at `memory/src/lib.rs:64` pointing at the new home. Consumers that must keep compiling unchanged: `bins/hone-feishu/src/handler.rs:261, 263, 303, 2205, 2210, 2215, 2228`; `memory/src/session.rs:14, 61, 170, 196, 1187, 1192, 1630`.

**Tests to PORT: 5** — `memory/src/session_sqlite.rs:539, 580, 704, 736, 778`. Three of them (`upsert_session_persists_rows`, `upsert_session_replaces_old_rows_and_stores_message_metadata_columns`, `list_sessions_orders_by_updated_at_desc`) assert relational-mirror semantics that `cloud_sessions` (whole-session JSONB, `crates/hone-core/src/cloud_runtime.rs:1372-1379`) does not have. Rewrite them as `cloud_sessions` round-trip assertions; do not drop the behaviour they cover. `list_sessions_skips_unreadable_rows` (`:778`) becomes a malformed-JSONB test.

### 4.2 `memory/src/web_auth.rs`

Delete: `WebAuthBackend::Sqlite` (`:146`), `WebAuthStorage::new` (`:172`), `init_schema` (`:202-300`) including the full `web_user_external_state` DDL at `:236-246`, `sqlite_conn` (`:303`), the SQLite arms of `load_external_state` (`:407-420`), `save_external_state` (`:437-465`), `find_external_user_by_email` (`:485-503`), `create_international_email_user` (`:1112-1156`), `export_cloud_records` (`:2070-2172`, only after §3.5 and P1-A backfill are both done and reconciled), `external_state_from_row` (`:2524`), and the `rusqlite::Row` signature of `external_state_from_values` (`:2528`).

**Tests: 29 total; 24 must be PORTED, 5 need rewriting or retiring with justification.**
Twenty-four call the shared `test_storage()` fixture and are backend-agnostic once the fixture points at PG.
The five that assert SQLite mechanics specifically:
- `memory/src/web_auth.rs:3393` `new_storage_adds_phone_and_revoked_columns_for_existing_database` — asserts the `ensure_column` ALTER-TABLE migration path. Its PG equivalent is the `ADD COLUMN IF NOT EXISTS` block at `crates/hone-core/src/cloud_runtime.rs:1387-1388` / `:1440-1447`. **Rewrite against those**, do not delete.
- `:2853`, `:2983`, `:3086`, `:3111`, `:3144`, `:3178` reference sqlite in fixture setup only — mechanical fixture swap.

`memory/src/web_auth.rs:3592` `cloud_web_user_external_state_round_trip` already runs against real PG; it becomes the template.

### 4.3 `memory/src/session.rs`

Delete `SessionRuntimeBackend::{Json, Sqlite}` (`:99`) — **both** (C-5) — plus `SessionStorageOptions.shadow_sqlite_db_path` / `shadow_sqlite_enabled`, the startup JSON→SQLite backfill, and `from_storage_config`'s SQLite arm.

**Tests: 22 total; all 22 PORTED.** Sixteen use `SessionStorage::new` and are mechanical. Five assert shadow/runtime-backend behaviour that ceases to exist: `:2028`, `:2065`, `:2140`, `:2211`, `:2243`. These do **not** get deleted — rewrite each as the equivalent `cloud_sessions` assertion (write-visibility, startup recovery, read-path authority, dual-write removal). `:2243` `cloud_runtime_backend_dual_writes_sqlite_shadow` becomes an explicit **negative** test: cloud writes must produce no local artifact.

### 4.4 `memory/src/billing.rs`

Delete `BillingBackend::Sqlite` (`:114`), `BillingStorage::new`, and the SQLite schema/rebuild path.
**Tests: 6 total; all 6 PORTED.** `:1308` `legacy_provider_tables_are_rebuilt_as_stripe_only` tests a SQLite table rebuild — its PG analogue is the `CHECK (provider IN ('stripe','domestic_invite'))` constraint at `crates/hone-core/src/cloud_runtime.rs:1416`. Rewrite as a constraint-violation test.

### 4.5 `memory/src/llm_audit.rs`

Delete the SQLite arm and `LlmAuditStorage::new`.
**Tests: 3 total; all 3 PORTED.** `:855` `migrate_legacy_schema_and_persist_tokens` tests a SQLite legacy-schema migration; rewrite against `cloud_llm_audit_records`.
Retain the paged exporter at `memory/src/llm_audit.rs:496-534` until §3.5-style reconciliation for `llm_audit` is signed off (it already is — 7 rows, ID set difference 0, per the handoff) — then delete it.

### 4.6 `memory/src/cron_job/`

Delete `CronJobStorage::with_sqlite` (`memory/src/cron_job/mod.rs:88`) **and** `CronJobStorage::new` (`:78`, the JSON-dir arm — C-5), plus the `sqlite_path` field (`:35`) and all of `memory/src/cron_job/history.rs`'s SQLite half.

**Tests: 33 total (31 in `mod.rs` + 2 in `history.rs`); all 33 PORTED.**
- 13 use `CronJobStorage::new` (JSON) → repoint at `new_cloud`.
- 12 use `with_sqlite` → repoint at `new_cloud`. Four of those (`:1470`, `:1581`, `:1825`, `:1899`) additionally open a raw `Connection` to inject stale/malformed rows; rewrite as direct PG inserts into `cloud_cron_job_runs`. These four cover the watchdog/stale-row-recovery paths that caused the 30–50% silent scheduler failure rate — **losing them is not acceptable.**
- 2 bridge tests (`:218`, `:247`) move to §4.0.
- The remainder are pure-logic and unaffected.

**Blocking dependency**: `crates/hone-scheduler/src/lib.rs:424, 425, 475, 476, 535, 536` calls `with_sqlite` three times and hardcodes `sessions.sqlite3`. It has **no rusqlite dependency of its own** (`crates/hone-scheduler/Cargo.toml` lists only hone-core/hone-memory/tokio/tracing/chrono/serde_json), so a manifest-driven grep misses it entirely. Fix it in the same commit.

Also resolve C-11 (`cron_job_runs`, 17 rows, no import channel) before the cold backup is deleted.

### 4.7 Phase 3 test-porting total

**93 tests across `memory/` must be ported, plus 155 in `hone-event-engine` from §3.6 = 248.** None may be deleted. Per `AGENTS.md` §7: a pure refactor may not weaken existing coverage. Review gate: compare `cargo test --workspace -- --list | wc -l` before and after; the count must not drop.

---

## 5. Phase 4 — zero-residue cleanup

### 5.1 Dependencies

| Item | Action | Notes |
|---|---|---|
| `bins/hone-cli/Cargo.toml:21` | **DELETE NOW** | Unused (C-10). Zero-risk, do it in the first commit of any phase |
| `memory/Cargo.toml:16` | Delete after §4 | `tokio-postgres` at `memory/Cargo.toml:24` (dev-dep) stays — it is what the ported tests use |
| `crates/hone-event-engine/Cargo.toml:19` | Delete after §3 | Add `hone-core` PG deps in the same commit |
| `Cargo.toml:72-73` (workspace) | **KEEP**, retitle the comment | Still required by `bins/hone-imessage/Cargo.toml`. Change the comment to state it is *exclusively* for `bins/hone-imessage`'s read-only macOS `chat.db` |
| `Cargo.lock` | Regenerate | `rusqlite` and its exclusive transitive deps drop to a single consumer |

**`bins/hone-imessage` is out of scope. Do not touch it, its Cargo.toml, or its `SQLITE_OPEN_READ_ONLY` handling. It reads macOS's own `~/Library/Messages/chat.db`.**

### 5.2 Config fields — the anchor problem must be solved first (C-8)

**Step 1 (blocking, separate commit).** Introduce a real data-root accessor on `StorageConfig` (`crates/hone-core/src/config/server.rs:395`) derived from `sessions_dir`'s parent — the pattern `ensure_runtime_dirs` already uses at `crates/hone-core/src/config/server.rs:472`. Repoint:
- `crates/hone-web-api/src/routes/research_store.rs:19`
- `crates/hone-web-api/src/routes/community_forum.rs:841`

**Step 2.** Delete the fields and their machinery:

| Item | Sites |
|---|---|
| `storage.session_sqlite_db_path` | field `crates/hone-core/src/config/server.rs:399-400`; default `:483-485`; `apply_data_root` `:435`; `ensure_runtime_dirs` `:457`, `:476` |
| `storage.session_sqlite_shadow_write_enabled` | field `:401-402`; `default_true`; materializer `crates/hone-core/src/config/materialize.rs:470, 473, 485, 490, 493` |
| `storage.session_runtime_backend` | field `:403-404`; default `:489-491`; parser in `memory/src/session.rs:99` |
| `storage.llm_audit_db_path` | field `:405-406`; default `:492-494`; `apply_data_root` `:440`; `ensure_runtime_dirs` `:472`, `:475` |
| `config.example.yaml` | `:500`, `:502`, `:503`, `:505`, `:506`, `:511`, `:521` |
| Env var `HONE_CLOUD_KEEP_SESSION_SQLITE_SHADOW` | `crates/hone-core/src/cloud_runtime.rs:6099-6103`; consumer `crates/hone-channels/src/core/bot_core.rs:89-90`; **also delete it from GCE `/etc/hone/runtime.env` and any systemd drop-in** |
| `local_durable_dependencies` | `crates/hone-core/src/cloud_runtime.rs:6111-6135` — after deletion only object storage remains. Note the pre-existing hole: `crates/hone-web-api/src/lib.rs:446-458` passes the raw shadow flag **without** the `keep_cloud_session_sqlite_shadow()` gate that `bot_core.rs:89-90` applies, while `local_durable_dependencies` ANDs that same gate at `:6128-6129` — so a shadow DB created via that path is never reported. Fix or delete both together |
| `effective_strict_no_local_storage` | `crates/hone-core/src/config/server.rs:652`. If it becomes constant-true, delete the switch rather than leave a lying toggle |

### 5.3 CloudMode deletion — the definitive site list

`crates/hone-core/src/config/server.rs:600-611` (`CloudMode` enum), `:613-619` (`from_config_value`, incl. the `_ => Self::Local` arm), `:621-627` (`as_str` + `is_cloud_authoritative`), `:630-637` (`effective_mode`), `:639-651` (`effective_enabled`), `:655-663` (the `validate` cloud branch).

**All 22 `is_cloud_authoritative()` call sites** (verified exhaustive at HEAD):

```
crates/hone-core/src/cloud_runtime.rs:6111
crates/hone-core/src/config/mod.rs:202
crates/hone-web-api/src/lib.rs:426
crates/hone-web-api/src/lib.rs:443
crates/hone-web-api/src/lib.rs:735
crates/hone-web-api/src/routes/web_users.rs:42
crates/hone-web-api/src/routes/portfolio.rs:48
crates/hone-web-api/src/routes/company_profiles.rs:53
crates/hone-web-api/src/routes/meta.rs:129
crates/hone-web-api/src/routes/users.rs:34
crates/hone-channels/src/response_finalizer.rs:217
crates/hone-channels/src/response_finalizer.rs:950
crates/hone-channels/src/agent_session/core.rs:2856
crates/hone-channels/src/attachments/ingest.rs:162
crates/hone-channels/src/attachments/ingest.rs:433
crates/hone-channels/src/core/bot_core.rs:78
crates/hone-channels/src/core/bot_core.rs:102
crates/hone-channels/src/core/bot_core.rs:353
crates/hone-channels/src/core/bot_core.rs:430
crates/hone-channels/src/core/bot_core.rs:461
crates/hone-channels/src/core/bot_core.rs:611
bins/hone-cli/src/main.rs:238
```

**Plus 5 non-`is_cloud_authoritative` cloud-mode branches** (C-7): `crates/hone-web-api/src/routes/meta.rs:146`, `crates/hone-web-api/src/routes/public.rs:166`, `crates/hone-web-api/src/routes/company_ratings.rs:442`, `crates/hone-web-api/src/lib.rs:1137`, `crates/hone-web-api/src/routes/meta.rs:295`. Reporting-only, keep or adjust: `bins/hone-cli/src/cloud.rs:2708-2709`.

**Four blockers that fire before any of the above compiles:**

| Blocker | Site | Required action |
|---|---|---|
| Frontend source-text contract | `packages/app/src/pages/public-dev-login-contract.test.ts:16` | Update the asserted string **in the same commit** as `crates/hone-web-api/src/routes/public.rs:177`. No Rust tooling will flag this |
| DB-free CI regression | `tests/regression/ci/test_billing_http_e2e.sh:58`, seeds at `:93-97`, `:145-149`, `:736` | Rework to run against the CI Postgres service, or gate it on PG availability. Runs on every push via `.github/workflows/ci.yml:72` → `tests/regression/run_ci.sh:9` |
| ~49 `HoneConfig::default()` / ~30 `HoneBotCore::new(HoneConfig::default())` unit tests | reach Local only via `crates/hone-core/src/config/server.rs:617` | They will hit `.expect(...)` panics at `crates/hone-channels/src/core/bot_core.rs:98` and `:106`. Provide a PG-backed test config helper before deleting the arm |
| `CloudConfig::validate` | `crates/hone-core/src/config/server.rs:655-663`, reached on every load via `crates/hone-core/src/config/mod.rs:150` | Every config lacking PG **and** OSS now fails to load. Decide whether OSS stays mandatory in the cloud-only world |

**Two silent-degradation fallbacks to remove, not just rewire:** `crates/hone-channels/src/core/bot_core.rs:611-622` and `bins/hone-cli/src/main.rs:242-251` (`&& let Ok(...)` → `with_sqlite`, no logging). `bins/hone-cli/src/main.rs:246-248` has a **third** arm (JSON-dir `CronJobStorage::new`) the discovery pass never listed.

**Two cloud-mode paths that already write local files and stay local** — document them, do not pretend they are gone: `crates/hone-channels/src/response_finalizer.rs:997` (OSS failure `warn!`s and returns a local absolute path into the user-visible reply) and `crates/hone-channels/src/attachments/ingest.rs:162` (every attachment lands in the local `upload_dir` first, in every mode).

**`handle_portfolio`** (C-7): cloud-aware through the process-global set at `crates/hone-channels/src/core/bot_core.rs:128`. Any process that never constructs a `HoneBotCore` silently uses the local JSON path. Make it explicit or leave a comment.

### 5.4 Scripts (whole-file deletes)

- `scripts/migrate_sessions_to_sqlite.py` (~590 lines)
- `scripts/diagnose_session_sqlite.py`
- `scripts/diagnose_event_engine_daily_pushes.py` — **write the PG replacement first**; this is the tool the scheduler-reliability investigation used
- `scripts/export_weekly_sessions_excel.py` — remove the `--sqlite-db` / `--source` machinery (`:51-60`, `:145-220`, `:534`) and keep the JSON path, or delete outright

### 5.5 Tests and fixtures

| File | Action |
|---|---|
| `tests/regression/ci/test_session_sqlite_migration.sh` | **DELETE the whole file.** It exists solely to exercise the two deleted Python scripts (`:69, 73, 78, 100, 131, 133, 135`). It is picked up by the `run_ci.sh:9` glob, so deleting the scripts without deleting this file breaks CI with no Rust compile error |
| `tests/regression/ci/test_billing_http_e2e.sh` | Rework (§5.3) |
| `tests/regression/manual/test_stripe_billing_lifecycle.sh` | `:311-320`, `:585-589` |
| `tests/regression/manual/test_multi_agent_runner.sh` | inline YAML `:30-34` |
| `tests/regression/manual/test_skill_runtime_cli.sh` | inline YAML `:74-78` |
| `tests/regression/manual/test_opencode_acp_skill_toggle.sh` | inline YAML `:239-243` |
| `bins/hone-cli/src/yaml_io.rs` | `:100`, `:130`, `:132` |
| `bins/hone-cli/src/common.rs` | `:177`, `:189`, `:191` |
| `crates/hone-core/src/config/tests.rs` | **8 assertion sites, not 2**: `:340-343`, `:439`, `:442-443`, `:556-583`, `:700`, `:706`, `:2029-2042`, `:2183-2186` |
| `crates/hone-channels/src/scheduler.rs` | `:8569-8572` |
| `crates/hone-channels/src/agent_session/tests.rs` | `:884`, `:917`, `:5213`, `:5216`, `:5806-5850`, `:6788` |
| `crates/hone-event-engine/src/tests.rs` | `:56`, `:60`, `:1156`, `:1475` (assertion messages) |
| `crates/hone-event-engine/src/global_digest/collector.rs:196`, `unified_digest/collector.rs:172`, `sources/synth.rs:92`, `sources/global.rs:86` | test fixture paths |

### 5.6 String literals — survive any type-driven refactor with zero compile errors

- `crates/hone-channels/src/runtime.rs:270`, `:283` — the **user-facing output sanitizer** regexes redacting `data/sessions\.sqlite3` / `sessions\.sqlite3` / `session_messages` / `session_metadata` from model replies. Tests at `:1498-1503`, `:1887-1897`. Update the regexes and their tests together; the replacement must redact PG table names instead.
- `crates/hone-channels/src/prompt.rs:53`, `:58`, `:63` — the system prompt forbidding the model to mention `sessions.sqlite3`/SQLite. Asserted at `crates/hone-channels/src/agent_session/tests.rs:5213`, `:5216`.
- `crates/hone-channels/src/core/logging.rs:175-178` — startup log line.
- `crates/hone-event-engine/src/router/dispatch.rs:221`, `:371`; `router/sink.rs:21`; `router/policy.rs:249` — `warn!` strings naming the four deleted methods.
- `.codex/automations/bug/automation.toml:17` and `.codex/automations/bug-2/automation.toml:53` — bug-patrol instructions telling the agent to inspect `data/sessions.sqlite3`.
- `.agents/skills/event-engine-push-review/SKILL.md` (8 hits), `.agents/skills/event-engine-baseline-testing/SKILL.md` (1).

### 5.7 Documentation — split into three buckets, do not rewrite all 200 files

**Bucket A — MUST change (current, load-bearing):**

| File | Note |
|---|---|
| `docs/runbooks/session-sqlite-shadow-backfill.md` | **Not just docs.** `crates/hone-core/src/config/tests.rs:2183-2186` reads this exact path and asserts its Chinese prose (`tests.rs:566`, `:582`). Deleting it fails `cargo test -p hone-core`; editing it without editing `tests.rs` also fails. Delete both together |
| `docs/invariants.md` | **8 lines, not 2**: `:156`, `:183`, `:184`, `:185`, `:186`, `:187`, `:221`, `:268` |
| `docs/session-sqlite-migration-plan.md` | ~620 lines. A **live, non-archived** plan whose entire subject is migrating sessions *to* SQLite — the exact opposite of the current decision. Move to `docs/archive/plans/` and note the supersession in `docs/current-plans/sqlite-to-postgres-migration-2026-08-16.md` |
| `docs/wiki.md` `:39`, `:114`, +1 | primary reference |
| `docs/technical-spec.md` (7 hits) | asserted at `crates/hone-core/src/config/tests.rs:700`, `:706` |
| `docs/repo-map.md` (10 hits) | |
| `docs/runbooks/backend-deployment.md` `:1` | |
| `docs/decisions.md` (11 hits) | |
| `docs/current-plan.md`, `docs/current-plans/*` | update to reflect C-1 |
| `CONTRIBUTING.md`, `docs/open-source-prep.md` | 1 hit each |
| `resources/architecture.html`, `resources/architecture.svg`, `docs/architecture.html`, `docs/architecture2.html` | architecture diagrams still show SQLite as a storage tier |
| **`packages/app/src/lib/public-content.ts:2379` (中文) and `:5028` (English)** | **Highest-consequence non-code item.** The shipped, public **privacy policy** states account and conversation data are stored in a local SQLite database. Git-tracked, on the public site, a legal statement. Must be corrected in the same release as the cutover |

**Bucket B — leave alone (historical records):** all of `docs/bugs/`, `docs/bugs/archive/`, `docs/archive/`, `docs/event-review/`, `docs/handoffs/`, `docs/releases/`, `docs/proposal/`. These are dated incident and decision records; rewriting them falsifies history. The one exception is `docs/releases/v0.4.1.md:38, 130`, which documents the `count_high_sent_since` contract — add a superseded note rather than editing the record.

**Bucket C — this plan and its handoffs:** `docs/current-plans/sqlite-to-postgres-migration-2026-08-16.md`, `docs/handoffs/2026-08-16-local-sqlite-to-pg-data-migration.md` keep their SQLite references by definition.

### 5.8 Final verification grep and the exact expected survivors

```bash
grep -rn -i "sqlite\|rusqlite" \
  --include='*.rs' --include='*.toml' --include='*.yaml' --include='*.yml' \
  --include='*.sh' --include='*.py' --include='*.ts' --include='*.tsx' \
  --include='*.html' --include='*.svg' --include='*.json' --include='Dockerfile*' \
  . \
  | sed 's|^\./||' \
  | grep -v '^target/' \
  | grep -v '^node_modules/' \
  | grep -v '^Cargo.lock' \
  | grep -v '^bins/hone-imessage/' \
  | grep -v '^docs/archive/' \
  | grep -v '^docs/bugs/' \
  | grep -v '^docs/handoffs/' \
  | grep -v '^docs/releases/' \
  | grep -v '^docs/proposal/' \
  | grep -v '^docs/event-review/' \
  | grep -v '^docs/current-plans/sqlite-to-postgres-migration-2026-08-16.md'
```

**Expected surviving hits — this exact set, nothing else:**

1. `Cargo.toml:72` — the retitled workspace comment naming `bins/hone-imessage` as the sole consumer
2. `Cargo.toml:73` — `rusqlite = { version = "0.31", features = ["bundled"] }`
3. `docs/wiki.md` / `docs/repo-map.md` — one line each describing `bins/hone-imessage` as the macOS `chat.db` reader

Any other hit must be fixed, or justified in writing in the handoff. Paste the raw output into the handoff — the plan doc requires it (§6.4).

**`Cargo.lock` is deliberately excluded** from the grep: `rusqlite` legitimately remains there for `bins/hone-imessage`. Verify separately that its consumer list has shrunk to one:
```bash
cargo tree -i rusqlite --workspace
```

---

## 6. Ordering and parallelism

### Must serialize

| Constraint | Reason |
|---|---|
| **§4.0 (shared bridge) → Phase 3** | After Phase 3 the seven per-call-`Runtime::new()` sites (C-4) are the *only* path, on every box. Deleting SQLite first ships a known CPU-burn regression that already cost 47 CPU-minutes in 26 wall-minutes on production hardware |
| **Phase 2 (event-engine, complete + accepted) → Phase 3** | Explicit plan-doc constraint and correct: deleting the SQLite backends first leaves the event engine with nowhere to fall back to during its own port. Both touch `bot_core.rs` construction |
| **P1-A step 1 (production backfill) → P1-A step 3 (delete legacy branch)** | Deleting the `#>>` fallback before the GCE backfill orphans every pre-`18ff42c2` external-state record. Local source count is 0, so local success is not evidence |
| **§5.2 step 1 (data-root anchor) → §5.2 step 2 (field deletion)** | `research_store.rs:19` and `community_forum.rs:841` break otherwise (C-8) |
| **§3.5 step 5 (delivery_log) → step 6 (delivered_push_context) → step 7 (setval)** | Id correspondence and sequence correctness. Nothing detects a violation at runtime |
| **§4.1 relocation of `InterruptedSessionInfo` → deletion of `session_sqlite.rs`** | `bins/hone-feishu` breaks otherwise (C-9) |
| **§5.4 (delete Python scripts) ↔ §5.5 (delete `test_session_sqlite_migration.sh`)** | Same commit, either order. Split them and CI breaks with no Rust compile error |
| **§5.3 (`public.rs` edit) ↔ `public-dev-login-contract.test.ts:16`** | Same commit. A source-text assertion no Rust tooling sees |
| **`docs/runbooks/session-sqlite-shadow-backfill.md` ↔ `crates/hone-core/src/config/tests.rs:2183-2186`** | Same commit. `cargo test -p hone-core` reads the file off disk |

### Can run concurrently in separate worktrees

| Track | Contents | Why it is disjoint |
|---|---|---|
| **A** | §5.1 `bins/hone-cli/Cargo.toml:21` deletion | Provably unused (C-10). Zero file overlap. Merge immediately |
| **B** | §4.0 shared bridge extraction | Touches `crates/hone-core/src/cloud_bridge.rs` (new) + 7 one-line call-site swaps in `memory/`. No overlap with `store.rs` |
| **C** | Phase 2 §3.0–§3.4 (event-engine port) | Confined to `crates/hone-event-engine/`, plus the four `EventStore::open` call sites. The only shared file with Track D is `crates/hone-core/src/cloud_runtime.rs` — and only the `ensure_schema` DDL block, which is append-only. Sequence the two DDL insertions or expect a trivial conflict |
| **D** | P1-A (external-state legacy-branch removal) | `cloud_runtime.rs:2875-2957` + `memory/src/web_auth.rs`. Disjoint from Track C's `cloud_runtime.rs:1450` insertion |
| **E** | §5.7 Bucket A docs, §5.6 string literals, `packages/app` privacy policy | Pure text. **Exception**: anything paired with a compiled assertion (runbook↔`config/tests.rs`, `public.rs`↔`.test.ts`) must ride with its code commit, not this track |
| **F** | PG-backed test-harness scaffolding for §3.6 and §4 (fixture helpers, per-test schema namespacing, `tests/regression/manual/` connectivity script) | Pure additive. **Start this first** — it is the long pole for 248 test ports and blocks the acceptance gate for both Phase 2 and Phase 3 |

**Recommended wall-clock ordering:** A + F immediately; then B and C and D in parallel; then Phase 3; then Phase 4. E rides alongside throughout except for the two paired items.

**Per-phase gate (unchanged from the plan doc §7.1):**
```bash
cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
bash tests/regression/run_ci.sh
```
`run_ci.sh` fails in a non-interactive shell because `rg` is a shell function on this machine, not a binary. Put a shim on PATH first: `exec -a rg "$CLAUDE_CODE_EXECPATH"`.

---

## 7. Open questions — do not invent answers

**OQ-1 (BLOCKING for Phase 2 cutover) — does GCE currently hold an `events.sqlite3`?**
`crates/hone-channels/src/core/bot_core.rs:113-125` opens it unconditionally, so under `role=all` (in force since 2026-08-15) it should exist. The plan doc's "zero sqlite3 files under `/srv/honeclaw/data/`" was measured under `role=web`. I could not probe the host from this Mac. **Run `ls -la /srv/honeclaw/data/*.sqlite3*` on the GCE box and record the result before cutover.** If a file exists with rows, it needs its own §3.5 migration pass and the plan doc's "GCE has no historical data to reconcile" is false.

**OQ-2 — is `hone-desktop` still a shipped product?**
`bins/hone-desktop/src/commands.rs:269` and `sidecar.rs:687` call `start_server(.., "local")` with no Postgres. It is excluded from `cargo test` (`.github/workflows/ci.yml:69`) and absent from `release.yml:143`'s binary list. If it is live, `CloudMode::Local` deletion breaks it. Flagged, not asserted.

**OQ-3 — does anything outside Rust read `events.sqlite3` directly?**
The store-methods discovery pass greps were `--include='*.rs'` over `crates/`, `bins/`, `tests/` only. `apps/`, `workers/`, `packages/`, `agents/` were not swept for non-Rust readers. My §5 sweep covered `*.ts`/`*.py`/`*.sh` repo-wide and found only `scripts/diagnose_event_engine_daily_pushes.py:27` — but I did not read `apps/` or `workers/` source for indirect access.

**OQ-4 — does the JSONL mirror survive PG?**
Its stated rationale (`crates/hone-event-engine/src/store.rs:20-21`) is "for when SQLite is corrupt". Under PG that reason is gone, but `data/events.jsonl` (278 MB) is currently the only independent reconciliation baseline for §3.5. **Keep it this round**; decide after Phase 2 acceptance. Not a blocker.

**OQ-5 — is OSS still mandatory in a cloud-only world?**
`CloudConfig::validate` (`crates/hone-core/src/config/server.rs:655-663`) requires PG **and** OSS whenever mode is cloud. With Local deleted, every config must satisfy both — including local dev, which has no OSS. The local Docker PG setup from `9ba2f7d7` does not provide one. Unresolved; must be decided before §5.3 lands.

**OQ-6 — the 225 `cloud_documents` rows point at `local:///tmp/hone-pg35-91227a9f.44BhUL/...`.**
Per `docs/handoffs/2026-08-16-local-sqlite-to-pg-data-migration.md` (Risks). Deleting that temp snapshot invalidates every URI. Not this migration's problem, but it constrains cleanup: **do not delete `/tmp/hone-pg35-91227a9f.44BhUL` as part of Phase 4.**

**OQ-7 — `cron_job_runs`, 17 rows, no import channel (C-11).**
Needs a decision: build the importer, or accept the loss and record it. Blocks deletion of the cold backup.

**OQ-8 — production PostgreSQL major version.**
`docker-compose.dev.yml` pins `postgres:16-alpine`. I could not verify the GCE server version. Confirm before relying on any version-specific behaviour (`FOR UPDATE SKIP LOCKED` is ≥9.5, so §3.4 is safe regardless; the concern is `jsonb` function availability and planner behaviour).