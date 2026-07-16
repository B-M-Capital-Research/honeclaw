# Archive Index

Last updated: 2026-07-16

## 2026-07-16

### Investment Guard Scheduler Routing Fix

- Status: done
- Date: 2026-07-16
- Plan: `docs/archive/plans/investment-guard-scheduler-routing-fix.md`
- Handoff: `docs/handoffs/2026-07-16-investment-guard-scheduler-routing-fix.md`
- Decision / ADR: follow-up scope clarification to `D-2026-07-15-03` in `docs/decisions.md`
- Related PRs / commits: this change set; follows incomplete mitigation `c776b808`
- Related runbooks / regressions: investment guard unit tests, full channel library tests, 12 CI-safe finance contracts, `hone-cli` build, isolated live scheduled-envelope probe, and runtime/API/storage/channel health checks
- Current conclusion: `repeat=daily/trading_day` was incorrectly parsed as ticker `REPEAT` because the direct single-stock guard scanned scheduler envelopes. Scheduler and heartbeat envelopes now bypass that interactive guard, generic report acronyms and multi-security inputs cannot masquerade as a single ticker, and search requires an exact symbol match. A live envelope containing `repeat=daily` plus “财报分析” completed successfully without any market-data preflight.
- Next entry point: `crates/hone-channels/src/investment_response_guard.rs` and `docs/handoffs/2026-07-16-investment-guard-scheduler-routing-fix.md`

## 2026-07-15

### Deep Single-Stock Evidence And Response Contract

- Status: done
- Date: 2026-07-15
- Plan: `docs/archive/plans/response-contract-enforcement.md`
- Handoff: `docs/handoffs/2026-07-15-response-contract-enforcement.md`
- Decision / ADR: `D-2026-07-15-01`, `D-2026-07-15-02`, and `D-2026-07-15-03` in `docs/decisions.md`
- Related PRs / commits: `c29de55c`
- Related runbooks / regressions: 117 core tests, 7 function-calling agent tests, full channel library tests, 12 CI-safe finance automation contracts, `hone-cli` build, and isolated live NBIS Web regression
- Current conclusion: the real NBIS incident was caused by the model ignoring a full prompt that had already been injected, not by prompt omission. Canonical `soul.md` and its runtime sync are restored, non-admin native-runner configuration routes through the actor-bound safety runner, and deep single-stock turns now prefetch same-symbol quote/profile/financial/news/calendar evidence before enforcing a nine-section final answer. Incomplete drafts are reset and retried once, then fail closed. The exact live question completed successfully with all sections after restart.
- Next entry point: `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/core.rs`, and `docs/handoffs/2026-07-15-response-contract-enforcement.md`

## 2026-07-13

### Public Community Original Assets And Navigation Repair

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-community-assets-navigation-fix.md`
- Handoff: `docs/handoffs/2026-07-12-public-community-readonly.md`
- Decision / ADR: N/A; source access controls remain the hard download boundary and immutable full-SHA objects remain the rollback boundary
- Related PRs / commits: `879e9722`, `af3cb605`, `7ab36682`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 118 core tests, 106 Web API tests with two credentialed tests ignored, 67 CLI tests, 242 Web tests, TypeScript check, public production build, 11 CI-safe regression scripts, desktop/390px browser QA, and the 651-file magic/size/SHA/OOXML audit
- Current conclusion: this follow-up supersedes the file/image counts in the 2026-07-12 community entry. The complete archive now has 649 content rows and 818 resources: 53 original-resolution images plus 765 file resources. Of those files, 651 verified originals (2,614,811,800 bytes) are stored in immutable R2 objects and linked from PG, 113 remain explicitly source-protected, and only resource 834 remains unresolved after independent visible-UI search. Desktop/mobile navigation includes a compact first-class Community tab, the backend runs the current `0.14.1` code, and production Pages serves `assets/index-BB8Wrwbl.js`.
- Next entry point: `docs/handoffs/2026-07-12-public-community-readonly.md`, `bins/hone-cli/src/cloud.rs`, `crates/hone-web-api/src/routes/public_community.rs`, and `packages/app/src/pages/public-community.tsx`

## 2026-07-12

### Public Read-only Community

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/public-community-readonly.md`, `docs/archive/plans/public-community-deployment-qa.md`
- Handoff: `docs/handoffs/2026-07-12-public-community-readonly.md`
- Decision / ADR: N/A; source-protected resources remain a hard no-download boundary
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, cloud doctor schema apply, 218 Rust tests (2 credentialed ignored), workspace check, 236 Web tests, TypeScript check, public production build, CI-safe regression suite, desktop/390px browser QA
- Current conclusion: the 616-row user-authorized archive is deployed as the shared authenticated `/community` timeline across Web/macOS/iOS. Content is newest-first and read-only; every source post remains one row with ordered media. The runtime is `0.14.1`, Cloudflare Pages serves `index-D-q3AOum.js`, R2-backed passive images/PDF use the hardened private preview route, and 764 protected source files deliberately remain metadata-only.
- Next entry point: `crates/hone-web-api/src/routes/public_community.rs` and `packages/app/src/pages/public-community.tsx`

### Public Chat Native Runner Streaming

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/public-chat-native-runner-streaming.md`
- Handoff: `docs/handoffs/2026-07-12-public-chat-startup-experience.md`
- Decision / ADR: `D-2026-07-12-03` in `docs/decisions.md`
- Related PRs / commits: `6d5075a4`, this CI follow-up change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 13 LLM tests, 7 function-calling tests, 500 channel tests, 101 Web API tests with two credentialed tests ignored, 218 frontend tests, frontend typecheck/public build, workspace all-target check excluding Apple clients, production asset/API/runtime checks
- Current conclusion: Codex/OpenCode ACP retains native ACP message chunks for trusted administrators, while ordinary public users remain actor-isolated and now receive real upstream OpenAI-compatible/OpenRouter tool-capable SSE through the strict function-calling runner. Fragmented parallel tool calls are assembled by index, internal reasoning stays hidden, transient tool preambles reset in place, final persistence remains one assistant turn, and the public client frame-batches deltas in the existing thinking card.
- Next entry point: `crates/hone-llm/src/provider.rs`, `agents/function_calling/src/lib.rs`, `crates/hone-channels/src/runners/tool_reasoning.rs`, and `packages/app/src/pages/chat.tsx`

### Server-owned Finance Calendar Images

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/server-owned-finance-calendar-images.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: `D-2026-07-12-02` in `docs/decisions.md`
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 216 frontend tests, frontend typecheck, public production build, 100 Web API tests with two credentialed tests ignored
- Current conclusion: each new finance-calendar message now persists validated desktop and mobile PNG paths as structured session metadata. Public bootstrap/history selects one path from the request User-Agent, legacy two-marker messages are selected server-side, and the client renders one stable authenticated image URL without calendar refetching, Canvas rebuilding, blob replacement, or source swapping. Image responses use private immutable browser caching.
- Next entry point: `crates/hone-web-api/src/routes/public_finance_calendar.rs`, `crates/hone-web-api/src/routes/history.rs`, and `packages/app/src/components/finance-calendar-message.tsx`

### Public Chat Startup Experience

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/public-chat-startup-experience.md`, `docs/archive/plans/public-chat-mobile-gesture-share-polish.md`
- Handoff: `docs/handoffs/2026-07-12-public-chat-startup-experience.md`
- Decision / ADR: N/A; public auth and actor-scoped history ownership remain unchanged
- Related PRs / commits: `22af864b`, `2f0c0e9e`, this follow-up change set
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 216 frontend tests, frontend typecheck, 96 Web API tests from the pagination phase, public production build, 390 x 844 browser QA
- Current conclusion: `/chat` now uses one recovery shell, starts from the newest 20 projected messages, cursor-loads older history upward, reserves media layout, represents every assistant run as one in-thread card, blocks accidental browser-level pinch outside the controlled calendar viewer, and centers user queries inside exported share bubbles.
- Next entry point: `docs/handoffs/2026-07-12-public-chat-startup-experience.md`

### Agent And Data Security Hardening

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/agent-data-security-hardening.md`
- Handoff: `docs/handoffs/2026-07-12-agent-data-security-hardening.md`
- Decision / ADR: `D-2026-07-12-01` in `docs/decisions.md`
- Related PRs / commits: `dbabbe77`, `a99bf096`
- Related runbooks / regressions: 495 channel tests, 115 core tests, 121 memory tests, 123 tool tests, 95 Web API tests, 12 Discord tests, 211 frontend tests, workspace check, CI-safe regression suite, cloud doctor, production/origin/CORS/runtime probes
- Current conclusion: non-admin actors can no longer use host-capable ACP/CLI runners and instead use actor-bound function calling; runtime/config/sandbox permissions and skill child environments are owner-only/secret-free; public CORS and actor-key data isolation are verified; production dependency alerts fell from 10 to two Tauri-only residual alerts. Admin ACP remains a trusted boundary and all credentials should be rotated.
- Next entry point: `docs/handoffs/2026-07-12-agent-data-security-hardening.md`

## 2026-07-11

### Public Web Visual Architecture And Finance Calendar V4

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/public-web-visual-architecture-refactor.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: `D-2026-07-11-03` in `docs/decisions.md`
- Related PRs / commits: `5b7b1d67`, `a3e0dbaa`
- Related runbooks / regressions: 211 frontend tests, typecheck, public build, direct Canvas dense-fixture review at 1500 x 2668 and 390px, 390 x 844 production browser QA, production asset/route/runtime checks
- Current conclusion: public visual ownership is split into foundation, shared component polish, chat shell, and component-local artifact layers. Mobile finance-calendar PNGs use one Canvas 2D renderer when a calendar is created, eliminating iOS html2canvas glyph clipping. The 2026-07-12 server-owned image follow-up supersedes client-side lazy upgrades: history now receives one backend-selected persisted image and never rebuilds v1-v3 artifacts in the viewer.
- Next entry point: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

### Mobile Finance Calendar Dual Layout And Gestures

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/mobile-finance-calendar-dual-layout.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: superseded in part by `D-2026-07-12-02` in `docs/decisions.md`; creation still renders/uploads both variants, while persistence and history selection are backend-owned
- Related PRs / commits: `2a6e7572`, `a4af378d`, `1a72b918`, `6ab39ee3`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, 209 frontend tests, 7 focused Rust tests, 14 focused migration tests, typecheck, public build, rendered 390 x 844 portrait/fit/300 percent/legacy-upgrade reviews, 15-event editorial design review, production asset/route/origin checks
- Current conclusion: new finance-calendar messages carry independently validated desktop and mobile PNGs. Since the 2026-07-12 follow-up, backend metadata owns both paths and history selects one by device; the viewer no longer lazily rebuilds or replaces portrait blobs. The controlled viewer gestures and the HONE monthly-brief artifact composition remain in place.
- Next entry point: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

### Mobile Finance Calendar And Navigation Polish

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/mobile-finance-calendar-nav-polish.md`
- Handoff: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`
- Decision / ADR: N/A; APIs, persistence, and module boundaries are unchanged
- Related PRs / commits: `31081106`, `e95b1049`
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, `bun run typecheck:web`, `bun run test:web` (207 passed), `bun run build:web:public`, local and production 390 x 844 browser QA
- Current conclusion: hone-claw.com now serves a bounded fit/125/150/200 percent finance-calendar viewer with fixed controls; Safari page zoom no longer combines with calendar canvas zoom. Production uses `index-D4wSdzNX.js` / `chat-ByxolQgf.js`; core routes and the public API proxy passed smoke checks.
- Next entry point: `docs/handoffs/2026-07-11-mobile-finance-calendar-nav-polish.md`

### HONE Client Brand And iOS Release

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/hone-client-brand-ios-release.md`
- Handoff: `docs/handoffs/2026-07-11-hone-client-brand-ios-release.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-02-use-one-hone-brand-and-remote-boundary-for-public-apple-clients`
- Related PRs / commits: `e33a467a`, `dc889ffa`, `aa32c818`, `6a14e3e7`; tag `v0.13.0`
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `docs/runbooks/public-user-ios-app.md`, `bash tests/regression/ci/test_hone_ios_contract.sh`, Apple Clients run `29139331210`, Release run `29139409377`
- Current conclusion: Public Web, focused macOS, and standalone iOS clients now use one uppercase HONE brand and polished navigation language. v0.13.0 ships a Universal macOS DMG, Xcode-built iOS Simulator app, complete iOS Xcode project, and Apple checksum manifest; device IPA/TestFlight and notarized macOS distribution still require Apple signing credentials.
- Next entry point: `docs/handoffs/2026-07-11-hone-client-brand-ios-release.md`

