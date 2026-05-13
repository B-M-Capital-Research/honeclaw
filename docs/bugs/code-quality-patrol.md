# Code Quality Patrol Findings

## 2026-05-13 - 错误与日志质量

### ACP parse-error audit records keep full raw protocol lines

- status: open
- direction: 错误与日志质量
- evidence: `crates/hone-channels/src/runners/acp_common/log.rs` documents `acp-events.log` as storing request/response/notification originals, and `log_acp_raw_parse_error` writes `"raw_line": raw_line` without redaction or length bounding. The same module already redacts and bounds stderr details for user-visible timeout/error messages through `redact_common_stderr_secrets` and `tail_for_log`.
- risk: parse-error lines can include malformed JSON-RPC payloads, tool arguments, paths, or copied provider diagnostics; changing this directly would alter the operator audit contract and may reduce replay/debug value for ACP runner incidents.
- suggested_fix: introduce an explicit audit policy for ACP event logs: either keep raw protocol payloads in a restricted artifact and add a separate redacted preview field, or replace parse-error `raw_line` with bounded/redacted text plus a documented opt-in raw capture mode. Cover with tests for Bearer/query/JSON secret redaction and long malformed lines.

## 2026-05-13 - 复杂度热点

### `crates/hone-channels/src/session_compactor.rs` session compaction orchestration is oversized

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-channels --tests -- -W clippy::cognitive_complexity -W clippy::too_many_lines` reports `SessionCompactor::compact_session` at `329/100` lines.
- risk: the function owns eligibility checks, transcript loading, prompt construction, auxiliary LLM execution, persistence, audit recording, and fallback/error handling in one async path. A patrol-sized extraction could change compaction trigger semantics, stored summary content, or audit side effects.
- suggested_fix: first split behavior-preserving private helpers for transcript selection, prompt/message assembly, summary persistence, and audit emission. Keep the public orchestration return type unchanged, then add focused tests around forced compaction, no-op eligibility, audit failure handling, and summary sanitization before larger refactors.

## 2026-05-12 - 死代码与废弃路径

### Public password storage helpers remain after public SMS login replaced password flows

- status: open
- direction: 死代码与废弃路径
- evidence: after the public app moved to `/api/public/auth/sms/send` and `/api/public/auth/sms/login`, `rg` finds no routed backend handler or frontend caller for password login, set-password, or change-password. The remaining password surface is now deeper compatibility/storage code: `memory/src/password.rs`, `memory/src/web_auth.rs` methods `find_by_phone_password_ready` / `set_password` / `change_password`, the `password_hash` / `password_set_at` columns, `PublicAuthUserInfo.has_password`, and the `argon2` / `password-hash` dependencies.
- risk: removing this directly could break existing databases with historical password columns, public API consumers that still read `has_password`, or downstream users of the public `hone_memory::password` module. Keeping it indefinitely leaves a stale auth model beside the SMS-only public login path.
- suggested_fix: make an explicit compatibility decision for historical password accounts. If password login is no longer supported, document the migration, stop exposing `has_password`, remove the public password module and web_auth password helpers in one focused change, and keep a SQLite migration/compatibility test for old databases. If compatibility is still required, reintroduce an explicit routed legacy endpoint or mark the storage helpers as retained compatibility code.

## 2026-05-12 - 死代码与废弃路径

### Desktop OpenRouter settings commands appear orphaned after frontend settings consolidation

- status: open
- direction: 死代码与废弃路径
- evidence: `packages/app/src/lib/backend.ts` no longer has callers for `loadDesktopOpenRouterSettings` / `saveDesktopOpenRouterSettings`, and `rg` finds no frontend references to those wrappers. The Desktop sidecar still registers `get_openrouter_settings` / `set_openrouter_settings` in `bins/hone-desktop/src/commands.rs` and keeps `OpenRouterSettings` plus the implementation pair in `bins/hone-desktop/src/sidecar.rs`.
- risk: removing the Rust commands directly could break older Desktop bundles or any external automation still invoking those Tauri command names. Keeping them indefinitely leaves a stale config-write path beside the newer agent/profile settings flow.
- suggested_fix: decide whether Desktop command compatibility for `get_openrouter_settings` / `set_openrouter_settings` is still required. If not, remove the commands, sidecar helpers, and tests/docs in one Desktop-focused cleanup; if compatibility is required, mark them as deprecated and route operators to the current agent/profile settings flow.

## 2026-05-12 - 错误与日志质量

### `crates/hone-channels/src/runners/gemini_cli.rs` exit errors can still surface full stderr upstream

- status: done
- direction: 错误与日志质量
- evidence: `stream_gemini_prompt` now truncates the warning log for non-empty stderr, but the `ExitFailure` error still formats `stderr_trimmed` into `AgentSessionError.message` when Gemini exits unsuccessfully before producing streamed output.
- risk: stderr is useful for operator diagnosis, but CLI stderr can also include verbose provider diagnostics, local paths, or copied request context. Changing the user-visible error string directly in a patrol could remove needed recovery detail or break tests/ops expectations, so this needs an explicit split between user-safe failure text and operator diagnostics.
- suggested_fix: introduce a small helper that returns both a user-safe stderr summary and a bounded operator stderr preview. Use the safe summary in `AgentSessionError.message`, emit the bounded preview through tracing or audit, and add tests for long stderr plus empty-output exit failures.
- resolution: 2026-05-13 patrol changed Gemini CLI exit failures to use bounded, redacted stderr details in `AgentSessionError.message` and tracing previews; covered by `stream_gemini_prompt_bounds_exit_stderr`.

## 2026-05-12 - 复杂度热点

### `crates/hone-channels/src/agent_session/core.rs` agent run path is too broad for local cleanup

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-channels --tests -- -W clippy::cognitive_complexity -W clippy::too_many_lines` reports `AgentSession::run` at cognitive complexity `51/25` and `431/100` lines.
- risk: the run path currently owns quota/domain short-circuiting, persisted message repair, runner execution, stream delivery, final response persistence, and audit emission in one async function. A drive-by extraction could change message ordering, quota semantics, or streamed-vs-final delivery behavior.
- suggested_fix: split behavior-preserving private helpers around pre-run guard decisions, execution request assembly, stream/final response delivery, and persistence/audit finalization. Add focused tests for domain short-circuit, streamed output, and final message persistence before moving side effects.

