# Proposal: External MCP Workspace Gateway for Hone Research Assets

status: proposed
priority: P2
created_at: 2026-05-21 08:04:04 +0800
owner: automation
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `docs/proposal/auto_p1_hone-cloud-api-contract.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p1_skill-trust-marketplace.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `bins/hone-mcp/src/main.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `crates/hone-tools/src/registry.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-tools/src/notification_prefs_tool.rs`
- `crates/hone-tools/src/cron_job_tool.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-channels/src/runners/codex_acp.rs`
- `crates/hone-channels/src/runners/opencode_acp.rs`
- `bins/hone-cli/src/main.rs`
- `bins/hone-desktop/src/sidecar.rs`
- `packages/app/src/pages/settings.tsx`

## 背景与现状

Hone 已经内置了一个可运行的 MCP 服务器：

- `bins/hone-mcp/src/main.rs` 只是薄入口，启动 `hone_channels::mcp_bridge::run_hone_mcp_stdio()`。
- `crates/hone-channels/src/mcp_bridge.rs` 能把 `ToolRegistry` 暴露为 MCP `tools/list` 和 `tools/call`，并从环境变量读取 `HONE_CONFIG_PATH`、`HONE_MCP_ACTOR_CHANNEL`、`HONE_MCP_ACTOR_USER_ID`、`HONE_MCP_ACTOR_SCOPE`、`HONE_MCP_SESSION_ID`、`HONE_MCP_ALLOWED_TOOLS`、`HONE_MCP_MAX_TOOL_CALLS`、`HONE_DATA_DIR`、`HONE_SKILLS_DIR` 和 `HONE_AGENT_SANDBOX_DIR`。
- `hone_mcp_servers()` 已经在 ACP runner request 中自动生成本地 `hone-mcp` server 配置，让 Codex/OpenCode ACP runner 在 Hone 管控下调用 `discover_skills`、`skill_tool`、`cron_job` 等工具。
- `mcp_bridge.rs` 已有最小安全边界：按 stage 限制 allowed tools、限制单 session tool call 次数、按 actor/channel target 创建 registry、记录脱敏工具调用日志。
- `ToolRegistry` 是统一工具注册和执行入口，已被 function-calling 和 MCP bridge 复用；工具执行前还可经过 `ToolExecutionGuard`。
- Release/desktop 打包已经把 `hone-mcp` 作为组件之一，`docs/repo-map.md` 也把它列为本地 stdio MCP server。

这些基础说明 Hone 已经具备“把投资工作台能力暴露给外部 agent 协议”的技术内核。但当前产品语义仍是内部桥：`hone-mcp` 主要服务 ACP runner，不是用户可理解、可配置、可授权、可审计的外部接入面。

与此同时，Hone 的资产越来越像一个个人投资研究 workspace：portfolio、watchlist、company portraits、scheduled tasks、notification prefs、event-engine 结果、research artifacts、session memory、skills 和本地 actor sandbox。用户并不总是只在 Hone Web/desktop 中工作；他们可能同时使用 Claude Desktop、Codex、OpenCode、其它 MCP 客户端或 IDE agent。如果 Hone 只提供聊天 API，外部 agent 能“问 Hone 一个问题”，但不能安全复用 Hone 的结构化投资资产和工具能力。

## 问题或机会

这是 P2：它不是当前核心可用性的阻塞，也不应抢在权限 broker、mutation ledger、readiness 等底座前强行开放写能力；但它有明确的生态、增长和高级用户价值。

1. **已有 MCP 能力不可发现。**  
   普通用户不知道 `hone-mcp` 可以怎么配置到外部 MCP 客户端，也不知道需要哪些环境变量、actor scope 和工具 allowlist。现在它更像 runner 内部实现细节，而不是产品能力。

2. **外部 agent 接入缺少最小安全模式。**  
   当前 `hone-mcp` 如果被手工启动且未设置 actor 或 allowlist，工具表和数据归属不够产品化。外部客户端需要明确的 profile：只读、研究、自动化草稿、管理员诊断，而不是直接复用内部 runner stage 环境变量。

