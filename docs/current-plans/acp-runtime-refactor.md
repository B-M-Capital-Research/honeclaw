# ACP Runtime Refactor

最后更新：2026-03-18
状态：进行中

## 目标

- 将现有 `AgentSession` 重构为统一编排层，消除 `run_blocking` / `run_gemini_streaming` 双轨执行。
- 引入 ACP 对齐的统一 runner/event 模型，并纳入 `function_calling`、`gemini_cli`、`gemini_acp`、`codex_cli`、`codex_acp` 与 `opencode_acp`。
- 重构 prompt 结构，冻结 session 时间，提升前缀缓存命中率。
- 以 breaking 方式升级 Web SSE、前端流式类型、配置项与 session 持久化结构。

## 涉及文件

- `crates/hone-core/src/agent.rs`
- `crates/hone-core/src/config.rs`
- `crates/hone-channels/src/agent_session.rs`
- `crates/hone-channels/src/core.rs`
- `crates/hone-channels/src/runners/mod.rs`
- `crates/hone-channels/src/runners/types.rs`
- `crates/hone-channels/src/runners/acp_common.rs`
- `crates/hone-channels/src/runners/gemini_cli.rs`
- `crates/hone-channels/src/runners/gemini_acp.rs`
- `crates/hone-channels/src/runners/codex_acp.rs`
- `crates/hone-channels/src/runners/opencode_acp.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-channels/src/sandbox.rs`
- `crates/hone-channels/src/lib.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-web-api/src/routes/chat.rs`
- `packages/app/src/lib/types.ts`
- `packages/app/src/context/sessions.tsx`
- `memory/src/session.rs`
- `bins/hone-discord/src/*`
- `bins/hone-feishu/src/main.rs`
- `bins/hone-imessage/src/main.rs`
- `bins/hone-telegram/src/main.rs`
- `bins/hone-cli/src/main.rs`
- `bins/hone-mcp/src/main.rs`
- `config.yaml`
- `config.example.yaml`
- `tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- `tests/regression/manual/test_codex_acp_initialize.sh`
- `docs/current-plan.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/runbooks/opencode-setup.md`
- `AGENTS.md`

## Todo

- 定义统一 runner/event 契约，并让 `AgentSession` 只保留会话编排职责。
- 重构 prompt 构建为静态 system / session 固定上下文 / 当前会话上下文三层。
- 升级 session schema：版本化、显式 summary/runtime metadata、冻结时间字段。
- 升级 `agent.provider` 为 breaking 的 runner 配置结构，纳入 `opencode_acp`、`codex_acp` 与 `gemini_acp`。
- 为渠道 actor 引入 repo 外的独立 sandbox root，并让附件落盘、CLI cwd 与 ACP cwd 全部指向该隔离目录。
- 升级 Web SSE 与前端流式事件协议，移除对旧 `ack/tool_status/segment/done` 语义的依赖。
- 迁移各渠道 listener 到新事件协议。
- 验证：`cargo check --workspace --all-targets --exclude hone-desktop`、针对性 `cargo test`、`bun run typecheck:web`。
- 文档同步：更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`，完成后补 `docs/handoffs/`.

## 当前进展

- 已确认参考实现来源：
  - `AionUi`：ACP backend 路由、渠道适配模式。
  - `opencode`：ACP agent、tool lifecycle、session/message/part 持久化模型。
- 已确认 Hone 当前主要约束：
  - `AgentSession` 仍是双轨执行。
  - prompt 中动态时间、`session_id`、summary 会破坏前缀缓存。
  - Web SSE 与前端依赖旧事件语义。
- 已完成第一阶段落地：
  - `AgentSession::run()` 已成为统一入口；渠道/Web 主调用链已切到该入口
  - session JSON 升级为 v2，显式保存 `summary` 与 `runtime.prompt.frozen_time_beijing`
  - prompt 重构为 bundle 分层，summary 已移出 static system prompt
  - Web SSE 与前端会话流升级为 `run_started / assistant_delta / tool_call / run_error / run_finished`
  - `agent.provider` 已 breaking 更名为 `agent.runner`
