# Hone-Financial Technical Specification

Last updated: 2026-05-15
Status: Aligned with the current implementation

## 1. Document Purpose

- Describe the technical architecture, entrypoints, data flow, configuration, and verification style that are already implemented in the repository
- Provide new sessions, new developers, and future refactors with a specification that is more complete than `README.md` while still treating the implementation as the source of truth

Source-of-truth priority:

1. Code and tests
2. `README.md`
3. Workspace manifests
4. This document

## 2. Product and Current Capabilities

Hone-Financial is a local-first AI research assistant. The current codebase has already been rewritten from the historical Python version into a Rust workspace.

Current capabilities:

- Multiple entrypoints: Web console, public Web API, CLI, iMessage, Discord, Feishu, Telegram, and Desktop
- Agent execution modes: `codex_acp` (default), `hone_cloud`, `opencode_acp`, `gemini_cli`, and `codex_cli`; `gemini_acp` remains deserializable legacy config but is rejected at runtime
- Local JSON file storage plus SQLite-backed session indexes/runtime reads, cron run history, Web auth sessions, and LLM audit records
- Multi-channel actor isolation by `channel + user_id + channel_scope`
- A Claude Code-style skill system that injects only query-relevant summaries, uses `discover_skills` for misses, and loads full `SKILL.md` content only when invoked
- Skills can optionally declare a default `script` entrypoint that `skill_tool` may execute explicitly inside the skill directory
- A Web console for session browsing, skill management, task management, holdings management, and research tasks
- Scheduled tasks that poll every minute and run on Beijing time

Current boundaries to keep in mind:

- Some older runner crates under `agents/` remain for compatibility, while the active runner factory lives in `crates/hone-channels/src/runners.rs`
- The `hone-tools` registry includes skills, cron jobs, portfolio, data fetch, web search, notification preferences, missed events, local file tools, deep research, and admin-only `restart_hone`

## 3. Technology Stack

Backend and runtime:

- Rust
- Tokio async runtime
- Reqwest HTTP client
- Tracing logs
- Rusqlite for reading the macOS `chat.db` used by iMessage

Frontend:

- SolidJS
- TypeScript
- Vite
- Tailwind CSS
- Shadcn-style UI primitives
- Playwright for end-to-end tests

Integrations and external capabilities:

- OpenRouter / Kimi / OpenAI-compatible model calls
- OpenRouter credentials use the config-owned `llm.providers.openrouter.api_key/api_keys` pool with legacy single-key config readable only for migration
- Local Gemini CLI, Codex CLI, Codex ACP, OpenCode ACP, and Hone Cloud adapters
- Tavily search
- Nano Banana image generation
- Feishu Go facade, connected from the Rust business process through local RPC

## 4. Workspace Structure

### 4.1 Workspace Overview

```text
Hone-Financial/
├── crates/
│   ├── hone-core
│   ├── hone-llm
│   ├── hone-tools
│   ├── hone-integrations
│   ├── hone-scheduler
│   ├── hone-channels
│   ├── hone-event-engine
│   └── hone-web-api
├── agents/
│   ├── gemini_cli
│   └── codex_cli
├── memory/
├── bins/
│   ├── hone-console-page
│   ├── hone-cli
│   ├── hone-imessage
│   ├── hone-discord
│   ├── hone-feishu
│   └── hone-telegram
├── packages/
│   ├── app
│   └── ui
├── skills/
├── data/
└── docs/
```

### 4.2 Module Responsibilities

#### `crates/hone-core`

- Defines `HoneConfig` plus the config façade in `src/config/mod.rs` and its `config/{agent,channels,event_engine,server}.rs` submodules
- Defines `ActorIdentity`
- Defines error types, logging initialization, the agent abstraction, and context types

#### `crates/hone-llm`

- Defines the `LlmProvider` abstraction
- Provides the OpenRouter provider implementation

#### `crates/hone-tools`

- Defines the `Tool` trait and `ToolRegistry`
- Implements the skill runtime, cron jobs, portfolio, data fetch, web search, notification preferences, missed events, actor-local file tools, deep research, and admin-only restart tools

#### `crates/hone-integrations`

- Wraps X, the Feishu facade, and Nano Banana

#### `crates/hone-scheduler`

- Scans scheduled tasks every minute and emits events

#### `crates/hone-channels`

- Provides the cross-channel shared `HoneBotCore`
- Wraps tool registration, agent creation, session compression, attachment helpers, and text chunking
- Splits the attachment pipeline into `attachments/{ingest,vision,vector_store}.rs` behind the `attachments.rs` façade

