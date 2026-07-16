# Decisions

Last updated: 2026-07-17

## D-2026-03-07-01 Maintain LLM Collaboration Context In-Repo

- Status: Accepted
- Decision: Keep the stable rules, repo map, current plan, decision log, and task handoffs as repository documents instead of relying on session history
- Impact: A new session should read `AGENTS.md`, `docs/repo-map.md`, and `docs/current-plan.md` first
- Details: See `docs/adr/0001-repo-context-contract.md`

## D-2026-03-07-02 Separate Long-Lived Rules From Task State

- Status: Accepted
- Decision: `AGENTS.md`, `repo-map`, `invariants`, and `decisions` only store long-lived information; `current-plan` and `handoffs` only store the current task and single handoff state
- Impact: Temporary state must not be piled into `AGENTS.md`, and long-term rules must not be written into handoffs

## D-2026-03-07-03 Make Documentation Updates Part of Done

- Status: Accepted
- Decision: Documentation maintenance is not optional; changes that affect behavior, structure, or workflow must update the matching context docs
- Impact: Delivery requires checking the context assets in addition to code and tests

## D-2026-03-07-04 Normalize User-Owned Data by `ActorIdentity`

- Status: Accepted
- Decision: All long-lived on-disk data that belongs to a user must use `ActorIdentity(channel, user_id, channel_scope)` as the ownership key instead of raw `user_id`
- Impact: Sessions, scheduled tasks, portfolios, generated image directories, and any future stores of the same kind should use the actor as the file key or query key
- Note: In direct chat, `channel_scope` is empty; in group or shared-context scenarios, the concrete channel fills in `channel_scope`

## D-2026-03-07-05 Switch Dynamic Plans to an Index Page Plus Single-Task Files

- Status: Accepted
- Decision: `docs/current-plan.md` is only an active-task index page; each parallel task uses its own `docs/current-plans/*.md`
- Impact: Parallel tasks no longer fight over one detailed plan file; starting or switching tasks now requires updating both the index page and the single-task plan page
- Note: The index page records the task name, status, ownership scope, and file links, while the detailed todo and progress live in the matching task file

## D-2026-03-07-06 Use a Minimal Handoff Policy

- Status: Accepted
- Decision: Handoffs are only kept for tasks that need transfer, pause-and-resume support, or medium-or-larger tasks with follow-up risk. When possible, prefer updating the original file instead of creating fragmented new ones.
- Impact: Small pure-execution tasks do not get a new handoff by default; handoffs should keep only the goal, result, verification, risk, and unfinished items, not a full activity log
- Note: A handoff is a relay document, not a complete operation log

## D-2026-03-08-01 Use Local SQLite for the Minimal LLM Audit Layer

- Status: Accepted
- Decision: Land the minimal viable LLM audit layer in the repo first, organizing call records by `ActorIdentity + session_id + created_at`
- Storage: Use local SQLite instead of one JSON file per call; enable WAL to balance write cost and later lookup by actor / session / time
- Coverage: function-calling agent, Gemini / Codex CLI agents, and session compression
- Retention: Keep a rolling 30-day window by default; clean once on startup and then incrementally every 100 writes
- Impact: New audit-chain changes should reuse the audit types in `hone-core` and `memory/src/llm_audit.rs` instead of reimplementing them in channel entrypoints

## D-2026-03-13-01 Unify `AgentSession` and the Channel Session Flow

- Status: Accepted
- Decision: Add `agent_session` in `hone-channels` to unify session lifecycle, system prompt construction, event listeners, and logging. Channel code should run through `AgentSession` while keeping placeholder / streaming adapters.
- Impact: Channel entrypoints should no longer build `restore_context`, `build_system_prompt`, or `ensure_session_exists` on their own; new channels or new flows must reuse `AgentSession` and `AgentSessionListener`

## D-2026-03-15-01 Disable iMessage by Default

- Status: Accepted
- Decision: The default project config keeps iMessage off, and the scheduler skips deliveries to disabled iMessage targets. The UI should no longer treat iMessage as a default channel.
- Impact: Unless runtime config is changed explicitly, iMessage will not start or receive scheduler deliveries; historical iMessage tasks are for viewing only and should not be added to any new active workstream.

## D-2026-03-17-01 Unify Agent Runtime Around Runner + Prompt Layering + Session V2

