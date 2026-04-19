# Repo Map

Last updated: 2026-04-19

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
  - `hone-llm`: model provider abstraction, OpenRouter integration, and generic OpenAI-compatible provider plumbing used by the desktop `multi-agent` search stage
  - `hone-tools`: tool traits, registry, and built-in tools; the skill subsystem centers on `src/skill_runtime.rs`, `skill_tool`, the local `discover_skills` index, the `skill_registry` enabled/disabled override layer, and the compatibility `load_skill` shim. `skill_tool` still parses structured script `stdout` and validates local image artifact roots/extensions before exposing them to the model.
  - `hone-integrations`: external integrations such as X, Feishu, and image generation
  - `hone-scheduler`: scheduled task orchestration
  - `hone-channels`: channel runtime, `HoneBotCore`, shared channel startup bootstrap, unified `agent_session` orchestration, the shared `execution` preparation layer, and the separate `runners` execution layer; it also hosts shared `ingress` (incoming envelope / actor scope / dedup / session lock / group pretrigger window), `outbound` (placeholder / reasoning / chunking / stream probes，以及把助手文本里的 `file://` 本地图片 marker 拆成有序 text/image 片段的共享逻辑), repo-external actor sandbox management, prompt-audit / session-compaction helpers, the cross-channel pre-session intercept layer for commands such as `/register-admin` and `/report`, plus shared attachment ingest / PDF preview helpers under `attachments/{ingest,vision,vector_store}.rs`. Feishu / Discord / Telegram attachment size and image-dimension gates are also centralized here.
- `agents/`
  - `function_calling`: function-calling agent core
  - `gemini_cli`, `codex_cli`: CLI agent adapters
  - `gemini_acp`, `codex_acp`, `opencode_acp`: agent runner adapters based on ACP stdio / JSON-RPC
  - `multi-agent`: two-stage runner wiring that combines a direct function-calling search pass with an ACP answer pass
- `memory/`
  - Local storage abstractions for sessions, identity quotas, portfolios, cron jobs, and LLM audit logs
  - `memory/src/company_profile/{mod,types,markdown,storage,transfer,tests}.rs` now splits company portraits into stable public types, Markdown/template parsing, actor-scoped storage CRUD, zip transfer helpers, and colocated regression tests; portraits still live under `company_profiles/<profile_id>/profile.md` plus append-only `events/*.md`, and both storage reads and transfer/import paths tolerate legacy plain Markdown files without frontmatter by synthesizing minimal metadata from titles, filenames, file mtimes, and bundle manifest timestamps
  - `memory/src/web_auth.rs` keeps web invite users and public-login cookie sessions in the shared SQLite DB; one invite code maps to one stable `channel=web` actor
  - `memory/src/session.rs` currently stores versioned session JSON (v3) and explicitly persists `summary`, legacy `runtime.prompt.frozen_time_beijing`, recoverable `tool` result messages, and the session ownership field `session_identity`; current prompt assembly no longer uses that legacy frozen timestamp as the displayed "当前时间"
  - `memory/src/session_sqlite.rs` hosts the SQLite-backed session persistence used by both shadow backfill and runtime reads/writes when `storage.session_runtime_backend=sqlite`
  - `memory/src/cron_job.rs` keeps cron definitions in per-actor JSON files and mirrors cron execution history into the shared SQLite DB so task detail can query per-run records
  - `memory/src/quota.rs` stores `success_count` / `in_flight` in JSON files by `ActorIdentity` and by Beijing date
- `bins/`
  - `hone-console-page`: Web console backend, static asset hosting, and API
  - `hone-cli`: local REPL
  - `hone-mcp`: local stdio MCP server that exposes Hone built-in tools to ACP runners
  - `hone-imessage`, `hone-telegram`, `hone-discord`, `hone-feishu`: channel entrypoints, with shared startup in `hone-channels::bootstrap` and per-channel sibling modules for scheduler / outbound / handlers where the protocol layer needs local ownership
