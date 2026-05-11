# Code Quality Patrol Findings

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

- status: open
- direction: 错误与日志质量
- evidence: `DeepResearchTool::execute` returns `{ "success": false, "error": "...", "raw": raw }` when the configured research API responds with a non-2xx status.
- risk: the research API is an external/internal service boundary, and raw error payloads can contain backend-only diagnostics, request metadata, or provider-specific details that are not meant for the final chat response. Removing `raw` directly could break an operator debugging workflow, so this needs an explicit UX/logging split rather than a drive-by patch.
- suggested_fix: keep the user/tool result to a sanitized status/message and move the full raw payload to an operator-only trace or debug log with size limits and secret redaction; add tests for non-2xx responses that assert the tool response omits backend-only fields while logs retain enough diagnostics.

## 2026-05-11 - 前端状态复杂度

### `packages/app/src/pages/settings.tsx` still combines several independent state machines in one page component

- status: open
- direction: 前端状态复杂度
- evidence: after the low-risk check-status cleanup, `settings.tsx` is still `2670` lines and owns language saves, agent runner/config edits, web invite CRUD, data API key lists, notification preferences, and channel settings in one Solid component. The web invite flow alone has six action handlers around lines 621-790, while channel settings repeatedly patch `channelDraft` for Feishu, Discord, Telegram, and iMessage around lines 2243-2636.
- risk: small UI edits now require reasoning across unrelated state machines, shared message/error signals, clipboard side effects, backend saving state, and tab visibility. Directly extracting everything in one patrol would be high risk because invite CRUD and channel settings touch externally visible configuration and secrets/tokens.
- suggested_fix: split the page into behavior-preserving child components by tab (`AgentSettingsPanel`, `DataApiKeysPanel`, `WebInvitePanel`, `ChannelSettingsPanel`) and move local state/helpers with each panel. Start with tests or smoke coverage around runner selection, invite action state, and channel draft round-trip before changing component boundaries.