### `crates/hone-channels/src/runners/opencode_acp.rs` runner loop mixes process setup, stream protocol, and transcript finalization

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-channels --tests -- -W clippy::cognitive_complexity -W clippy::too_many_lines` reports `run_opencode_acp` at cognitive complexity `32/25` and `284/100` lines.
- risk: the function currently resolves the bundled command, prepares environment and working directories, starts stdio JSON-RPC, streams ACP events, tracks session metadata, persists tool calls, and finalizes the runner response in one async path. A patrol-sized extraction could change startup diagnostics, event ordering, or tool-call transcript behavior.
- suggested_fix: split behavior-preserving private helpers for command/process setup, ACP initialize/session-new handshakes, prompt send/wait, and response finalization. Keep event-state mutation centralized until tests cover resumed sessions, tool-call updates, and error exits.

## 2026-05-11 - 死代码与废弃路径

### `crates/hone-channels` exposes internal runner and execution types as unreachable `pub`

- status: done
- direction: 死代码与废弃路径
- evidence: `RUSTFLAGS='-W unreachable-pub' cargo check --workspace --all-targets --exclude hone-desktop` reports 43 unreachable `pub` warnings in `crates/hone-channels`, concentrated in `execution.rs`, `prompt_audit.rs`, `runners.rs`, runner implementations, `runners/types.rs`, and `session_compactor.rs`.
- risk: these items are not externally reachable today, but the `pub` surface makes internal runner/execution boundaries look broader than they are. Drive-by fixes are risky because the warnings span runner factory wiring, prompt audit persistence, session compaction, and tests that may rely on current module visibility.
- suggested_fix: handle as a focused `hone-channels` visibility pass: first map which items are used only by sibling modules or tests, then narrow them to `pub(crate)` or `pub(super)` in coherent groups and validate with `cargo check -p hone-channels --tests` plus the runner/session focused tests.
- resolution: 2026-05-13 patrol narrowed the unreachable `pub` surface across execution, prompt audit, runner implementations, runner reexports, runner timeouts, and session compaction. `AgentRunner` and request/result traits remain public because they are still part of the visible runner factory/MCP helper boundary. `RUSTFLAGS='-W unreachable-pub' cargo check -p hone-channels --tests` now passes with no warnings.

## 2026-05-11 - 复杂度热点

### `crates/hone-event-engine/src/engine.rs` event-engine startup orchestration is oversized

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-event-engine --tests -- -W clippy::too_many_lines -W clippy::cognitive_complexity` reports `Engine::start` at cognitive complexity `70/25` and `558/100` lines.
- risk: startup now owns source construction, registry refresh, poller scheduling, sink wiring, digest jobs, and long-running task orchestration in one function. Local fixes to one source or sink can accidentally affect startup ordering or cancellation behavior elsewhere.
- suggested_fix: split startup into behavior-preserving private builders for subscriptions/registry refresh, source task spawning, digest scheduling, and sink setup; keep `Engine::start` as orchestration glue and add focused tests around enabled-source combinations before moving logic.