- Status: Accepted
- Decision: `AgentSession` now funnels through the `run()` entrypoint; executor selection comes from `agent.runner`; prompts are split into three layers - static system, session-fixed context, and dynamic session context; session storage is upgraded to versioned JSON v2 and explicitly stores `summary` and `runtime.prompt.frozen_time_beijing`
- Impact:
  - Channels and the Web UI no longer split into `run_blocking` / `run_gemini_streaming`
  - The Web chat SSE protocol is now `run_started / assistant_delta / tool_call / run_error / run_finished`
  - Session summaries are no longer encoded as fake `system` messages
  - The breaking config key moved from `agent.provider` to `agent.runner`
- Note: `opencode_acp` connects through `opencode acp` over stdio / JSON-RPC. Later runner hardening moved ACP continuity back to Hone's local transcript restore: `opencode_acp` and `codex_acp` both seed fresh ACP sessions from local context instead of relying on remote `session/load` replay.
- Details: See `docs/adr/0002-agent-runtime-acp-refactor.md`

## D-2026-03-18-01 Make Dynamic Plans Opt-In

- Status: Accepted
- Decision: `docs/current-plan.md` and `docs/current-plans/*.md` only track tasks that need ongoing follow-up; only medium-or-larger items, cross-turn / cross-module changes, behavior / structure / workflow changes, or tasks that need parallel collaboration, handoff, or blocker management should enter the dynamic plan docs
- Impact: Small tasks such as a single commit / sync / rebase, light script or config fixes, no-behavior-change patches, and pure copy or formatting changes are no longer mechanically written into the dynamic plan docs; the simple task todo can stay in the delivery context
- Note: The dynamic plan docs are meant to support handoff and parallel work, not to log every action

## D-2026-04-08-01 Reuse One Execution-Preparation Path for Session and Scheduler Runs

- Status: Accepted
- Decision: Keep `AgentSession` as the public session entrypoint, but move prompt-audit write, tool registry creation, runner creation, and actor-sandbox-backed request assembly into a shared `execution` layer inside `hone-channels`
- Impact:
  - Scheduler heartbeat / transient task flows must not instantiate concrete runners directly
  - `AgentSession` stays focused on session semantics such as quota, slash skill handling, and transcript persistence
  - Session compaction and prompt audit are now explicit support services rather than ad hoc logic inside the main orchestrator

## D-2026-04-09-01 Normalize Active Plans, Handoffs, and Archive Index

- Status: Accepted
- Decision: Keep `docs/current-plan.md` as an active-only index, require a concrete `docs/current-plans/*.md` file for every active tracked task, move completed plan pages into `docs/archive/plans/*.md`, and use `docs/archive/index.md` as the stable entry point for historical work. Standardize new plan / handoff / decision documents around the templates in `docs/templates/*.md`.
- Impact: Agents can no longer leave active-task links dangling without backing files, and historical work no longer depends on `docs/current-plan.md` retaining a growing "recently completed" section. Future task closure should update the archive index and, when applicable, move the plan page into `docs/archive/plans/*.md`.
- Note: Existing older documents may keep legacy formatting, but any touched or newly created task-tracking document should carry the minimal metadata and structure defined in `AGENTS.md`.

## D-2026-04-11-01 Centralize Runtime Overrides and Channel Bootstrap

- Status: Accepted
- Decision: Keep runtime data-root / skills-dir override logic in `hone-core::HoneConfig`, and keep channel startup scaffolding in `hone-channels::bootstrap_channel_runtime`. Entry binaries should only add channel-specific protocol wiring after these shared layers complete.
- Impact:
  - `hone-web-api` and channel binaries should not rewrite storage, session, or runtime directory paths ad hoc
  - New channel entrypoints should reuse shared logging, enabled-check, process-lock, and heartbeat bootstrap instead of cloning startup logic
  - Channel-local scheduler or outbound helpers should live beside the channel module instead of expanding the binary entry file

## D-2026-04-12-01 Let OpenCode Own Its User Config By Default

- Status: Accepted
- Decision: `opencode_acp` should inherit the user's local OpenCode provider/auth/model config by default. Hone may still inject a narrow custom `OPENCODE_CONFIG` for permissions and explicit overrides, but it must not replace the user's global OpenCode config root or force an OpenRouter-centric default route when `agent.opencode.*` is empty.
- Impact:
  - First-install CLI onboarding should tell users to finish provider setup in `opencode` itself
  - `config.example.yaml` should leave `agent.opencode.model / variant / api_base_url / api_key` empty by default
  - The runner should not override `XDG_CONFIG_HOME` just to apply Hone's ACP permission policy
- Note: Explicit Hone-side overrides through `agent.opencode.*` or `hone-cli models set ...` remain supported for users who want Hone to pin a different route than their local OpenCode default.

## D-2026-04-17-01 Use Inline `file://` Markers As The Canonical Local Image Contract

