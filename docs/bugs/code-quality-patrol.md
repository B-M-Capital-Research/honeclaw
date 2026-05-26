# Code Quality Patrol Findings

## 2026-05-26 - 复杂度热点

### `hone-cli configure` still mixes prompts, section routing, and mutation assembly

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-cli --bin hone-cli --tests -- -W clippy::too_many_lines -W clippy::cognitive_complexity` reports `bins/hone-cli/src/configure.rs::run_configure` at `357/100` lines. The same scan showed the now-fixed `bins/hone-cli/src/mutations.rs::build_model_mutations` candidate, but `run_configure` still owns section selection, prompt defaults, secret prompts, per-channel allowlist prompting, provider key parsing, mirror writes, and final canonical-config mutation application in one function.
- risk: a drive-by split could change interactive prompt order, default values, secret-presence handling, or which canonical config paths are mirrored together. This path is operator-facing and shares semantics with `hone-cli models set`, `hone-cli channels set`, and onboarding.
- suggested_fix: in a focused CLI-configure pass, extract behavior-preserving private builders for agent, channel, and provider sections; reuse the mutation helpers from `bins/hone-cli/src/mutations.rs` where possible; keep prompt order stable; then validate with focused configure/mutation tests plus `cargo check -p hone-cli --tests`.

### Channel onboarding builder owns too many recovery and mutation branches

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-cli --bin hone-cli --tests -- -W clippy::too_many_lines -W clippy::cognitive_complexity` reports `bins/hone-cli/src/onboard.rs::build_channel_onboard_mutations` at `265/100` lines. The function combines platform skipping, enable prompts, prerequisite copy, allowlist warnings, required-field recovery, disabled-channel detection, chat-scope prompts, per-channel allowlist prompts, iMessage target-handle prompting, and final mutation assembly.
- risk: the function is interactive and stateful: a user can abandon one required field and the code must reset that channel to `enabled=false` without losing earlier channels. A local split must preserve prompt order, recovery wording, `enabled_channels` side effects, and per-channel mutation order.
- suggested_fix: extract behavior-preserving private helpers for channel enablement, required-field collection, disabled-channel detection, chat-scope mutation, and allowlist prompts. Keep the current `ChannelOnboardSpec` data shape initially, add tests for required-field abandon paths where possible, then rerun `cargo check -p hone-cli --tests`.