### Standalone Public User macOS App

- Status: done
- Date: 2026-07-11
- Plan: `docs/archive/plans/standalone-public-user-macos-app.md`
- Handoff: `docs/handoffs/2026-07-11-standalone-public-user-macos-app.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
- Related PRs / commits: this change set
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `cargo test -p hone-user-app`, `cargo check -p hone-user-app`, `bash scripts/build_user_app.sh`, Universal architecture/bundle/signature inspection, packaged `/chat` launch smoke
- Current conclusion: Hone now ships a focused Universal macOS user client that enters production `/chat` through a polished local startup shell and intentionally excludes local runtime, ACP, MCP, channels, config, skills, and data directories. The 16 MB app / 5.7 MB DMG are ad-hoc signed on this machine and require Developer ID signing plus notarization before public distribution.
- Next entry point: `docs/runbooks/public-user-macos-app.md`

## 2026-07-10

### Web Scheduled Push Mobile Hotfix

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/web-scheduled-push-mobile-hotfix.md`
- Handoff: `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-10-01-project-web-scheduled-results-into-a-durable-push-inbox`
- Related PRs / commits: `383058fe`
- Related runbooks / regressions: `cargo test -p hone-web-api --lib`, targeted memory legacy-push tests, `bun run test:web`, `bun run typecheck:web`, `bun run build:web:public`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`, actor-scoped HTTP backfill smoke
- Current conclusion: Production mobile blank message shells were caused by deploying the scheduled-push backend without the matching public bundle. The mobile push bell now sits outside the hidden desktop nav, and pre-upgrade scheduled messages are lazily imported into the durable inbox; the affected actor imported 79 messages in 93ms without clearing existing sessions or read state. Cloudflare Pages now serves `index-BeqwKSm5.js`, and the production Worker returns the same 79-message inbox through authenticated public API traffic.
- Next entry point: `crates/hone-web-api/src/routes/public_pushes.rs`, `packages/app/src/components/public-nav.tsx`, and `packages/app/src/components/public-push-center.tsx`

### Web Scheduled Push Inbox

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/web-scheduled-push-inbox.md`
- Handoff: `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-10-01-project-web-scheduled-results-into-a-durable-push-inbox`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api --lib`, `cargo check -p hone-channels -p hone-web-api`, `bun run test:web`, `bun run typecheck:web`, `bun run build:web:public`, local PostgreSQL + HTTP mark-through smoke
- Current conclusion: Web scheduled results now render as compact summary cards, open full content on demand, persist actor-scoped read state, and collect in a rail/mobile push center with an aggregate unread dot; reading the latest push clears the dot while Feishu and other channels remain unchanged.
- Next entry point: `crates/hone-web-api/src/routes/public_pushes.rs`, `memory/src/cron_job/history.rs`, and `packages/app/src/components/public-push-center.tsx`

### Public Finance Calendar Polish

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/public-finance-calendar-polish.md`
- Handoff: `docs/handoffs/2026-06-29-public-finance-calendar.md`
- Decision / ADR: N/A; module boundaries and upload/send architecture are unchanged
- Related PRs / commits: N/A
- Related runbooks / regressions: finance-calendar helper smoke and changed TS/TSX syntax parse passed; `bash tests/regression/run_ci.sh` passed available checks before stopping at missing `cargo`; Rust/Bun suites remain pending in a provisioned environment
- Current conclusion: the public finance calendar now opens on the current month with an immediate image preview, compact month navigation, explicit loading/error/source states, a redesigned 1080 x 1350 share image, and 17 verified July 2026 macro events in Beijing time.
- Next entry point: `packages/app/src/pages/chat.tsx`, `packages/app/src/components/finance-calendar-card.tsx`, and `crates/hone-web-api/src/routes/public_finance_calendar.rs`

## 2026-06-29

### Public Finance Calendar

- Status: done
- Date: 2026-06-29
- Plan: `docs/archive/plans/public-finance-calendar.md`
- Handoff: `docs/handoffs/2026-06-29-public-finance-calendar.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api finance_calendar` and `bun run test:web` still need to be rerun where Rust/Bun are installed; `bash scripts/ci/check_fmt_changed.sh` skipped because no base ref was discoverable in this workspace.
- Current conclusion: public chat now includes a “我的财经日历” quick action that fetches actor-scoped macro/FMP earnings data, renders a month-view PNG in the browser, uploads it through the current-user public upload root, appends an assistant image message, and broadcasts `push_message`.
- Next entry point: `crates/hone-web-api/src/routes/public_finance_calendar.rs`, `packages/app/src/pages/chat.tsx`, and `packages/app/src/components/finance-calendar-card.tsx`

## 2026-06-24

### ACP `hone-mcp` Process Cleanup

- Status: done
- Date: 2026-06-24
- Plan: `docs/current-plans/acp-runtime-refactor.md`
- Handoff: `docs/handoffs/2026-06-24-acp-hone-mcp-process-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels acp_child_guard_terminates_grandchild_process_group -- --nocapture`, `cargo test -p hone-channels codex_acp -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: ACP CLI children now run in an isolated process group and are cleaned up through `AcpChildGuard`, so `codex_acp` / `opencode_acp` success, error, and timeout paths terminate stdio MCP grandchildren such as `hone-mcp` instead of leaving local process leaks.
- Next entry point: `crates/hone-channels/src/runners/acp_common/process.rs`

## 2026-06-21

### Feishu Direct Cron Result Recovery

- Status: done
- Date: 2026-06-21
- Plan: N/A, single-session active bug fix did not need dynamic plan tracking
- Handoff: `docs/handoffs/2026-06-21-feishu-direct-cron-tool-result-recovery.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels finalize_agent_response_recovers_cron_job_ --lib -- --nocapture`, `cargo test -p hone-channels finalize_agent_response_recovers_portfolio_confirmation --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: Feishu direct 定时任务治理相关 turn 在真实 `cron_job` 工具已经返回结果时，最终回复现在会优先恢复任务列表、创建/更新确认或删除确认，而不是继续退化成过渡句或通用“定时任务管理暂时不可用”提示。
- Next entry point: `docs/bugs/feishu_direct_cron_management_tool_unavailable_internal_state_exposed.md`

## 2026-05-31

### Cloud PG / OSS Runtime Migration

- Status: done
- Date: 2026-05-31
- Plan: `docs/archive/plans/cloud-pg-oss-runtime-migration.md`
- Handoff: `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo test --offline -p hone-memory company_profile --lib`, `cargo test --offline -p hone-event-engine mainline_distill --lib`, `cargo test --offline -p hone-channels normalize_local_image_references --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-event-engine -p hone-channels -p hone-web-api -p hone-cli --tests`, `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json`, `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`
- Current conclusion: `cloud.mode=cloud` now uses PG/R2 for all current runtime durable dependencies covered by cloud doctor: sessions, web auth, quota, cron, skill registry, notification prefs, portfolio, LLM audit, company profiles, uploads/attachments, generated images, and cloud document indexing. The final doctor result is `local_durable_dependency_count=0`; local mode remains compatible and reports 0 cloud durable dependencies.
- Next entry point: `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md` for operational notes and `docs/runbooks/backend-deployment.md` for migration commands.

## 2026-05-27

### v0.12.4 Formal Release

- Status: done
- Date: 2026-05-27
- Plan: N/A, single-session formal release execution
- Handoff: `docs/handoffs/2026-05-27-v0.12.4-release.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bash tests/regression/run_ci.sh`, `bash scripts/prepare_release_notes.sh v0.12.4 /tmp/release-notes-v0.12.4.md`
- Current conclusion: `v0.12.4` ships the Cloud PG / OSS runtime config slice, public upload OSS proxy path, scheduler commodity guard false-positive fix, Feishu/external error diagnostics, guarded live smoke wrappers, refreshed architecture SVG, and release notes.
- Next entry point: `docs/releases/v0.12.4.md`, then `docs/archive/plans/cloud-pg-oss-runtime-migration.md` for the completed cloud storage follow-up record.

## 2026-05-23

### Heartbeat Structured Status Hardening

- Status: done
- Date: 2026-05-23
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-23-heartbeat-structured-status-hardening.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`, `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: heartbeat status parsing now tolerates common nonstandard noop/triggered status aliases and complete internal-only no-op reasoning, while the prompt blocks tool/task/profile configuration fragments as final output.
- Next entry point: `docs/bugs/scheduler_heartbeat_unknown_status_silent_skip.md`

### Heartbeat Context Overflow Status Boundary

- Status: done
- Date: 2026-05-23
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-23-heartbeat-context-overflow-status.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels heartbeat_context_overflow_error_is_not_classified_as_noop --lib -- --nocapture`, `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: heartbeat context-window overflow is no longer treated as a legitimate noop; it is classified as `context_window_overflow` and lands as `execution_failed + skipped_error` for auditability.
- Next entry point: `docs/bugs/scheduler_heartbeat_context_window_limit_no_recovery.md`

### Heartbeat Max-Iterations Budget

- Status: done
- Date: 2026-05-23
- Plan: N/A, single active-bug fix did not need dynamic plan tracking
- Handoff: `docs/handoffs/2026-05-23-heartbeat-max-iterations-budget.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels heartbeat_prompt_requires_noop_json_for_contract_conflicts --lib -- --nocapture`, `cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`, `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`, `cargo check -p hone-channels --tests`
- Current conclusion: heartbeat auxiliary function-calling now gets 18 iterations instead of 10, and the heartbeat prompt explicitly requires minimal tool use so sector/multi-symbol heartbeat jobs are less likely to burn their whole budget confirming noop.
- Next entry point: `docs/bugs/scheduler_heartbeat_iteration_exhaustion_skips_alert.md`