- `hone-desktop`: Tauri desktop host with a thin `main.rs` façade, command handlers in `commands.rs`, backend / sidecar lifecycle in `sidecar.rs`, sidecar concern modules in `sidecar/{processes,runtime_env,settings}.rs`, tray extension points in `tray.rs`, and the desktop window packaging flow
- `config.yaml` / `data/runtime/`
  - `config.yaml` is the canonical user-writable config; dev uses the repo root copy, and packaged installs seed one under the user config dir
  - `data/runtime/effective-config.yaml` is the generated runtime snapshot for processes that want a materialized runtime config file
  - legacy `data/runtime/config_runtime.yaml` and sibling `.overrides.yaml` should not be recreated
- Actor sandbox research docs live under `agent-sandboxes/<channel>/<scope__user>/company_profiles/<profile_id>/profile.md` plus `events/*.md`; this actor-local directory is the source of truth for company portraits and long-term fundamental tracking
- `packages/`
  - `app`: SolidJS web console
  - `ui`: shared UI components and context
- `skills/`
  - In-repo skill definitions; runtime also supports `data/custom_skills/<id>/SKILL.md` and nested `.hone/skills/<id>/SKILL.md` with nearer dynamic directories taking precedence
  - `SKILL.md` frontmatter now also supports an opt-in `script` entrypoint that `skill_tool(..., execute_script=true)` can run from the skill directory
  - `skills/chart_visualization/` 是内置图表 skill：`SKILL.md` 定义 chart spec 与 `file:///abs/path.png` 输出契约，`scripts/render_chart.py` 用 Python `matplotlib` 把 PNG 写进 Hone runtime 的 `gen_images` 目录
  - `skills/company_portrait/` now follows a lighter Codex-style pattern: keep the trigger/workflow contract in `SKILL.md`, and move the detailed portrait framework / event template / research-trail guidance into `references/`
- `data/runtime/skill_registry.json`
  - Global skill enabled/disabled override layer for registered skills
- `tests/regression/`
  - `ci/`: CI-safe
  - `manual/`: manual regression tests that depend on an external CLI or account

## Key Entry Points

- Web console backend: `bins/hone-console-page/src/main.rs`
- Web console frontend: `packages/app/src/app.tsx`
  - 管理端与用户端现在按端口和构建产物分离：管理端默认走 `HONE_WEB_PORT` + `packages/app/dist`，用户端默认走 `HONE_PUBLIC_WEB_PORT` + `packages/app/dist-public`
  - 用户可见的长期研究记忆入口现只保留 `/memory` 下的公司画像视图；KB 页面与知识记忆 tab 已移除
- CLI: `bins/hone-cli/src/main.rs`
  - `hone-cli` now has explicit subcommands for `chat`, `config`, `configure`, `models`, `channels`, `status`, `doctor`, and `start`; no-subcommand mode still drops into the local chat REPL
- Channel runtime export: `crates/hone-channels/src/lib.rs`
- Shared channel bootstrap: `crates/hone-channels/src/bootstrap.rs`
- `AgentSession` abstraction: `crates/hone-channels/src/agent_session.rs`
  - Owns turn-0 skill listing disclosure, related-skill hints, slash-skill expansion, and invoked-skill restoration after compaction
- Shared execution preparation: `crates/hone-channels/src/execution.rs`
  - Centralizes prompt-audit write, tool registry creation, runner creation, and actor-sandbox-backed `AgentRunnerRequest` assembly for both session and transient task flows
- Shared ingress model: `crates/hone-channels/src/ingress.rs`
- Shared outbound model: `crates/hone-channels/src/outbound.rs`
  - 同时也是 canonical 本地图片 marker 解析入口；Web 历史提取与外部通道图片投递都复用这里的 `file:///abs/path.png` 分段规则
