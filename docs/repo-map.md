# Repo Map

Last updated: 2026-03-26

## Purpose

- Give a new session or model a low-cost entry point: understand the structure first, then read the source in depth
- Record only high-value, relatively stable structural information; task-level state belongs in `docs/current-plan.md`

## Source of Truth

1. Code and tests
2. `README.md`
3. `Cargo.toml` and each crate `Cargo.toml`
4. `package.json` and each package config
5. `config.example.yaml`

`docs/technical-spec.md` has been refreshed to match the current implementation and can be read as a structured supplement, but its priority is still lower than code, tests, README, and the various manifests.

## Repository Overview

- `docs/`
  - `current-plan.md`: active task index
  - `current-plans/`: single-task plan pages for parallel work
  - `handoffs/`: handoff summaries that only keep information needed for the next person
  - `open-source-prep.md`: allowlist / denylist and cleanup checklist before copying to a public repo
- `crates/`
  - `hone-core`: foundational capabilities such as the config façade / submodules, logging, errors, and agent context
  - `hone-llm`: model provider abstraction and OpenRouter integration
  - `hone-tools`: tool traits, registry, and built-in tools
  - `hone-integrations`: external integrations such as X, Feishu, XHS MCP, and image generation
  - `hone-scheduler`: scheduled task orchestration
  - `hone-channels`: channel runtime, `HoneBotCore`, unified `agent_session` orchestration, and the separate `runners` execution layer; it also hosts shared `ingress` (incoming envelope / actor scope / dedup / session lock / group pretrigger window), `outbound` (placeholder / reasoning / chunking / stream probes), repo-external actor sandbox management, and the attachment ingest / KB persistence pipeline split across `attachments/{ingest,vision,vector_store}.rs`. Feishu / Discord / Telegram attachment size and image-dimension gates are also centralized here.
- `agents/`
  - `function_calling`: function-calling agent core
  - `gemini_cli`, `codex_cli`: CLI agent adapters
  - `gemini_acp`, `codex_acp`, `opencode_acp`: agent runner adapters based on ACP stdio / JSON-RPC
- `memory/`
  - Local storage abstractions for sessions, identity quotas, portfolios, cron jobs, and LLM audit logs
  - `memory/src/session.rs` currently stores versioned session JSON (v3) and explicitly persists `summary`, `runtime.prompt.frozen_time_beijing`, recoverable `tool` result messages, and the session ownership field `session_identity`
  - `memory/src/session_sqlite.rs` hosts the SQLite-backed session persistence used by both shadow backfill and runtime reads/writes when `storage.session_runtime_backend=sqlite`
  - `memory/src/quota.rs` stores `success_count` / `in_flight` in JSON files by `ActorIdentity` and by Beijing date
- `bins/`
  - `hone-console-page`: Web console backend, static asset hosting, and API
  - `hone-cli`: local REPL
  - `hone-mcp`: local stdio MCP server that exposes Hone built-in tools to ACP runners
  - `hone-imessage`, `hone-telegram`, `hone-discord`, `hone-feishu`: channel entrypoints, with Feishu / Telegram now split into thin `main.rs` façades plus sibling modules
  - `hone-desktop`: Tauri desktop host with a thin `main.rs` façade, command handlers in `commands.rs`, backend / sidecar lifecycle in `sidecar.rs`, tray extension points in `tray.rs`, and the desktop window packaging flow
- `config.yaml` / `data/runtime/`
  - `config.yaml` is the read-only seed template and source of default comments / values
  - `data/runtime/config_runtime.yaml` is the effective runtime base created on first startup
  - `data/runtime/config_runtime.overrides.yaml` stores Desktop / automation overrides and is merged on load
- `packages/`
  - `app`: SolidJS web console
  - `ui`: shared UI components and context
- `skills/`
  - In-repo skill definitions
- `tests/regression/`
  - `ci/`: CI-safe
  - `manual/`: manual regression tests that depend on an external CLI or account

## Key Entry Points