- Status: Accepted
- Decision: Hone assistant-visible local images use inline `file:///abs/path/to/image.png` markers in the final assistant text. Web keeps that marker in message content and renders it through the local file proxy, while outbound chat channels must parse the same text into ordered `text` / `local-image` segments and send real images instead of leaking raw local paths.
- Impact:
  - Skill or tool scripts that generate charts or other local images should expose absolute artifact paths and instruct the model to place the exact `file://` URI where the image should appear in the answer
  - `skill_tool` is responsible for validating image artifacts before they become model-visible paths
  - Web history extraction must recognize inline local image markers as attachments
  - Feishu / Telegram / Discord outbound adapters must preserve interleaved `text -> image -> text` order and replace local markers with actual uploaded channel images
- Note: v1 keeps this contract entirely inside the final assistant text and does not introduce a separate SSE media event type.

## D-2026-04-24-01 Route Price Alerts Through Directional Band Lanes

- Status: Accepted
- Decision: Price alerts use daily low/close ids plus directional intraday band ids instead of one `price:{symbol}:{date}` id per day. Intraday bands are keyed as `price_band:{symbol}:{date}:{up|down}:{band_bps}` and default to `6%` with `2pp` steps.
- Impact:
  - A low-magnitude price move can enter digest first, then a later same-day cross of `+6%/+8%` or `-6%/-8%` can still dispatch as a distinct event.
  - Price bands use `price_realert_step_pct` to form intraday lanes and `price_band_min_advance_pct` to decide whether a newer band advances far enough for a direct push.
  - Close price alerts are digest-only by default through `price_close_direct_enabled=false`.
  - Digest buffering treats price alerts as latest-state rows per actor/symbol/date/window, replacing older queued price rows instead of appending duplicates.
- Note: This preserves old event rows in SQLite; it only changes ids and routing for new price observations.

## D-2026-05-11-01 Make LLM Credentials Config-Only

- Status: Accepted
- Decision: LLM provider/profile/auxiliary/agent credentials are configured only through `config.yaml`; runtime must not use `*_API_KEY`, `*_BASE_URL`, or `api_key_env` fallback as a second truth source
- Impact:
  - `llm.providers.<symbol>.api_key/api_keys` is the preferred provider credential path, and `llm.profiles.*.provider` references that symbol
  - Legacy `llm.openrouter.api_key/api_keys` remains readable only as config-owned migration fallback, not as an env bridge
  - CLI/Desktop settings should write inline config keys and mask API-key fields in display/logs
  - Missing credentials should fail with a migration hint pointing to `config.yaml`
- Note: Child-process bridges such as passing a config-owned OpenRouter key into `opencode` are allowed when the underlying CLI has no config API, but Hone must not read parent process env vars as user LLM config.

## D-2026-07-10-01 Project Web Scheduled Results Into A Durable Push Inbox

- Status: Accepted
- Decision: Keep the canonical scheduled-task transcript unchanged for agent context and channel delivery, but project deliverable `web` scheduler results into a separate actor-scoped push store with stable push ids, deterministic summaries, full content, delivery timestamps, and server-owned `read_at` state. Public chat history and SSE render this projection as summary cards; Feishu and other channel adapters keep their existing output.
- Read semantics: Opening push `N` marks every push owned by the same Web actor with an order at or before `N` as read. Opening an older push therefore preserves newer unread state, while opening the latest push clears the aggregate unread indicator. Opening the push center itself acknowledges the latest item known at that instant; pushes arriving afterward remain unread and restore the aggregate red dot.
- API impact: Public list responses carry summaries and unread counts only; full content is fetched from an authenticated actor-scoped detail endpoint. Read state is used to drive the aggregate red dot but is not rendered as per-message read/unread copy.
- Compatibility: Historical scheduled turns without push ids are lazily backfilled on the actor's first push-list request with deterministic `legacy:*` ids and durable read state. The import is actor-scoped and idempotent, preserves any existing `read_at`, and makes pre-upgrade messages available in the same inbox as new scheduler pushes.

## D-2026-07-11-01 Separate The Public macOS App From The Local Runtime Desktop

- Status: Accepted
- Decision: Package the production public user experience as a dedicated remote-only Tauri app in `bins/hone-user-app` instead of extending the full `hone-desktop` bundle. The app opens `https://hone-claw.com/chat`, keeps only first-party Hone navigation in its WebView, and owns no backend, sidecar, ACP, MCP, channel, skill, config, or local-data lifecycle.
- Impact: Public macOS releases use `scripts/build_user_app.sh`; full local-runtime desktop packaging remains a separate lane. The user app is tested and built on macOS rather than included in the default Linux workspace gate.
- Security: The local shell ships a restrictive CSP, permits in-app HTTPS navigation only to Hone-owned hosts, and sends unrelated HTTP(S)/mail links to the system browser. The release bundle must be inspected for unexpected resources and external binaries before distribution.
- Distribution: Unsigned development machines may produce an ad-hoc signed Universal `.app` / `.dmg` for internal use. Public distribution still requires an Apple Developer ID identity and notarization.