- Runtime config override source of truth: `crates/hone-core/src/{config.rs,config/server.rs}`
- ACP MCP bridge: `crates/hone-channels/src/mcp_bridge.rs`
- Actor sandbox: `crates/hone-channels/src/sandbox.rs`
- Attachment ingest / preview helpers: `crates/hone-channels/src/attachments.rs` and `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
  - Enforces shared attachment gates across channels: 5 MB for generic attachments, 3 MB for images, plus rejection of extreme aspect ratio, resolution, or pixel-count cases. Rejected attachments never enter the prompt.
- Runner contract and ACP / Gemini execution layer: `crates/hone-channels/src/runners/`
  - `mod.rs`: runner exports
  - `types.rs`: shared runner trait / request / event / result types
  - `acp_common.rs`: shared helpers for ACP stdio / JSON-RPC
  - `gemini_cli.rs`, `gemini_acp.rs`, `codex_acp.rs`, `opencode_acp.rs`, `multi_agent.rs`: runner implementations
- Prompt layering: `crates/hone-channels/src/prompt.rs`
  - Injects the global finance-domain constraints in one place: no stock-picking recommendations, reject non-finance questions, warn users not to blindly follow buy or sell advice, and keep greetings short
- Session compaction service: `crates/hone-channels/src/session_compactor.rs`
- Prompt audit writer: `crates/hone-channels/src/prompt_audit.rs`
- Tool registry entry point: `crates/hone-tools/src/lib.rs`
- Skill runtime source of truth: `crates/hone-tools/src/skill_runtime.rs`
- Desktop sidecar helpers: `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs`
- Feishu channel split: `bins/hone-feishu/src/{handler.rs,scheduler.rs,outbound.rs}`
- Feishu image upload client: `bins/hone-feishu/src/client.rs`
- Telegram scheduler split: `bins/hone-telegram/src/scheduler.rs`
- Telegram outbound text/image interleave handling: `bins/hone-telegram/src/listener.rs`
- Discord outbound text/image interleave handling: `bins/hone-discord/src/utils.rs`
- Settings page pure state helpers: `packages/app/src/pages/settings-model.ts`
- Config sample: `config.example.yaml`
- GitHub install script: `scripts/install_hone_cli.sh`

## Main Flow

1. A channel entrypoint or the Web API receives user input and performs protocol parsing, allowlist checks, and explicit-trigger detection on the channel side
2. Before entering `AgentSession::run()`, channel entrypoints may short-circuit shared pre-session intercept commands in `hone-channels::core`, including runtime admin registration and the local report-workflow bridge (`/report 公司名`, `/report 进度`)
3. `hone-channels::ingress` centralizes actor scope, chat mode, deduplication, session serialization, shared group pretrigger buffering, and `IncomingEnvelope`
4. `hone-channels::AgentSession::run()` orchestrates session semantics such as fast user-message persistence, slash skill expansion, quota, and response persistence; it now needs an explicit distinction between:
    - `ActorIdentity`: who is executing this request
    - `SessionIdentity`: which history this message should be written into (group-chat shared sessions are controlled by it)
5. `hone-channels::execution` builds the concrete execution plan for both persistent conversations and transient tasks: prompt audit, tool registry, runner selection, and actor-sandbox-backed `AgentRunnerRequest`
6. `hone-channels::runners` executes the chosen runtime based on `agent.runner` and maps provider / CLI events back into unified session events. ACP runners now include a local `hone-mcp` server so Hone built-in tools are exposed as MCP tools to the underlying agent. Channel runners default to a repo-external actor sandbox.
7. `hone-channels::AgentSession::run()` stores parseable tool-call results returned by the runner into the session for future cross-turn recovery; `hone-channels::outbound` and each channel adapter consume the unified events and finish placeholder / reasoning / chunked / streaming responses according to platform capability。当前本地图表等媒体仍通过最终 assistant 文本里的 inline `file://` marker 传递：Web 保留 marker 并内联渲染，Feishu / Telegram / Discord 则按顺序把它转成真实图片消息。
8. `hone-tools` provides data, skills, search, scheduled-task, and other capabilities
   - Skill disclosure is now two-phase: the model first sees a compact listing, and full `SKILL.md` bodies are only expanded into the turn after `skill_tool(...)` or a user slash skill is invoked
   - Invoked skill prompts are persisted in session metadata so context restoration can re-inject them after compression instead of relying on historic tool results
   - 用户可见的研究记忆相关 skill 目前只保留 `company_portrait`
