# Active Bug Burn-down 2026-04-28

- title: Active Bug Burn-down 2026-04-28
- status: in_progress
- created_at: 2026-04-28
- updated_at: 2026-04-29 00:00 CST
- owner: Codex
- related_files:
  - `docs/bugs/README.md`
  - `docs/bugs/*.md`
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-event-engine/src/**`
  - `crates/hone-web-api/src/routes/**`
  - `memory/src/**`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `launch.sh`
  - `bins/hone-desktop/src/**`
  - `packages/app/src/**`
- related_docs:
  - `docs/current-plans/feishu-p1-reliability-batch.md`
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/current-plans/canonical-config-runtime-apply.md`

## Goal

Clear the current active bug queue as far as software changes can responsibly do so, prioritizing shared reliability contracts over one-off compatibility hacks for transient network/provider/model behavior.

## Scope

- Triage the 21 active bugs in `docs/bugs/README.md`.
- Group fixes by shared root cause: scheduler delivery/status, ACP unfinished-tool handling, session mirroring, Feishu/Web outbound, heartbeat status/duplicate behavior, event-engine quality, desktop runtime restart, Telegram stale runtime files.
- Implement durable hardening and tests for controllable code paths.
- Keep issues caused by external credentials, provider outages, model nondeterminism, or live network failures documented when code can only improve classification, retry boundaries, or observability.

## Progress

- 2026-04-28: Started from a clean `main...origin/main` workspace, then pulled latest before the burn-down.
- 2026-04-28: Closed 6 active bugs with code changes and moved them to `Fixed` in `docs/bugs/README.md`:
  - Desktop bundled restart 8077 port conflict
  - Web scheduler SSE-only delivery false failure
  - `sessions.sqlite3` mirror disabled by default
  - Once scheduler absolute date loss
  - Telegram `GetMe` startup failure leaving dead pid / heartbeat
  - Event-engine `immediate_kinds` resurrecting Low news
- 2026-04-28: Added hardening for related active scheduler paths without marking the user-facing bugs fully fixed:
  - scheduler event now carries authoritative schedule fields into the channel prompt
  - heartbeat max-iteration failures are no longer treated as compatibility noops
  - web/imessage scheduler now records an initial `running + pending` run before executing
- 2026-04-28: Continued the burn-down and moved the active queue from 15 to 2:
  - blocked cron jobs whose prompt `【触发时间】HH:MM` conflicts with structured schedule, including historical bad data at due-time scan
  - hardened OpenAI-compatible 4xx handling so numeric `error.code` responses preserve the real upstream message instead of collapsing to serde `invalid type`
  - added deterministic heartbeat duplicate suppression against recently delivered previews
  - made empty heartbeat output and empty-status JSON fail the heartbeat contract instead of silently becoming `noop`
  - raised heartbeat auxiliary function-calling max iterations from 6 to 10 so shared heartbeat execution has enough budget without model/provider-specific hacks
  - strengthened heartbeat source attribution rules so oil/geopolitics claims cannot cite Reuters/WSJ/Bloomberg/official sources unless current tool results substantiate that source
  - added single-contact current-app open_id resolution for event-engine Feishu direct sends to avoid stale cross-app actor ids
  - made Web scheduler persist a user-visible failure message for failed runs, including internally-suppressed unfinished-tool failures
  - fixed `launch.sh` zombie child detection for disabled channel pid cleanup
  - suppressed internal Feishu scheduler failure fallbacks for `codex acp prompt ended before tool completion`
  - reviewed event-engine news classifier and convergence guard code paths and moved stale active docs to `Fixed`
- 2026-04-28: Active bug queue is now 2. Remaining items are Feishu direct empty/invalid answer fallback quality and Telegram invalid token/live connectivity.

## Validation

- Run targeted Rust unit tests for changed crates/modules.
- Run targeted frontend tests when web UI or API client behavior changes.
- Run formatting checks for changed Rust files where practical.
- Re-check `git status` and active bug documentation before closing.

Completed this round:

- `cargo check -p hone-memory -p hone-scheduler -p hone-tools -p hone-web-api -p hone-event-engine -p hone-channels --tests`
- `cargo test -p hone-memory once_jobs_with_future_date_do_not_run_today --lib`
- `cargo test -p hone-event-engine per_actor_immediate_kinds_does_not_resurrect_low_signal_news --lib`
- `cargo check -p hone-telegram --tests`
- `cargo check -p hone-desktop --tests`
- `cargo test -p hone-channels heartbeat_prompt --lib`
- `cargo test -p hone-memory prompt_schedule_time_mismatch --lib`
- `cargo test -p hone-llm extracts_ --lib`
- `cargo test -p hone-channels heartbeat_duplicate_preview_match --lib`
- `cargo test -p hone-channels heartbeat_prompt_requires_source_grounding_for_geopolitics --lib`
- `cargo test -p hone-channels heartbeat_empty --lib`
- `cargo test -p hone-event-engine direct_contact --lib`
- `cargo test -p hone-event-engine first_batch_get_open_id --lib`
- `cargo test -p hone-web-api scheduler_failure_trace_required --lib`
- `cargo test -p hone-channels user_visible_error_message_or_none --lib`
- `cargo check -p hone-memory -p hone-llm -p hone-channels --tests`
- `bash -n launch.sh`

Known verification limitation:

- `bash scripts/ci/check_fmt_changed.sh` cannot run under the system Bash 3 environment because it uses `mapfile`; changed Rust files were formatted directly with `rustfmt --edition 2024`.

## Documentation Sync

- Update `docs/bugs/README.md` and touched bug documents with status, fix notes, and verification.
- Update this plan while work is active.
- When the batch is closed or paused, write a handoff and either keep this plan active or move it to archive with an `docs/archive/index.md` entry.

## Risks / Open Questions

- Some active bugs depend on production credentials, live Feishu/Telegram/OpenRouter behavior, or model output quality; these should be hardened through contracts and evidence, not hidden behind brittle provider-specific special cases.
- The active queue spans several ongoing plans, so edits must avoid reverting unrelated in-progress work.