3. **Hone Cloud API 与 MCP 解决的是不同问题。**  
   `auto_p1_hone-cloud-api-contract.md` 聚焦 OpenAI-compatible chat endpoint、developer console、API key、stream/error/usage contract。MCP gateway 应聚焦“工具和资产可被外部 agent 调用”：列 portfolio、读取画像、查询通知偏好、生成任务草稿、读取 schedule forecast 等。这不是同一条 API 面。

4. **桌面与本地部署的差异化没有充分利用。**  
   Hone 桌面 bundled 模式天然能成为本机投资研究 hub。外部 MCP 客户端通过本机 stdio 访问 Hone，比把用户资产上传给第三方 SaaS 更符合本地优先体验。但当前桌面设置页没有“一键生成 MCP 配置 / 权限预览 / 断开访问”的入口。

5. **后续 skill 和 playbook 生态缺少标准外部入口。**  
   `skill-trust-marketplace`、`investment_playbook_launcher`、`company_portrait` 等能力若只在 Hone 内部 UI 可用，生态扩散有限。MCP gateway 可以让外部 agent 用同一批经过 Hone 权限和 actor 隔离包装的工具，而不是复制一套插件。

6. **没有稳定接入契约会诱发危险手工配置。**  
   高级用户可能自行把 `hone-mcp` 加进任意 MCP 客户端。如果没有官方 profile、工具分级、actor 绑定、审计和文档，容易出现跨 actor 读写、误建 cron、泄漏本地路径或让外部 agent 直接修改长期资产的问题。

## 方案概述

新增 **External MCP Workspace Gateway**：把现有 `hone-mcp` 内部桥包装成一个面向用户和外部 agent 客户端的受控接入产品层。

第一版目标是保守的：

- 只正式支持本地 stdio MCP，不先做公网 MCP server。
- 默认只读 profile，不默认开放写 portfolio、写画像、建 cron 或执行 skill script。
- 所有 profile 都绑定明确 `ActorIdentity`，不能以 anonymous actor 访问用户资产。
- 对外暴露的工具表按 `McpAccessProfile` 生成，不让用户手写底层 `HONE_MCP_ALLOWED_TOOLS`。
- Web/desktop/CLI 能生成 Claude Desktop、Codex/OpenCode、其它 MCP 客户端可粘贴的配置片段。
- 工具调用记录进入现有日志，后续可接入 `agent-permission-broker`、`agent-mutation-ledger` 和 operator audit。

核心对象：

- `McpAccessProfile`
  - `readonly_research`: 读 portfolio、company profile 摘要、schedule overview、notification prefs 摘要、skill listing。
  - `research_assistant`: 在只读基础上允许生成 chat draft / review prompt / evidence context，但不直接写长期资产。
  - `automation_draft`: 允许创建 cron draft 或 playbook preview，但 apply 需要 Hone UI 确认。
  - `admin_diagnostics`: 管理员本机诊断 profile，可读 readiness/log summary，但默认不读用户敏感全文。

- `McpClientRegistration`
  - 本机客户端名称、profile、actor、created_at、last_used_at、allowed_tools、max_tool_calls、expires_at、enabled。

- `McpToolSurface`
  - 面向外部 client 的稳定工具名与 schema。内部工具可继续存在，但外部工具应避免暴露低层实现细节和中文错误字符串。

- `McpGatewayAudit`
  - 记录 client、actor、tool、result、duration、redacted args/result excerpt、permission decision 和 request id。

## 用户体验变化

### 用户端

- Public Web 不直接开放 MCP 配置给普通试用用户；如果后续提供 hosted MCP，也必须先完成 API key、policy consent、entitlement 和权限边界。本提案 v1 只面向本地/桌面和管理员。
- 本地用户可以在 desktop/settings 中看到 “Connect Hone to external AI apps”：
  - 选择当前 workspace / actor。
  - 选择只读或草稿 profile。
  - 复制 Claude Desktop / Codex / OpenCode MCP 配置。
  - 查看已连接客户端和最近工具调用。
