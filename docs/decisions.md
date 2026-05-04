# Decisions

Last updated: 2026-05-04

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
  - Price bands bypass the generic same-symbol cooldown and use price-specific `price_intraday_min_gap_minutes` plus `price_symbol_direction_daily_cap`.
  - Close price alerts are digest-only by default through `price_close_direct_enabled=false`.
  - Digest buffering treats price alerts as latest-state rows per actor/symbol/date/window, replacing older queued price rows instead of appending duplicates.
- Note: This preserves old event rows in SQLite; it only changes ids and routing for new price observations.
