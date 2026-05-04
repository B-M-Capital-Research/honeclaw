# Active Bug Burn-down 2026-04-28

- title: Active Bug Burn-down 2026-04-28
- status: in_progress
- created_at: 2026-04-28
- updated_at: 2026-05-04 21:15 CST
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
- 2026-04-29: Rebased the burn-down work onto latest `origin/main`, then closed four newly active scheduler defects:
  - heartbeat near-threshold false triggers now hit a shared `near_threshold_suppressed` send gate instead of reaching the user when the message itself admits the threshold was only approached
  - Feishu scheduler terminal execution events now update the matching `running + pending` started row by `delivery_key`
  - heartbeat duplicate history now includes the same actor's recent heartbeat deliveries across sibling jobs, not only the current job
- 2026-04-29: Active bug queue is now 3. Remaining active items are Feishu direct empty/invalid answer quality, `sessions.sqlite3` mirror stalled evidence, and Telegram invalid token/live connectivity. Telegram remains a credential/live configuration issue; do not add hard-coded compatibility behavior for it.
- 2026-04-30: Addressed event-engine digest readability feedback from the last 24h push log review: macro digest rows now include actual/expected/previous values or a clear future publish time, earnings surprise rows label EPS explicitly, and digest links render as source-host anchors in Telegram HTML, Discord embeds, and Feishu cards while retaining exact href targets.
- 2026-05-01: Closed the active P1 Feishu `open_id cross app` event-engine regression by widening the Feishu direct current-app fallback from “exactly one email or exactly one mobile” to “all stable contacts resolve to exactly one open_id”. This covers single-user configs that keep both email and mobile while preserving the no-guessing rule for ambiguous multi-user contact sets.
- 2026-05-01: Closed the active P2 watchlist near-threshold regression by extending the heartbeat send gate to parse watchlist price phrases such as `跌至 69.85` and suppress `triggered` outputs that claim `已触及或低于触发价 69.83` while the parsed current price is still above the configured lower trigger line.
- 2026-05-01: Closed the reopened P1 Feishu direct quota rejection bug by preserving quota rejection text before internal-error suppression, including wrapped forms such as `工具执行错误: 已达到今日对话上限...`, and by logging Feishu failure fallback sends as `reply.send failure_fallback` so placeholder updates are auditable.
- 2026-05-02: Closed the daily macOS isolated-config `soul.md` startup bug by copying safe relative `system_prompt_path` assets from bundle/repo resources into the canonical config directory before desktop runtime config loads it. Also moved two stale active entries back to `Fixed` based on current code/test evidence instead of old production samples: Web scheduler offline SSE delivery status and provider numeric `HTTP 400` error preservation.
- 2026-05-02 11:03: Latest bug ledger refresh reopened Web scheduler offline SSE and provider numeric `HTTP 400` based on newer local evidence; keep those active for separate review instead of carrying forward the stale Fixed conclusion.
- 2026-05-02: Closed the Feishu scheduler started-row finalization regression by hardening both sides of the matching contract: scheduler terminal detail now replaces unusable `delivery_key` values, and cron history storage can safely fallback-update the latest recent `phase=started` pending row for the same actor/job/target/heartbeat when exact key matching fails.
- 2026-05-02 17:35: Reopened P1 Feishu direct empty/invalid answer bug is now back to `Fixing` after narrowing `response_finalizer`'s `planning_sentence_suppressed` heuristic. Clarification questions such as “请先确认具体是哪只股票/资产的 ticker？” are no longer treated as empty-success fallbacks, and targeted `hone-channels` regression tests now cover both the helper and full finalizer path. No live Feishu runtime recheck yet because this automation does not restart services.
- 2026-05-03 18:06: Closed the active Web `tool_call_update.rawOutput` leak by hardening shared session event emission instead of transcript persistence: `SessionEventEmitter` now relativizes `ToolStatus.tool/message/reasoning`, suppresses internal prompt markers such as `【Invoked Skill Context】` / `Base directory for this skill:`, and drops structured JSON payloads from user-visible progress events while preserving raw ACP evidence for restore/debug. Targeted `hone-channels` emitter tests and `cargo check -p hone-channels --tests` passed. Feishu direct empty/invalid answer remains the only active P1 because this automation run does not restart services or generate new live Feishu samples.
- 2026-05-04 21:15: Tightened the remaining active P1 Feishu direct-answer path again. `multi_agent` search results backed only by read-only local file tools (`local_list_files` / `local_search_files` / `local_read_file`) may now return directly when the answer is already concise and single-paragraph, which covers attachment / local-state confirmation turns that were still being forced into the more failure-prone ACP answer stage. Added targeted `hone-channels` tests to keep verbose local file summaries on the answer path while letting concise confirmations bypass it. No live Feishu runtime recheck yet because this automation does not restart services.

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
- `cargo test -p hone-memory execution_terminal_event_updates_matching_pending_row -- --nocapture`
- `cargo test -p hone-channels heartbeat_near_threshold_trigger_is_suppressed -- --nocapture`
- `cargo test -p hone-channels heartbeat_watchlist_above_trigger_price_is_suppressed -- --nocapture`
- `cargo test -p hone-scheduler heartbeat_history_includes_actor_cross_job_deliveries -- --nocapture`
- `cargo check -p hone-memory -p hone-scheduler -p hone-channels --tests`
- `bun run test:web`
- `HONE_DATA_DIR=/tmp/honeclaw-validate-runtime HONE_WEB_PORT=18087 HONE_PUBLIC_WEB_PORT=18088 HONE_DISABLE_AUTO_OPEN=1 cargo run -p hone-console-page` 启动隔离用户端实例，in-app browser 打开 `http://127.0.0.1:18088/chat`，确认登录页可渲染且 console error 为 0
- `cargo test -p hone-event-engine direct_contact --lib -- --nocapture`
- `cargo test -p hone-event-engine unique_batch_get_open_id --lib -- --nocapture`
- `cargo test -p hone-event-engine sinks::feishu --lib -- --nocapture`
- `rustfmt --edition 2024 --check crates/hone-event-engine/src/sinks/feishu.rs`
- `cargo check -p hone-event-engine -p hone-web-api --tests`
- `cargo test -p hone-channels heartbeat_watchlist_ --lib -- --nocapture`
- `cargo test -p hone-channels user_visible_error_message --lib -- --nocapture`
- `cargo test -p hone-feishu failed_reply_text -- --nocapture`
- `cargo test -p hone-channels run_rejects_over_daily_limit_with_user_turn_and_friendly_error -- --nocapture`
- `cargo check -p hone-channels -p hone-feishu --tests`
- `rustfmt --edition 2024 --check crates/hone-channels/src/runtime.rs bins/hone-feishu/src/handler.rs`
- `rustfmt --edition 2024 bins/hone-desktop/src/sidecar/runtime_env.rs`
- `git diff --check`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop runtime_env -- --nocapture`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop --tests`
- `git diff --check`
- `cargo test -p hone-llm openrouter -- --nocapture`
- `cargo test -p hone-event-engine --lib`
- `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- `OPENROUTER_API_KEY=<config value> env RUN_EVENT_ENGINE_LLM_BASELINE=1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`；15/15 live OpenRouter baseline matched, reported cost `0.000640`.
- `HONE_FMP_API_KEY=<config value> cargo test -p hone-event-engine pollers::news::tests::live_fmp_news_smoke --lib -- --ignored --nocapture`
- `HONE_FMP_API_KEY=<config value> cargo test -p hone-event-engine pollers::macro_events::tests::live_fmp_macro_smoke --lib -- --ignored --nocapture`
- Live FMP digest probe against `telegram::::8039067465` holdings (`TEM,RKLB,MU,CAI,COHR,GOOGL,AAPL,SNDK,GEV,AAOI,VST,BE,AMD`) produced 50 news events, 737 macro events, 5 holding-matched news rows, and channel-rendered source-host links.
- `cargo test -p hone-scheduler execution_detail_with_delivery_key --lib -- --nocapture`
- `cargo test -p hone-memory execution_terminal_event_ --lib -- --nocapture`
- `cargo check -p hone-memory -p hone-scheduler -p hone-feishu --tests`
- `cargo test -p hone-channels finalize_agent_response_marks_planning_sentence_as_failure -- --nocapture`
- `cargo test -p hone-channels transitional_clarification_question_is_not_treated_as_planning_sentence -- --nocapture`
- `cargo test -p hone-channels finalize_agent_response_keeps_user_facing_clarification_question -- --nocapture`
- `cargo check -p hone-channels --tests`
- `rustfmt --edition 2024 crates/hone-channels/src/runtime.rs crates/hone-channels/src/agent_session/tests.rs`
- `cargo test -p hone-channels session_event_emitter_ -- --nocapture`
- `cargo check -p hone-channels --tests`
- `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session/emitter.rs crates/hone-channels/src/agent_session/tests.rs`
- `cargo test -p hone-channels concise_local_file_answer_can_return_directly -- --nocapture`
- `cargo test -p hone-channels multiline_local_file_summary_still_requires_answer_stage -- --nocapture`
- `cargo test -p hone-channels runners::multi_agent::tests -- --nocapture`
- `cargo check -p hone-channels --tests`

Known verification limitation:

- `bash scripts/ci/check_fmt_changed.sh` cannot run under the system Bash 3 environment because it uses `mapfile`; changed Rust files were formatted directly with `rustfmt --edition 2024`.
- 本轮再次尝试 `bash scripts/ci/check_fmt_changed.sh`，仍因系统 Bash 3 缺少 `mapfile` 失败；已改用 `rustfmt --edition 2024 --check` 覆盖本轮改动 Rust 文件。
- `cargo fmt --all -- --check` still fails on pre-existing formatting drift outside this patch (`crates/hone-channels/src/agent_session/tests.rs`, `crates/hone-core/src/quiet.rs`, `crates/hone-event-engine/src/digest/curation.rs`, `crates/hone-event-engine/src/prefs.rs`, `crates/hone-event-engine/src/router/policy.rs`, `crates/hone-tools/src/notification_prefs_tool.rs`); touched event-engine files pass direct `rustfmt --edition 2024 --check`.

## Documentation Sync

- Update `docs/bugs/README.md` and touched bug documents with status, fix notes, and verification.
- Update this plan while work is active.
- When the batch is closed or paused, write a handoff and either keep this plan active or move it to archive with an `docs/archive/index.md` entry.

## Risks / Open Questions

- Some active bugs depend on production credentials, live Feishu/Telegram/OpenRouter behavior, or model output quality; these should be hardened through contracts and evidence, not hidden behind brittle provider-specific special cases.
- The active queue spans several ongoing plans, so edits must avoid reverting unrelated in-progress work.