## 2026-05-21

### Public Blog Module

- Status: done
- Date: 2026-05-21
- Plan: `docs/archive/plans/public-blog-module.md`
- Handoff: `docs/handoffs/2026-05-21-public-blog-module.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bun --filter @hone-financial/app test`, `bun --filter @hone-financial/app typecheck`, `HONE_APP_OUT_DIR=dist-public HONE_APP_SURFACE=public bun --filter @hone-financial/app build`
- Current conclusion: hone-claw.com public surface now has a bilingual static Blog index and Rust article route, with navigation/homepage entry points and local Chinese/English article images copied from the provided source links.
- Next entry point: `packages/app/src/lib/public-blog.ts`, `packages/app/src/pages/public-blog.tsx`, and `packages/app/src/pages/public-blog-post.tsx`

### Public Blog Share Metadata

- Status: done
- Date: 2026-05-21
- Plan: `docs/archive/plans/public-blog-share-metadata.md`
- Handoff: `docs/handoffs/2026-05-21-public-blog-module.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bun --filter @hone-financial/app test`, `bun --filter @hone-financial/app typecheck`, `HONE_APP_OUT_DIR=dist-public HONE_APP_SURFACE=public bun --filter @hone-financial/app build`
- Current conclusion: Blog article pages now show both Chinese and English titles, include a card to switch language versions, inject article-specific metadata at runtime, and use Cloudflare Worker HTML metadata injection for crawlers that do not execute the SPA. README top navigation and Rust-stack sections now link to the Blog with matching language labels.
- Next entry point: `packages/app/public/_worker.js`, `packages/app/src/pages/public-blog-post.tsx`, and `README_ZH.md`

## 2026-05-20

### Heartbeat Mimo 429 Key-Pool Fallback

- Status: done
- Date: 2026-05-20
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-20-heartbeat-mimo-429-key-pool.md`
- Decision / ADR: N/A
- Related PRs / commits: GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)
- Related runbooks / regressions: `cargo test -p hone-llm chat_with_tools_falls_back_to_next_key_after_http_429 -- --nocapture`, `cargo test -p hone-channels heartbeat_provider_429_quota_error_is_classified --lib -- --nocapture`
- Current conclusion: OpenAI-compatible non-streaming routes now honor provider key pools for non-OpenRouter profiles, so a single exhausted mimo key no longer drops the whole heartbeat batch when fallback keys are configured.
- Next entry point: `docs/bugs/scheduler_heartbeat_mimo_429_quota_exhausted.md`

## 2026-05-12

### Public SMS Verification Login

- Status: done
- Date: 2026-05-12
- Plan: `docs/archive/plans/public-sms-login.md`
- Handoff: `docs/handoffs/2026-05-12-public-sms-login.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth::tests::active_invite_user_by_phone_is_sms_login_whitelist`, `cargo test -p hone-memory web_auth::tests::record_tos_acceptance_updates_public_login_terms`, `cargo test -p hone-web-api aliyun_sms::tests`, `cargo check -p hone-web-api`, `bun run --cwd packages/app typecheck`, `bun run --cwd packages/app test:e2e -- --project=public public-sms-login.spec.ts`, optional live SMS smoke `HONE_ALIYUN_SMS_LIVE_PHONE=13871396421 cargo test -p hone-web-api aliyun_sms::tests::live_send_verify_code_smoke -- --ignored --nocapture`
- Current conclusion: 用户端登录已切换为手机号 + 阿里云短信验证码；管理端现有 Web invite 用户手机号作为白名单来源，旧邀请码仅保留为兼容管理字段。
- Next entry point: `crates/hone-web-api/src/aliyun_sms.rs`, `crates/hone-web-api/src/routes/public.rs`, and `packages/app/src/components/public-login-form.tsx`

## 2026-05-11

### LLM Profile Registry POC

- Status: done
- Date: 2026-05-11
- Plan: N/A, single-session POC did not need active plan tracking
- Handoff: `docs/handoffs/2026-05-11-llm-profile-poc.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `tests/regression/manual/test_llm_profile_poc.sh`, `cargo test -p hone-core config::tests`, `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- Current conclusion: The proposed `llm.providers` + `llm.profiles` shape can parse model profiles with `reasoning`, `response_format`, and other generation params, and OpenRouter accepted a live profile-derived request with `reasoning_present=true`.
- Next entry point: Runtime migration is tracked in `docs/archive/plans/llm-profile-runtime-migration.md`.

### LLM Profile Runtime Migration

- Status: done
- Date: 2026-05-11
- Plan: `docs/archive/plans/llm-profile-runtime-migration.md`
- Handoff: `docs/handoffs/2026-05-11-llm-profile-poc.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core config::tests`, `cargo check -p hone-channels --tests`, `cargo check -p hone-web-api --tests`, `cargo test -p hone-llm resolver`, `cargo test -p hone-event-engine global_digest_llm_providers_can_be_wired_per_stage`, `cargo test -p hone-web-api validate_global_digest`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop --bin hone-desktop sidecar`, `bun run typecheck:web`, `bun run test:web`, `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- Current conclusion: `llm.providers` + `llm.profiles` is now a runtime-supported profile registry for event-engine and auxiliary LLM paths; Settings UI can edit profile routing and profile params; legacy OpenRouter/Auxiliary fields remain fallback-compatible.
- Next entry point: `crates/hone-llm/src/resolver.rs`, `crates/hone-web-api/src/lib.rs`, and `packages/app/src/pages/settings.tsx`

### LLM Config Env Removal

- Status: done
- Date: 2026-05-11
- Plan: `docs/archive/plans/llm-config-env-removal.md`
- Handoff: `docs/handoffs/2026-05-11-llm-profile-poc.md`
- Decision / ADR: `docs/decisions.md#d-2026-05-11-01-make-llm-credentials-config-only`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core config::tests`, `cargo test -p hone-llm resolver`, `cargo test -p hone-cli mutations`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop --bin hone-desktop sidecar`, `bun run test:web`, `bun run typecheck:web`, `cargo run -p hone-cli -- config validate --json`, `cargo run -p hone-cli -- status --json`, `cargo run -p hone-cli -- probe --channel cli --user-id cli_smoke --query '只输出 HONE_CLI_LLM_OK' --show-events false`, `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- Current conclusion: LLM credentials are now config-only. Runtime no longer consumes `api_key_env` or parent-process `*_API_KEY` fallback for LLM provider/profile/auxiliary paths; CLI/Desktop OpenRouter writes now target `llm.providers.openrouter.api_keys`, while legacy `llm.openrouter.*` remains a config-only fallback/migration path.
- Next entry point: `crates/hone-core/src/config/agent.rs`, `crates/hone-llm/src/resolver.rs`, and `config.example.yaml`

## 2026-05-10

### Source CLI Start And Launch Retirement

- Status: done
- Date: 2026-05-10
- Plan: `docs/archive/plans/source-cli-start-retire-launch.md`
- Handoff: `docs/handoffs/source-cli-start-retire-launch-2026-05-10.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `docs/runbooks/source-web-startup.md`, `docs/runbooks/desktop-dev-runtime.md`, `cargo test -p hone-cli start`, `bash tests/regression/ci/test_source_cli_start_contract.sh`, `bash tests/regression/ci/test_install_hone_cli_path_resolution.sh`, CLI channel configuration smoke, source startup smoke with `/api/meta` on port `19077`, `cargo test -p hone-cli`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bun run typecheck:web`, `bun run test:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: Source checkout startup now uses `cargo run -p hone-cli -- start --build`, installed users continue with packaged `hone-cli start`, active docs no longer recommend source launcher flows, and the previous channel configuration changes were verified through real CLI commands against a temporary config.
- Next entry point: `docs/runbooks/hone-cli-install-and-start.md` for install/source startup and `docs/runbooks/desktop-dev-runtime.md` for desktop dev lanes.

### Channel Delivery Config Borrowing

- Status: done
- Date: 2026-05-10
- Plan: `docs/archive/plans/channel-delivery-config-borrowing.md`
- Handoff: `docs/handoffs/channel-delivery-config-borrowing-2026-05-10.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-tools cron_job_tool_add_preserves_origin_channel_target`, `cargo test -p hone-cli build_channel_mutations_supports_allowlists`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop desktop_channel_settings`, `cargo test -p hone-cli`, `bun run test:web`, `cargo test -p hone-memory channel_target`, `cargo test -p hone-scheduler scheduler_records_missing_channel_target_without_dispatching`, `cargo test -p hone-cli cli_parses_channels_targets_command`, `cargo test -p hone-memory`, `cargo test -p hone-scheduler`, `cargo test -p hone-web-api cron`, `bun run typecheck:web`, `cargo check --workspace --all-targets --exclude hone-desktop`
- Current conclusion: Hermes-style channel improvements were borrowed without adding platforms or a `home_channel` default. Honeclaw now keeps origin-bound delivery, exposes existing allowlists / `chat_scope` / iMessage `target_handle` through CLI and Desktop/Web settings, rejects or records missing scheduled delivery targets deterministically, and provides a typed cron-backed channel-target directory through `hone-cli channels targets`.
- Next entry point: Add a Web/Desktop selector backed by `CronJobStorage::list_channel_targets()` if users need clickable target discovery; do not introduce `home_channel` unless a separate no-origin system task flow is designed.

## 2026-05-09

### Event Engine Poller Timeout Boundary

- Status: done
- Date: 2026-05-09
- Plan: `docs/archive/plans/event-engine-poller-timeout-boundary.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine spawner::tests --lib -- --nocapture`, `cargo test -p hone-event-engine pollers::earnings_surprise::tests::quality_review_applies_successful_earnings_event --lib -- --nocapture`, `cargo test -p hone-event-engine --lib`, `cargo check -p hone-event-engine --tests`, changed-file `rustfmt --edition 2024 --check`
- Current conclusion: event-engine unified poller ticks now have a bounded timeout, so a stuck `poll().await` / `run_once().await` records a failed tick and releases the loop for the next scheduled cadence instead of suppressing `poller ok` indefinitely
- Next entry point: `docs/bugs/archive/event_engine_poller_cadence_stall_without_restart.md`

### Event Engine Mainline Distill Token Cap

- Status: done
- Date: 2026-05-09
- Plan: `docs/archive/plans/event-engine-mainline-distill-token-cap.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api mainline_distill_uses_short_completion_budget --lib -- --nocapture`, `cargo check -p hone-web-api --tests`, changed-file `rustfmt --edition 2024 --check`
- Current conclusion: mainline distill cron now uses its own OpenRouter provider capped at 1200 completion tokens instead of inheriting global `llm.openrouter.max_tokens`, closing the HTTP 402 preauthorization failure for short investment-mainline summaries
- Next entry point: `docs/bugs/event_engine_mainline_distill_openrouter_402.md`

## 2026-05-08

### Event-engine Push Quality Hardening