- 外部 agent 可以回答：
  - “列出我当前关注的公司画像健康摘要。”
  - “读取我的 portfolio 只读摘要，帮我准备一个周末复盘 prompt。”
  - “根据 Hone 的通知偏好解释今天会收到哪些摘要。”
  但不能默认直接修改 portfolio、删除画像或创建定时任务。

### 管理端

- Settings 增加 MCP Gateway 区块：
  - 显示 `hone-mcp` binary 是否可用、版本、当前 backend data dir、profile 列表。
  - 为指定 actor 生成一次性配置片段。
  - 禁用某个本地 client registration。
  - 查看最近工具调用和拒绝原因。
- 用户详情页可以生成 actor-scoped readonly profile，便于 support 在本机外部 agent 中做只读分析；高风险 profile 需要后续 operator access/audit 能力支撑。

### 桌面端

- Bundled mode 是第一优先级：
  - 桌面知道 bundled binary 路径、runtime data dir、canonical config。
  - 可以生成无需用户手写 env 的 MCP 配置。
  - 可以在 UI 中 revoke 或 rotate registration token。
- Remote mode 只展示远端是否支持 MCP gateway；默认不把远端用户资产通过本机 `hone-mcp` 暴露，避免用户误以为本地 client 能访问远端 backend 数据。

### 多渠道

- IM 通道不直接暴露 MCP gateway。
- 如果用户在 Feishu/Telegram/Discord 中问“能不能把 Hone 接到 Claude/Codex”，agent 可以解释需要在 desktop/admin settings 中生成本地 MCP 配置，而不是在聊天里输出敏感环境变量。
- 群聊上下文不允许生成个人 actor 的 MCP profile。

## 技术方案

### 1. 明确 external vs internal MCP mode

当前 `hone-mcp` 只从环境变量恢复上下文，适合 ACP runner 内部临时 server。建议保留该路径，并新增外部模式：

```text
hone-mcp serve --profile <profile_id>
hone-cli mcp config --actor <actor> --profile readonly_research --client claude
hone-cli mcp list
hone-cli mcp revoke <registration_id>
```

内部 ACP mode 继续使用现有 `HONE_MCP_*` 环境变量，不受本提案第一阶段改动影响。

外部 mode 从本地 registration store 读取：

- actor
- access profile
- allowed tools
- max calls per session / per day
- optional expiry
- redaction policy
- client label

第一版 store 可用本地 SQLite 或 `data/runtime/mcp_gateway/registrations.json`。由于这是可撤销授权，长期更适合 SQLite。

### 2. Access profile 到工具表映射

不要让 UI 直接暴露 `HONE_MCP_ALLOWED_TOOLS`。定义稳定 profile：

```rust
enum McpAccessProfileKind {
    ReadonlyResearch,
    ResearchAssistant,
    AutomationDraft,
    AdminDiagnostics,
}
```

示例工具分层：

- `readonly_research`
  - `portfolio(view only)`
  - `notification_prefs(get only)`
  - `schedule_view`
  - `discover_skills`
  - future `company_profile_summary`
  - future `operations_calendar_forecast`

- `research_assistant`
  - readonly 工具
  - `skill_tool` 仅允许不执行脚本的 prompt expansion / read-only workflow context
  - future `evidence_review(list/create_draft)`

- `automation_draft`
  - research assistant 工具
  - future `cron_job(preview only)` 或 `automation_intent(create draft)`

- `admin_diagnostics`
  - readiness summary
  - redacted support summary
  - logs summary
  - 不默认读取 full prompt audit、raw session transcript 或 secrets。

如果现有工具还没有 read-only action，需要先加外部 facade 工具，而不是把完整 mutable tool 暴露出去。例如不要把完整 `portfolio_tool` 原样给 `readonly_research`，而是提供 `portfolio_summary` 或在 tool 内强制 `action=view`。

### 3. Actor binding and sandbox boundary

每个 registration 必须绑定明确 actor：

- `channel`
- `user_id`
- `channel_scope`
- display label

启动外部 MCP 时如果无法解析 actor，直接拒绝 `tools/list` 或只返回一个 `gateway_status` 工具说明缺少绑定。