## D-2026-07-11-02 Use One HONE Brand And Remote Boundary For Public Apple Clients

- Status: Accepted
- Decision: Public Web, macOS, and iOS clients use `HONE` as the sole user-facing product brand and share one mark/wordmark language. The native Apple shells open the production public Web experience rather than duplicating chat, push, calendar, or account business logic.
- iOS boundary: `apps/hone-ios/` is an independent SwiftUI/WKWebView project. It persists the normal WebKit login store, allows in-app navigation only to HONE-owned HTTPS hosts, hands external links to iOS, and contains no local runtime or sidecar lifecycle.
- Release impact: Tag releases upload the Universal macOS DMG, iOS Simulator app, iOS Xcode source, and Apple checksums in addition to existing CLI assets. A device IPA is only valid when Apple signing/provisioning credentials are configured and must not be inferred from an unsigned Simulator artifact.
- Compatibility: `packages/app` remains the public feature source of truth; backend API, session, push-read, channel, and storage contracts are unchanged by the native shells or brand refresh.

## D-2026-07-11-03 Layer Public Visual Ownership And Version Generated Images

- Status: Accepted
- Superseded in part by: `D-2026-07-12-02`; the CSS ownership decision remains active, while visible-history lazy calendar regeneration and client-side variant selection are retired.
- Decision: The public user client uses a layered CSS ownership model instead of page-local style strings and late ad hoc overrides. `public-foundation.css` owns HONE tokens and interaction foundations, `public-polish.css` owns shared public navigation/push components, `public-chat.css` owns the chat shell, and generated visual artifacts keep component-local styles beside their render component.
- Rendering: Generated mobile finance calendars use one Canvas 2D renderer for both new sends and visible-history lazy upgrades. Text is painted with explicit baselines and coordinates rather than rasterizing DOM line boxes. Every material artifact redesign increments the mobile filename marker; visible older versions are lazily rebuilt in the browser without mutating conversation history.
- Impact: New public visual work must extend the narrowest owning layer rather than adding a `<style>` block to a page component. Shared token changes require public-page and authenticated-chat checks; calendar composition changes require direct Canvas source-size and 390px visual verification on an iOS-compatible path.
- Compatibility: API, persistence, desktop-calendar output, Feishu, and other channel behavior remain unchanged. The version marker affects only which Web mobile image is selected or lazily regenerated.

## D-2026-07-12-01 Reserve Native Agent Runners For Trusted Administrators

- Status: Accepted
- Superseded in part by: `D-2026-07-13-01`; the in-process function-calling fallback is retired, while the native-runner administrator trust boundary and fail-closed requirement remain active.
- Decision: Treat local CLI/ACP runners as trusted-host execution rather than actor sandboxes. When a non-admin actor reaches shared execution preparation and the configured runner can access the host, route the turn through the in-process function-calling runner with an actor-bound tool registry; if no such LLM is available, fail closed. Explicit administrators retain the configured native runner.
- Data boundary: Runtime configuration/data/actor-sandbox roots use owner-only Unix permissions. Skill scripts clear inherited server environment before execution. Public credentialed CORS accepts only HONE production origins, local development origins, and exact operator-configured origins.
- Supply-chain boundary: Keep lockfile security patches current; Discord uses Serenity's native TLS backend so its fixed `tokio-tungstenite 0.21` dependency does not retain the unpatched `rustls-webpki 0.102` branch.
- Impact: Public Web users and non-admin channel users can use registered tools but cannot ask an ACP/CLI agent to inspect repository, config, database, home-directory, or process data. Native ACP remains an administrator trust boundary and must not be presented as a strict filesystem sandbox.
- Verification: Runner-selection tests cover non-admin fallback, admin retention, and fail-closed behavior; permissions, skill environment, CORS, file traversal, and cross-actor push-read tests cover the supporting boundaries.

## D-2026-07-12-02 Persist Finance Calendar Variants And Select Them Server-side