- Status: done
- Date: 2026-05-08
- Plan: `docs/archive/plans/event-engine-push-quality-hardening.md`
- Handoff: `docs/handoffs/2026-04-23-event-engine-push-quality.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine --lib`, `cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, changed-file `rustfmt --edition 2024 --check`; full `cargo fmt --all -- --check` currently blocked by unrelated formatting debt
- Current conclusion: 基于近期 event review 与 POC 结论，event engine 已补 analyst 同源文章 fanout 降噪、RSS 标题级保守实体链接，以及 Zacks 泛化模板回归证明；本轮没有新增 LLM 调用或 summary/body 宽匹配
- Next entry point: `docs/handoffs/2026-04-23-event-engine-push-quality.md#2026-05-08-poc-后续收口`

### Event Engine Earnings Quality Review

- Status: done
- Date: 2026-05-08
- Plan: `docs/archive/plans/event-engine-earnings-quality-review.md`
- Handoff: `docs/handoffs/2026-05-08-event-engine-earnings-quality-review.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine pollers::earnings_surprise`, `cargo test -p hone-event-engine pollers::earnings_quality`, `cargo test -p hone-event-engine --lib`, `cargo test -p hone-core --lib`, `cargo check -p hone-web-api`, changed-file `rustfmt --edition 2024 --check`; full `cargo fmt --all -- --check` currently blocked by unrelated formatting debt
- Current conclusion: `EarningsReleased` 已移除 EPS-only 推送，并新增 best-effort LLM 综合财报 review；AAOI / CAI / CRWV POC 结论落地为 SEC 8-K 上下文 + `x-ai/grok-4.1-fast` 风格 JSON judgement，失败、缺上下文或低置信时跳过 candidate
- Next entry point: `docs/handoffs/2026-05-08-event-engine-earnings-quality-review.md`

## 2026-04-30

### Feishu P1 直聊与定时任务可靠性修复批次

- Status: done
- Date: 2026-04-30
- Plan: `docs/archive/plans/feishu-p1-reliability-batch.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels runners::multi_agent::tests`, `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries`, `cargo check -p hone-channels`
- Current conclusion: 活跃 Feishu `P1` 已全部移出活跃队列；multi-agent 对 `cron_job` / `portfolio` 可信本地结果的直返放宽到多行与较长正文，避免“我的定时任务”这类本地状态答案已生成却仍被硬送进容易空回复的 answer 阶段
- Next entry point: `docs/bugs/README.md#活跃待修复`

## 2026-04-29

### Admin Notification Log and Actor Picker

- Status: done
- Date: 2026-04-29
- Plan: `docs/archive/plans/admin-notification-log-actor-picker.md`
- Handoff: `docs/handoffs/2026-04-29-admin-notification-log-actor-picker.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api routes::notifications`, `cargo test -p hone-event-engine list_recent_delivery_logs`, `cargo test -p hone-event-engine store::tests::delivery_log_is_append_only_across_retries`, `bun --filter @hone-financial/app typecheck`, `git diff --check`
- Current conclusion: 管理端推送日志已从只读 cron 执行记录改为合并 cron 与 event-engine `delivery_log`；默认排除 no-actor router 与 digest item 内部行，避免真实 Discord / sink 送达记录被淹没；前端现在显示 `events.kind_json.type` 的业务事件类型，推送日志和推送日程均改为 actor 下拉选择
- Next entry point: `docs/handoffs/2026-04-29-admin-notification-log-actor-picker.md`

## 2026-04-26

### 后端部署文档与 public chat 顶部菜单修复

- Status: done
- Date: 2026-04-26
- Plan: `docs/archive/plans/backend-deployment-and-public-chat-nav.md`
- Handoff: `docs/handoffs/2026-04-26-backend-deployment-public-chat-nav.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/backend-deployment.md`, `bun run typecheck:web`, `bun run build:web:public`, Chrome/Playwright local preview screenshots for `/chat`
- Current conclusion: 后端部署流程已落到 runbook，公开文档统一使用后端 origin 口径；public chat 顶部菜单样式已收敛到共享 public CSS，Cloudflare Pages SPA fallback 已加入 public 静态资源
- Next entry point: `docs/runbooks/backend-deployment.md`

### Non-P1 Fixing Bug Batch

- Status: done
- Date: 2026-04-26
- Plan: `docs/archive/plans/non-p1-fixing-bug-batch.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: `0e917fe fix scheduler fixing bug batch`
- Related runbooks / regressions: `cargo test -p hone-channels scheduler::tests`, `cargo test -p hone-channels prompt::tests`, `cargo test -p hone-channels`, `cargo test -p hone-feishu failed_reply_text`, `git diff --check`
- Current conclusion: 非 P1 `Fixing` 批次已完成代码止血与文档同步；按新口径，已代码修复但只待真实窗口复核的缺陷统一标记为 `Later`，不再占活跃队列，后续复现时改回 `New`
- Next entry point: `docs/bugs/README.md#later--待复现`

### Remove Truth Social Source

- Status: done
- Date: 2026-04-26
- Plan: N/A, single-session deletion did not need dynamic plan tracking
- Handoff: `docs/handoffs/2026-04-26-remove-truth-social-source.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo check -p hone-web-api`
- Current conclusion: Truth Social 已从 event-engine 活跃 source 集合删除；`truth_social_accounts` 配置、`TruthSocialPoller` 模块、engine 装配、主配置启用项和本机 ignored effective config 均已移除，历史 403 断流 bug 标记为 Closed
- Next entry point: `docs/handoffs/2026-04-26-remove-truth-social-source.md`

## 2026-04-24

### Price Event Lane 增量改造

- Status: done
- Date: 2026-04-24
- Plan: `docs/archive/plans/price-event-lane.md`
- Handoff: `docs/handoffs/2026-04-24-price-event-lane.md`
- Decision / ADR: `docs/decisions.md#d-2026-04-24-01-route-price-alerts-through-directional-band-lanes`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine price --lib`, `cargo test -p hone-event-engine router --lib`, `cargo test -p hone-event-engine digest --lib`, `cargo test -p hone-core --lib`, `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`, `cargo test --workspace --all-targets --exclude hone-desktop`
- Current conclusion: 价格事件已从日级去重改为 low/band/close 分层 id；盘中 `price_band:{symbol}:{date}:{up|down}:{band_bps}` 可在同日多次跨新档时形成独立事件，router 使用价格专属 gap/cap 控频，digest 对同一 actor/symbol/date/window 保留最新价格态，收盘价格默认摘要化
- Next entry point: `docs/handoffs/2026-04-24-price-event-lane.md`

### Event Engine Close Price 与 Truth Social 后续修复

- Status: done
- Date: 2026-04-24
- Plan: `docs/archive/plans/event-engine-close-price-truth-social-followup.md`
- Handoff: `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine --lib`, `cargo fmt --all -- --check`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, `cargo test -p hone-event-engine pollers::news::tests::live_news_classifier_baseline_source_policy_is_stable --lib`, `env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`, `python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-23 --actor telegram::::8039067465`, `python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-24 --actor telegram::::8039067465 --include-body`
- Current conclusion: Truth Social poller 已补 status / content-type / body-prefix 失败诊断，`price_close` 高波动已恢复 High / immediate 路由；真实模型 baseline 已从 12 条 LLM 样本扩到 15 条并 15/15 matched；2026-04-24 Telegram digest 省略项已可通过 `digest_item omitted` 审计，低信号 news/social/macro/no-op analyst 噪声已降噪
- Next entry point: `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`

## 2026-04-23

### Event Engine 推送质量全量修复

- Status: done
- Date: 2026-04-23
- Plan: `docs/archive/plans/event-engine-push-quality.md`
- Handoff: `docs/handoffs/2026-04-23-event-engine-push-quality.md`
- Decision / ADR: N/A
- Related PRs / commits: `0ff23d4 feat(event-engine): improve push quality routing`, `df820ca feat(event-engine): add daily push calibration export`
- Related runbooks / regressions: `cargo fmt --all -- --check`, `cargo test -p hone-event-engine --lib`, `cargo test -p hone-core --lib`, `cargo check -p hone-web-api`, `bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- Current conclusion: event engine 的 24 项推送质量清单已全部收口，新增 digest 去重 / min-gap / topic memory、source/channel 偏好、分类预算、方向性价格阈值、macro/earnings 时窗、delivery observability，以及 `amazon/nova-lite-v1` 不确定来源新闻分类基线
- Next entry point: `docs/handoffs/2026-04-23-event-engine-push-quality.md`

### Core Runtime 职责与类型收敛

- Status: done
- Date: 2026-04-23
- Plan: `docs/archive/plans/core-runtime-type-consolidation.md`
- Handoff: `docs/handoffs/2026-04-23-core-runtime-type-consolidation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels agent_session`, `cargo test -p hone-channels runners::tests`, `cargo test -p hone-event-engine subscription`, `cargo test -p hone-web-api routes::history`, `bun run test:web`, `bun --filter @hone-financial/app typecheck`, `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`
- Current conclusion: `AgentSession` 的 prompt/skill turn 构建与 response finalization 已从主编排里拆出，runner/session 内部事件收敛到 canonical `run_event`，runner kind / CLI probe 逻辑有了统一 helper，前端历史附件类型已和 Rust 对齐，本地图片 marker 也补了 Rust/前端共享 fixture
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-22

### Git Hook Auto Format

- Status: done
- Date: 2026-04-22
- Plan: `docs/archive/plans/git-hook-auto-format.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `.githooks/pre-commit` hook smoke test with staged Rust formatting, `bash -n .githooks/pre-commit`, `bash -n scripts/install_gitleaks.sh`
- Current conclusion: 本地 Git hook 现在会在 commit 前自动格式化已暂存 Rust 文件并重新暂存，push 前的 rustfmt / gitleaks 仍作为兜底门禁；同一 Rust 文件如果同时有已暂存和未暂存改动，pre-commit 会停止以避免把未选择的内容混入 commit
- Next entry point: `.githooks/pre-commit`

## 2026-04-20

### Hone 内置技能高置信度收敛

- Status: done
- Date: 2026-04-20
- Plan: `docs/archive/plans/hone-skill-consolidation.md`
- Handoff: `docs/handoffs/2026-04-20-hone-skill-consolidation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_finance_automation_contracts.sh`, `cargo test -p hone-tools load_skill_and_direct_invocation_accept_aliases`, `cargo fmt --all --check`
- Current conclusion: Hone 的高重叠金融 skill 已收敛到更小的维护面：`one_sentence_memory` 被删除，`major_alert` 被并入 `scheduled_task`，`valuation` 与 `stock_selection` 被并入带兼容 alias 的 `stock_research`；finance regression 已改为验证新的 canonical skill 形态
- Next entry point: `skills/stock_research/SKILL.md`

## 2026-04-19

### Hone 半小时健康巡检补齐用户端静态资源检查

- Status: done
- Date: 2026-04-19
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-19-hone-health-automation-public-web-check.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `sed -n '1,220p' ~/.codex/automations/hone-health-30m/automation.toml`, `bun run build:web:public`, `curl http://127.0.0.1:8088/`, `ls packages/app/dist-public`
- Current conclusion: `hone-health-30m` 现在不会再把“`8088` 正在监听”误判成用户端健康；它新增了 `packages/app/dist-public/index.html` 与 `8088` HTML 返回检查，并在只缺用户端静态资源时优先执行 `bun run build:web:public` 做最小止血，只有仍不健康时才整套重启
- Next entry point: `.codex/automations/hone-health-30m/automation.toml`