actor sandbox 规则仍由 `crates/hone-channels/src/sandbox.rs` 和相关工具执行：

- 不把 repo root 当作外部 MCP workspace。
- 不暴露 sandbox 外绝对路径。
- 不允许外部 MCP profile 绕过 public/admin 权限读取其它 actor 数据。

### 4. Tool schema and result contract

外部 MCP client 比内部 runner 更依赖稳定 schema。建议新增一层 `external_mcp_tools` facade：

- 工具名稳定且英文，例如 `hone_portfolio_summary`、`hone_company_profile_list`、`hone_notification_overview`。
- 返回结构化 JSON，附 `display_text` 供普通 MCP client 展示。
- 错误使用稳定 code，例如 `actor_not_bound`、`profile_denied_tool`、`read_only_profile`、`resource_not_found`、`permission_required`。
- 所有结果默认压缩，不返回超长 Markdown 或完整 session transcript。

内部 `ToolRegistry` 可以继续保留原工具；external facade 只是把危险/复杂工具包装成更窄的 MCP surface。

### 5. CLI / Desktop / Web settings

CLI：

- `hone-cli mcp profiles`
- `hone-cli mcp register --actor web:<id> --profile readonly_research --client claude`
- `hone-cli mcp config --registration <id> --client claude`
- `hone-cli mcp revoke <id>`
- `hone-cli mcp audit --registration <id>`

Desktop/Web settings：

- 调用 backend API 创建 registration。
- 展示 profile 解释、工具列表、风险提示、过期时间。
- 复制配置片段时只包含 profile id / command / env，不包含 raw provider secrets。
- Remote mode 明确标注 unsupported 或由远端生成。

### 6. Audit and permission integration

第一版先写本地 MCP audit log，不等待所有相邻提案落地：

- `tool_list`
- `tool_call_allowed`
- `tool_call_denied`
- `tool_call_error`
- `registration_created`
- `registration_revoked`

后续衔接：

- `agent-permission-broker`：高风险 tool call 从 hard deny 升级为可确认 permission request。
- `agent-mutation-ledger`：外部 MCP 触发的 draft/apply 变更必须进入 mutation ledger。
- `operator-access-audit`：管理员为他人 actor 生成 profile 或读取 audit 时进入 operator audit。
- `usage-entitlement-ledger`：hosted/remote MCP 未来按 tool call、read volume 或 automation draft 计入权益。

## 实施步骤

### Phase 1: External read-only contract

- 定义 `McpAccessProfile`、registration store 和 profile -> allowed tools 映射。
- 新增 CLI `hone-cli mcp register/config/list/revoke` 的最小版本。
- 为 portfolio、notification prefs、schedule overview 提供 read-only external MCP facade 工具。
- `hone-mcp serve --profile` 支持从 registration store 启动。
- 增加 audit log，记录 tool list/call/deny。

### Phase 2: Desktop setup surface

- Desktop bundled settings 增加 MCP Gateway 区块。
- 自动定位 bundled `hone-mcp` 路径，生成 Claude Desktop / Codex / OpenCode 配置片段。
- 显示 registration 的 last used、allowed tools、expires_at 和 revoke 操作。
- Runtime Readiness 可消费 `hone-mcp` binary/profile 状态。

### Phase 3: Draft-only workflow tools

- 增加 company profile summary、operations calendar forecast、evidence review draft、automation intent draft 等只读/草稿工具。
- `skill_tool` 外部模式只允许 prompt expansion 和 read-only context，不执行 script。
- 对任何写长期资产的动作返回 `permission_required` 或 `draft_created`，不直接 apply。

### Phase 4: Hosted/remote exploration

- 在 Hone Cloud API contract、entitlement、policy consent、permission broker 和 audit 底座足够稳定后，再评估远端 MCP over HTTP。
- 远端模式必须使用 API key、scope、rate limit、policy consent 和 revoke。
- 不把本地 stdio MCP profile 自动迁移成远端授权。

## 验证方式