- Status: Accepted
- Supersedes: The finance-calendar rendering and compatibility portion of `D-2026-07-11-03`; generated-artifact CSS ownership remains unchanged.
- Decision: Every new public finance-calendar message must persist validated desktop and mobile PNG paths plus its month in structured session metadata. Public bootstrap/history inspect the authenticated request User-Agent and project exactly one actor-owned `finance_calendar.image_path` and variant. The user client renders that stable path and must not fetch calendar data, repaint Canvas, create a blob URL, or replace the source while restoring visible history.
- Compatibility: Historical calendar messages without metadata are projected server-side by parsing their existing image markers. Mobile requests select the second marker when available; truly desktop-only legacy messages retain the desktop image. Compatibility projection does not mutate the transcript or expose paths outside the authenticated image proxy.
- Cache: Stored image paths are immutable upload artifacts. Authenticated image responses use private immutable browser caching so repeated restores do not refetch or regenerate the artifact.
- Impact: Calendar creation still renders the two artifacts once before upload, but display and history restoration are backend-owned. A send without both validated variants fails instead of creating another incomplete message.

## D-2026-07-12-03 Stream Public Replies From The Active Safe Runner

- Status: Accepted
- Superseded in part by: `D-2026-07-13-01`; function-calling streaming and tool-loop behavior are retired, while native runner streaming, client event handling, and no-replay-after-stream-start remain active.
- Decision: Public chat streaming follows the runner selected by the shared execution boundary. Trusted administrators using Codex/OpenCode ACP keep native `agent_message_chunk` streaming; non-admin actors remain on the strict actor-bound function-calling runner and receive native OpenAI-compatible/OpenRouter SSE content and tool-call deltas. Streaming must never be implemented by slicing an already-complete response.
- Tool-loop contract: Provider streams expose structured content, reasoning, indexed tool-call fragments, and optional usage. Function calling assembles parallel tool ids/names/arguments by index, executes only the actor-bound registry, hides internal reasoning blocks across chunk boundaries, and emits visible `StreamDelta` events. A model preamble emitted before a later tool call is withdrawn with transient `StreamReset`; only the final normalized assistant response is persisted.
- Retry contract: API-key fallback is allowed only before an upstream stream starts. Once any successful streaming response has begun, transport errors surface through the existing run error state and must not replay the request, preventing duplicated output or duplicated tool execution. Providers without native tool streaming retain a single-response compatibility fallback.
- Client impact: The public user client edits the existing in-thread thinking card, batches token deltas once per animation frame, handles reset in place, and preserves an explicit error phase on failed streams. The public SSE protocol therefore includes `assistant_reset` in addition to the existing `run_started / assistant_delta / tool_call / run_error / run_finished` events.
- Security: This decision does not reopen native ACP/CLI access for ordinary users and does not change final transcript ownership, quota, actor isolation, or persistence boundaries established by `D-2026-07-12-01`.

## D-2026-07-13-01 Retire In-Process Function Calling and Multi-Agent

- Status: Accepted
- Superseded in part by: `D-2026-07-15-01` restores the strict actor-bound function-calling safety fallback; `D-2026-07-15-02` restores the full investment workflow and response-format prompt; `D-2026-07-15-03` adds code-level enforcement for deep single-stock replies. `multi-agent` and user-selectable `function_calling` remain retired.
- Supersedes: The in-process fallback portion of `D-2026-07-12-01` and the function-calling stream/tool-loop portions of `D-2026-07-12-03`.
- Decision: Remove the in-process `function_calling` agent and the sequential `multi-agent` runner. Use the unified configured runner for both conversation and transient heartbeat execution, with `codex_acp` as the default local path.
- Default: `gpt-5.6-sol` with `xhigh` reasoning effort through `@openai/codex >= 0.144.1` and `@agentclientprotocol/codex-acp >= 1.1.2`.
- Compatibility: Old `function_calling` and `multi-agent` config values fail explicitly; they do not silently select another runner. Historical records may retain the old names.
- Security: Native CLI/ACP runners remain administrator-only trusted-host capabilities. With the actor-bound fallback removed, non-admin native-runner requests fail closed and should use `hone_cloud`; this change does not present an ACP/CLI subprocess as a strict filesystem sandbox.
- Prompt impact: Keep `soul.md` as a compact persona layer, keep hard runtime policies in Rust, attach only query-relevant skill summaries to the current turn, and use `discover_skills` instead of injecting the full catalog into every system prompt.

## D-2026-07-15-01 Restore The Strict Actor-Bound Safety Runner

- Status: Accepted
- Supersedes: The non-admin fail-without-routing portion of `D-2026-07-13-01`; preserves its retirement of `multi-agent` and of `function_calling` as a user-selectable primary runner.
- Decision: Keep `codex_acp` as the default primary runner for explicit administrators, but route every non-admin persistent conversation or transient scheduler task to the in-process function-calling runner whenever the configured primary runner is a trusted-host CLI/ACP. The fallback receives only the actor-bound Hone tool registry and never launches the native subprocess.
- Security: If the actor-bound LLM is unavailable, fail closed. Do not weaken the administrator boundary or present ACP working directories as filesystem sandboxes.