### `crates/hone-event-engine/src/unified_digest/scheduler.rs` digest tick path is too broad

- status: open
- direction: 复杂度热点
- evidence: the same clippy scan reports `UnifiedDigestScheduler::tick_once` at cognitive complexity `64/25` and `343/100` lines; `get_or_build_global_cache` is `160/100` lines and `run_quiet_flush` is `132/100` lines.
- risk: tick scheduling, cache construction, per-actor filtering, quiet-hour flushing, and delivery decisions are tightly interleaved. This makes digest timing changes hard to review and raises regression risk around duplicate sends or missed quiet-hour flushes.
- suggested_fix: extract pure planning helpers for slot eligibility, global cache lookup/build, actor delivery plan, and quiet-hour flush decisions; preserve storage and sink side effects at the edges, then cover each helper with deterministic unit tests.

### `crates/hone-channels/src/scheduler.rs` scheduled execution entrypoint mixes guards and side effects

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-channels --tests -- -W clippy::too_many_lines -W clippy::cognitive_complexity` reports `execute_scheduler_event` at cognitive complexity `37/25` and `303/100` lines.
- risk: quiet-hour bypass, heartbeat execution, failure rollback, delivery metadata, persistence, and user-visible status are coupled in one async path. Past scheduler bugs often sit at those boundaries, so direct large edits are high regression risk.
- suggested_fix: split into a deterministic execution plan plus small side-effect functions for quiet-hour skip, heartbeat run, persistence rollback, and delivery recording; add scheduler tests around each plan outcome before changing orchestration.

### `bins/hone-feishu/src/handler.rs` inbound message handler is too broad for safe drive-by cleanup

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-feishu --tests -- -W clippy::too_many_lines -W clippy::cognitive_complexity` reports `process_incoming_message` at cognitive complexity `182/25` and `704/100` lines. The same path includes repeated failure and empty-response fallback send branches around `failure_fallback` / `empty_fallback` logging.
- risk: one function owns Feishu ingress guards, contact resolution, actor/session identity, attachment handling, prompt setup, streaming CardKit updates, persistence, and final reply delivery. A direct refactor can easily change externally visible channel behavior or miss a failure-path log/persist boundary.
- suggested_fix: first extract behavior-preserving private helpers for inbound context construction, attachment/user-input assembly, placeholder setup, and final reply/fallback delivery. Add focused tests around group vs direct message context, panic/failure fallback, and placeholder-vs-CardKit delivery before changing orchestration.