### Top-level CLI dispatch is too broad for safe patrol-sized splitting

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-cli --bin hone-cli --tests -- -W clippy::too_many_lines -W clippy::cognitive_complexity` reports `bins/hone-cli/src/main.rs::run_cli` at cognitive complexity `33/25` and `340/100` lines. The function parses top-level commands and owns config get/set/unset/validate, models status/set, channel list/set/toggle/targets, web, start, cleanup, probe, doctor, and onboard dispatch paths in one match tree.
- risk: splitting this directly can change command output ordering, JSON/text behavior, language resolution, mutation application messages, or default `Chat` behavior. It also intersects with large enum-size clippy warnings around `Commands`, `ModelsCommands`, and `ChannelsCommands`, which may require Clap boxing decisions rather than a mechanical extraction.
- suggested_fix: in a focused CLI-entrypoint pass, extract private `run_config_command`, `run_models_command`, `run_channels_command`, and `run_web_command` helpers while keeping `Cli::parse()` and the default chat branch in `run_cli`. Preserve existing parse tests and add focused output smoke tests for config/model/channel text versus JSON branches before considering enum boxing.

## 2026-05-26 - 用户文案

### Symbol drawer bypasses the admin bilingual content tree

- status: open
- direction: 用户文案
- evidence: `packages/app/src/components/symbol-drawer.tsx` is an admin-console surface opened from user/profile/research flows, but visible copy is hardcoded in Chinese instead of routing through `packages/app/src/lib/admin-content/*`. Examples include tab labels (`公司画像`, `研究记录`, `相关会话`, `操作`), fallback states (`先选定用户`, `该用户暂无会话`), watchlist/research actions, feedback text, and the close button `aria-label="关闭"`.
- risk: English-locale admin users can switch most console navigation and page copy to English, then open the symbol drawer and get a mixed-language workflow. Migrating it directly in this patrol would touch several interaction states, feedback messages, date formatting, and navigation labels in one component, so it needs a focused UI-content pass.
- suggested_fix: add a small `admin-content/symbol-drawer.ts` tree, wire `SymbolDrawer` and its tab subcomponents through that content, and keep dynamic labels (`{symbol}`, `{user_id}`, counts, timestamps) as placeholders. Validate with the existing admin content shape test plus a focused component/model smoke for profile, research, sessions, and actions states.

### Channel status badge ignores the admin locale switch

- status: open
- direction: 用户文案
- evidence: `packages/app/src/components/channel-status-badge-model.ts` and `packages/app/src/components/channel-status-badge.tsx` return and render Chinese strings directly for global admin chrome copy such as `运行中`, `管理端后端未连接`, `渠道加载中`, `系统连接`, `渠道监听`, `清理多余进程`, and duplicate-process hints. The sidebar and page titles around the badge already use `packages/app/src/lib/admin-content/shared.ts`, so switching the admin console to English leaves the top-right runtime status mixed in Chinese.
- risk: this is global chrome shown on every admin page and has test-covered model helpers. A safe fix needs to preserve the current status derivation while injecting locale-specific labels into both the model tests and component rendering.
- suggested_fix: move status labels, connection labels, summary templates, and cleanup button/hint text into the shared admin content tree or a dedicated `admin-content/channel-status.ts`. Then update the model helpers to accept a copy bundle or return stable status tokens that the component formats through content, with tests for both locales.

## 2026-05-26 - 注释准确性

### Periodic task convention and mainline distill cron disagree on missed-tick behavior

- status: open
- direction: 注释准确性
- evidence: `docs/conventions/periodic_tasks.md` says periodic loops must set `MissedTickBehavior::Delay` and describes `Delay` as the shared convention for avoiding burst recovery after long work. `crates/hone-event-engine/src/global_digest/mainline_cron.rs` instead sets `MissedTickBehavior::Skip` in `distill_cron_loop`, while the same document lists `mainline_cron` among the periodic tasks in scope.
- risk: changing `Skip` to `Delay` directly could alter how missed hourly distillation windows recover after machine sleep or long-running LLM calls; changing the convention directly would weaken a cross-task workflow rule. This needs an explicit decision about whether mainline distillation is an intentional exception or should follow the standard loop behavior.
- suggested_fix: in a focused periodic-task pass, decide whether `mainline_cron` should use `Delay` like other internal tasks or stay as a documented exception. If aligning behavior, cover machine-sleep / long-tick recovery expectations with a small unit or manual regression note; if keeping `Skip`, update `docs/conventions/periodic_tasks.md` with a bounded exception and rationale.

## 2026-05-23 - 测试可维护性

### Hone-tools skill script tests trip strict clippy on async env locking and module layout

- status: open
- direction: 测试可维护性
- evidence: `cargo clippy -p hone-tools --all-targets --no-deps -- -D warnings` reports `clippy::items-after-test-module` in `crates/hone-tools/src/skill_tool.rs` because the large `#[cfg(test)] mod tests` sits before later production helpers and the `impl Tool for SkillTool`. The same strict run reports `clippy::await-holding-lock` for several async skill-tool tests that keep the test-only `std::sync::MutexGuard` from `env_lock()` across `.await` while serializing environment mutations.
- risk: the current tests pass and the lock intentionally serializes process-wide env changes, but the layout and sync-guard pattern make future strict clippy adoption noisy. A drive-by move would create a large diff around skill execution helpers and could accidentally weaken env isolation in concurrent async tests.
- suggested_fix: in a focused test-maintenance pass, move the skill-tool test module to the end of the file or split production helpers before tests, then replace the sync env guard pattern with a small async-aware test harness or a scoped helper that performs env mutation and cleanup around awaited calls without holding `std::sync::MutexGuard` across `.await`. Rerun `cargo test -p hone-tools skill_tool` and the skill-runtime CI regression scripts.

### Ignored live smoke tests remain scattered outside manual regression entry points

- status: open
- direction: 测试可维护性
- evidence: `AGENTS.md` and `docs/invariants.md` say external-account, external-CLI, or local-machine-state checks should live under `tests/regression/manual/`, but `rg "#\\[ignore\\]|live_|HONE_.*KEY|HONE_.*TOKEN"` still finds credential-backed ignored tests in crate modules. Examples include `crates/hone-web-api/src/aliyun_captcha.rs::live_probe_smoke`, `crates/hone-web-api/src/aliyun_sms.rs::live_send_verify_code_smoke`, and event-engine poller smokes such as `crates/hone-event-engine/src/pollers/news.rs::live_fmp_news_smoke`, `pollers/price.rs::live_fmp_price_smoke`, `pollers/earnings.rs::live_fmp_earnings_smoke`, `pollers/analyst_grade.rs::live_fmp_analyst_grade_smoke`, `pollers/corp_action.rs::live_fmp_corp_action_smoke`, `pollers/earnings_surprise.rs::live_fmp_earnings_surprise_smoke`, and `pollers/macro_events.rs::live_fmp_macro_smoke`.
- risk: these tests are ignored and do not block CI, but their command surface is hard to discover from `tests/regression/manual/` and remains mixed into unit-test modules. Moving them in a patrol-sized patch could lose useful smoke commands, required environment notes, or fixture setup for live provider checks.
- suggested_fix: create manual regression wrappers for Aliyun SMS/Captcha and event-engine FMP poller smokes, preserving required env vars, command examples, and expected success criteria. Keep deterministic parsing/auth/signature coverage in Rust unit tests, then update `tests/regression/README.md` if new manual entry points are added.

## 2026-05-22 - 注释准确性

### Global digest broadcast dedup channel no longer matches unified scheduler audit writes

- status: open
- direction: 注释准确性
- evidence: `crates/hone-event-engine/src/global_digest/collector.rs` documents cross-batch dedup against `GLOBAL_DIGEST_CHANNEL = "global_digest"` and `excludes_already_broadcast_event_ids` only logs that channel in the fixture. The current unified digest path in `crates/hone-event-engine/src/unified_digest/scheduler.rs` logs delivered global items under `delivery_log.channel = "global_digest_item"` and filtered items under the same channel; `rg` finds no production writer for `"global_digest"`. As a result the collector's `broadcasted_event_ids_since(GLOBAL_DIGEST_CHANNEL, ...)` appears to miss the channel that production writes now use.
- risk: changing the constant directly would alter global-news cross-batch dedup behavior and could hide or newly suppress stories across actors and slots, so it needs a focused event-engine regression pass rather than a comment-only patrol fix.
- suggested_fix: decide whether the canonical broadcast-dedup channel should be `global_digest_item` or a separate broadcast-level `global_digest` marker. Then align `GLOBAL_DIGEST_CHANNEL`, scheduler delivery-log writes, collector tests, and any audit/report wording in one behavior-preserving change with coverage for delivered and focus-filtered global picks.

## 2026-05-22 - 测试可维护性

### Event-engine live integration checks still live in the crate unit-test module

- status: open
- direction: 测试可维护性
- evidence: `AGENTS.md` and `docs/invariants.md` both say external-account or local-machine-state checks should live under `tests/regression/manual/`, but `crates/hone-event-engine/src/tests.rs` still contains ignored live tests such as `live_engine_e2e`, `live_telegram_push_demo`, `live_telegram_push_llm_polished_demo`, `live_portfolio_backtest_push`, and `live_social_engine_e2e`. These tests read `HONE_FMP_API_KEY`, `HONE_TG_BOT_TOKEN`, `HONE_TG_CHAT_ID`, `HONE_OPENROUTER_KEY`, local `data/portfolio/...`, or live Telegram/FMP network state directly from the crate test module.
- risk: the tests are ignored, so they do not block CI, but their placement makes the manual verification contract hard to discover from `tests/regression/manual/` and keeps long external workflows mixed with unit/integration test code. Moving them directly in a patrol could lose useful operator commands or accidentally change the live smoke setup.
- suggested_fix: migrate the live event-engine checks into one or more `tests/regression/manual/test_event_engine_*.sh` wrappers or documented manual fixtures, keeping deterministic contract/unit coverage in Rust. Preserve the current trigger commands, required env vars, and expected artifacts, then update `docs/repo-map.md` if the manual regression entry points change.

## 2026-05-14 - 配置文档漂移

### `hone-cli onboard` does not validate multi-agent's OpenCode answer dependency

- status: open
- direction: 配置文档漂移
- evidence: `crates/hone-channels/src/core/bot_core.rs` builds the `multi-agent` answer stage from `agent.opencode` / `agent.multi_agent.answer.*` and runs it through `OpencodeAcpRunner`, while `crates/hone-core/src/config/agent.rs` maps `AgentRunnerKind::MultiAgent::cli_probe()` to `opencode --version`. In contrast, `bins/hone-cli/src/onboard.rs` currently returns `None` for `OnboardRunnerKind::MultiAgent::binary_probe()`, so selecting multi-agent does not check that `opencode` exists even though runtime needs it for the answer stage.
- risk: new users can choose multi-agent, complete onboarding, and only discover the missing local `opencode` dependency at first runtime use. Changing the wizard probe directly alters onboarding behavior and should be handled with a focused CLI UX/test pass rather than a documentation-only patrol.
- suggested_fix: align `OnboardRunnerKind::MultiAgent` with runtime runner requirements: probe `opencode --version`, show the same install/setup guidance used for OpenCode ACP, and add a CLI test that locks the multi-agent probe contract against `AgentRunnerKind::MultiAgent::cli_probe()`.

### Event-engine admin writes still use config overlay files after canonical-config migration

- status: open
- direction: 配置文档漂移
- evidence: `docs/invariants.md` says no steady-state runtime path should read or write sibling `.overrides.yaml` files anymore, and `docs/repo-map.md` says legacy `data/runtime/config_runtime.yaml` and sibling `.overrides.yaml` should not be recreated. But `crates/hone-web-api/src/routes/event_engine_admin.rs` still documents and implements PUT/POST/DELETE writes through `apply_overlay_mutations`, and its user-facing restart hint says changes were written to `config.overrides.yaml`. `crates/hone-core/src/config/mutation.rs` still documents `apply_overlay_mutations` as the management-surface path for runtime knobs such as schedules, model switching, and RSS feeds. `crates/hone-web-api/src/routes/meta.rs` also uses `apply_overlay_mutations` for `PUT /api/meta/language`.
- risk: operators can see two conflicting configuration contracts: most CLI/Desktop settings mutate canonical `config.yaml`, while event-engine admin changes still land in an overlay file that the long-term docs say should not exist. Moving these writes directly to canonical config could rewrite comments or change restart/apply behavior, so it needs a focused config-apply design pass.
- suggested_fix: decide whether event-engine admin should migrate to canonical `config.yaml` mutations like other settings surfaces or whether the invariant needs an explicit, temporary exception. If migrating, update the route implementation, restart hint, config tests, and repo-map/invariants together; if retaining overlay semantics, document its bounded scope and ownership.

## 2026-05-14 - 复杂度热点

### Company profile import parsing and resolution orchestration are oversized

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-memory --tests -- -W clippy::cognitive_complexity -W clippy::too_many_lines` reports `CompanyProfileStorage::apply_import_resolution` at `107/100` lines and `parse_company_profile_bundle` at `106/100` lines. The same module also had a duplicated conflict-reason branch that the 2026-05-14 patrol simplified, but the import orchestration remains broad.
- risk: these paths own conflict preview reuse, bundle parsing, resolution strategy branching, profile writes, event import decisions, zip manifest validation, markdown parsing, and event sorting. A drive-by split could change import conflict semantics, overwrite behavior, or accepted bundle shape.
- suggested_fix: split behavior-preserving private helpers around non-conflict resolution result construction, conflicted strategy application, manifest/profile entry validation, and parsed-event assembly. Keep the public result structs unchanged, then cover replace/merge/skip, plain markdown import, duplicate profile paths, and invalid manifest paths with the existing company profile test surface.

### Cron execution history terminal-row reconciliation is oversized

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-memory --tests -- -W clippy::cognitive_complexity -W clippy::too_many_lines` reports `CronJobStorage::record_execution_event` at `159/100` lines in `memory/src/cron_job/history.rs`.
- risk: the function owns SQLite connection selection, preview truncation, delivery-key terminal update, recent-started fallback update, and insert fallback. A patrol-sized split could accidentally create duplicate cron run rows or fail to finalize legacy started rows.
- suggested_fix: extract private helpers for terminal input detection, shared update parameters, delivery-key update, recent-started fallback update, and insert fallback. Preserve the current update-before-insert order, then rerun the existing heartbeat/terminal-row tests plus stale-started recovery coverage.

### Event-engine router dispatch mixes policy, prefs, and sink side effects

- status: open
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-event-engine --lib -- -W clippy::collapsible_if -W clippy::too_many_lines -W clippy::cognitive_complexity` reports `crates/hone-event-engine/src/router/dispatch.rs` `Router::dispatch` at cognitive complexity `60/25` and `391/100` lines.
- risk: the function owns global policy demotion, per-actor preference checks, same-symbol cooldown, price-band advance gating, quiet-hours holding, delivery-log writes, immediate sink sends, digest enqueue fallback, and status accounting in one async path. A drive-by split could change which users receive immediate alerts versus digest-only events, or alter delivery-log accounting.
- suggested_fix: split behavior-preserving private helpers around policy evaluation, per-actor route decision, quiet-hours hold decision, and final delivery/enqueue side effects. Keep the current sent/enqueued counters and log statuses intact, then cover high severity immediate delivery, quiet hold, cooldown demotion, price-band advance, and disabled prefs with the existing router test surface.

## 2026-05-13 - 用户文案

### Public portfolio page is not localized while the surrounding public site is bilingual

- status: open
- direction: 用户文案
- evidence: `packages/app/src/pages/public-portfolio.tsx` imports `PublicNav`, `PublicFooter`, and `PublicLoginForm`, but does not import `CONTENT` or `useLocale`; visible strings such as `查看画像`, `投资上下文`, `加载失败`, `立即刷新`, `整体投资风格`, and `公司画像 inventory` are hardcoded in Chinese. The adjacent public chat, login, home, roadmap, and contact surfaces already route visible copy through the bilingual `CONTENT` tree.
- risk: English-locale users can navigate from the bilingual public site into `/portfolio` and see a mixed-language account surface. Migrating this in a patrol-sized patch would touch many strings plus date formatting and refresh/error messages, with UI regression risk on an authenticated page.
- suggested_fix: add a focused public-portfolio localization pass: move portfolio copy and relative-date labels into `packages/app/src/lib/public-content.ts`, switch timestamps through locale-aware formatting, and validate `/portfolio` in both `zh` and `en` locales with a lightweight UI smoke or model test around loading, error, empty, and refreshed states.

## 2026-05-13 - 错误与日志质量

### ACP parse-error audit records keep full raw protocol lines

- status: done
- direction: 错误与日志质量
- evidence: `crates/hone-channels/src/runners/acp_common/log.rs` documents `acp-events.log` as storing request/response/notification originals, and `log_acp_raw_parse_error` writes `"raw_line": raw_line` without redaction or length bounding. The same module already redacts and bounds stderr details for user-visible timeout/error messages through `redact_common_stderr_secrets` and `tail_for_log`.
- risk: parse-error lines can include malformed JSON-RPC payloads, tool arguments, paths, or copied provider diagnostics; changing this directly would alter the operator audit contract and may reduce replay/debug value for ACP runner incidents.
- suggested_fix: introduce an explicit audit policy for ACP event logs: either keep raw protocol payloads in a restricted artifact and add a separate redacted preview field, or replace parse-error `raw_line` with bounded/redacted text plus a documented opt-in raw capture mode. Cover with tests for Bearer/query/JSON secret redaction and long malformed lines.
- resolution: 2026-05-14 patrol replaced parse-error `raw_line` with `raw_line_chars`, `raw_line_truncated`, and bounded/redacted `raw_line_preview`; the same pass redacted ACP stop-diagnostic `prompt_result` excerpts and ACP error-response messages.

## 2026-05-13 - 复杂度热点

### `memory/src/cron_job/storage.rs` cron job due-job selection mixes filtering and scheduling rules

- status: done
- direction: 复杂度热点
- evidence: `cargo clippy -p hone-channels --tests -- -W clippy::cognitive_complexity -W clippy::too_many_lines` reports `CronJobStorage::get_due_jobs` at cognitive complexity `27/25` and `138/100` lines while checking `hone-memory` as a dependency.
- risk: the function currently combines actor enumeration, per-job schedule matching, channel filtering, disabled-state filtering, and return shaping. A drive-by extraction could change scheduled delivery eligibility or make hidden cron jobs fire/skip unexpectedly.
- suggested_fix: split the pure eligibility checks into private helpers for actor/job iteration, channel match, schedule match, and disabled-state filtering. Keep storage reads and final return order unchanged, then cover with focused tests for multi-actor jobs, channel-restricted jobs, disabled jobs, and day/hour/minute boundary matching.
- resolution: 2026-05-14 patrol extracted channel, due-window, repeat/day, already-ran, dedup-key, and actor-list helpers while preserving storage reads and return order. `cargo clippy -p hone-memory --tests -- -W clippy::cognitive_complexity -W clippy::too_many_lines` no longer reports `CronJobStorage::get_due_jobs`.

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
- evidence: current `packages/app/src/lib/backend.ts` no longer exports `loadDesktopOpenRouterSettings` / `saveDesktopOpenRouterSettings`, and `rg` finds no frontend references to `get_openrouter_settings` / `set_openrouter_settings`. The remaining surface is the Desktop compatibility layer: `bins/hone-desktop/src/commands.rs` still registers `get_openrouter_settings` / `set_openrouter_settings`, and `bins/hone-desktop/src/sidecar.rs` still keeps `OpenRouterSettings` plus the implementation pair.
- risk: removing the Rust commands directly could break older Desktop bundles or any external automation still invoking those Tauri command names. Keeping them indefinitely leaves a stale config-write path beside the newer agent/profile settings flow, but the frontend wrapper cleanup itself is already done.
- suggested_fix: decide whether Desktop command compatibility for `get_openrouter_settings` / `set_openrouter_settings` is still required. If not, remove the commands, sidecar helpers, and tests/docs in one Desktop-focused cleanup; if compatibility is required, mark the Rust commands as deprecated compatibility shims and route operators to the current agent/profile settings flow.

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
- additional_evidence: public/admin mainline views still keep separate load/refresh/modal state machines even after the low-risk derived-state helpers were extracted. The remaining shared UI extraction would need to keep session-scoped public APIs separate from actor-scoped admin APIs.
- risk: future changes to mainline refresh, profile modal loading, skipped ticker handling, or error presentation can drift between public and admin surfaces. Direct extraction in a patrol could accidentally mix session auth with actor-scoped admin APIs.
- suggested_fix: introduce a small shared model/helper for the pure view state and ticker derivation first, then consider a shared presentational panel that receives API callbacks for public vs admin data sources. Keep API/auth boundaries explicit and cover both public and admin refresh paths with smoke or model tests before extracting the UI.
- progress: 2026-05-13 patrol extracted the pure derived profile ticker set into `profileTickerSet` with unit coverage. 2026-05-14 patrol added a shared `firstProfileTicker`, moved public refresh/timestamp/button derivation into `public-portfolio-model`, aligned the public profile modal fetch trigger with `createEffect`, and cleared selected modal tickers on close. 2026-05-23 patrol added shared holding-card and profile-inventory row derivation for public/admin mainline views. The cross-view load/refresh/modal state machines remain open because they still cross public-session and admin-actor API boundaries.

### `packages/app/src/pages/settings.tsx` still combines several independent state machines in one page component

- status: open
- direction: 前端状态复杂度
- evidence: after several low-risk cleanup passes, `settings.tsx` is still about `2600` lines and owns language saves, agent runner/config edits, web invite CRUD, data API key lists, notification preferences, and channel settings in one Solid component. The web invite flow still has multiple CRUD/copy handlers around lines 589-765, while channel settings still keep Feishu, Discord, Telegram, and iMessage field state in the same page even though simple draft patches now flow through `updateChannelDraft`.
- risk: small UI edits now require reasoning across unrelated state machines, shared message/error signals, clipboard side effects, backend saving state, and tab visibility. Directly extracting everything in one patrol would be high risk because invite CRUD and channel settings touch externally visible configuration and secrets/tokens.
- suggested_fix: split the page into behavior-preserving child components by tab (`AgentSettingsPanel`, `DataApiKeysPanel`, `WebInvitePanel`, `ChannelSettingsPanel`) and move local state/helpers with each panel. Start with tests or smoke coverage around runner selection, invite action state, and channel draft round-trip before changing component boundaries.
- progress: 2026-05-23 patrol moved the invite-list prepend/replace transforms into `settings-model` with tests. A later 2026-05-23 patrol consolidated FMP/Tavily API-key values plus visibility into one tested draft-state helper and wrapped repeated invite action bookkeeping in the page. The invite CRUD side effects, channel settings state, and panel/component split remain open.