## D-2026-07-15-02 Restore The Full Investment And Response-Format Prompt

- Status: Accepted
- Supersedes: The compact-`soul.md` portion of `D-2026-07-13-01`; runner retirement, Codex ACP defaults, and query-relevant skill disclosure remain unchanged.
- Decision: Treat `soul.md` as the complete behavioral contract for investment reasoning and answer composition, using the pre-`71a4498e` large prompt as the baseline. It must retain task routing, single-stock and sector analysis order, fact/inference/conclusion/action separation, valuation discipline, Bull/Bear/Base framing, financial-comparison fields, answer ordering, anti-repetition rules, and user-facing capability behavior.
- Layering: Rust prompt policy remains the hard enforcement layer for live-data truth sources, privacy, storage, channels, cron, and security. Channel-specific Markdown/HTML guidance takes precedence over old generic formatting examples.
- Runtime truth: Canonical `soul.md` is authoritative. Effective-config generation refreshes the runtime prompt copy even when a stale destination exists; direct edits to `data/runtime/soul.md` are not durable configuration.

## D-2026-07-15-03 Enforce Deep Single-Stock Evidence And Reply Structure In Code

- Status: Superseded in scope by `D-2026-07-16-01`; its nine-section deep-analysis answer contract and fail-closed final validation remain active.
- Decision: A recognized security plus deep-analysis/outlook intent (including quarter outlook, “起飞”, valuation, fundamentals, earnings, or buyability) is a code-enforced route, not only a system-prompt suggestion. Before the runner starts, Hone resolves the entity and verifies a same-symbol positive quote; deep routes additionally fetch current profile, financial statements, company news, and—when the question is forward-looking—the next 120 days of earnings-calendar evidence.
- Answer contract: The turn suffix requires nine numbered sections in the established order: conclusion; company/business model; moat; industry/competitors; financial quality; at least two suitable valuation methods; Bull/Bear/Base; catalysts/risks/falsification; and an actionable buy/wait/reduce/sell/observe recommendation. It also requires a data timestamp and explicit fact/inference separation.
- Enforcement: The final answer is validated before persistence. A non-conforming candidate is retried once with the missing contract items. Since `D-2026-07-16-02`, candidate text and retry resets remain internal; if the retry still fails, Hone fails closed without first exposing a superficial or unverified investment conclusion.
- Scope: Simple quote-only questions still receive concise answers, but they must pass the same entity and same-symbol quote preflight. Missing or mismatched quote/financial evidence stops numeric conclusions; history, profiles, model memory, and unrelated symbol fields cannot fill the gap. The guard applies only to direct, uniquely identified single-security turns: scheduler/heartbeat envelopes and multi-security comparisons retain their own execution contracts, generic finance/report acronyms are not ticker hints, and entity search must return an exact symbol match rather than silently selecting its first approximate result.

## D-2026-07-16-01 Make Security Entity Resolution The First Investment Stage