## 2026-05-11 - 错误与日志质量

### `crates/hone-tools/src/deep_research.rs` returns raw backend error payloads to the tool caller

- status: done
- direction: 错误与日志质量
- evidence: `DeepResearchTool::execute` returns `{ "success": false, "error": "...", "raw": raw }` when the configured research API responds with a non-2xx status.
- risk: the research API is an external/internal service boundary, and raw error payloads can contain backend-only diagnostics, request metadata, or provider-specific details that are not meant for the final chat response. Removing `raw` directly could break an operator debugging workflow, so this needs an explicit UX/logging split rather than a drive-by patch.
- suggested_fix: keep the user/tool result to a sanitized status/message and move the full raw payload to an operator-only trace or debug log with size limits and secret redaction; add tests for non-2xx responses that assert the tool response omits backend-only fields while logs retain enough diagnostics.
- resolution: 2026-05-13 patrol removed the non-2xx `raw` tool response, added bounded/redacted operator response previews, redacted Bearer/query/JSON secret fields, and covered the HTTP error path with `execute_http_error_hides_raw_payload`.

## 2026-05-11 - 前端状态复杂度

### Public/admin mainline views duplicate parallel state machines

- status: open
- direction: 前端状态复杂度
- evidence: `packages/app/src/pages/public-portfolio.tsx` `PortfolioContextView` and `packages/app/src/components/user-mainline-view.tsx` `UserMainlineView` both maintain the same shape of state: context payload, loading/error, refresh progress/message, profile modal open state, selected ticker, load/refresh handlers, and derived profile ticker sets. The public view is session-scoped while the admin view is actor-scoped, so the duplication is not a safe one-file cleanup.
- additional_evidence: `packages/app/src/pages/public-portfolio.tsx` `ProfileModal` still uses a hand-rolled `maybeFetch()` call to emulate reactive fetching, while `packages/app/src/components/user-mainline-view.tsx` `ProfileMarkdownModal` uses `createEffect`. Aligning this may change modal loading behavior, so it needs targeted UI/model coverage rather than a drive-by patrol edit.
- risk: future changes to mainline refresh, profile modal loading, skipped ticker handling, or error presentation can drift between public and admin surfaces. Direct extraction in a patrol could accidentally mix session auth with actor-scoped admin APIs.
- suggested_fix: introduce a small shared model/helper for the pure view state and ticker derivation first, then consider a shared presentational panel that receives API callbacks for public vs admin data sources. Keep API/auth boundaries explicit and cover both public and admin refresh paths with smoke or model tests before extracting the UI.
- progress: 2026-05-13 patrol extracted the pure derived profile ticker set into `profileTickerSet` with unit coverage. The cross-view load/refresh/modal state machines remain open because they still cross public-session and admin-actor API boundaries.

### `packages/app/src/pages/settings.tsx` still combines several independent state machines in one page component

- status: open
- direction: 前端状态复杂度
- evidence: after two low-risk cleanup passes, `settings.tsx` is still `2600` lines and owns language saves, agent runner/config edits, web invite CRUD, data API key lists, notification preferences, and channel settings in one Solid component. The web invite flow still has six action handlers around lines 589-765, while channel settings still keep Feishu, Discord, Telegram, and iMessage field state in the same page even though simple draft patches now flow through `updateChannelDraft`.
- risk: small UI edits now require reasoning across unrelated state machines, shared message/error signals, clipboard side effects, backend saving state, and tab visibility. Directly extracting everything in one patrol would be high risk because invite CRUD and channel settings touch externally visible configuration and secrets/tokens.
- suggested_fix: split the page into behavior-preserving child components by tab (`AgentSettingsPanel`, `DataApiKeysPanel`, `WebInvitePanel`, `ChannelSettingsPanel`) and move local state/helpers with each panel. Start with tests or smoke coverage around runner selection, invite action state, and channel draft round-trip before changing component boundaries.