- Web console backend: `bins/hone-console-page/src/main.rs`
- Web console frontend: `packages/app/src/app.tsx`
- CLI: `bins/hone-cli/src/main.rs`
- Channel runtime export: `crates/hone-channels/src/lib.rs`
- `AgentSession` abstraction: `crates/hone-channels/src/agent_session.rs`
- Shared ingress model: `crates/hone-channels/src/ingress.rs`
- Shared outbound model: `crates/hone-channels/src/outbound.rs`
- ACP MCP bridge: `crates/hone-channels/src/mcp_bridge.rs`
- Actor sandbox: `crates/hone-channels/src/sandbox.rs`
- Attachment ingest / KB pipeline: `crates/hone-channels/src/attachments.rs` and `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
  - Enforces shared attachment gates across channels: 5 MB for generic attachments, 3 MB for images, plus rejection of extreme aspect ratio, resolution, or pixel-count cases. Rejected attachments never enter the prompt or KB.
- Runner contract and ACP / Gemini execution layer: `crates/hone-channels/src/runners/`
  - `mod.rs`: runner exports
  - `types.rs`: shared runner trait / request / event / result types
  - `acp_common.rs`: shared helpers for ACP stdio / JSON-RPC
  - `gemini_cli.rs`, `gemini_acp.rs`, `codex_acp.rs`, `opencode_acp.rs`: runner implementations
- Prompt layering: `crates/hone-channels/src/prompt.rs`
  - Injects the global finance-domain constraints in one place: no stock-picking recommendations, reject non-finance questions, warn users not to blindly follow buy or sell advice, and keep greetings short
- Tool registry entry point: `crates/hone-tools/src/lib.rs`
- Config sample: `config.example.yaml`

## Main Flow

1. A channel entrypoint or the Web API receives user input and performs protocol parsing, allowlist checks, and explicit-trigger detection on the channel side
2. `hone-channels::ingress` centralizes actor scope, chat mode, deduplication, session serialization, shared group pretrigger buffering, and `IncomingEnvelope`
3. `hone-channels::AgentSession::run()` orchestrates the session, prompt layering, tool registration, actor sandbox selection, and persistence; it now needs an explicit distinction between:
    - `ActorIdentity`: who is executing this request
    - `SessionIdentity`: which history this message should be written into (group-chat shared sessions are controlled by it)
4. `hone-channels::runners` executes the chosen runtime based on `agent.runner` and maps provider / CLI events back into unified session events. ACP runners now include a local `hone-mcp` server so Hone built-in tools are exposed as MCP tools to the underlying agent. Channel runners default to a repo-external actor sandbox.
5. `hone-channels::AgentSession::run()` stores parseable tool-call results returned by the runner into the session for future cross-turn recovery; `hone-channels::outbound` and each channel adapter consume the unified events and finish placeholder / reasoning / chunked / streaming responses according to platform capability
6. `hone-tools` provides data, skills, search, scheduled-task, and other capabilities
7. `memory` reads and writes local sessions, quotas, portfolios, and cron jobs
    - `memory/src/quota.rs` keeps a daily successful-reply quota for each user-initiated conversation
    - `memory/src/llm_audit.rs` uses SQLite to record LLM call audit logs archived by `ActorIdentity`
    - Session persistence is controlled by `storage.session_runtime_backend`; `json` reads from local files, `sqlite` reads from `storage.session_sqlite_db_path`, and JSON can still be dual-written as a rollback mirror through `storage.session_sqlite_shadow_write_enabled`
    - Heartbeat-style cron jobs are still stored in the same cron store; they are identified by `repeat=heartbeat` and a `heartbeat` tag, then polled every 30 minutes instead of a fixed clock time
8. Responses are sent back to the originating channel; the Web console streams `run_started / assistant_delta / tool_call / run_error / run_finished` via v2 SSE events

## Desktop Structure

- The Tauri host lives in `bins/hone-desktop/`
- `bins/hone-desktop/src/{main.rs,commands.rs,sidecar.rs,tray.rs}` now separates the builder façade, Tauri command handlers, backend lifecycle, and tray extension point
- The desktop app supports two backend modes:
  - `bundled`: Tauri starts the built-in `hone-console-page` sidecar and points the frontend API at a local loopback address
  - `remote`: Tauri does not start a local backend; the frontend connects directly to a remote HTTP base URL
- Runtime configuration is layered: `config.yaml` seeds `data/runtime/config_runtime.yaml`, and writable changes are isolated in `data/runtime/config_runtime.overrides.yaml`; deleting the runtime base is the supported reset path
- Desktop agent settings now expose both the primary opencode/OpenRouter model and `llm.openrouter.sub_model`; the sub-model is reserved for cheaper background work such as heartbeat checks and session compression
- In `bundled` mode, Tauri also starts or stops `hone-imessage` / `hone-discord` / `hone-feishu` / `hone-telegram` according to the layered runtime config in the application data directory; each channel process writes a `runtime/*.heartbeat.json` with its PID every 30 seconds, and the console backend uses that to determine runtime status
- Frontend backend runtime lives in `packages/app/src/context/backend.tsx` and `packages/app/src/lib/backend.ts`
- `hone-console-page` `/api/meta` handles version and capability negotiation

## Web Console Structure

- Route entrypoint: `packages/app/src/app.tsx`
- Pages: `packages/app/src/pages/`
- Domain state: `packages/app/src/context/`
- Composite components: `packages/app/src/components/`
- API access and data transformation: `packages/app/src/lib/`

## Common Coupled Changes

- Adding a tool:
  - Change `crates/hone-tools/src/*`
  - Update `agents/function_calling` if needed
  - If the Web UI needs to show it, also update `bins/hone-console-page/src/main.rs` and the frontend pages
- Adding a Web page or dashboard:
  - Change `packages/app/src/pages/*`
  - Change `packages/app/src/context/*` and / or `packages/app/src/lib/*`
  - If the backend API is insufficient, add the Web bin API
- Adjusting desktop backend switching or sidecar lifecycle:
  - Change `bins/hone-desktop/src/{main.rs,commands.rs,sidecar.rs,tray.rs}`
  - Change `packages/app/src/context/backend.tsx` and / or `packages/app/src/lib/backend.ts`
  - Update `bins/hone-console-page/src/main.rs` and the runtime config loading for the channel bins if needed
- Adding channel behavior:
  - Change the matching `bins/*`
  - Update `hone-channels`, `hone-core`, or `memory` if needed
  - If the change touches incoming envelopes, dedup, actor scope, placeholder / streaming delivery, or attachment persistence, start with `crates/hone-channels/src/ingress.rs`, `crates/hone-channels/src/outbound.rs`, and `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
- Adjusting persistence structure:
  - Start with `memory/`
  - Then check the Web API, channel entrypoints, and frontend pages that depend on it
- Adjusting identity quotas or limits:
  - Start with `memory/src/quota.rs` and `memory/src/cron_job.rs`
  - Then check `crates/hone-channels/src/agent_session.rs` and `crates/hone-channels/src/scheduler.rs`
  - If the Web UI is affected, also check `crates/hone-web-api/src/routes/chat.rs`, `crates/hone-web-api/src/routes/cron.rs`, and `packages/app/src/lib/api.ts`
- Adjusting the agent execution path:
  - Start with `crates/hone-channels/src/agent_session.rs`
  - Then check `crates/hone-channels/src/prompt.rs`, `crates/hone-channels/src/core.rs`, and `crates/hone-channels/src/sandbox.rs`
  - If the Web UI is affected, also check `crates/hone-web-api/src/routes/chat.rs` and `packages/app/src/context/sessions.tsx`
- Adjusting LLM audit:
  - Start with `memory/src/llm_audit.rs`
  - Then check `crates/hone-channels/src/core.rs` and `agents/*`

## Fragile Areas / Notes

- `docs/technical-spec.md` is aligned with the current Rust implementation, but if module boundaries or default wiring change again, it still needs to be kept in sync so it does not drift
- Channel runners now start from a repo-external sandbox root by default; if a CLI starts reading higher-level repo rule files again, check `crates/hone-channels/src/sandbox.rs` and the runner `cwd` / config injection logic first
- `ChatMode` only means "this message came from a direct chat or a group chat"; do not treat it as the source of truth for session ownership. Use `SessionIdentity` for shared group context.
- Telegram / Discord / Feishu group chats now share one model: untriggered text is buffered in a short pretrigger window, and only an explicit `@bot` / reply-to-bot trigger flushes that buffered text into the shared group session before `AgentSession::run()`.
- Scripts in `tests/regression/manual/` depend on local environment state or external accounts and must not be promoted to default CI gates
- iMessage capabilities depend on local macOS permissions and cannot be assumed to work in CI or on non-macOS environments
- Desktop packaging depends on a local Rust + Tauri toolchain; if `cargo` or `bun` is missing, only static changes are possible, not a full compile verification

## Suggested Reading Order

1. `AGENTS.md`
2. `docs/repo-map.md`
3. `docs/invariants.md`
4. `docs/current-plan.md`
5. The matching `docs/current-plans/*.md`
6. The relevant entry files and tests