- Status: Accepted
- Supersedes: The single-security-only scope, text-envelope detection, acronym denylist, and final-stage ticker inference in `D-2026-07-15-03`. Its deep-analysis evidence set, nine-section answer format, one internal retry, and fail-closed behavior remain active.
- Decision: Every investment turn enters one entity-resolution stage before company-specific planning or generation. The stage produces one of four explicit outcomes: no named security for a genuinely broad macro/industry request, one or more normalized securities, an ambiguous candidate set, or an unresolved entity. Named companies, aliases, tickers, share classes, and comparison lists must be resolved against current-turn DataFetch `search` results; history and model memory cannot establish identity.
- Matching: An explicit ticker requires an exact ticker candidate. A company name or alias may be ranked against provider candidates, but equally plausible listings or share classes remain ambiguous and must be clarified; the runtime must never select the first search result by position. All resolved securities require a positive same-symbol current quote before numeric conclusions.
- Bare ticker path: A normal bare ticker such as `NBIS` or contextual lowercase `nbis` is a first-class lexical candidate and must not depend on auxiliary-LLM JSON success. It becomes a security entity only after DataFetch search returns an exact symbol; assignment keys, report periods, broad industry/metric acronyms, and unrelated lowercase words must remain outside this fast path. Complex names and aliases still use structured extraction, and a valid empty structured result must not be repopulated from unconfirmed uppercase tokens.
- Data flow: `AgentSession` prepares one structured investment contract and carries it through execution, response validation, reset, and retry. `AgentTurnOrigin` is typed as interactive, scheduled, or heartbeat. Scheduled execution passes the original task body separately from its delivery envelope, so metadata such as `REPEAT` can never be parsed as a security and scheduler/heartbeat cannot bypass entity resolution.
- Asset routing: After exact-symbol search and a positive same-symbol quote, every non-crypto security must be classified from the exact profile's structured fields before deep evidence is selected. `isEtf` / `isFund` routes to fund evidence (`profile + ETF holdings + news`), an explicit company profile routes to equity evidence (`profile + meaningful financial statements + news`), and an exact crypto-market search result routes to crypto evidence (`quote + news`). An empty company financial response cannot prove that an unknown instrument is a fund; unknown asset types fail closed. Fund and crypto routes must not call company financials or earnings calendars, and crypto must not require a stock profile.
- Answer contracts: Interactive deep single-security requests retain an asset-specific nine-section contract. Equity answers cover the company/business, moat, competition, financial quality, valuation, scenarios, risks and action trigger; fund answers cover mandate, holdings/concentration, exposure, liquidity, costs/tracking, valuation context, scenarios, risks and action trigger; crypto answers cover market role, structure, liquidity, token/supply evidence when available, valuation context, scenarios, risks and action trigger. Multi-security comparisons must cover every resolved symbol with its own asset-appropriate evidence, state a data timestamp, and include risks or falsification conditions. Scheduled and heartbeat turns use the same entity, quote and asset-route gate but keep their delivery-specific response shape.
- Provider semantics and retry audit: A successful semantic-empty response is distinct from an authentication/provider failure. Empty profile/search/quote/financial/holdings payloads are not cached. Only authentication, quota, or rate-limit failures may rotate FMP keys; transport, parse, ordinary provider, and server errors stop without fanning out across credentials. Forbidden evidence calls are accumulated across the initial draft and retry so a clean retry cannot erase an invalid fund/crypto company-data call. Response-format and numeric validation must use the same sanitized visible content that SSE, persistence, and final delivery expose; raw `<think>` or tool-protocol blocks remain internal and cannot create or hide user-facing sections.
- Failure behavior: Ambiguous or unresolved identity, unavailable entity search, missing same-symbol quotes, or a response that omits required entities or evidence fails closed with a user-facing clarification or availability message. Internal parser labels, provider payloads, and guard diagnostics must not leak into the answer.

## D-2026-07-16-02 Commit Investment Replies Only After Validation

- Status: Accepted
- Supersedes: The user-visible `StreamReset` withdrawal behavior for investment contract retries in `D-2026-07-15-03` and `D-2026-07-16-01`; their evidence, retry, and fail-closed requirements remain active.
- Decision: Investment runner attempts execute behind a session-owned deferred-output boundary. Progress and sanitized tool status may remain visible, but candidate `StreamDelta`, `StreamReset`, `StreamThought`, and attempt-local `Error` events never reach clients. After entity/evidence checks, contract validation, final sanitization, and attachment processing succeed, the session emits exactly one canonical final answer.
- Retry and failure semantics: Empty-success, transient-runner, investment-contract, and context-overflow retries all reuse the deferred boundary. Attempt flags cannot claim that hidden text or a hidden error was user-visible. A terminal failure is emitted once by the outer session; a successful run emits one terminal result. Web must not synthesize a second `run_finished` after the session listener has handled `Done`.
- Scope: Non-investment turns keep native live streaming and tool-branch reset behavior. This decision only trades token-by-token display for validated one-shot display on guarded investment turns, where correctness and stable rendering take priority over draft latency.

## D-2026-07-16-03 Make Active Chat Runs Server-Authoritative

- Status: Accepted
- Decision: A running Web chat turn is identified by a server-owned `run_id`, `started_at_ms`, phase, safe status text, and update time in an actor/session-scoped active-run registry. Public bootstrap/history and live SSE read the same state. Conversation quota `in_flight` remains a billing reservation count and must never be used as proof that a runner is alive.
- Refresh and interruption semantics: Refresh only reattaches the UI observer to the existing detached runner; it never reposts the prompt. The recovered timer uses the original server start time. When the current process has no matching active run and the persisted interactive transcript still ends with an unanswered user turn, bootstrap returns an explicit interrupted state instead of fabricating a new thinking timer. Scheduled-push projections do not hide an older unanswered interactive turn.
- Progress and security: Deferred investment output remains final-only. `run_progress` and `tool_call.public_status_text` may expose only fixed user-safe phase text; raw tool names, provider messages, reasoning, attempt errors, draft deltas, and resets are not a substitute for progress and must not leak through the public UI.
- Lifecycle and operations: One session may have only one registered active run. An RAII guard clears the matching `run_id` after terminal persistence, including error and early-return paths. Controlled `hone-cli start` shutdown checks `/api/runtime/active-chat-runs` and waits for zero active turns up to a bounded timeout before terminating children. A process crash is represented as an interrupted turn after restart rather than a still-running task.
- Deployment boundary: The current registry is process-local because the production Web runner is single-instance. A future multi-Web-instance topology must replace it with a shared leased/fenced run registry or enforce sticky ownership; quota counters are not an acceptable distributed substitute.