9. `memory` reads and writes local sessions, quotas, portfolios, and cron jobs
  - `memory/src/quota.rs` keeps a daily successful-reply quota for each user-initiated conversation; the runtime limit now comes from `agent.daily_conversation_limit`, and `0` means unlimited
    - `memory/src/llm_audit.rs` uses SQLite to record LLM call audit logs archived by `ActorIdentity`
    - Session persistence is controlled by `storage.session_runtime_backend`; `json` reads from local files, `sqlite` reads from `storage.session_sqlite_db_path`, and JSON can still be dual-written as a rollback mirror through `storage.session_sqlite_shadow_write_enabled`
    - Session compaction is now boundary-based: compacted sessions write a `Conversation compacted` marker plus a compact summary message, and the active context window is restored from the most recent boundary forward
    - `codex_acp` and `opencode_acp` session turns now persist restorable assistant/tool transcript structure locally as `assistant(tool_calls)` + `tool` messages; `codex_acp` uses it to reseed recreated ACP sessions, while `opencode_acp` injects the restored transcript into each fresh ACP session prompt because OpenCode does not safely replay prior sessions
    - `AgentSession::run()` now also supports explicit `/compact` requests, reusing the same compaction pipeline without charging user conversation quota or persisting the slash command as a normal transcript message
    - Heartbeat-style cron jobs are still stored in the same cron store; they are identified by `repeat=heartbeat` and a `heartbeat` tag, then polled every 30 minutes instead of a fixed clock time
9. Responses are sent back to the originating channel; the Web console streams `run_started / assistant_delta / tool_call / run_error / run_finished` via v2 SSE events

## Desktop Structure

