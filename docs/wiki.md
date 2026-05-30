# Hone Wiki

Last updated: 2026-05-27

This page is the practical wiki entry for Honeclaw. It explains the repository layout, the main runtime pieces, and the common ways to install, configure, start, stop, and verify the project.

中文用户可以直接按本页命令操作；英文 README 保持产品介绍为主，本页承担更完整的工程入口职责。

## Table Of Contents

- [What Hone Runs](#what-hone-runs)
- [Repository Directory Guide](#repository-directory-guide)
- [Runtime Directory Guide](#runtime-directory-guide)
- [Prerequisites](#prerequisites)
- [Install And Start From Release](#install-and-start-from-release)
- [Start From Source](#start-from-source)
- [Desktop Startup Modes](#desktop-startup-modes)
- [Web Startup Modes](#web-startup-modes)
- [Configuration](#configuration)
- [Model and Runner Setup](#model-and-runner-setup)
- [Channel Setup](#channel-setup)
- [Common URLs And Ports](#common-urls-and-ports)
- [Stop, Restart, And Cleanup](#stop-restart-and-cleanup)
- [Verification Commands](#verification-commands)
- [Troubleshooting](#troubleshooting)
- [Contributor Reading Map](#contributor-reading-map)

## What Hone Runs

Hone is a multi-process local assistant stack:

- `hone-console-page`: local Web backend and static asset server.
- `hone-cli`: local command-line entrypoint for doctor, onboarding, config, chat, and runtime start.
- `hone-mcp`: local MCP server used by ACP runners to expose Hone tools.
- `hone-desktop`: Tauri desktop host.
- `hone-imessage`, `hone-discord`, `hone-feishu`, `hone-telegram`: optional channel listeners.
- `packages/app`: SolidJS Web UI for admin console, public chat, memory, settings, and runtime views.
- `skills/`: built-in skill prompts and optional script-backed skill entrypoints.
- `memory/`: local persistence for sessions, cron jobs, portfolios, quotas, company profiles, and audit logs.

At runtime, user messages enter through Web, desktop, or an IM channel. The channel builds a normalized request, `hone-channels` selects and runs the configured agent runner, `hone-tools` exposes tools and skills, `memory` persists state, and the final response is rendered back to the source channel.

## Repository Directory Guide

Top-level layout:

| Path | Purpose |
| --- | --- |
| `README.md`, `README_ZH.md`, `README_EN.md` | Product-facing introduction and quick start. |
| `AGENTS.md` | Collaboration rules, testing contract, CI/CD expectations, and documentation maintenance policy. |
| `Cargo.toml` | Rust workspace manifest. |
| `package.json` | Bun workspace scripts for Web and desktop frontend flows. |
| `config.example.yaml` | Canonical example config. Copy to `config.yaml` for source runs. |
| `launch.sh` | Compatibility shim that points to the CLI startup path. Source checkout startup goes through `cargo run -p hone-cli -- start --build`. |
| `crates/` | Shared Rust libraries. |
| `bins/` | Runnable Rust binaries. |
| `agents/` | Agent adapters and runner implementations. |
| `memory/` | Storage crate. |
| `packages/` | Frontend workspaces. |
| `skills/` | Built-in skills. |
| `scripts/` | Install, build, packaging, and maintenance scripts. |
| `tests/regression/` | CI-safe and manual regression scripts. |
| `docs/` | Wiki, runbooks, architecture notes, plans, handoffs, bug ledger, and release notes. |
| `resources/` | Images and architecture HTML used by README/docs. |

Important Rust crates:

| Crate | Role |
| --- | --- |
| `crates/hone-core` | Config facade, logging, errors, and shared core types. |
| `crates/hone-channels` | Shared channel runtime, ingress/outbound handling, agent sessions, runner orchestration, response finalization, MCP bridge, and actor sandboxes. |
| `crates/hone-tools` | Tool registry, built-in tools, skill runtime, and script-backed skill execution. |
| `crates/hone-llm` | LLM provider abstraction and OpenAI-compatible/OpenRouter plumbing. |
| `crates/hone-scheduler` | Scheduled task orchestration. |
| `crates/hone-integrations` | External service integrations. |
| `crates/hone-web-api` | Web API routes used by the console backend. |

Runnable binaries:

| Binary | Source | Purpose |
| --- | --- | --- |
| `hone-cli` | `bins/hone-cli` | CLI setup, config, doctor, chat, cleanup, and runtime start. |
| `hone-console-page` | `bins/hone-console-page` | Admin/public Web backend and static asset server. |
| `hone-desktop` | `bins/hone-desktop` | Tauri desktop app and sidecar lifecycle manager. |
| `hone-mcp` | `bins/hone-mcp` | MCP bridge for ACP runners. |
| `hone-feishu` | `bins/hone-feishu` | Feishu/Lark listener and scheduler delivery. |
| `hone-discord` | `bins/hone-discord` | Discord listener and outbound delivery. |
| `hone-telegram` | `bins/hone-telegram` | Telegram listener and outbound delivery. |
| `hone-imessage` | `bins/hone-imessage` | iMessage integration on macOS. |

Frontend layout:

| Path | Purpose |
| --- | --- |
| `packages/app/src/app.tsx` | Route entrypoint. |
| `packages/app/src/pages/` | Main pages such as chat, memory, settings, and admin views. |
| `packages/app/src/context/` | Domain state providers. |
| `packages/app/src/components/` | Composite UI components. |
| `packages/app/src/lib/` | API clients, message parsing, public chat helpers, and data transforms. |
| `packages/ui/` | Shared UI package. |

## Runtime Directory Guide

Source checkout defaults:

| Path | Purpose |
| --- | --- |
| `config.yaml` | Canonical local config for source runs. |
| `data/` | Runtime data root. |
| `data/runtime/effective-config.yaml` | Generated runtime config snapshot for spawned processes. |
| `data/runtime/logs/` | Runtime log files. |
| `data/runtime/*.pid` | Runtime supervisor pid files. |
| `data/runtime/locks/` | Process lock files. |
| `data/sessions.sqlite3` | SQLite session runtime store when enabled. |
| `agent-sandboxes/` | Actor-scoped workspaces and company profile docs. |

Installed release defaults:

| Path | Purpose |
| --- | --- |
| `~/.honeclaw/current` | Active release bundle symlink. |
| `~/.honeclaw/config.yaml` | Canonical user config. |
| `~/.honeclaw/data` | Runtime data. |
| `~/.honeclaw/data/runtime/effective-config.yaml` | Generated runtime config. |
| `~/.honeclaw/current/share/honeclaw/skills` | Bundled skills. |
| `~/.honeclaw/current/share/honeclaw/web` | Bundled admin Web assets. |
| `~/.honeclaw/current/share/honeclaw/web-public` | Bundled public/user Web assets. |

## Prerequisites

Recommended local environment:

- macOS or Ubuntu.
- Rust toolchain and Cargo.
- Bun for frontend and Tauri dev/build flows.
- Git.
- Optional: Homebrew for macOS/Linux package install.
- Optional: `gh` for GitHub issue/PR workflows.
- Optional runners: Codex CLI/ACP or OpenCode ACP.

For source checkout development:

```bash
rustup update
bun install
cp config.example.yaml config.yaml
```

If `config.yaml` already exists, keep it. It is the canonical local config and may contain credentials or machine-specific settings.

## Install And Start From Release

Use this path when you want to run Hone without cloning the repository.

### One-line install

```bash
curl -fsSL https://raw.githubusercontent.com/B-M-Capital-Research/honeclaw/main/scripts/install_hone_cli.sh | bash
hone-cli doctor
hone-cli onboard
hone-cli start
```

### Homebrew

```bash
brew install B-M-Capital-Research/honeclaw/honeclaw
hone-cli doctor
hone-cli onboard
hone-cli start
```

`hone-cli onboard` guides runner selection, optional channel setup, and provider API key setup. `hone-cli start` starts the local backend plus enabled channel listeners in the foreground.

See the detailed runbook: [`docs/runbooks/hone-cli-install-and-start.md`](./runbooks/hone-cli-install-and-start.md).

## Start From Source

Clone and prepare:

```bash
git clone https://github.com/B-M-Capital-Research/honeclaw.git
cd honeclaw
cp config.example.yaml config.yaml
bun install
```

Build the local CLI/runtime binaries and start backend plus enabled channel listeners:

```bash
cargo run -p hone-cli -- start --build
```

Start admin/public Vite frontends through the CLI wrapper after the backend is ready:

```bash
cargo run -p hone-cli -- web admin-ui --dev
cargo run -p hone-cli -- web user-ui --dev
```

The direct Bun scripts remain available for frontend-only work: `bun run dev:web` and `bun run dev:web:public`.

For the full source Web startup checklist and macOS Rollup/Node signing pitfall, see [`docs/runbooks/source-web-startup.md`](./runbooks/source-web-startup.md).

Desktop development uses explicit Tauri commands:

```bash
bun run tauri:prep:dev -- --skip-dev-command
bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json
```

For desktop work against an already running CLI backend, prepare only the shell side and then run Tauri:

```bash
bun run tauri:prep:dev -- --skip-dev-command --shell-only
bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json
```

Build release desktop assets directly when needed:

```bash
bun run build:desktop
```

Stop a foreground source runtime:

```bash
Ctrl-C
```

## Desktop Startup Modes

Use the mode that matches what you are testing:

| Command | Best For | What It Starts |
| --- | --- | --- |
| `bun run tauri:prep:dev -- --skip-dev-command` + `bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json` | Bundled desktop integration checks. | Vite + Tauri dev; desktop starts embedded backend and enabled channels. |
| `bun run tauri:prep:dev -- --skip-dev-command --shell-only` + Tauri dev | Daily desktop UI work while keeping backend/channels stable. | Tauri dev shell connected to an existing CLI-started backend. |
| `bun run build:desktop` | Long-running desktop verification / packaging prep. | Builds release desktop assets and bundled sidecars. |

For daily development, keep `cargo run -p hone-cli -- start --build` running in one terminal and use Tauri dev in another when backend/channel processes should not restart on every desktop shell rebuild.

## Web Startup Modes

| Command | Best For |
| --- | --- |
| `cargo run -p hone-cli -- start --build` | Runtime-only backend/channel smoke from source. |
| `cargo run -p hone-cli -- web admin-ui --dev` | CLI-managed admin Vite frontend when backend is already running. |
| `cargo run -p hone-cli -- web user-ui --dev` | CLI-managed public/user Vite frontend when public backend is already running. |
| `bun run dev:web` | Frontend-only admin UI work when backend is already running. |
| `bun run dev:web:public` | Frontend-only public chat UI work when public backend is already running. |
| `bun run build:web` | Build admin Web assets. |
| `bun run build:web:public` | Build public Web assets. |
| `bun run build:web:desktop` | Build desktop Web assets with relative asset paths. |

Run the CLI backend and Vite frontends as separate foreground processes.

## Configuration

Source checkout config:

```bash
cp config.example.yaml config.yaml
```

Installed config:

```bash
hone-cli config file
```

Useful CLI config commands:

```bash
hone-cli doctor
hone-cli onboard
hone-cli configure --section agent --section channels --section providers
hone-cli config get agent.runner
hone-cli config set agent.hone_cloud.api_key "<api-key>"
hone-cli config set agent.runner opencode_acp
hone-cli models set --runner opencode_acp --model openrouter/openai/gpt-5.4 --variant medium
```

Important config areas:

- `agent.*`: runner choice, model routing, `daily_conversation_limit`, and timeout behavior.
- `llm.*`: provider keys and OpenAI-compatible/OpenRouter routes.
- `imessage.*`, `feishu.*`, `telegram.*`, `discord.*`: channel enablement, credentials, allowlists, and chat scope.
- `web.*`: Web console auth token and workflow/research integration settings.
- `storage.*`: session data paths and backend selection, especially `sessions_dir`, `session_sqlite_db_path`, `session_sqlite_shadow_write_enabled`, `session_runtime_backend`, `conversation_quota_dir`, `llm_audit_db_path`, and `notif_prefs_dir`.
- `cloud.*`: migration-time managed Postgres / OSS env references. `cloud.enabled` may also become effectively enabled when `HONE_CLOUD_ENABLED` is true or the referenced env vars are present; `cloud.strict_no_local_storage` / `HONE_CLOUD_STRICT_NO_LOCAL_STORAGE` fail startup while local storage dependencies remain.
- `admins.*`: channel admin identities and runtime admin registration passphrase.
- `event_engine.*`: market/news event monitoring and delivery.
- `logging.*`: runtime log level and local UDP sink port. `udp_port: null` uses the default `18118` UDP sink, with no config-level disable switch today; `console` and `file` remain parsed compatibility fields until file/console sinks are wired.
- `security.*`: actor isolation and tool-guard policy.
- `nano_banana.*`: OpenRouter-backed image generation defaults.
- `search.*`, `fmp.*`: external data/search providers.
- `language`: UI / CLI display language (`zh` or `en`).

Admin/public Web ports are runtime environment settings, primarily `HONE_WEB_PORT` and `HONE_PUBLIC_WEB_PORT`, rather than `config.yaml` keys.
Public SMS login and optional Aliyun Captcha are also runtime environment settings; use `config.example.yaml` and `docs/runbooks/backend-deployment.md` as the reference for `ALIBABA_CLOUD_*`, `HONE_ALIYUN_SMS_*`, `HONE_ALIYUN_CAPTCHA_*`, and `HONE_PUBLIC_SECURE_COOKIE`. Active admin-created Web invite users remain the public-login invite-list admission source. For the public session cookie, `HONE_PUBLIC_SECURE_COOKIE` accepts `true/1/yes` and `false/0/no`; invalid non-empty values keep `Secure=true`.
For OpenRouter credentials, prefer the `llm.providers.openrouter.api_key/api_keys` pool; legacy `llm.openrouter.*` key fields are migration fallbacks only.
For Tavily web search, the current runtime tool reads `search.api_keys` and `search.max_results`; `search.provider`, `search.search_depth`, and `search.topic` are preserved schema fields but are not wired into the request yet.
For managed cloud storage, keep actual `DATABASE_URL`, `HONE_POSTGRES_*`, and `HONE_OSS_*` values outside committed config. Current cloud mode has PG hot paths for sessions, Web auth, conversation quota, and cron jobs/runs; `cloud.strict_no_local_storage` is only safe after the remaining local LLM audit, portfolio, notification preference, generated image, skill registry, and actor sandbox stores have managed backends.

Never commit local secrets in `config.yaml`.

## Model and Runner Setup

Hone can use Hone Cloud, local CLI/ACP runners, or OpenAI-compatible cloud APIs.

Common runner choices:

| Runner | Use When |
| --- | --- |
| `hone_cloud` | You want the hosted Hone service seeded by `config.example.yaml`; set `agent.hone_cloud.api_key` before starting. |
| `opencode_acp` | You want Hone to inherit local OpenCode provider/model config. |
| `codex_acp` | You use Codex ACP and want ACP session integration. |
| `codex_cli` | You use Codex CLI directly. |
| `function_calling` | You want the built-in OpenAI-compatible function-calling path. |
| `multi-agent` | You want separate search and OpenCode ACP answer stages; search keys come from `agent.multi_agent.search.api_key` or legacy `llm.auxiliary.api_key`, while answer can inherit the `llm.providers.openrouter.api_key/api_keys` pool when `agent.multi_agent.answer.api_key` is empty. |

Typical OpenCode setup:

```bash
curl -fsSL https://opencode.ai/install | bash
opencode # run /connect and set the default provider/model
hone-cli config set agent.runner opencode_acp
hone-cli start
```

Typical model override:

```bash
hone-cli models set --runner opencode_acp --model openrouter/openai/gpt-5.4 --variant medium
```

If using Hone Cloud, keep `agent.runner=hone_cloud` and set `agent.hone_cloud.api_key`. If using other cloud APIs, configure keys through `hone-cli onboard`, `hone-cli configure`, or direct config edits.

## Channel Setup

Channels are optional. Enable only the ones you actually use.

```bash
hone-cli channels list
hone-cli channels set telegram --enabled true --bot-token "<token>"
hone-cli channels set discord --enabled true --bot-token "<token>"
```

For Feishu/Lark, Discord, Telegram, and iMessage, check `config.example.yaml` for required fields and comments. The onboarding wizard also prints prerequisite notes when enabling channels.

Channel reminders:

- iMessage is macOS-only and depends on local permissions.
- Feishu/Lark requires tenant app credentials and target resolution.
- Discord and Telegram require bot tokens and correct bot/channel permissions.
- Group chat behavior depends on channel `chat_scope` and explicit trigger rules.

## Common URLs And Ports

Defaults in source checkout:

| Service | Default |
| --- | --- |
| Admin backend/API | `http://127.0.0.1:8077` |
| Public backend/API | `http://127.0.0.1:8088` |
| Admin Vite frontend | `http://127.0.0.1:3000` |
| Public Vite frontend | `http://127.0.0.1:3001` |
| Health/meta check | `http://127.0.0.1:8077/api/meta` |

Override ports with environment variables:

```bash
HONE_WEB_PORT=8078 HONE_PUBLIC_WEB_PORT=8089 cargo run -p hone-cli -- start --build
```

## Stop, Restart, And Cleanup

Stop a foreground `hone-cli start` or source runtime:

```bash
Ctrl-C
```

Clean installed Hone runtime data:

```bash
hone-cli cleanup
```

Non-interactive full cleanup:

```bash
hone-cli cleanup --all --yes
```

Homebrew package removal:

```bash
brew uninstall honeclaw
```

## Verification Commands

General checks:

```bash
hone-cli doctor
curl -fsS http://127.0.0.1:8077/api/meta
```

Rust checks:

```bash
cargo check --workspace --all-targets --exclude hone-desktop
cargo test --workspace --all-targets --exclude hone-desktop
```

Frontend checks:

```bash
bun run typecheck:web
bun run test:web
```

CI-safe regression scripts:

```bash
bash tests/regression/run_ci.sh
```

Desktop crate type check without bundled resource validation:

```bash
HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop
```

Manual regression scripts live in `tests/regression/manual/` and may require external accounts or local machine state.

## Troubleshooting

### `hone-cli` not found

```bash
command -v hone-cli || ls -l ~/.local/bin/hone-cli
export PATH="$HOME/.local/bin:$PATH"
```

### Source startup says `config.yaml` is missing

```bash
cp config.example.yaml config.yaml
```

Then edit `config.yaml` or run CLI configuration commands.

### Bun is missing

Install Bun and rerun:

```bash
curl -fsSL https://bun.sh/install | bash
exec "$SHELL" -l
bun install
```

### Vite fails with Rollup native addon code-signing errors on macOS

If `bun run dev:web` fails with `@rollup/rollup-darwin-arm64`, `ERR_DLOPEN_FAILED`, or a Team ID code-signing mismatch, make sure Homebrew Node comes before app-bundled Node in `PATH`:

```bash
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH cargo run -p hone-cli -- web admin-ui --dev
```

See the detailed source Web startup runbook: [`docs/runbooks/source-web-startup.md`](./runbooks/source-web-startup.md).

### Port already occupied

Inspect the process holding the port before stopping it:

```bash
lsof -nP -iTCP:8077 -sTCP:LISTEN
lsof -nP -iTCP:3000 -sTCP:LISTEN
```

### Backend starts but Web assets are missing

For source checkout:

```bash
bun run build:web
bun run build:web:public
```

For installed release, reinstall the latest bundle and confirm:

```bash
ls ~/.honeclaw/current/share/honeclaw/web/index.html
ls ~/.honeclaw/current/share/honeclaw/web-public/index.html
```

### A channel exits during startup

Check config and logs:

```bash
tail -200 data/runtime/logs/*.log
hone-cli channels list
```

A disabled channel may exit intentionally with the configured skip path. Missing credentials, invalid tokens, lock conflicts, or port conflicts are the usual real failures.

### Desktop opens but shows a blank page

For source desktop release mode, rebuild desktop Web assets with the desktop-specific asset base:

```bash
bun run build:web:desktop
bun run build:desktop
```

For desktop dev, prefer:

```bash
bun run tauri:prep:dev -- --skip-dev-command --shell-only
bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json
```

## Contributor Reading Map

Read in this order when changing the codebase:

1. [`AGENTS.md`](../AGENTS.md): collaboration, testing, docs, and release contract.
2. [`docs/repo-map.md`](./repo-map.md): stable architecture map and key files.
3. [`docs/invariants.md`](./invariants.md): constraints that should not be casually broken.
4. [`docs/current-plan.md`](./current-plan.md): active tracked work.
5. Relevant runbooks under [`docs/runbooks/`](./runbooks/).
6. Relevant source entrypoints and tests.

Useful engineering docs:

| Doc | Use |
| --- | --- |
| [`docs/technical-spec.md`](./technical-spec.md) | Detailed implementation supplement. |
| [`docs/conventions/periodic_tasks.md`](./conventions/periodic_tasks.md) | Periodic task tracing and observer conventions. |
| [`docs/bugs/README.md`](./bugs/README.md) | Bug ledger and repair backlog. |
| [`docs/decisions.md`](./decisions.md) | Long-lived decisions and ADR pointers. |
| [`docs/runbooks/hone-cli-install-and-start.md`](./runbooks/hone-cli-install-and-start.md) | Installed CLI setup and runtime start. |
| [`docs/runbooks/desktop-dev-runtime.md`](./runbooks/desktop-dev-runtime.md) | Desktop dev/runtime mode guidance. |
| [`docs/runbooks/desktop-release-app-runtime.md`](./runbooks/desktop-release-app-runtime.md) | Release desktop runtime operations. |
| [`docs/runbooks/backend-deployment.md`](./runbooks/backend-deployment.md) | Backend deployment notes. |

## Maintenance Notes

- Update this wiki when startup commands, ports, major directories, or first-run setup change.
- Update `docs/repo-map.md` when module boundaries, entrypoints, or major data flows change.
- Update runbooks when operational procedures change.
- Keep secrets out of docs, examples, and committed config.