### Web 邀请码手机号绑定与固定端口切换

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-invite-phone-and-fixed-ports.md`
- Handoff: `docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`, `bun run build:web`, `bun run build:web:public`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bun run tauri:prep:build`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`, `curl http://127.0.0.1:8077/api/meta`, `curl http://127.0.0.1:8088/api/public/auth/me`, `curl -I http://127.0.0.1:8088/chat`
- Current conclusion: bundled desktop 现在固定使用管理端 `8077` 与用户端 `8088`；Web 邀请码已改为与手机号强绑定，管理端发码必须填手机号，用户端登录必须同时提交邀请码和手机号。新的 release app 已按 runbook 切换到 `.app` runtime；`discord` / `feishu` 在线，`telegram` 仍因配置里的 `Invalid bot token` 处于 `degraded`
- Next entry point: `docs/handoffs/2026-04-19-web-invite-phone-and-fixed-ports.md`

### 用户可见内部工作说明泄露修复

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/user-visible-internal-working-note-fix.md`
- Handoff: `docs/handoffs/2026-04-19-user-visible-internal-working-note-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels runners::tests -- --nocapture`, `cargo test -p hone-channels agent_session -- --nocapture`, `cargo test -p hone-web-api -- --nocapture`, `bun run test:web`
- Current conclusion: public web 不再把 `company_profiles/`、actor 用户空间、目录结构这类内部工作说明直接作为最终答复或执行中状态暴露给用户；ACP runner 在本轮发生工具调用时只接受“最后一个 tool 之后的 assistant 文本”作为最终答复候选，session 成功态也会对明显的内部 working note 触发安全 fallback
- Next entry point: `crates/hone-channels/src/agent_session.rs`

### Company Profile Optional Frontmatter

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-optional-frontmatter.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-optional-frontmatter.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile -- --nocapture`
- Current conclusion: 公司画像与事件现在不再在读取、列出、bundle preview/import 时硬依赖 YAML frontmatter；legacy plain Markdown 本地画像与 plain-Markdown 画像包都会推断最小 metadata 继续工作，不再因为 `缺少 frontmatter` 直接失败
- Next entry point: `memory/src/company_profile/markdown.rs`

### 公司画像包导入导出与傻瓜式导入流

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-transfer.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-transfer.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile`, `cargo test -p hone-web-api`, `bun run test:web`, `bun run typecheck:web`, `bun run build:web`, `bun run --cwd packages/app test:e2e`, `cargo check -p hone-memory -p hone-web-api -p hone-channels`
- Current conclusion: 公司画像现在支持 actor 私有画像包导入导出；Memory 页面左侧已收敛成单一“目标用户空间”列表，当前空间里的公司切换放到右侧详情内部；右侧会先自动扫描导入包，只在存在冲突时要求逐家公司选择“保留当前”或“用导入版本替换”，并在存在替换时自动生成导入前备份供用户下载；legacy plain Markdown 画像即使缺少 frontmatter，也能被 transfer 导出、自动备份并参与冲突判断
- Next entry point: `packages/app/src/context/company-profiles.tsx`

### Company Profile 模块拆分

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/company-profile-module-split.md`
- Handoff: `docs/handoffs/2026-04-19-company-profile-transfer.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo fmt --all`, `cargo test -p hone-memory company_profile`, `cargo test -p hone-web-api`, `cargo check -p hone-memory -p hone-web-api -p hone-channels`
- Current conclusion: `hone-memory` 里的 company profile 已按职责拆成 `types / markdown / storage / transfer / tests` 子模块，保留原有 `hone_memory::*` 导出面和导入导出语义，后续继续改画像能力时不需要再在单个超大文件里同时处理类型、Markdown、zip 和存储细节
- Next entry point: `memory/src/company_profile/mod.rs`

Use this file as the historical entry point for completed or paused work that should remain discoverable.

### Web 管理端 / 用户端端口隔离与公网暴露加固

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-admin-public-isolation.md`
- Handoff: `docs/handoffs/2026-04-19-web-admin-public-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`, `./launch.sh --web`, `curl http://127.0.0.1:8077/api/public/auth/me`, `curl http://127.0.0.1:8088/api/meta`
- Current conclusion: Web 管理端和 invite 用户端已按端口与可访问路由拆开；管理端默认监听 `8077` 并只提供 `/api/*` 与 console SPA，用户端默认监听 `8088` 并只提供 `/api/public/*` 与 `/chat`。后续安全加固已经补上 public 邀请码失败冷却、邀请码停用 / 恢复 / 重置与会话清退、单邀请码单活跃 session、HTTPS 场景 `Secure` cookie，以及 public API 默认去掉 `CORS: *`；公网暴露时仍必须确保管理端不被反代出去，并在反向代理 / WAF 层继续做 IP 级限流
- Next entry point: `crates/hone-web-api/src/routes/public.rs`