- The Tauri host lives in `bins/hone-desktop/`
- `bins/hone-desktop/src/{main.rs,commands.rs,sidecar.rs,tray.rs}` now separates the builder façade, Tauri command handlers, backend lifecycle, and tray extension point
- `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs` keeps process supervision, runtime environment/path wiring, and persisted desktop settings / overlay writes out of the main Tauri command surface
- Desktop sidecars are prepared by `scripts/prepare_tauri_sidecar.mjs`, which detects the target triple, builds the supported channel bins plus `hone-mcp`, resolves/bundles macOS `opencode`, copies them into `bins/hone-desktop/binaries/`, and writes `bins/hone-desktop/tauri.generated.conf.json` for `bunx tauri dev/build`
- The same script also supports target-override / skip-build self-checks, so macOS packaging expectations can be verified by regenerating config for `*-apple-darwin` without requiring a full build
- Root `make_dmg_release.sh` is the macOS release entrypoint: it prepares bundled binaries for `aarch64-apple-darwin` and `x86_64-apple-darwin`, runs `tauri build --target`, and collects DMGs into `dist/dmg/`
- Tag release workflow emits installable CLI bundles (`honeclaw-darwin-aarch64.tar.gz`, `honeclaw-darwin-x86_64.tar.gz`, `honeclaw-linux-x86_64.tar.gz`) containing `hone-cli`, runtime binaries, built Web assets, `skills/`, `config.example.yaml`, and `soul.md`; it also requires a checked-in user-facing release note at `docs/releases/vX.Y.Z.md` instead of relying on GitHub auto-generated notes. `scripts/install_hone_cli.sh` consumes those assets for the `curl | bash` path, prefers installing the wrapper into an already-on-PATH writable user bin directory, and falls back to `~/.local/bin` with an explicit export hint. The same workflow also uploads `SHASUMS256.txt` and pushes the generated `honeclaw.rb` into the dedicated tap repo `B-M-Capital-Research/homebrew-honeclaw` so `brew install B-M-Capital-Research/honeclaw/honeclaw` resolves without a custom remote
- Release-oriented Rust builds are warmed in two layers: `.github/workflows/release-cache-warm.yml` prebuilds the three shipped targets on `main`, `Swatinem/rust-cache` stores dependency/`target` state per release target, and `sccache` stores compiler outputs so tag releases mostly reuse warmed caches instead of compiling cold
- Windows desktop packaging intentionally excludes `hone-imessage`; macOS packaging keeps it, and runtime support still uses `cfg!(target_os = "macos")` as the source of truth
- `./launch.sh --desktop` is intentionally single-runtime: it starts Vite + Tauri dev only, and the desktop sidecar is responsible for starting the bundled `hone-console-page` plus enabled channel listeners. Do not start the external backend/channel set in parallel for desktop dev, or logs and incoming updates will split across duplicate processes
- `./launch.sh --desktop --remote` is the dev mode to use when desktop/UI hot reload should not interrupt long-running backend or channel listeners: the launcher starts the normal external `hone-console-page` + channel set first, writes the desktop backend config to `remote`, and then starts Vite + Tauri dev against that remote base URL
- `./launch.sh --release` starts a release desktop binary without `tauri dev` hot reload. It is intended for long-running desktop verification when source edits should not automatically restart the desktop host. By default the launcher pins `HONE_DESKTOP_DATA_DIR` to the repo `data/` directory and `HONE_DESKTOP_CONFIG_DIR` to `data/runtime/desktop-config/` so the release desktop instance can reuse project-local runtime data instead of silently drifting to an app-specific config/data root
- `hone-cli onboard` is the first-install guided setup path for bundled CLI installs and repo-local use: it can detect local `codex` / `codex-acp` / `opencode`, switch to `opencode_acp` without forcing Hone-side provider config, guide channel enablement with mandatory local fields plus prerequisite notes, let the user back out of a mistaken channel enablement by disabling that channel mid-flow, and require an explicit configure-or-skip decision for `FMP` / `Tavily` API keys
- `hone-cli start` is the runtime-only local launch entry for bundled CLI installs and repo-local use: it loads canonical `config.yaml`, generates `data/runtime/effective-config.yaml`, starts `hone-console-page`, waits for `/api/meta`, then starts enabled channel listeners without going through `launch.sh`
- `hone-cli cleanup` is the explicit installed-layout teardown helper: it can interactively remove `~/.honeclaw` config, runtime data, and downloaded release bundles before the user runs `brew uninstall honeclaw` or removes the wrapper manually
- Desktop startup now uses per-process runtime lock files under `data/runtime/locks/` (or the app runtime dir in packaged mode). `hone-desktop` must hold its own lock, each standalone channel/backend binary must hold its own lock, and bundled desktop mode preflights the full `hone-console-page` + enabled-channel set before startup. When the conflict still points at a live matching Hone process, desktop startup now attempts one lock-targeted cleanup by pid and then retries before surfacing the blocking error.
- The desktop app supports two backend modes:
  - `bundled`: Tauri starts the built-in `hone-console-page` sidecar and points the frontend API at a local loopback address
  - `remote`: Tauri does not start a local backend; the frontend connects directly to a remote HTTP base URL