#### `agents/*`

- Legacy reasoning-agent crates for `gemini_cli` and `codex_cli`; current channel runtime creates runners through `crates/hone-channels/src/runners.rs`

#### `memory/`

- JSON and SQLite storage layer
- Stores sessions, portfolios, cron jobs, web auth, delivery logs, and related runtime indexes

#### `bins/*`

- The actual runtime entrypoints for each channel and for the Web / CLI apps
- `hone-feishu`, `hone-telegram`, and `hone-desktop` now keep only a thin `main.rs` façade while sibling modules hold the handler / command / sidecar logic

#### `packages/app`

- Web console frontend
- Pages include `sessions`, `skills`, `tasks`, `portfolio`, and `research`

#### `packages/ui`

- Shared UI primitives, Markdown rendering, and theme context

## 5. Core Runtime Architecture

### 5.1 Main Execution Chain

```text
Channel entrypoint / Web API
  -> HoneBotCore
  -> ActorIdentity / SessionStorage
  -> ToolRegistry
  -> AgentRunner(codex_acp | hone_cloud | opencode_acp | gemini_cli | codex_cli)
  -> memory / integrations / scheduler
  -> Channel reply or Web response
```

### 5.2 `HoneBotCore`

`crates/hone-channels/src/core/mod.rs` plus its sibling modules are the current backend assembly point. They are responsible for:

- Loading the resolved `HoneConfig`; CLI/Desktop settings mutate canonical `config.yaml`, while child runtime processes usually consume the generated `data/runtime/effective-config.yaml`
- Initializing `SessionStorage`
- Creating the LLM provider
- Creating the default `ToolRegistry`
- Creating the concrete agent
- Starting the scheduler
- Emitting unified logs
- Running long-session compression

### 5.3 Actor Isolation

All user-level data is keyed by `ActorIdentity`:

```text
ActorIdentity {
  channel,
  user_id,
  channel_scope?
}
```

Rules:

- `storage_key()` encodes a stable file name
- `session_id()` is always `Actor_<storage_key>`
- Group scenarios are distinguished by `channel_scope`, for example Discord group channels such as `g:<guild>:c:<channel>`
- Direct chat uses the `direct` scope by default

This rule is already applied to:

- Session history
- Holdings
- Scheduled tasks
- Some generated-file directories and Web query parameters

### 5.4 Agent Layer

#### `gemini_cli`

- Uses the local `gemini --prompt ... -o json`
- Combines the system prompt, tool schema, and recent conversation into a single prompt
- The CLI itself currently handles the reasoning required for non-native tool calls

#### `codex_cli`

- Uses the local `codex exec`
- Writes the system prompt, tool schema, and recent conversation to stdin
- Recovers the final response from a temporary output file

#### `codex_acp`

- Uses `codex-acp` over stdio / JSON-RPC
- Defaults to `gpt-5.6-sol` with `xhigh` reasoning effort
- Requires `@openai/codex >= 0.144.1` and `@agentclientprotocol/codex-acp >= 1.1.2` before starting a turn
- Creates a fresh ACP session per Hone turn and seeds it from Hone's restored context

#### `opencode_acp`

- Uses `opencode acp` over stdio / JSON-RPC
- Inherits the user's local OpenCode provider/model/auth config when `agent.opencode.*` overrides are empty
- Applies explicit `agent.opencode.model` / `variant` through ACP `session/set_model` when configured

#### `hone_cloud`

- Calls the configured Hone Cloud OpenAI-compatible chat endpoint
- Uses `agent.hone_cloud.base_url`, `api_key`, and `model`

### 5.5 Tool Layer

Default registered tools:

- `skill_tool`
- `discover_skills`
- `load_skill` (compatibility shim)
- `cron_job`
- `portfolio`
- `notification_prefs`
- `missed_events`
- Actor-local `list_files`, `search_files`, and `read_file` tools when an actor is available
- `data_fetch`
- `web_search`
- `deep_research`
- `restart_hone` (admin only)

That means:

- `skills/` frontmatter can declare these tool names
- When writing skills or extending the default tool set, update `HoneBotCore::create_tool_registry` as well

### 5.6 Skill System

The runtime skill format is:

1. `skills/<name>/SKILL.md`
2. `data/custom_skills/<name>/SKILL.md`
3. A closer `.hone/skills/<name>/SKILL.md`

If older environments still contain `skills/<name>.yaml|yml`, run the migration script first and then let runtime load the converted files.

Skill metadata includes:

- `name`
- `description`
- `when_to_use`
- `allowed-tools`
- `user-invocable`
- `model`
- `effort`
- `context` (`inline` or `fork`)
- `agent`
- `paths`
- `hooks`
- `arguments`
- `script`
- `shell`
- `aliases` and legacy `tools` are still parsed as migration fallbacks
- Markdown prompt body

Skill execution is two-phase:

1. The system prompt and discovery text expose only a compact listing of available or relevant skills.
2. `skill_tool(skill_name="...")` or a user slash command such as `/<skill-name>` injects the fully rendered skill body into the active turn, including `Base directory for this skill` and placeholder expansion for `${HONE_SKILL_DIR}` and `${HONE_SESSION_ID}`.
3. If the caller explicitly sets `execute_script=true`, `skill_tool` resolves the declared `script`, maps `script_arguments` according to frontmatter `arguments`, runs it with `${HONE_SKILL_DIR}` as cwd, and returns stdout/stderr plus exit status.

Timing and restore semantics:

- Full skill context is injected only at the actual invocation boundary, not during discovery/listing.
- The persisted restore payload is the stable invoked skill context itself.
- If a slash invocation also contains one-shot user instructions, those stay in the current turn input and are not persisted as long-lived skill context during compaction/resume.
- Session compaction now follows a boundary model: a compacted session writes a `Conversation compacted` boundary plus a compact-summary message, and subsequent context restoration slices from the most recent boundary forward.
- Active invoked skills are now materialized into post-compact skill snapshot messages inside the compacted window. Restore logic uses those snapshots when present and avoids re-injecting the same skill prompt again from metadata.
- Manual `/compact` uses that same compaction path, accepts optional user instructions for the summarizer, does not consume conversation quota, and does not persist the slash command text as a normal transcript message.
- History APIs may still return compact summaries and compact skill snapshots for debugging/recovery, but they are marked transcript-only so the primary chat timeline can hide them while preserving compact boundaries as visible separators.

At runtime, skills are aggregated from two places:

- The in-repo `skills/` directory
- The user-customizable `./data/custom_skills`
- Nested `.hone/skills` directories discovered under the current workspace

Precedence is:

- closer dynamic `.hone/skills`
- `data/custom_skills`
- repo `skills/`

`discover_skills` is the metadata/discovery entrypoint:

- Search relevant skills for the current task
- Respect `paths`-gated activation based on touched file paths
- Return compact summaries suitable for prompt disclosure or UI index views

`skill_tool` is the execution entrypoint:

- Load one resolved skill
- Return the full expanded prompt
- Surface runtime metadata such as `allowed_tools`, `model`, `effort`, `context`, `agent`, `paths`, and `hooks`
- Persist invoked skill prompt state into session metadata for compaction / resume recovery

`load_skill` remains as a compatibility shim over the same runtime and should not be taught as the main path.

## 6. Storage and Data Model

### 6.1 Storage Strategy

The backend still keeps core runtime state local-first today. JSON files are the default session runtime read path, while SQLite-backed session indexes/runtime reads are available through `storage.session_sqlite_db_path` and `storage.session_runtime_backend`; cron run history, Web auth sessions, and LLM audit records also use local SQLite tables. `cloud.postgres` now records the managed Postgres env contract for the migration target, but PG-backed repositories are not yet the default runtime storage path.

Main directories come from `config.storage.*`:

- `sessions_dir`: `./data/sessions`
- `session_sqlite_db_path`: `./data/sessions.sqlite3`
- `portfolio_dir`: `./data/portfolio`
- `cron_jobs_dir`: `./data/cron_jobs`
- `gen_images_dir`: `./data/gen_images`
- `notif_prefs_dir`: `./data/notif_prefs`
- `conversation_quota_dir`: `./data/conversation_quota`
- `llm_audit_db_path`: `./data/llm_audit.sqlite3`

When `cloud.oss` is configured through runtime env, public Web uploads are stored in OSS under `cloud.oss.public_upload_prefix`, and `/api/public/image` / `/api/public/file` can proxy managed `oss://bucket/key` paths. Other generated files remain under `config.storage.*` until their cloud storage adapters land.

### 6.2 Session

`memory/src/session.rs`

- In `json` mode, one session corresponds to one JSON file; in `sqlite` mode, `storage.session_sqlite_db_path` is the runtime read source while JSON can remain a rollback mirror
- The session structure contains `actor`, the message list, metadata, and summary
- The Web UI, CLI, and every channel reuse the same persistence layer

Session compression rules:

- Direct sessions trigger compression when the active message count exceeds 20 or active content exceeds about 80 KB
- Group sessions use `group_context.compress_threshold_messages` and `group_context.compress_threshold_bytes` from `config.yaml`; defaults are 24 messages and 48 KB
- Ask the configured auxiliary LLM route to generate a "watch list + conversation summary"
- Keep one system summary plus the latest 6 messages for direct sessions; group sessions use `group_context.retain_recent_after_compress`, defaulting to 8

### 6.3 Portfolio

`memory/src/portfolio.rs`

- File names are isolated by actor: `portfolio_<actor_key>.json`
- The structure contains `holdings[]` and `updated_at`
- The Web API is currently the most complete portfolio read / write entrypoint

### 6.4 Cron Job

`memory/src/cron_job/mod.rs`

- File names are isolated by actor: `cron_jobs_<actor_key>.json`
- Each actor can have up to 20 enabled jobs
- Scheduled time uses Beijing time
- A 5-minute tolerance window exists to avoid missing minute boundaries because of LLM latency

## 7. Channel Entrypoints

### 7.1 Web Console Backend

Entrypoint: `bins/hone-console-page/src/main.rs`

Responsibilities:

- Host frontend static assets
- Provide APIs and SSE
- Maintain endpoints for users, history, skills, scheduled tasks, holdings, and research tasks
- Push scheduler events to the browser

Main routes:

- `/api/meta`
- `/api/channels`
- `/api/users`
- `/api/history`
- `/api/chat`
- `/api/events`
- `/api/skills`
- `/api/skills/{id}`
- `/api/cron-jobs*`
- `/api/portfolio*`
- `/api/research/*`
- `/api/image`
- `/api/file`

The research capability currently proxies the external deep-research service in the Web layer and also provides PDF generation / download endpoints.

### 7.2 CLI

Entrypoint: `bins/hone-cli/src/main.rs`

Characteristics:

- Interactive REPL
- Uses a fixed actor: `cli / cli_user`
- Mainly used for local debugging

### 7.3 iMessage

Entrypoint: `bins/hone-imessage/src/main.rs`

Characteristics:

- macOS only
- Polls `~/Library/Messages/chat.db`
- Sends messages through AppleScript
- Requires Full Disk Access

### 7.4 Discord

Entrypoint: `bins/hone-discord/src/main.rs`

Characteristics:

- Already has fairly complete DM handling
- Supports a group-chat aggregation reply window
- Has attachment understanding and categorization
- Uses `channel_scope` to distinguish group-channel context

### 7.5 Feishu

Entrypoint: `bins/hone-feishu/src/main.rs`

Characteristics:

- Rust owns the business logic
- The Go facade owns the official SDK long connection and event integration
- Rust and the facade cooperate through local JSON-RPC / HTTP
- Already supports direct messages, `post` bodies, image attachments, and file attachment parsing
- The Rust side is organized into sibling modules (`card.rs`, `handler.rs`, `listener.rs`, `markdown.rs`, `types.rs`) behind the `main.rs` façade

### 7.6 Telegram

Entrypoint: `bins/hone-telegram/src/main.rs`

Current status:

- Uses teloxide polling for real message receiving
- Supports direct triggers, group mentions/replies, placeholder/progress updates, HTML-safe response splitting, local image/file segments, and scheduler delivery
- The Rust side is organized into sibling modules (`handler.rs`, `listener.rs`, `markdown_v2.rs`, `types.rs`) behind the `main.rs` façade

## 8. Web Console Frontend

Entrypoint: `packages/app/src/app.tsx`

Frontend structure:

- Routes: `/sessions`, `/skills`, `/tasks`, `/portfolio`, `/research`
- Contexts: `console`, `sessions`, `skills`, `tasks`, `portfolio`, `research`
- API wrapper: `packages/app/src/lib/api.ts`
- Shared UI: `packages/ui`

Current frontend capabilities:

- Session history browsing and chat
- Channel status polling
- Skill list and skill details
- Scheduled task CRUD
- Portfolio CRUD
- Stock research task creation, polling, preview, and PDF export

Web and backend integration:

- REST for lists and detail fetching
- Streaming responses for `/api/chat`
- SSE push for scheduler events on `/api/events`

## 9. Configuration System

Long-lived config sources:

1. Canonical `config.yaml`
2. `config.example.yaml` as the sample

Runtime materialization:

- CLI/Desktop settings mutate canonical `config.yaml`
- Startup generates `data/runtime/effective-config.yaml` for backend/channel child processes
- `HONE_CONFIG_PATH` points runtime children at the effective snapshot, while installed wrappers use `HONE_USER_CONFIG_PATH` for canonical user config

Key config sections:

- `llm`
- `agent`
- `imessage`
- `feishu`
- `telegram`
- `discord`
- `group_context`
- `nano_banana`
- `fmp`
- `search`
- `storage`
- `cloud`
- `logging`
- `admins`
- `web`
- `security`
- `event_engine`
- `language`

Implementation note:

- The config loader lives in `crates/hone-core/src/config/mod.rs` as a façade, with the concrete type definitions split into `src/config/{agent,channels,event_engine,server}.rs`

Important constraints:

- LLM provider/profile credentials are config-owned. Prefer `llm.providers.<symbol>.api_key/api_keys`; legacy `llm.openrouter.*` remains readable only as a config fallback during migration.
- Tavily web search currently consumes `search.api_keys` and `search.max_results`; `search.provider`, `search.search_depth`, and `search.topic` remain schema/compatibility fields and are not wired into requests until the search tool request builder is widened.
- `cloud.enabled` is effectively enabled when set directly, when `HONE_CLOUD_ENABLED` is true, or when `cloud.postgres` / `cloud.oss` resolve enough env-backed credentials to be configured.
- `cloud.postgres` and `cloud.oss` prefer env references such as `DATABASE_URL`, `HONE_POSTGRES_*`, and `HONE_OSS_*`; committed config should keep the actual credentials empty.
- `cloud.strict_no_local_storage` / `HONE_CLOUD_STRICT_NO_LOCAL_STORAGE` fail startup while declared local storage dependencies remain, so they should stay false until managed replacements exist for all listed stores.
- `logging.udp_port: null` uses the default local UDP log sink port `18118`; there is currently no config-level disable switch for UDP logging.
- `logging.console` and `logging.file` are parsed compatibility fields; `setup_logging` currently installs the console formatter unconditionally and does not create a file appender from `logging.file`.
- External-account capabilities must not enter the default CI gate
- iMessage is treated as a local privileged capability by default
- Admin tools are exposed by channel allowlist

## 10. Scheduler and Async Tasks

The scheduler lives in `crates/hone-scheduler/src/lib.rs`.

Behavior:

- Align to the next minute after startup
- Scan all actor tasks every 60 seconds
- Emit a `SchedulerEvent` when a task fires
- Let the channel entrypoint or Web backend consume the event and deliver it

Current consumption:

- The Web console converts events into SSE pushes
- Other channels handle them in their own entrypoints
- Telegram consumes scheduled delivery through its channel entrypoint

## 11. Testing and Delivery Contract

Default verification commands:

```bash
bash scripts/ci/check_fmt_changed.sh
cargo check --workspace --all-targets
cargo test --workspace --all-targets
bash tests/regression/run_ci.sh
```

Common Web frontend verification:

```bash
bun run typecheck:web
bun run test:web
bun run build:web
```

Testing organization follows `AGENTS.md`:

- Rust unit tests live next to the implementation by default
- Integration tests live in `tests/integration/`
- CI-safe regression tests live in `tests/regression/ci/`
- Scripts that depend on accounts or local machine state live in `tests/regression/manual/`

## 12. Known Differences and Future Work

- `docs/technical-spec.md` has been refreshed from the historical Python document to the current implementation, but it still needs to evolve with the code
- Some skills in `skills/` still mention older CRUD-style `skill_tool` or `load_skill` guidance and need continued migration to the execution-style runtime contract
- Skill fields such as `hooks`, `allowed-tools`, `model`, and `effort` are now parsed and exposed, but strict runner-side enforcement is not fully implemented yet
- Persistence is still local-first; any further database-backed paths must keep `ActorIdentity` as the isolation source of truth

## 13. Key Entrypoint Index

- Web backend: `bins/hone-console-page/src/main.rs`
- Web frontend: `packages/app/src/app.tsx`
- CLI: `bins/hone-cli/src/main.rs`
- iMessage: `bins/hone-imessage/src/main.rs`
- Discord: `bins/hone-discord/src/main.rs`
- Feishu: `bins/hone-feishu/src/main.rs` plus sibling modules
- Telegram: `bins/hone-telegram/src/main.rs` plus sibling modules
- Core assembly: `crates/hone-channels/src/core/mod.rs`
- Config structure: `crates/hone-core/src/config/mod.rs` and `crates/hone-core/src/config/{agent,channels,event_engine,server}.rs`
- Actor isolation: `crates/hone-core/src/actor.rs`
- Session storage: `memory/src/session.rs`
- Scheduled task storage: `memory/src/cron_job/mod.rs`
- Portfolio storage: `memory/src/portfolio.rs`