- Rust 单元测试：
  - profile -> allowed tools 映射符合只读/草稿边界。
  - external MCP 启动缺 actor/profile 时拒绝工具访问。
  - readonly profile 调用 mutable tool 返回 `read_only_profile` 或不出现在 `tools/list`。
  - redaction 覆盖 args/result 中的 token、api_key、password、secret、Bearer。

- CLI contract 测试：
  - `hone-cli mcp register` 创建 registration。
  - `hone-cli mcp config --client claude` 输出包含 `hone-mcp serve --profile` 的配置片段。
  - revoke 后 `hone-mcp serve --profile` 不能继续列出工具。

- MCP protocol 测试：
  - `initialize`、`tools/list`、`tools/call` 仍符合当前 JSON-RPC shape。
  - 内部 ACP mode 现有 `HONE_MCP_ALLOWED_TOOLS`、`HONE_MCP_MAX_TOOL_CALLS` 行为不回归。
  - 外部 mode 的工具名/schema 稳定，不暴露内部中文错误或本地绝对路径。

- 前端/桌面验证：
  - Bundled mode 能显示 `hone-mcp` binary ready。
  - Remote mode 不误生成本机访问远端数据的配置。
  - Revoke 后 UI 状态刷新。

- 手工验收：
  - 用生成配置接入 Claude Desktop 或等价 MCP client，只能读取当前 actor 的 portfolio/notification/schedule 摘要。
  - 尝试让外部 client 创建 cron 或改 portfolio，默认被拒绝或只生成 draft。

## 风险与取舍

- 风险：过早开放外部 MCP 会扩大数据泄露面。  
  取舍：v1 只支持本地 stdio、明确 actor binding、默认只读、registration 可撤销、工具 facade 窄化。

- 风险：外部 MCP client 对工具调用的展示和确认能力不一致。  
  取舍：不依赖 client 做安全确认；Hone 端 profile 和 permission 先行裁决。

- 风险：直接复用内部 mutable tools 会绕过产品确认。  
  取舍：外部 gateway 使用 facade 工具；写操作先走 draft 或直接拒绝。

- 风险：用户混淆 Hone Cloud API 与 MCP gateway。  
  取舍：文案明确：API 是聊天 endpoint，MCP 是本地工具/资产 gateway；远端 MCP 属于后续探索。

- 风险：registration store 变成又一套权限系统。  
  取舍：第一版保持小范围，只记录本地 MCP client 授权；后续与 permission broker、operator audit、entitlement 合并口径。

- 不做：不做公网 MCP server、不做第三方 OAuth、不允许默认写长期资产、不开放 secrets/raw logs/full prompt audit、不解决跨 actor workspace 合并。

## 与已有提案的差异

查重范围：

- `docs/proposal/` 全部现有自动提案。
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 不重复 `auto_p1_hone-cloud-api-contract.md`：该提案定义 OpenAI-compatible chat/API key/developer console；本提案定义 MCP 工具和 workspace asset gateway。
- 不重复 `auto_p1_agent-permission-broker.md`：permission broker 处理运行时权限裁决；本提案定义外部 MCP client 如何注册、拿到哪类工具、以什么只读/草稿 profile 接入。
- 不重复 `auto_p1_agent-mutation-ledger.md`：mutation ledger 记录和确认状态变更；本提案默认不 apply mutation，只在未来需要写操作时消费 mutation ledger。
- 不重复 `auto_p1_skill-trust-marketplace.md`：skill marketplace 处理第三方 skill 安装、审查、升级；本提案处理外部 agent client 调用 Hone 工具和资产。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：readiness 判断 runner/MCP/渠道是否可用；本提案提供一个用户可配置的外部 MCP 接入面，readiness 只会消费其状态。
- 不重复 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该历史提案关注 skill runtime 与 Claude Code / multi-agent 语义对齐，以及内部 MCP stage allowlist；本提案关注将 `hone-mcp` 产品化给外部 MCP 客户端使用。

本轮只新增 proposal，不开始执行实现，因此不更新 `docs/current-plan.md`，也无需归档计划页。若后续实际落地本提案，应新建或复用 `docs/current-plans/external-mcp-workspace-gateway.md`，并在改变 MCP/CLI/Desktop/API 行为时同步更新 `docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。