## D-2026-07-17-01 Make Investment Facts And Repair Server-Authoritative

- Status: Accepted
- Supersedes in part: The model-authored timestamp/current-price wording and whole-answer retry behavior left implicit by `D-2026-07-16-01` and `D-2026-07-16-02`. Their entity-first, asset-aware, deferred-output, and fail-closed contracts remain active.
- Entity scope: Every nonempty turn first resolves to exactly one of `securities`, actor `portfolio`, `broad`, `confirmed no entity`, or `needs clarification`. Exact ticker mentions use a deterministic DataFetch exact-symbol path. Possible named companies that remain unresolved may use the auxiliary LLM, but the extraction has a 15-second bound and timeout/provider/malformed/incomplete results fail closed without silently keeping a partial comparison set. The auxiliary schema distinguishes a valid ordinary-finance result (`entities=[]`, `unresolved_mentions=[]`) from an actually named but unresolved security (`unresolved_mentions` nonempty); only the former may enter confirmed-no-entity.
- Portfolio scope: Personal portfolio/watchlist requests read the actor-scoped portfolio tool once as the membership/cost truth source. If the user asks for current price, change, or performance analysis, an explicitly requested ticker, or a bounded service-selected portfolio subset when no ticker was named, must still enter the exact DataFetch entity/quote contract; portfolio state is not live-market evidence. The snapshot records total/included/truncated and market-symbol total/included/omitted counts, and the answer must disclose limited coverage instead of silently describing it as the full portfolio. A read preflight does not satisfy a requested write/delete operation.
- Canonical visible prefix: The server, not the model, owns the first Beijing data-time line, normalized security identity, same-symbol quote, price change, quote source time, and verified-fact labels. Those fields are rendered from the prepared investment contract before the sanitized model body. A model cannot omit, contradict, or falsely deny current quote capability after DataFetch supplied a verified quote; a later format failure retains the verified time/entity/quote instead of rewriting it as a provider outage.
- Evidence semantics: Provider payloads are reduced to typed, symbol-matched evidence before prompt injection. Profile snapshot-price fields are removed; company financial evidence uses explicitly labeled annual income-statement metrics and does not reinterpret net income as cash flow or net cash; missing cash/debt/FCF/capex, consensus, forward, or peer/history valuation data is explicit. Exact asset fields route equity, ETF/fund, and crypto through separate evidence contracts. Entity-filtered news excludes acronym collisions such as mortgage `RMBS`; every claimed current event must match a verified real absolute date and full source domain in the same clause, while mixed-market research uses the relevant exchange-local date for each requested market.
- Repair semantics: The initial sanitized candidate remains the repair source. One contract repair may rewrite missing structure, but it must not re-run the original user request or erase already verified facts. Persistent operations are execute-once: ambiguous native-runner traces suppress automatic replay rather than risk duplicate writes. Guarded attempts remain hidden and publish one canonical answer only after validation.
- Stream terminal semantics: The session `Done` event is the single terminal authority. Web sends at most one `run_finished`, closes the stream immediately after it, ignores late frames, and preserves the server's original run start time across refresh. This prevents a completed answer from flashing into a second run or resetting its elapsed timer.
- Operational diagnosis: Live FMP/DataFetch probes and Tavily search diagnostics succeeded during this repair. The reported NBIS/RMBS/INTL symptoms were caused by Hone's internal entity/asset routing and response-format validation/repair path, not by a general FMP or Tavily outage.
- Migration and rollback: This decision changes runtime orchestration, validation, and Web stream handling only; it has no database or durable-storage migration. Roll back by restoring the prior server/frontend revision, rebuilding assets/binaries, and performing the controlled runtime restart. No actor session, portfolio, or other durable data transformation is required.
- Verification: Unit and integration coverage must include lowercase/common tickers (`RMBS`, `NBIS`, `INTL`), exact symbol and asset routing, quote timestamp/freshness, unsupported financial claims, market/sector templates, repair preservation, execute-once operations, terminal fencing, and live DataFetch probes.
