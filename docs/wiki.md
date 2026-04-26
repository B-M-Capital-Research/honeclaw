# Hone Wiki

Last updated: 2026-04-26

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
- [Model And Runner Setup](#model-and-runner-setup)
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
| `launch.sh` | Source checkout launcher for backend, Web, desktop, and release-desktop modes. |
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
| `data/runtime/*.pid` | Launcher pid files. |
| `data/runtime/locks/` | Process lock files. |
| `data/sessions.sqlite3` | SQLite session runtime store when enabled. |
| `agent-sandboxes/` | Actor-scoped workspaces and company profile docs. |

Installed release defaults:

| Path | Purpose |
| --- | --- |
| `~/.honeclaw/current` | Active release bundle symlink. |
| `~/.honeclaw/config.yaml` | User config. |
| `~/.honeclaw/data` | Runtime data. |
| `~/.honeclaw/data/runtime/effective-config.yaml` | Generated runtime config. |
| `~/.honeclaw/current/share/honeclaw/skills` | Bundled skills. |
| `~/.honeclaw/current/share/honeclaw/web` | Bundled Web assets. |

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

`hone-cli onboard` guides runner selection, optional channel setup, and provider/API-key setup. `hone-cli start` starts the local backend plus enabled channel listeners in the foreground.

See the detailed runbook: [`docs/runbooks/hone-cli-install-and-start.md`](./runbooks/hone-cli-install-and-start.md).

## Start From Source

Clone and prepare:

```bash
git clone https://github.com/B-M-Capital-Research/honeclaw.git
cd honeclaw
cp config.example.yaml config.yaml
bun install
```

Start backend and enabled channel listeners only:

```bash
./launch.sh
```

Start backend, enabled channels, and both admin/public Vite frontends:

```bash
./launch.sh --web
```

Start desktop development mode where the desktop owns bundled backend/channel sidecars:

```bash
./launch.sh --desktop
```

Start backend/channels outside the desktop host, then launch desktop in remote mode:

```bash
./launch.sh --desktop --remote
```

Start release desktop mode without Tauri dev hot reload:

```bash
./launch.sh --release
```

Stop launcher-managed processes:

```bash
./launch.sh stop
```

## Desktop Startup Modes

Use the mode that matches what you are testing:

| Command | Best For | What It Starts |
| --- | --- | --- |
| `./launch.sh --desktop` | Bundled desktop integration checks. | Vite + Tauri dev; desktop starts embedded backend and enabled channels. |
| `./launch.sh --desktop --remote` | Daily desktop UI work while keeping backend/channels stable. | Backend + channels + Vite + Tauri dev connected to remote backend config. |
| `./launch.sh --release` | Long-running desktop verification without Rust hot reload. | Builds release desktop assets and runs the release desktop binary. |

For daily development, prefer `--desktop --remote` when backend/channel processes should not restart on every desktop shell rebuild. Use `--desktop` when validating bundled sidecar startup, process locks, and desktop-managed runtime behavior.

## Web Startup Modes

| Command | Best For |
| --- | --- |
| `./launch.sh` | Runtime-only backend/channel smoke. |
| `./launch.sh --web` | Browser UI development against local backend/channels. |
| `bun run dev:web` | Frontend-only admin UI work when backend is already running. |
| `bun run dev:web:public` | Frontend-only public chat UI work when public backend is already running. |
| `bun run build:web` | Build admin Web assets. |
| `bun run build:web:public` | Build public Web assets. |
| `bun run build:web:desktop` | Build desktop Web assets with relative asset paths. |

`./launch.sh --web` starts both the admin and public Vite servers after the backend is ready.

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
hone-cli config set agent.runner opencode_acp
hone-cli models set --runner opencode_acp --model openrouter/openai/gpt-5.4 --variant medium
```

Important config areas:

- `agent.*`: runner choice, model routing, timeout behavior.
- `llm.*`: provider keys and OpenAI-compatible/OpenRouter routes.
- `channels.*`: Feishu, Discord, Telegram, and iMessage enablement and credentials.
- `web.*`: admin/public ports and host behavior.
- `storage.*`: JSON/SQLite session backend and data paths.
- `scheduler.*`: scheduled task and heartbeat behavior.
- `event_engine.*`: market/news event monitoring and delivery.
- `search.*`, `fmp.*`: external data/search providers.

Never commit local secrets in `config.yaml`.

## Model And Runner Setup

Hone can use local CLI/ACP runners or OpenAI-compatible cloud APIs.

Common runner choices:

| Runner | Use When |
| --- | --- |
| `opencode_acp` | You want Hone to inherit local OpenCode provider/model config. |
| `codex_acp` | You use Codex ACP and want ACP session integration. |
| `codex_cli` | You use Codex CLI directly. |
| `function_calling` | You want the built-in OpenAI-compatible function-calling path. |
| `multi-agent` | You want separate search and answer stages. |

Typical OpenCode setup:

```bash
curl -fsSL https://opencode.ai/install | bash
hone-cli config set agent.runner opencode_acp
hone-cli start
```

Typical model override:

```bash
hone-cli models set --runner opencode_acp --model openrouter/openai/gpt-5.4 --variant medium
```

If using cloud APIs, configure keys through `hone-cli onboard`, `hone-cli configure`, or direct config edits.

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
HONE_WEB_PORT=8078 HONE_PUBLIC_WEB_PORT=8089 ./launch.sh --web
```

## Stop, Restart, And Cleanup

Stop processes started by `launch.sh`:

```bash
./launch.sh stop
```

Stop a foreground `hone-cli start`:

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

### `./launch.sh` says `config.yaml` is missing

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

### Port already occupied

Try the managed stop first:

```bash
./launch.sh stop
```

If a process is still holding a port, inspect it:

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
./launch.sh --release
```

For desktop dev, prefer:

```bash
./launch.sh --desktop --remote
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