- Persistent user config now lives in canonical `config.yaml`; CLI/start flows and desktop-managed sidecars export the generated `data/runtime/effective-config.yaml`, while settings surfaces mutate the canonical file through shared config services. Desktop dev/runtime uses the desktop config dir as the canonical location and may only promote missing values one-way from legacy `data/runtime/config_runtime.yaml`, including runner, multi-agent, enabled channels, Tavily search keys, and FMP keys
- In packaged desktop mode, runtime data, locks, logs, and actor sandboxes live under the app sandbox data directory by default; the desktop host also hydrates key login-shell environment variables and exports bundled binary paths (`HONE_MCP_BIN`, bundled `opencode`, `HONE_AGENT_SANDBOX_DIR`) before starting the embedded backend or channel sidecars
- Desktop agent settings now expose the primary opencode/OpenRouter model, a dedicated `llm.auxiliary` OpenAI-compatible background route for heartbeat/session compression, and the nested `multi-agent` search/answer config. `llm.openrouter.sub_model` remains only as the legacy fallback model name for the auxiliary path; it is not reused as the `multi-agent` search model
- In `bundled` mode, Tauri also starts or stops `hone-imessage` / `hone-discord` / `hone-feishu` / `hone-telegram` according to the layered runtime config in the application data directory; each channel process now posts heartbeat snapshots carrying `channel + pid` back to the console backend via `HONE_CONSOLE_URL`, and `/api/channels` aggregates those live registrations into per-channel multi-process status. Desktop channel status also merges OS process scanning so duplicate listener processes are visible even when an older instance is not bound to the current backend heartbeat registry, and the desktop shell exposes a cleanup command that keeps only one process per channel. The legacy `runtime/*.heartbeat.json` files still exist as a compatibility fallback for non-desktop paths
- Desktop log pages read from `/api/logs`; the backend route now merges the in-memory log ring with recent `data/runtime/logs/*.log` tails so bundled desktop mode can display channel/runtime logs even when they were written by sibling processes instead of the current web process
- Frontend backend runtime lives in `packages/app/src/context/backend.tsx` and `packages/app/src/lib/backend.ts`
- Assistant message parser for inline local images: `packages/app/src/lib/messages.ts`
- `hone-console-page` `/api/meta` handles version and capability negotiation
- `hone-console-page` admin app only serves `/api/*` and console SPA on the admin port; the public app only serves `/api/public/*` plus `/chat` on the public port for invite-based web users
- `hone-console-page` `/api/skills*` serves the skill management surface: registered listing, detail view, enable/disable mutation, and reset
- `hone-console-page` `/api/company-profiles*` now serves actor-space listing, portrait detail, full deletion, and actor-scoped portrait bundle transfer (`export`, `import/preview`, `import/apply`) for actor-local portrait docs; portrait creation and section/event updates still rely on runner-native file operations inside the actor sandbox rather than dedicated mutation APIs
- `packages/app/src/context/company-profiles.tsx` now acts as the memory-page transfer orchestrator: it merges portrait actor spaces with recent session users into one target-selector model, supports manual target entry for first-time imports, runs bundle preview/apply, keeps post-import highlights plus optional pre-import backup blobs, and auto-selects the first company in the current target space so the right panel does not fall back to a false empty state

## Web Console Structure

- Route entrypoint: `packages/app/src/app.tsx`
- Pages: `packages/app/src/pages/`
  - admin surface keeps `/start` and the management console routes
  - public surface only exposes `/` and `/chat`, both pointing at the invite-login chat experience
- Settings page state helpers: `packages/app/src/pages/settings-model.ts`
- Domain state: `packages/app/src/context/`
- Composite components: `packages/app/src/components/`
- API access and data transformation: `packages/app/src/lib/`

## Common Coupled Changes

- Adding a tool:
  - Change `crates/hone-tools/src/*`
  - Update `agents/function_calling` if needed
  - If the Web UI needs to show it, also update `bins/hone-console-page/src/main.rs` and the frontend pages
- Adjusting the skill runtime:
  - Start with `crates/hone-tools/src/skill_runtime.rs`, `crates/hone-tools/src/{skill_registry.rs,skill_tool.rs}`
  - Then check `crates/hone-channels/src/{agent_session.rs,core.rs,prompt.rs,mcp_bridge.rs,runtime.rs}`
  - If the Web UI is affected, also check `crates/hone-web-api/src/routes/skills.rs` and `packages/app/src/{context/skills.tsx,components/skill-*.tsx,lib/skill-command.ts}`
- Adding a Web page or dashboard:
  - Change `packages/app/src/pages/*`
  - Change `packages/app/src/context/*` and / or `packages/app/src/lib/*`
  - If the backend API is insufficient, add the Web bin API
  - Invite-based public user flows also require checking `memory/src/web_auth.rs` and `crates/hone-web-api/src/routes/public.rs` instead of wiring directly into the console-only `/api/chat` / `/api/history` / `/api/users` routes
