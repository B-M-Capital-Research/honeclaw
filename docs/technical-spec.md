# Hone-Financial Technical Specification

Last updated: 2026-03-22
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

- Multiple entrypoints: Web console, CLI, iMessage, Discord, and Feishu
- Multiple agent execution modes: `function_calling`, `gemini_cli`, and `codex_cli`
- Local JSON persistence for sessions, holdings, scheduled tasks, drafts, and generated files
- Multi-channel actor isolation by `channel + user_id + channel_scope`
- A skill system that dynamically loads skill definitions from `skills/` and `data/custom_skills/`
- A Web console for session browsing, skill management, task management, holdings management, and research tasks
- Scheduled tasks that poll every minute and run on Beijing time

Current boundaries to keep in mind:

- The Telegram binary and configuration already exist, but the message-delivery path is still a placeholder and should not be treated as a fully usable channel
- The `hone-tools` crate already contains implementations such as `data_fetch`, `web_search`, and `portfolio`, but `HoneBotCore::create_tool_registry` currently registers only `load_skill`, `skill_tool`, `cron_job`, and the admin-only `restart_hone`

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

- OpenRouter / Kimi model calls
- Local Gemini CLI / Codex CLI agent adapters
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
│   └── hone-channels
├── agents/
│   ├── function_calling
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

- Defines `HoneConfig` plus the config façade in `src/config.rs` and its `config/{agent,channels,server}.rs` submodules
- Defines `ActorIdentity`
- Defines error types, logging initialization, the agent abstraction, and context types

#### `crates/hone-llm`

- Defines the `LlmProvider` abstraction
- Provides the OpenRouter provider implementation

#### `crates/hone-tools`

- Defines the `Tool` trait and `ToolRegistry`
- Implements `load_skill`, `skill_tool`, `cron_job`, and `restart_hone`
- Keeps `data_fetch`, `web_search`, and `portfolio` implementations around for future wiring and the skill system

#### `crates/hone-integrations`

- Wraps X, the Feishu facade, and Nano Banana

#### `crates/hone-scheduler`

- Scans scheduled tasks every minute and emits events

#### `crates/hone-channels`

- Provides the cross-channel shared `HoneBotCore`
- Wraps tool registration, agent creation, session compression, attachment helpers, and text chunking
- Splits the attachment pipeline into `attachments/{ingest,vision,vector_store}.rs` behind the `attachments.rs` façade

#### `agents/*`

- `function_calling`: LLM + tool loop
- `gemini_cli`: wrapper around the local `gemini` CLI
- `codex_cli`: wrapper around the local `codex exec`

#### `memory/`

- JSON file storage layer
- Stores sessions, portfolios, and cron jobs

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
  -> Agent(function_calling | gemini_cli | codex_cli)
  -> memory / integrations / scheduler
  -> Channel reply or Web response
```

### 5.2 `HoneBotCore`

`crates/hone-channels/src/core.rs` is the current backend assembly point. It is responsible for:

- Loading configuration from `config.yaml`
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

#### `function_calling`

- The default provider
- Runs multiple LLM turns
- Passes the tool schema to the LLM
- Executes tools in order when the model returns `tool_calls`
- Stops after `max_iterations`

#### `gemini_cli`

- Uses the local `gemini --prompt ... -o json`
- Combines the system prompt, tool schema, and recent conversation into a single prompt
- The CLI itself currently handles the reasoning required for non-native tool calls

#### `codex_cli`

- Uses the local `codex exec`
- Writes the system prompt, tool schema, and recent conversation to stdin
- Recovers the final response from a temporary output file

### 5.5 Tool Layer

Default registered tools:

- `load_skill`
- `skill_tool`
- `cron_job`
- `restart_hone` (admin only)

Tools that already exist but are not registered by default in `HoneBotCore`:

- `data_fetch`
- `web_search`
- `portfolio`

That means:

- `skills/` frontmatter can declare these tool names
- Without extra wiring, the agent runtime may not actually be able to call them
- When writing skills or extending the default tool set, update `HoneBotCore::create_tool_registry` as well

### 5.6 Skill System

Only one skill format is supported:

1. `skills/<name>/SKILL.md`

If older environments still contain `skills/<name>.yaml|yml`, run the migration script first and then let runtime load the converted files.

Skill metadata includes:

- `name`
- `description`
- `aliases`
- `tools`
- Markdown prompt body

At runtime, skills are aggregated from two places:

- The in-repo `skills/` directory
- The user-customizable `./data/custom_skills`

`skill_tool` allows runtime skill management:

- List skills
- Create custom skills
- Update custom skills
- Delete custom skills

System skills cannot be modified through the tool.

## 6. Storage and Data Model

### 6.1 Storage Strategy

The backend currently uses local JSON files by default and does not depend on a database service.

Main directories come from `config.storage.*`:

- `./data/sessions`
- `./data/portfolio`
- `./data/cron_jobs`
- `./data/reports`
- `./data/x_drafts`
- `./data/gen_images`

### 6.2 Session

`memory/src/session.rs`

- One session corresponds to one JSON file
- The session structure contains `actor`, the message list, metadata, and summary
- The Web UI, CLI, and every channel reuse the same persistence layer

Session compression rules:

- Trigger compression when the number of effective user / assistant messages exceeds 40
- Ask the current LLM to generate a "watch list + conversation summary"
- Keep one system summary plus the latest 4 messages after compression

### 6.3 Portfolio

`memory/src/portfolio.rs`

- File names are isolated by actor: `portfolio_<actor_key>.json`
- The structure contains `holdings[]` and `updated_at`
- The Web API is currently the most complete portfolio read / write entrypoint

### 6.4 Cron Job

`memory/src/cron_job.rs`

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

- The config structure, scheduler integration, and startup scaffolding already exist
- Real message receiving and reply handling are still placeholder-mode
- The docs and README should not describe it as a mature capability on par with Discord or iMessage
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

Config sources:

1. `config.yaml`
2. `config.example.yaml` as the sample

Key config sections:

- `llm`
- `agent`
- `imessage`
- `feishu`
- `telegram`
- `discord`
- `x`
- `nano_banana`
- `fmp`
- `storage`
- `logging`
- `admins`

Implementation note:

- The config loader lives in `crates/hone-core/src/config.rs` as a façade, with the concrete type definitions split into `src/config/{agent,channels,server}.rs`

Important constraints:

- LLM keys are read from environment variables first and fall back to the config file
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
- Telegram is still a placeholder-log mode

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
- Some skills in `skills/` depend on `data_fetch`, `web_search`, or `portfolio`, but not all of those tools are wired into the default registry yet
- Telegram still needs a real bot integration
- Persistence is still local JSON-first; if a database is introduced later, `ActorIdentity` must remain the isolation source of truth

## 13. Key Entrypoint Index

- Web backend: `bins/hone-console-page/src/main.rs`
- Web frontend: `packages/app/src/app.tsx`
- CLI: `bins/hone-cli/src/main.rs`
- iMessage: `bins/hone-imessage/src/main.rs`
- Discord: `bins/hone-discord/src/main.rs`
- Feishu: `bins/hone-feishu/src/main.rs` plus sibling modules
- Telegram: `bins/hone-telegram/src/main.rs` plus sibling modules
- Core assembly: `crates/hone-channels/src/core.rs`
- Config structure: `crates/hone-core/src/config.rs` and `crates/hone-core/src/config/{agent,channels,server}.rs`
- Actor isolation: `crates/hone-core/src/actor.rs`
- Session storage: `memory/src/session.rs`
- Scheduled task storage: `memory/src/cron_job.rs`
- Portfolio storage: `memory/src/portfolio.rs`
