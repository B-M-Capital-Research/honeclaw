# Storage API 全面 async 化

- title: Storage API 全面 async 化并删除通用 sync-to-async 桥接
- status: in_progress
- created_at: 2026-08-16
- updated_at: 2026-08-16
- owner: Codex
- related_files:
  - `memory/src/*.rs`
  - `memory/src/cron_job/*.rs`
  - `memory/src/company_profile/storage.rs`
  - `crates/hone-core/src/cloud_sync.rs`
  - `crates/hone-event-engine/src/store.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-event-engine/src/**/*.rs`
  - `crates/hone-channels/src/**/*.rs`
  - `crates/hone-web-api/src/**/*.rs`
  - affected tests and callers discovered by compilation
- related_docs:
  - `AGENTS.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
  - `docs/archive/index.md`

## Goal

Remove the repository-wide `run_cloud_sync`, `run_cloud_cron`,
`run_cloud_web_auth`, and `run_cloud_notification_prefs` bridges now that runtime
storage is PostgreSQL-only. Make storage operations and all ordinary callers async
without changing SQL or product behavior.

Preserve the PostgreSQL cross-process delivered-push-context claim protocol and
the process-local one-time `EventStore::ensure_schema` guard.

## Scope

- Capture the pre-refactor ignored `replay_push_quality_audit` output.
- Convert storage APIs in small module-oriented slices and compile/test each slice.
- Propagate async through event-engine public collectors, channels, web-api, tools,
  scheduler, tests, and other compile-discovered call sites.
- Convert affected tests to async tests; do not remove coverage.
- Delete `crates/hone-core/src/cloud_sync.rs` and its module export once no caller
  remains. Do not edit its runtime construction parameters before deletion.
- Do not modify `bins/hone-imessage` or `docs/handoffs/`; do not push.
- Split work into independently revertible commits. Every task commit includes
  `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`.

Planned commit slices:

1. Smaller memory stores (`quota`, `llm_audit`, `billing`, `portfolio`, company
   profile and their callers).
2. Session and Web authentication stores and their callers.
3. Cron storage/history and scheduler/tool/channel callers.
4. Event-engine `EventStore`, notification preferences, internal/public async
   propagation, and cross-crate callers.
5. Bridge deletion, final tests, and plan/archive documentation.

The actual boundaries may be adjusted when compilation reveals unavoidable shared
call sites; each resulting commit must still compile or have its dependency stated
in the commit message.

## Validation

Run focused compilation/tests after each module slice. Final acceptance:

```bash
cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
cargo test -p hone-memory --all-targets -- --ignored
bash tests/regression/run_ci.sh
bun run test:web
grep -rn "run_cloud_sync\|run_cloud_cron\|run_cloud_web_auth\|run_cloud_notification_prefs" --include="*.rs" .
cargo test -p hone-event-engine replay_push_quality_audit --lib -- --ignored --nocapture
```

Also run the repository skill's focused event-engine test and formatting checks:

```bash
cargo test -p hone-event-engine --lib
cargo fmt --all -- --check
```

The post-refactor replay output must match the pre-refactor push results item by
item, ignoring build/test harness timing noise.

## Documentation Sync

- Track the active task in `docs/current-plan.md` and this file because the work
  crosses module/crate boundaries and changes public async APIs.
- Update `docs/repo-map.md` or `docs/invariants.md` only if the implementation
  changes a documented boundary or invariant; a signature-only propagation does
  not by itself require either update.
- On completion, move this plan to `docs/archive/plans/`, remove the active index
  entry, and add a concise record to `docs/archive/index.md`.
- Do not create or edit `docs/handoffs/` per the task boundary.

## Risks / Open Questions

- `EventStore::claim_delivered_push_context*` must retain its transaction and
  `FOR UPDATE SKIP LOCKED` semantics across separate connections.
- `EventStore::ensure_schema` must remain process-local once-only on hot paths.
- Async propagation may cross exported collector/source APIs and trait boundaries;
  any irreducible synchronous entry point must be minimal, documented in code,
  and listed in the final report.
- Tests may rely on deterministic local file fallbacks even though runtime storage
  is PostgreSQL-only; preserve test semantics while changing signatures.
- Do not combine SQL cleanup or storage behavior changes with this refactor.