- 已完成第二阶段落地：
  - 新增 `crates/hone-channels/src/runners/` 目录，按 runner/type/common 拆分实现，移除 giant file `runners.rs`
  - `AgentSession` 不再直接分叉 `gemini_cli` / blocking 路径，只保留会话编排、持久化和监听事件映射
  - `HoneBotCore::create_runner()` 已接管 runner 选择；`gemini_cli`、`codex_cli`、`function_calling` 都通过统一入口执行
  - `opencode_acp` 已接入真实 stdio/JSON-RPC runner；ACP session id 会回写到 Hone session metadata
  - `opencode_acp` 已支持通过 `agent.opencode.model` / `agent.opencode.variant` 显式固定 ACP 会话模型；当前默认示例为 `openrouter/openai/gpt-5.4` + `medium`
  - `codex_acp` 已接入真实 `codex-acp` stdio/JSON-RPC runner；启动前会校验本机 `codex` / `codex-acp` 版本，不满足已验证矩阵时直接 fail-fast 并提示安装命令
  - `codex_acp` 已支持通过 `agent.codex_acp` 显式透传 sandbox / approval / sandbox_permissions / extra `-c` overrides；若启用 `dangerously_bypass_approvals_and_sandbox`，会向 Codex 透传 `danger-full-access` + `never`，并在启动日志中打印
  - `gemini_acp` 已接入 `gemini --experimental-acp` runner；启动前会校验 `gemini >= 0.30.0`，并优先复用本机 `gemini-cli` 登录态；若配置了 `GEMINI_API_KEY` 等环境变量，则优先使用显式 API key
  - `gemini_acp` / `codex_acp` / `opencode_acp` 已开始通过本地 `hone-mcp` stdio server 暴露 Hone 内置工具；`session/new` / `session/load` 现在会附带 `mcpServers`
  - 已新增 `bins/hone-mcp` 与 `crates/hone-channels/src/mcp_bridge.rs`；`AgentRunnerRequest` 会携带 actor / channel_target / allow_cron / config_path，并通过环境变量传给 MCP 子进程重建 `ToolRegistry`
  - 已修复 `skill_tool` 的数组参数 schema，避免 ACP runner 把 Hone tools 暴露给 provider 后因非法 JSON Schema 触发 400
  - 已新增手工回归 `tests/regression/manual/test_opencode_acp_hone_mcp.sh`，可直接验证 `opencode acp` + Hone MCP bridge 的 `initialize/session/new/session/prompt` 全链路
  - 已新增 `docs/runbooks/opencode-setup.md`，沉淀新电脑安装 `opencode`、连接 OpenRouter、设置默认模型/variant，以及接入 Hone 的 SOP；`AGENTS.md` 已显式要求相关任务优先参考该 runbook
  - 已新增 `scripts/migrate_sessions.py`，支持对历史 session 做 `dry-run / --write / --validate-only` 迁移；CI-safe 回归覆盖在 `tests/regression/ci/test_session_migration.sh`
- 已完成第三阶段收敛（隔离硬化）：
  - 新增 `crates/hone-channels/src/sandbox.rs`；渠道 actor 的默认 sandbox 根目录位于 repo 外，默认落到系统临时目录下的 `hone-agent-sandboxes/`，可通过 `HONE_AGENT_SANDBOX_DIR` 覆盖
  - `AgentSession` 不再把 runner `cwd` 指向仓库根目录；渠道 actor 与 KB analysis 都改为使用独立 sandbox 目录
  - 附件 ingest 改为写入 actor sandbox 下的 `uploads/<session_id>/`，避免底层 runner 再经由 repo 内 `data/uploads` 看到项目目录层级
  - `codex_acp` 默认强制 `workspace-write + never`，并忽略危险 bypass；`gemini_cli` 继续默认 `--sandbox --approval-mode plan`
  - `gemini_acp` 不再强制 `--sandbox`；本机实测 `gemini --experimental-acp --sandbox --approval-mode plan` 会在 `initialize` 前直接退出，仅保留 `approval-mode=plan`
  - `opencode_acp` 会为每个 actor sandbox 生成最小权限 `opencode.jsonc`，禁用 `edit/bash/webfetch/websearch/skill` 并拒绝 `external_directory`
  - ACP `session/prompt` 现在显式拼接 Hone 的 system prompt，不再依赖 repo `cwd` 下被底层 CLI 自动发现的 `AGENTS.md` / `GEMINI.md`
  - 新增 `tests/regression/manual/test_actor_sandbox_isolation.sh`；实测结论为：`gemini_cli` 与最小权限 `opencode` 能拦住 repo 外文件，而 `codex workspace-write` 仍可直接读取 repo 文件，因此 `codex_cli` / `codex_acp` 已在严格 actor sandbox 路径上被 guard 拒绝
  - 新增 `tests/regression/manual/test_gemini_acp_initialize.sh`，覆盖 `gemini_acp` 的 `initialize/session/new`，防止 `--sandbox` 回归导致渠道侧再次出现 `initialize timeout`
  - 新增 `tests/regression/manual/test_codex_acp_initialize.sh`，覆盖 `codex_acp` 的 `initialize/session/new` 和 Hone MCP bridge，便于定位 `codex acp session/new timeout`

## 阻塞

- `opencode_acp` 当前只接入了最小 prompt-turn 与 `session/update` 映射；ACP permission/file-system/terminal 能力尚未完整接入 Hone 侧策略层。
- `gemini_acp` 目前已放宽为可复用本机 `gemini-cli` 登录态；仍需补一轮真实 `session/prompt` 手工回归，确认本机登录态在 Hone 托管子进程下稳定可用。
- `opencode_acp` 当前的最小权限配置基于已确认的常用权限键；若后续升级 opencode 并新增默认工具，需要再补一轮权限矩阵回归。
- `hone-cli` 仍固定读取项目根 `config.yaml`，无法仅通过 `HONE_CONFIG_PATH` 临时切换 runner；排查 ACP runner 时不要误以为它会自动复用 runtime 配置入口。

## 风险

- 该任务为全栈 breaking 重构，影响配置、会话存储、SSE 与各渠道运行时。
- 若中途发现渠道入口存在未覆盖的旧事件分支，需要优先统一事件适配，避免运行时分叉继续扩大。
- 旧 session / cron / skill 数据不再由运行时兼容读取，升级环境前需要先执行迁移脚本完成批量重写。
- 渠道 runner 现在默认运行在 repo 外 sandbox 中；任何依赖“直接看到仓库源码”的旧提示词或人工 SOP 都需要同步调整。