- Adjusting desktop backend switching or sidecar lifecycle:
  - Change `bins/hone-desktop/src/{main.rs,commands.rs,sidecar.rs,tray.rs}`
  - If the change is process supervision, runtime env, or persisted overlay wiring, start with `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs`
  - Change `packages/app/src/context/backend.tsx` and / or `packages/app/src/lib/backend.ts`
  - Update `bins/hone-console-page/src/main.rs` and the runtime config loading for the channel bins if needed
- Adding channel behavior:
  - Change the matching `bins/*`
  - If the change is startup / enable checks / heartbeat / process lock wiring, start with `crates/hone-channels/src/bootstrap.rs`
  - Feishu scheduled delivery and outbound rendering now live in `bins/hone-feishu/src/{scheduler.rs,outbound.rs}`; Telegram scheduled delivery lives in `bins/hone-telegram/src/scheduler.rs`
  - Update `hone-channels`, `hone-core`, or `memory` if needed
  - If the change touches incoming envelopes, dedup, actor scope, placeholder / streaming delivery, or attachment persistence, start with `crates/hone-channels/src/ingress.rs`, `crates/hone-channels/src/outbound.rs`, and `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
- Adjusting persistence structure:
  - Start with `memory/`
  - Then check the Web API, channel entrypoints, and frontend pages that depend on it
- Adjusting company portraits:
  - Start with `memory/src/company_profile/{mod,types,markdown,storage,transfer}.rs`
  - Then check `crates/hone-channels/src/{sandbox.rs,prompt.rs,core.rs}` and `crates/hone-web-api/src/routes/company_profiles.rs`
  - If the Web UI is affected, also check `packages/app/src/{context/company-profiles.tsx,components/company-profile-*.tsx,pages/memory.tsx}`
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
- Telegram / Discord / Feishu now gate direct-vs-group ingress through per-channel `chat_scope` (`DM_ONLY | GROUPCHAT_ONLY | ALL`), while group chats still share one model: untriggered text is buffered in a short pretrigger window, and only an explicit `@bot` / reply-to-bot trigger flushes that buffered text into the shared group session before `AgentSession::run()`.
- Group explicit triggers now expose a busy lifecycle: if one group session is still processing, the next explicit trigger gets an immediate “wait for the previous message” reply and its text is re-buffered into the pretrigger window instead of starting a second concurrent run.
- Scripts in `tests/regression/manual/` depend on local environment state or external accounts and must not be promoted to default CI gates
- iMessage capabilities depend on local macOS permissions and cannot be assumed to work in CI or on non-macOS environments
- Desktop packaging depends on a local Rust + Tauri toolchain; if `cargo` or `bun` is missing, only static changes are possible, not a full compile verification
- Default repo-wide Rust verification should keep using `cargo check --workspace --all-targets --exclude hone-desktop`; desktop packaging is a separate validation lane.
- For local IDE / syntax checks on the desktop crate itself, use `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop` so Tauri skips bundled sidecar existence validation while still type-checking Rust code.
- Real desktop packaging validation must still use the generated Tauri config / prepared sidecars path (`bun run tauri:prep:*` + `bunx tauri dev/build`); the skip flag is not a substitute for release-time resource checks.
- `opencode_acp` now treats the user's local OpenCode config as the default source of provider/auth/model truth. The Hone runner may still inject a small custom `OPENCODE_CONFIG` for ACP permissions and explicit `agent.opencode.*` overrides, but it should not hide `~/.config/opencode/opencode.json` / `opencode.jsonc` by replacing the entire OpenCode config home.

## Suggested Reading Order

1. `AGENTS.md`
2. `docs/repo-map.md`
3. `docs/invariants.md`
4. `docs/current-plan.md`
5. The matching `docs/current-plans/*.md`
6. The relevant entry files and tests