### Public Web 邀请码与公网暴露安全加固

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/public-web-security-hardening.md`
- Handoff: `docs/handoffs/2026-04-19-web-admin-public-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api`, `cargo check -p hone-web-api -p hone-memory`, `bun run typecheck:web`, `bun run test:web`
- Current conclusion: public 邀请码登录已从“无防刷、无撤销、无会话止血”状态提升到具备应用层失败冷却、邀请码停用 / 恢复 / 重置、旧 session 立即失效、HTTPS `Secure` cookie 和同源默认访问的基础安全面；剩余长期暴露风险主要转移到反向代理 / WAF 限流策略与管理端误暴露治理
- Next entry point: `crates/hone-web-api/src/routes/web_users.rs`

### Web 邀请码用户端与管理端入口拆分

- Status: done
- Date: 2026-04-19
- Plan: `docs/archive/plans/web-invite-chat-user-surface.md`
- Handoff: `docs/handoffs/2026-04-19-web-invite-chat-user-surface.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth -- --nocapture`, `cargo test -p hone-web-api -- --nocapture`, `bun run test:web`, `cd packages/app && bun run typecheck && bun run build`
- Current conclusion: 管理端现在可以在设置页生成邀请码并复制，侧边栏“开始”旁新增了用户端跳转 icon；用户侧新增 `/chat` 页面，通过邀请码登录并进入单会话 SSE 聊天窗口，过程卡片会展示 `Hone 思考中 -> 工具执行 -> 最终回复`；后端新增 `/api/public/*` 与 SQLite `web_auth` 存储，公开接口严格从 cookie 登录态反解 `web` actor，不再接受外部传入的 `channel/user_id/session_id`
- Next entry point: `crates/hone-web-api/src/routes/public.rs`

## 2026-04-17

### 群聊中间进度改为 compact 可见

- Status: done
- Date: 2026-04-17
- Plan: `docs/archive/plans/group-chat-compact-progress-visibility.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels outbound::tests -- --nocapture`, `cargo test -p hone-feishu listener -- --nocapture`, `cargo check --workspace --all-targets --exclude hone-desktop`
- Current conclusion: Telegram / Discord / Feishu 群聊现在都会显示处理中间进度，但默认收敛到 compact 粒度，只暴露“搜索信息 / 获取数据 / 执行命令 / 执行技能”等阶段，不再把 query、命令行和目录路径这类细节直接刷进群消息；当 runner 只吐出 `Tool` 这类泛化标签时，会结合 reasoning 回退成粗粒度动作文案，且连续多轮相同类型的工具调用也会像单聊一样逐轮追加
- Next entry point: `crates/hone-channels/src/outbound.rs`

### 对话额度改为可配置并支持无限制

- Status: done
- Date: 2026-04-17
- Plan: `docs/archive/plans/conversation-quota-config.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core`, `cargo test -p hone-channels run_success_commits_daily_conversation_quota -- --nocapture`, `cargo test -p hone-channels run_rejects_over_daily_limit_without_persisting_user_message -- --nocapture`, `cargo test -p hone-channels run_zero_daily_conversation_limit_bypasses_quota -- --nocapture`, `cargo run -q -p hone-cli -- config validate`
- Current conclusion: 用户每日成功对话额度不再固定写死为 `12`；现在由 `agent.daily_conversation_limit` 控制，`0` 表示无限制。本地 repo `config.yaml` 已切到 `0`，当前运行环境不再限制用户每日对话数
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-16

### Feishu 直聊 placeholder 假启动收口

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/feishu-direct-busy-placeholder-gap.md`
- Handoff: `docs/handoffs/2026-04-16-feishu-direct-busy-placeholder-gap.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- Current conclusion: Feishu 私聊当前已不再在 session 冲突时先发 placeholder 再卡死等待，而是会在入口直接返回 busy 提示；这条修复针对的是“placeholder 假启动”问题，不等同于已经完全根除所有深层长时间持锁根因
- Next entry point: `docs/handoffs/2026-04-16-feishu-direct-busy-placeholder-gap.md`

### 搜索失败提示主根因修复与 Tavily 复核

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/search-failure-tavily-and-tool-call-fix.md`
- Handoff: `docs/handoffs/2026-04-16-search-failure-tavily-and-tool-call-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels sanitize_search_context -- --nocapture`, `env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json`
- Current conclusion: `web_search` 工具确实走 Tavily，但当前统一失败提示的主根因不是 Tavily 全局不可用，而是 multi-agent 搜索阶段历史上下文清洗不完整，遗留 assistant `tool_calls` 与被删除的 `tool` 结果失配，触发 OpenAI-compatible provider `tool call result does not follow tool call (2013)`；该问题现已修复并完成定向测试与 desktop release 打包验证
- Next entry point: `docs/handoffs/2026-04-16-search-failure-tavily-and-tool-call-fix.md`

### Desktop 启动坑位沉淀与会话列表恢复

- Status: done
- Date: 2026-04-16
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-16-session-list-runtime-recovery.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`, `cargo test -p hone-core actor::tests::session_identity_can_be_restored_from_actor_session_id -- --exact`, `cargo test -p hone-memory session_sqlite::tests::list_sessions_skips_unreadable_rows -- --exact`, `cargo test -p hone-web-api routes::users::tests::actor_session_id_is_enough_for_listing_identity -- --exact`, `curl http://127.0.0.1:8077/api/meta`, `curl http://127.0.0.1:8077/api/users`, `curl http://127.0.0.1:8077/api/channels`
- Current conclusion: repo-local `honeclaw/data` 并未丢失，会话为空的主因是 backend session-listing 在部分脏数据路径上直接失败，导致 `/api/users` 错误返回空数组；现在列表会跳过损坏的 `normalized_json` 并从 `session_id` 回推 actor identity，desktop release runtime runbook 和 `bug-2` automation 也已经把锁文件、detached 启动静默失败、desktop/backend 分离排障、正式接口验证等坑位写清楚
- Next entry point: `docs/handoffs/2026-04-16-session-list-runtime-recovery.md`

### Desktop Agent 配置隔离修复

- Status: done
- Date: 2026-04-16
- Plan: `docs/current-plans/canonical-config-runtime-apply.md`
- Handoff: `docs/handoffs/2026-04-16-desktop-agent-config-isolation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core promote_legacy_runtime_agent_settings`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop build_agent_setting_updates_keeps_opencode_and_multi_agent_answer_isolated`
- Current conclusion: desktop legacy agent config promotion no longer overwrites canonical `agent.opencode` when the canonical `api_key` is intentionally blank, and desktop settings save no longer lets `multi-agent.answer` silently overwrite `agent.opencode`; both P1 bug docs and the bug navigation table are now updated to `Fixed`
- Next entry point: `docs/handoffs/2026-04-16-desktop-agent-config-isolation.md`

## 2026-04-15

### Bug 台账导航页与自动化文档模式升级

- Status: done
- Date: 2026-04-15
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-15-bug-index-and-automation-doc-mode.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `ls docs/bugs`, `sed -n '1,40p' docs/bugs/*.md`, `sed -n '1,220p' .codex/automations/bug/automation.toml`, `sed -n '1,220p' .codex/automations/bug-2/automation.toml`
- Current conclusion: `docs/bugs/README.md` 现在作为 bug 目录导航和状态总表存在，集中展示活跃待修复、已修复/关闭和历史分析条目；`bug` 与 `bug-2` 两个 automation 都被要求在任何 bug 状态变化时同步维护这张表
- Next entry point: `docs/bugs/README.md`

### Bug 每小时巡检自动化升级

- Status: done
- Date: 2026-04-15
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-15-hourly-bug-audit-automation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `sqlite3 data/sessions.sqlite3 '.tables'`, `sqlite3 data/sessions.sqlite3 'pragma table_info(session_messages);'`, `find data/runtime -maxdepth 2 -type f`
- Current conclusion: 每小时 `bug` automation 现在会优先巡检最近一小时真实会话与运行日志，并把“AI 返回不及预期、结构/格式错误、返回质量不佳但不影响功能链路”的问题统一按 `P3` 建档；只有真正影响功能链路、正确性、稳定性或投递结果的问题，才继续提升到 `P0`-`P2`。2026-04-26 起，新增或确认仍活跃的 `P1` 还必须通过 `gh issue create` 创建脱敏 GitHub issue，正文标记 `Reporter: hone-scanner` 并 `CC: @chet-zzz @Finn-Fengming`
- Next entry point: `docs/handoffs/2026-04-15-hourly-bug-audit-automation.md`

### Desktop 日志接口与 multi-agent 运行态恢复

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/runtime-logs-runner-recovery.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api logs`, `cargo test -p hone-core from_file_applies_runtime_overlay`, `curl http://127.0.0.1:8077/api/logs`, `curl http://127.0.0.1:8077/api/channels`
- Current conclusion: `/api/logs` 现在能容忍非 UTF-8 日志内容与日志缓冲锁中毒，不再因为多字节明文切片直接断开连接；`HoneConfig::from_file()` 也会正确合并 runtime overlay，渠道与 desktop 运行态恢复后能够稳定回到 `multi-agent`
- Next entry point: `crates/hone-web-api/src/routes/logs.rs`

### 持仓记忆补齐持有期限与策略信息

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/portfolio-memory-horizon-strategy.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory portfolio`, `cargo test -p hone-tools portfolio_`, `cargo test -p hone-web-api portfolio`, `bun run typecheck:web`, `bun run test:web`
- Current conclusion: 持仓记忆现在除标的、数量、成本和备注外，还会稳定保留 `holding_horizon`（`long_term` / `short_term`）和 `strategy_notes`；前端表单已允许负成本价输入，底层存储 / tool / API / UI 都兼容负成本与新增策略字段
- Next entry point: `memory/src/portfolio.rs`

### GitHub Security / Quality 高优问题收口

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/security-quality-remediation.md`
- Handoff: `docs/handoffs/2026-04-15-security-quality-remediation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bun run build:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: 已收口 research proxy URL 校验、session / company profile 路径组件校验、console 明文 user id 日志、Actions workflow 权限与一批高优 transitive dependency；剩余值得关注但未继续深挖的主要是 desktop GTK/Tauri 链上的 `glib` 告警，以及 `feishu-sdk -> salvo_core` 带入的低优 `rand 0.10.0`
- Next entry point: `docs/handoffs/2026-04-15-security-quality-remediation.md`

### Pre-Compact KV Cache 稳定性收口

- Status: done
- Date: 2026-04-15
- Plan: `docs/archive/plans/kvcache-stability-before-compaction.md`
- Handoff: `docs/handoffs/2026-04-15-kvcache-stability-before-compaction.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-agent-codex-cli`
- Current conclusion: Hone 现在不会在下一次 compact 之前，由自身更小的 recent restore window、按当前用户输入动态变化的 system prompt related-skill block，或 `codex_cli` 的额外 20 条裁剪，提前制造可避免的 cache miss；compact 之后 prefix 变化仍视为正常边界
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-13

### Multi-Agent 输出净化与 think/tool_call 泄漏修复

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/multi-agent-output-sanitization.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-feishu`, `cargo test -p hone-channels sanitize_user_visible_output -- --nocapture`, `cargo test -p hone-channels restore_context_sanitizes_polluted_assistant_history -- --nocapture`, `cargo test -p hone-channels internal_search_note_does_not_skip_answer_stage -- --nocapture`
- Current conclusion: 统一新增用户可见输出净化层后，multi-agent 搜索阶段不再把带 `<think>` / `<tool_call>` 的内部工作稿直接返回给用户；`AgentSession`、`restore_context`、`session_compactor` 会在持久化、恢复与压缩路径上拦截或清洗污染内容；Feishu / Telegram / Discord / iMessage 用户可见回复现统一隐藏 `<think>`，Feishu / iMessage 流式 formatter 也会吞掉 `<tool_call>` / `<tool_result>` / `<tool_use>` 内部块
- Next entry point: `crates/hone-channels/src/runtime.rs`

### 跨渠道富文本分段渲染修复

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/cross-channel-rich-text-segmentation.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-telegram -p hone-discord -p hone-feishu`, `cargo test -p hone-channels outbound::tests::split_html_segments_rebalances_open_tags_across_segments -- --exact`, `cargo test -p hone-channels outbound::tests::split_markdown_segments_rebalances_code_fences_across_segments -- --exact`
- Current conclusion: 共享分段层现在新增 HTML / Markdown 两种 format-aware segmenter；Telegram 长回复会在分段边界自动补全并重开 HTML tag，Discord / Feishu 会在 Markdown 代码块跨段时自动补全并重开 fence，避免富文本结构在长回复发送时被切坏后降级或回退纯文本
- Next entry point: `crates/hone-channels/src/outbound.rs`

### 飞书表格语法护栏

- Status: done
- Date: 2026-04-13
- Plan: `docs/archive/plans/feishu-table-sanitization.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-feishu markdown`, `cargo test -p hone-channels prompt`
- Current conclusion: 飞书提示词已明确禁止模型手写原始 `<table .../>` 卡片标签；运行时会继续自动把标准 Markdown 表格转换成飞书表格，同时对损坏、截断或 schema 错误的 raw table 做规范化/降级，避免坏标签直接投递到用户侧
- Next entry point: `docs/archive/plans/feishu-table-sanitization.md`

### Skill Runtime 对齐 Claude Code 与 Multi-Agent 优化提案

- Status: done
- Date: 2026-04-13
- Plan: N/A
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 已完成一份 proposal，对比 Claude Code 官方 skill 模型与 Hone 当前实现差异，并分析 `multi-agent` runner 下 skill 的实际使用模式；提案建议把 active skill state 提升为 runner 一等状态，随后再补 `allowed-tools` / `context: fork` / supporting files 等执行与作者体验能力
- Next entry point: `docs/proposals/skill-runtime-multi-agent-alignment.md`

## 2026-04-14

### 会话上下文超限自动恢复与错误净化

- Status: done
- Date: 2026-04-14
- Plan: `docs/archive/plans/context-overflow-recovery.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo test -p hone-channels context_overflow_auto_compacts_and_retries_successfully -- --nocapture`, `cargo test -p hone-channels context_overflow_failure_is_rewritten_to_friendly_message -- --nocapture`
- Current conclusion: `AgentSession` 现在会识别上下文超限错误并在同一 turn 内先强制 compact 当前 session、再重新准备 execution 自动重试一次；若恢复后仍失败，用户只会看到稳定友好的提示，不再看到 `bad_request_error`、`invalid params`、`context window exceeds limit` 等底层 provider 原始报错
- Next entry point: `crates/hone-channels/src/agent_session.rs`

## 2026-04-12

### v0.1.10 CLI Onboarding Provider 配置补齐

- Status: done
- Date: 2026-04-12
- Plan: N/A
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-cli`, `bash scripts/prepare_release_notes.sh v0.1.10 /tmp/release-notes-v0.1.10.md`
- Current conclusion: `hone-cli onboard` 现在会明确要求用户对 `FMP` 和 `Tavily` API key 做出“填写或跳过”的选择；`FMP` 首装写入改为优先使用 `fmp.api_keys`，并清空旧的 `fmp.api_key` 兼容字段；对应 release notes 已补齐到 `docs/releases/v0.1.10.md`
- Next entry point: [v0.1.10 release](https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.1.10)

### v0.1.9 Release 失败修复与补发

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/release-v0.1.9-publish-fix.md`
- Handoff: N/A
- Decision / ADR: N/A
- Related PRs / commits: `a505060` (`docs: restore v0.1.9 release notes`)
- Related runbooks / regressions: `bash scripts/prepare_release_notes.sh v0.1.9 /tmp/release-notes-v0.1.9.md`, GitHub Actions `Release` run `24307695528`
- Current conclusion: 已补齐 `docs/releases/v0.1.9.md` 并重推 `v0.1.9` tag；`ensure-release` 不再因缺失 release notes 失败，三套发布产物与 `SHASUMS256.txt` 已成功上传，Homebrew formula 同步发布完成
- Next entry point: [v0.1.9 release](https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.1.9)

### 公司画像与长期基本面追踪

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/company-portrait-tracking.md`, `docs/archive/plans/company-portrait-skill-framework.md`, `docs/archive/plans/company-research-actor-spaces.md`, `docs/archive/plans/remove-kb-memory-surface.md`
- Handoff: `docs/handoffs/2026-04-12-company-portrait-tracking.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile -- --nocapture`, `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`, `bun run --cwd packages/app typecheck`
- Current conclusion: Hone 已具备 Markdown 形式的公司画像与事件时间线、按 actor 展示的画像 Web 视图（允许彻底删除），以及更贴近投研档案的 `company_portrait` skill；画像文档现在直接落在 actor sandbox 的 `company_profiles/` 中，由 agent 使用 runner 原生文件读写维护，不再依赖专用 mutation tool、公共画像目录或 KB 记忆入口
- Next entry point: `docs/handoffs/2026-04-12-company-portrait-tracking.md`

### CLI 首装 Onboarding 与安装向导

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/cli-onboarding-install-wizard.md`
- Handoff: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `bash tests/regression/manual/test_install_bundle_smoke.sh`, `cargo check -p hone-cli`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已支持首装 `onboard/setup` TUI，能够探测本机 runner、在不强迫 Hone 侧填写 OpenCode provider 配置的前提下切到 `opencode_acp`，并按渠道逐个引导启用与填写本地必填字段；GitHub release 安装脚本在交互终端下会询问是否立即运行该向导
- Next entry point: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`

### Desktop Rust Check 与 IDE 语法检查解耦

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/desktop-rust-check-workflow.md`
- Handoff: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check --workspace --all-targets`
- Current conclusion: 默认 workspace Rust 检查继续排除 `hone-desktop`；desktop crate 新增开发态 sidecar 校验豁免开关，VSCode rust-analyzer 默认携带该 env，因此 IDE / 本地 `cargo check` 不再被缺失的 Tauri bundled binaries 阻塞
- Next entry point: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`

### Hone CLI Config MVP 与可安装启动流

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/hone-cli-config-mvp.md`
- Handoff: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test -p hone-core`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已具备 `config / configure / models / channels / status / doctor / start` 管理面；shared runtime overlay service 已供 CLI 与 desktop 共用；macOS / release 安装链路支持 `hone-cli start`，且已补齐首次 runtime config seed 行为
- Next entry point: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`

### Local 私有 Workflow Runner（公司研报 v1）

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/local-workflow-runner.md`
- Handoff: `docs/handoffs/2026-04-12-local-workflow-runner.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cd local/workflow && bun test`, `cd local/workflow && bun run bootstrap-config`, `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts --outdir /tmp/local-workflow-build`, `WORKFLOW_RUNNER_PORT=3213 bun run start`
- Current conclusion: 在 `local/workflow/` 下新增独立本地 workflow runner，并在后续迭代中补齐紧凑工作台、运行级 prompt override、SSE 去重续流、停止接口、单实例串行、Python UTF-8/旧版本注解兼容，以及结构化进度与节点详情观测；当前 `company_report` 入口既可在页面里运行/观察/停止，也可通过 `bun run client` 从本机其它位置发起并监听进度
- Next entry point: `docs/handoffs/2026-04-12-local-workflow-runner.md`

## 2026-04-11

### 金融自动化合同回归闭环

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/finance-automation-contract-loop.md`
- Handoff: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_finance_automation_contracts.sh`, `bash tests/regression/run_ci.sh`
- Current conclusion: finance 固定 9 样本合同切片已从 `success=5 review=1 fail=3` 收口到 `success=9 review=0 fail=0`；剩余 skill policy wording 漂移已全部修正
- Next entry point: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`

### 大文件物理拆分重构

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/large-files-refactor.md`
- Handoff: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`
- Decision / ADR: `docs/decisions.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: runtime override和渠道启动已收口到共享层；desktop sidecar、Feishu / Telegram 渠道热点与前端 settings 纯状态逻辑已按职责拆开，验证矩阵已跑通
- Next entry point: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`

## 2026-03-31

### macOS DMG Release 打包收口

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `make_dmg_release.sh`
- Current conclusion: 新增 `make_dmg_release.sh` 并真实产出 Apple Silicon / Intel 两套 DMG；release 包内置 `hone-mcp` 与 macOS `opencode`，并补齐 packaged/runtime 启动环境与启动锁重试路径
- Next entry point: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`

### 定时任务输出净化与 Tavily 失败隔离

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-tools`, `cargo test -p hone-channels`
- Current conclusion: heartbeat / 定时任务会抽出真正 JSON 结果；Tavily 临时失败会返回脱敏 unavailable 结构，且不再持久化进会话工具上下文
- Next entry point: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`

## 2026-03-29

### 额度与定时任务可靠性修复

- Status: done
- Date: 2026-03-29
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory`, `cargo test -p hone-channels`
- Current conclusion: 普通用户每日额度调整为 12；非 heartbeat 定时任务补上“同日单次补触发”；heartbeat JSON 解析失败会安全抑制
- Next entry point: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`

## 2026-03-27

### 单一聊天范围配置与群聊忙碌态控制

- Status: done
- Date: 2026-03-27
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-core -p hone-channels`
- Current conclusion: `dm_only` 收敛为 `chat_scope`；群聊忙碌态在显式触发场景具备统一控制
- Next entry point: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`

## 2026-03-26

### 子模型配置与心跳任务调度

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory -p hone-scheduler -p hone-tools -p hone-core -p hone-web-api -p hone-channels`, `cargo check -p hone-desktop`
- Current conclusion: Desktop 支持 OpenRouter 子模型配置，会话压缩切到子模型，cron 新增 heartbeat 任务类型
- Next entry point: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`

### Session SQLite 影子写入与运行时切换

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_session_sqlite_migration.sh`
- Current conclusion: SessionStorage 已支持 `json | sqlite` 切换；SQLite shadow write 与 runtime 主读都已接入
- Next entry point: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`

## 2026-03-24

### 群聊预触发窗口统一改造

- Status: done
- Date: 2026-03-24
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-channels -p hone-core`
- Current conclusion: Telegram / Discord / 飞书群聊统一为“未触发先静默缓存、显式触发再执行”的预触发窗口模型
- Next entry point: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`

## 2026-03-22

### 多渠道附件工程化卡点

- Status: archived
- Date: 2026-03-22
- Plan: `docs/archive/plans/channel-attachment-gate.md`
- Handoff: `docs/handoffs/2026-03-22-channel-attachment-gate.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: 共享附件 ingest 已统一拦截超限附件与异常图片，并把拦截原因透出到渠道 ack
- Next entry point: `docs/handoffs/2026-03-22-channel-attachment-gate.md`

## 2026-03-19

### 真群聊共享 Session 落地

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-shared-session.md`
- Handoff: `docs/handoffs/2026-03-19-group-shared-session.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`, `cargo test -p hone-memory -p hone-channels`
- Current conclusion: 群聊会话归属改为显式 `SessionIdentity`；三渠道群消息共享上下文，Web 控制台按真实 `session_id` 浏览
- Next entry point: `docs/handoffs/2026-03-19-group-shared-session.md`

### 群聊回复追加链路统一

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-reply-append-chain.md`
- Handoff: `docs/handoffs/2026-03-19-group-reply-append-chain.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-discord -p hone-telegram`
- Current conclusion: 群聊占位符、首条 `@用户` 与多段 reply 链已在 Discord / Telegram / Feishu 统一
- Next entry point: `docs/handoffs/2026-03-19-group-reply-append-chain.md`

## 2026-03-18

### 渠道运行态心跳替代 pid 判活

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-web-api -p hone-desktop -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage`, `cargo test -p hone-core -p hone-web-api`
- Current conclusion: `/api/channels` 已改为基于 `runtime/*.heartbeat.json` 的心跳新鲜度呈现状态
- Next entry point: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`

### launch.sh 真实进程清理修复

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash -n launch.sh`, `cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: `launch.sh` 已直接启动真实 debug 二进制，pid 文件改为记录真实服务进程
- Next entry point: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`

### Discord 重复“正在思考中”排查

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- Current conclusion: 结论偏向入口被多个 consumer / 进程重复消费，而不是单次 `opencode_acp` run 自行双发 thinking
- Next entry point: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`

### Runner 切换到 Gemini 3.1 Pro

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_gemini_streaming.sh`
- Current conclusion: 默认 runner 已切到 `gemini_acp`，模型固定为 `gemini-3.1-pro-preview`
- Next entry point: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`

## 2026-03-17

### IM 渠道共享入口收口

- Status: archived
- Date: 2026-03-17
- Plan: `docs/archive/plans/attachment-ingest-unify.md`
- Handoff: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`, `cargo test -p hone-channels`
- Current conclusion: 共享 `ingress` / `outbound` 抽象已收口；Discord / 飞书附件 ingest 与 KB 管线下沉到 `hone-channels`
- Next entry point: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`

### 文档计划与 handoff 清理

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-doc-context-cleanup.md`
- Decision / ADR: `docs/adr/0001-repo-context-contract.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 清空已完成计划、合并零碎 handoff，并把 `docs/current-plan.md` 恢复为活跃任务入口
- Next entry point: `docs/handoffs/2026-03-17-doc-context-cleanup.md`

### Legacy 兼容移除与数据迁移

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体迁移细节见 handoff
- Next entry point: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`

### 项目清理（会话稳定性 / 渠道收敛）

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-project-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体清理结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-project-cleanup.md`

### 架构收敛与稳定性审计

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体审计结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`

### Identity 限额策略

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-identity-quota-policy.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体策略结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-identity-quota-policy.md`

### 运行时管理员口令拦截

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-register-admin-intercept.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体拦截链路见 handoff
- Next entry point: `docs/handoffs/2026-03-17-register-admin-intercept.md`

### Telegram 管理员白名单支持

- Status: done
- Date: 2026-04-16
- Plan: `docs/archive/plans/telegram-admin-whitelist.md`
- Handoff: `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-core`, `cargo test -p hone-channels`
- Current conclusion: `admins` 正式支持 `telegram_user_ids`，共享管理员判定已接入 Telegram，当前私聊 identity `8039067465` 已写入本地配置
- Next entry point: `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`

### 活跃计划清理

- Status: done
- Date: 2026-04-16
- Plan: N/A
- Handoff: `docs/handoffs/2026-04-16-current-plan-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: `docs/current-plan.md` 已从 10 个活跃任务收口到 4 个；6 个长期失焦或仅剩占位语义的计划已移入 `docs/archive/plans/`
- Next entry point: `docs/handoffs/2026-04-16-current-plan-cleanup.md`

### Public Website Mobile Responsive Pass

- Status: done
- Date: 2026-04-26
- Plan: `docs/archive/plans/public-mobile-responsive-pass.md`
- Handoff: `docs/handoffs/2026-04-26-public-mobile-responsive-pass.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bun run build:web:public`, `bun run typecheck:web`, Playwright mobile overflow audit
- Current conclusion: 公开站共享移动端样式已收口，首页、对话页、路线图和基础文档页在 360/390/430/768 宽度下不再横向撑宽，header 保持在视口内
- Next entry point: `packages/app/src/pages/public-site.css`

### Hone Cloud Runner + Web User API Key

- Status: done
- Date: 2026-05-04
- Plan: `docs/archive/plans/hone-cloud-runner-api-key.md`
- Handoff: `docs/handoffs/2026-05-04-hone-cloud-runner-api-key.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth -- --nocapture`, `cargo check -p hone-web-api`, `cargo check -p hone-desktop`, `tsc -p packages/app/tsconfig.json --noEmit`
- Current conclusion: 客户端新增可见 `Hone Cloud` runner，并隐藏 legacy multi-agent / standalone codex CLI 入口；Web 邀请码用户现在拥有只存 hash 的 per-user API Key，public app 提供 Bearer 鉴权的 OpenAI-compatible `/api/public/v1/chat/completions`
- Next entry point: `docs/handoffs/2026-05-04-hone-cloud-runner-api-key.md`

### Public Web Multi-Session Auth

- Status: done
- Date: 2026-05-05
- Plan: `docs/archive/plans/public-web-multi-session-auth.md`
- Handoff: `docs/handoffs/2026-05-05-public-web-multi-session-auth.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory web_auth -- --nocapture`, `cargo check -p hone-web-api -p hone-memory`, `cargo test -p hone-web-api public -- --nocapture`
- Current conclusion: public web 普通登录不再清除同一用户其它活跃 session，避免每小时健康检查自动化、用户浏览器和多设备登录互相踢掉 `hone_web_session`
- Next entry point: `memory/src/web_auth.rs`

### SEC Enrichment OpenRouter Token Cap

- Status: done
- Date: 2026-05-07
- Plan: `docs/archive/plans/sec-enrichment-openrouter-token-cap.md`
- Handoff: `docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-web-api sec_filings_enrichment --lib`, `cargo test -p hone-event-engine sec_filings_enrichment --lib`, `cargo check -p hone-web-api`
- Current conclusion: SEC filing enrichment now uses a dedicated OpenRouter provider capped by `event_engine.sec_filings.enrichment.max_summary_tokens`, so short summary output no longer inherits the global 30k completion budget that triggered OpenRouter `HTTP 402`.
- Next entry point: `crates/hone-web-api/src/lib.rs`

### SEC Enrichment Section Excerpts

- Status: done
- Date: 2026-05-07
- Plan: `docs/archive/plans/sec-enrichment-section-excerpts.md`
- Handoff: `docs/handoffs/2026-05-07-sec-enrichment-openrouter-token-cap.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-event-engine sec_enrichment --lib`
- Current conclusion: SEC filing enrichment now selects filing-aware excerpts before the LLM call. 10-Q/10-K prioritize MD&A, strategic/capital/risk/legal windows and Risk Factors; 8-K prioritizes the front-loaded exhibit/news-release narrative. The default excerpt budget is now `10_000` chars, with `7_000` / `4_500` / `2_800` retries on `Prompt tokens limit exceeded`, covering the follow-up OpenRouter failures where TEM filings still hit `5198 > 3256` and `3956 > 3256` after the first section-aware pass.
- Next entry point: `crates/hone-event-engine/src/pollers/sec_enrichment.rs`

### Public Login Production Hotfix

- Status: done
- Date: 2026-05-13
- Plan: N/A
- Handoff: `docs/handoffs/2026-05-13-public-login-prod-hotfix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`, `bun --filter @hone-financial/app test -- chat.test.ts`, `cargo test -p hone-web-api routes::public::tests::sms_phone_candidates_accept_plus_86_and_local_numbers`, Chrome headless public chat smoke
- Current conclusion: Public chat now tolerates legacy malformed history rows without crashing on `content.split`; public SMS login accepts `+86...` numbers against local-number whitelist rows and sends Aliyun requests in local-number form; production was switched to rebuilt `0.11.2` release app, with `web`, `discord`, and `feishu` reporting running.
- Next entry point: `docs/handoffs/2026-05-13-public-login-prod-hotfix.md`

### Web Direct Sandbox Isolation Hotfix

- Status: done
- Date: 2026-05-14
- Plan: `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- Handoff: `docs/handoffs/2026-05-14-web-direct-sandbox-isolation-hotfix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels sandbox --lib -- --nocapture`, `cargo test -p hone-channels prepare_ignores_repo_internal_sandbox_override --lib -- --nocapture`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop runtime_env -- --nocapture`, `cargo check -p hone-channels --tests`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- Current conclusion: Actor sandboxes no longer default to repo `data/agent-sandboxes`; repo-internal sandbox roots now fall back to a repo-external temp directory, desktop sidecar propagates that explicit sandbox root, and sandbox initialization removes legacy portfolio files before native-file runners can read them.
- Next entry point: `docs/handoffs/2026-05-14-web-direct-sandbox-isolation-hotfix.md`

### Public Login ToS Runtime Mismatch

- Status: done
- Date: 2026-05-20
- Plan: N/A
- Handoff: `docs/handoffs/2026-05-20-public-login-tos-runtime-mismatch.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/desktop-release-app-runtime.md`, `bun run build:web:public`, `cargo build --release -p hone-console-page`, `bun --filter @hone-financial/app test -- public-sms-login`
- Current conclusion: Public login failure came from runtime artifact skew: the public bundle and `hone-console-page` binary did not agree on `TOS_VERSION`. Port 8088 now serves the rebuilt public bundle with ToS `2.1`, and the rebuilt backend accepts `2.1` while rejecting stale `2.0`.
- Next entry point: `docs/handoffs/2026-05-20-public-login-tos-runtime-mismatch.md`

### FMP/Tavily Usage Throttle

- Status: done
- Date: 2026-06-21
- Plan: N/A
- Handoff: `docs/handoffs/2026-06-21-fmp-tavily-usage-throttle.md`
- Decision / ADR: N/A
- Related PRs / commits: this change set
- Related runbooks / regressions: `cargo test -p hone-tools --lib`, `cargo test -p hone-agent --lib`, `cargo test -p hone-event-engine pollers::price --lib`, `cargo test -p hone-channels heartbeat_tool --lib`, `cargo check --workspace --all-targets --exclude hone-desktop`, `bash scripts/diagnose_fmp_tavily.sh --tavily-query 'health check'`
- Current conclusion: Tavily search now uses low-bandwidth Bearer requests, usage logging, and key cooldowns; FMP data_fetch now has TTL caching; heartbeat tool calls are capped; FMP price polling now runs only during US regular-session windows by default.
- Next entry point: `docs/handoffs/2026-06-21-fmp-tavily-usage-throttle.md`

### Public Mobile Overlay And Calendar Hotfix

- Status: done
- Date: 2026-07-10
- Plan: `docs/archive/plans/public-mobile-overlay-calendar-hotfix.md`
- Handoff: `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-10-01-project-web-scheduled-results-into-a-durable-push-inbox`
- Related PRs / commits: this change set
- Related runbooks / regressions: `packages/app/e2e/public-mobile-overlays.spec.ts`, `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`
- Current conclusion: mobile push center and detail layers no longer collide with the fixed nav, inbox-open acknowledgement reliably clears the red dot without consuming future arrivals, and the finance calendar has a full-screen zoomable viewer above all page stacking contexts.
- Next entry point: `packages/app/src/pages/chat.tsx`

### v0.14.0 Apple User Client Release

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/v0.14.0-apple-user-client-release.md`
- Handoff: `docs/handoffs/2026-07-12-v0.14.0-apple-user-client-release.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
- Related PRs / commits: `60ef12c8`, tag `v0.14.0`, GitHub Actions run `29181306840`
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `cargo test -p hone-user-app`, `bash tests/regression/ci/test_hone_ios_contract.sh`, `bun run test:web`
- Current conclusion: v0.14.0 published verified macOS Universal DMG, iOS Simulator App, and Xcode assets; macOS now has a verifiable bundle-level ad-hoc signature, and Apple checksum files use portable basenames with generation-time self-validation.
- Next entry point: `docs/handoffs/2026-07-12-v0.14.0-apple-user-client-release.md`

### v0.14.1 macOS Session And Calendar Release

- Status: done
- Date: 2026-07-12
- Plan: `docs/archive/plans/v0.14.1-macos-session-calendar-release.md`
- Handoff: `docs/handoffs/2026-07-12-v0.14.1-macos-session-calendar-release.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
- Related PRs / commits: `44b39aad`, tag `v0.14.1`, GitHub Actions run `29189572109`
- Related runbooks / regressions: `docs/runbooks/public-user-macos-app.md`, `cargo test -p hone-user-app`, `bash tests/regression/ci/test_hone_ios_contract.sh`, `bun run test:web`
- Current conclusion: v0.14.1 published a verified Universal macOS DMG whose stable named WebKit data store preserves login cookies across restarts/upgrades, while the PC finance-calendar modal now remains inside short viewports with internal scrolling.
- Next entry point: `docs/handoffs/2026-07-12-v0.14.1-macos-session-calendar-release.md`

### Public Agent Workspace Redesign

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-agent-workspace-redesign.md`
- Handoff: `docs/handoffs/2026-07-13-public-agent-workspace-redesign.md`
- Decision / ADR: N/A
- Related PRs / commits: `63e91795`
- Related runbooks / regressions: `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`, responsive browser QA at 1440 x 900 and 390 x 844
- Current conclusion: `/chat` now enters a responsive HONE Agent research workspace backed by existing community, calendar, push, account, and conversation data; desktop uses three columns, mobile uses five primary tabs, and history selection or prompt send returns to the unchanged single conversation runtime without navigation.
- Next entry point: `docs/handoffs/2026-07-13-public-agent-workspace-redesign.md`

### Public Workspace Page Unification

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-workspace-page-unification.md`
- Handoff: `docs/handoffs/2026-07-13-public-workspace-page-unification.md`
- Decision / ADR: N/A
- Related PRs / commits: `affa8836`
- Related runbooks / regressions: `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`, responsive browser QA at 1440 x 900 and 390 x 844
- Current conclusion: restore, Insights, Tracking/calendar, and Account now share the Agent workspace chrome; Insights is a continuous research stream, Tracking uses a desktop month grid plus a separate mobile agenda, and Account uses a lightweight action surface.
- Next entry point: `docs/handoffs/2026-07-13-public-workspace-page-unification.md`

### Public Chat Silent Restore And History Entry

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/public-chat-history-entry.md`
- Handoff: `docs/handoffs/2026-07-13-public-agent-workspace-redesign.md`
- Decision / ADR: N/A
- Related PRs / commits: this change set
- Related runbooks / regressions: `bun run typecheck:web`, `bun run test:web`, `bun run build:web:public`, responsive browser QA at 390 x 844 and 1365 x 850
- Current conclusion: authenticated chat now renders its full shell immediately, silently restores the latest 20 messages at the bottom, and exposes mobile conversation history with stable message navigation and cursor-based older-page loading; empty histories still land on the Agent overview.
- Next entry point: `packages/app/src/pages/chat.tsx`

### GPT-5.6 Codex ACP Runtime Simplification

- Status: done
- Date: 2026-07-13
- Plan: `docs/archive/plans/gpt-5-6-codex-acp-simplification.md`
- Handoff: `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`
- Decision / ADR: `docs/decisions.md#d-2026-07-13-01-retire-in-process-function-calling-and-multi-agent`
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `tests/regression/manual/test_codex_acp_initialize.sh`, `tests/regression/run_ci.sh`
- Current conclusion: the in-process function-calling crate and sequential multi-agent runner are removed; Codex ACP now defaults to GPT-5.6 Sol/xhigh on Codex 0.144.1 and Agent Client Protocol adapter 1.1.2; static prompts no longer carry the full skill catalog.
- Next entry point: `docs/handoffs/2026-07-13-gpt-5-6-codex-acp-simplification.md`
