# Proposal: Agent Permission Broker for Runtime Actions

status: proposed
priority: P1
created_at: 2026-05-19 02:04:23 +0800
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
- `docs/proposal/auto_p1_skill-trust-marketplace.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `config.example.yaml`
- `crates/hone-tools/src/guard.rs`
- `crates/hone-tools/src/registry.rs`
- `crates/hone-tools/src/skill_runtime.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `crates/hone-channels/src/sandbox.rs`
- `crates/hone-channels/src/runners/codex_acp.rs`
- `crates/hone-channels/src/runners/opencode_acp.rs`
- `crates/hone-core/src/config/server.rs`
- `crates/hone-core/src/config/agent.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/skills.tsx`
- `packages/app/src/components/skill-detail.tsx`

## 背景与现状

Honeclaw 的 agent runtime 已经从单一路径演进成多 runner、多工具、多技能、多渠道的执行系统：

- `crates/hone-channels/src/execution.rs` 是当前统一执行准备层，负责 prompt audit、tool registry、runner selection、actor sandbox 和 `AgentRunnerRequest` 组装。
- `crates/hone-channels/src/sandbox.rs` 会为每个 `ActorIdentity` 创建 repo-external actor sandbox，并清理遗留 portfolio 文件，避免把用户持仓误放进 runner 工作目录。
- `crates/hone-channels/src/mcp_bridge.rs` 把 Hone tools 暴露给 ACP runner，并通过 `HONE_MCP_ALLOW_CRON`、`HONE_MCP_ALLOWED_TOOLS` 和 `HONE_MCP_MAX_TOOL_CALLS` 做 stage 级裁剪。
- `crates/hone-tools/src/guard.rs` 已有 `ToolExecutionGuard`，可以按 tool name 和参数 deny pattern 在 block / audit 模式下拦截危险工具调用。
- `crates/hone-tools/src/skill_runtime.rs` 已解析 `allowed-tools`、`script`、`shell`、`paths` 等 skill frontmatter；`skill_tool.rs` 支持显式 `execute_script=true`，并校验生成图片 artifact 的路径和类型。
- `crates/hone-channels/src/runners/opencode_acp.rs` 会生成一个窄权限的临时 `OPENCODE_CONFIG`：读、列目录、glob、grep 默认 allow，edit/bash/webfetch/websearch/skill 和外部目录默认 deny；当 opencode 发出 `session/request_permission` 时，当前实现会选择一次性拒绝并记录 progress。
- `crates/hone-channels/src/runners/codex_acp.rs` 支持 `sandbox_mode`、`approval_policy`、`sandbox_permissions` 和 `dangerously_bypass_approvals_and_sandbox`，但 locked-down 情况下会强制收敛到 `workspace-write` 与 `never`。
- `config.example.yaml` 已暴露 runner sandbox、approval、multi-agent answer `max_tool_calls` 和 `security.tool_guard` 等配置，但这些仍是分散配置，不是统一的产品权限模型。

这些基础说明 Hone 已经有多层安全控制：actor sandbox、MCP allowlist、tool guard、skill stage constraints、runner 原生权限和局部拒绝逻辑。但它们目前分散在不同模块里，缺少一个可解释、可配置、可审计、可向用户确认的 **运行时权限 broker**。

当前安全姿态偏保守，尤其 opencode ACP 默认拒绝外部权限请求。这对早期稳定性是合理的；但当 Hone 要支持更复杂的桌面工作台、用户文件、skill script、投资文档处理、本地图表、团队 skill pack 和协作式 agent 时，单纯“拒绝或放行”会限制产品能力，也会让用户不知道某次 agent 为什么不能完成任务。

## 问题或机会

这是 P1 级问题。它不一定像 secret 泄露或投资输出错误那样立刻触发 P0 风险，但会显著影响核心体验、安全可解释性、桌面能力、技能生态和后续商业部署。

主要缺口：

1. **权限控制分散，用户无法理解失败原因。**
   MCP allowlist、skill stage constraints、ToolGuard、opencode permission、codex sandbox、script execution gate 分别生效。一个任务失败时，用户只看到工具不可用、permission rejected 或 runner error，很难知道应该配置 runner、启用 skill、批准脚本、切换桌面本地模式，还是请求管理员。

2. **没有统一的 permission request 对象。**
   opencode 的 `session/request_permission` 目前只被立即拒绝；Hone tool 调用只有 guard block；skill script 只有 `execute_script=true`；Codex sandbox 权限是启动参数。系统没有一个标准对象表达“agent 想做什么动作、访问什么资源、由谁批准、批准多久、是否可撤销”。

3. **技能权限声明和运行时权限没有真正闭环。**
   Skill frontmatter 可以声明 tools、script、shell、paths，但运行时只做可见性和部分校验。`Skill Trust Marketplace` 可以解决安装前审查；真正执行时仍需要 session-level broker 判断当前 actor、channel、runner、surface、文件路径和风险级别是否允许。

4. **桌面端与远程端需要不同权限心智。**
   Desktop bundled 模式天然适合用户批准读取本地文件、运行本机脚本或写入 actor sandbox；remote backend 模式则必须更保守，因为文件会上传或命令会在远端执行。当前配置无法把这种产品差异清楚地展示给用户。

5. **多渠道弱交互缺少安全降级。**
   Feishu / Telegram / Discord / iMessage 不适合弹复杂权限窗口。高风险请求应转成短确认码、只读降级、Web/desktop 待审批队列或明确拒绝，而不是每个 channel 自己处理。

6. **审计缺少“被拒绝的能力需求”。**
   Run trace 和 mutation ledger 关注已发生或已提出的运行与状态变更；但被权限阻断的动作同样有产品价值。它能告诉管理员“用户频繁想导入文件但 remote mode 不支持本地读”“某 skill 经常要求 bash”“某模型频繁请求外部目录”，这些是改进能力和安全策略的输入。

机会是新增一个轻量但统一的 **Agent Permission Broker**：把 runner 原生权限请求、Hone MCP tool call、skill script/shell/path gate、本地文件访问和高风险 tool 参数统一映射为可裁决的 permission request。第一版不需要放开危险能力，只要把当前分散的拒绝和 allowlist 变成结构化、可解释、可审计、可逐步批准的产品层。

## 方案概述

新增 `AgentPermissionBroker`，作为每次 agent execution 的运行时权限裁决层。它不替代 sandbox、ToolGuard 或 skill registry，而是把这些控制点统一到一个可观测接口：

- `PermissionRequest`：一次运行时权限请求，例如 read_file、write_file、run_shell、execute_skill_script、call_tool、network_fetch、external_directory_access、cron_mutation、secret_read。
- `PermissionDecision`：allow、deny、ask_user、ask_admin、defer_to_surface、allow_once、allow_for_session、allow_for_skill_version。
- `PermissionPolicy`：按 actor、surface、channel、runner、skill、tool、path、risk 和 deployment mode 生效的策略。
- `PermissionGrant`：用户或管理员批准后的短期授权，带 scope、expires_at、source、revoked_at 和 audit id。
- `PermissionEvent`：所有 allow / deny / ask / timeout / revoke 都写入审计，供 run trace、support bundle 和 readiness matrix 使用。

第一版目标：

1. 不默认扩大权限。现有高风险请求仍默认 deny。
2. 把拒绝原因结构化，并对用户展示可操作下一步。
3. 将 opencode `session/request_permission`、MCP tool allowlist miss、ToolGuard block、skill script request 统一记录成 `PermissionEvent`。
4. 在 Web/desktop 增加只读权限事件视图和少量低风险批准能力。
5. 为后续“允许读取用户选择的文件”“允许执行某个官方 skill script”“允许本轮调用 cron_job”留下标准接口。

## 用户体验变化

### 用户端

- 当 agent 无法完成任务时，回答不再只说“工具不可用”或“权限被拒绝”，而是解释：
  - 请求类型：例如“读取 sandbox 外文件”“执行 skill 脚本”“调用 cron_job 修改任务”。
  - 当前 surface：public web、desktop bundled、remote backend、IM。
  - 默认策略：例如“public web 不允许本地文件读取”“群聊不允许个人状态变更”“该 skill script 尚未批准”。
  - 下一步：进入 desktop 批准、切到私聊、让管理员启用权限、或上传文件而不是让 agent 读取本地路径。
- Public Web 默认只允许上传后的文件、只读工具和低风险 skill；高风险权限显示为不可用，不向普通用户暴露复杂策略。
- 对低风险、用户明确发起的动作，可以出现短确认：
  - “允许 Hone 在本轮读取你刚上传的 3 个文件？”
  - “允许执行官方 chart_visualization 脚本生成 PNG？”
  - “允许在当前 actor sandbox 写入公司画像事件？”

### 管理端

- 新增 `Permissions` 或接入 `Run Trace Workbench`：
  - 最近 permission requests、decisions、blocked reason、runner、skill、tool、actor、surface。
  - 高频 deny 分析：哪个 skill、tool、runner 或 channel 最常请求被拒权限。
  - policy preview：某 actor / channel / runner 当前能做哪些动作。
- `/skills` 的详情页除安装权限外，展示运行期权限事件：
  - 这个 skill 最近请求过哪些 tools / scripts / paths。
  - 是否因为 stage 或 broker policy 被拒绝。
  - 是否有待管理员批准的 grant。
- Settings 中把 `security.tool_guard`、Codex sandbox、OpenCode permission 和 MCP tool limits 归入一个“Agent permissions”说明区，避免用户以为这些是互不相关的开关。

### 桌面端

- Desktop bundled 模式可以提供更自然的权限确认 UI：
  - 读取用户通过文件选择器指定的文件。
  - 执行经过 skill package 审查的本地脚本。
  - 写入 actor sandbox 内的公司画像、图表和研究产物。
- Desktop remote backend 必须明确展示“该权限会在远端服务执行”，默认不允许本地路径读写。
- channel sidecar 状态页可以显示“某 channel 最近被权限策略阻断 N 次”，帮助用户区分配置错误和安全拒绝。

### 多渠道

- IM 私聊中使用简短语义：
  - `需要批准：执行图表脚本。本轮输入：确认 4821 / 取消`
  - `已拒绝：群聊不能写入个人 portfolio，请私聊继续。`
- 群聊默认不允许产生个人权限 grant；只允许触发者本人在私聊或 Web/desktop 确认。
- 弱交互渠道无法展示复杂 policy 时，permission broker 返回 `defer_to_surface`，回复引导用户打开 Web/desktop。

## 技术方案

### 1. 新增 permission 类型与服务

建议在 `hone-core` 定义稳定类型，在 `memory` 或 `hone-web-api` 接入存储：

```rust
pub enum PermissionAction {
    ReadFile,
    WriteFile,
    RunShell,
    ExecuteSkillScript,
    CallTool,
    NetworkFetch,
    ExternalDirectoryAccess,
    ReadSecret,
    MutateRuntimeState,
}

pub struct PermissionRequest {
    pub request_id: String,
    pub actor: ActorIdentity,
    pub session_id: String,
    pub channel: String,
    pub surface: PermissionSurface,
    pub runner: String,
    pub skill_id: Option<String>,
    pub tool_name: Option<String>,
    pub action: PermissionAction,
    pub resource: PermissionResource,
    pub risk: PermissionRisk,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}
```

存储第一版可用 SQLite：

```text
permission_events (
  id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  session_id TEXT,
  runner TEXT,
  surface TEXT NOT NULL,
  action TEXT NOT NULL,
  resource_json TEXT NOT NULL,
  risk TEXT NOT NULL,
  decision TEXT NOT NULL,
  reason TEXT,
  grant_id TEXT,
  created_at TEXT NOT NULL
)

permission_grants (
  grant_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  scope_json TEXT NOT NULL,
  granted_by TEXT NOT NULL,
  granted_from_surface TEXT NOT NULL,
  expires_at TEXT,
  revoked_at TEXT,
  created_at TEXT NOT NULL
)
```

### 2. Broker 插入点

优先接入已有集中点，避免每个 channel 重写：

- `ExecutionService::prepare()` 创建 `PermissionContext`，包含 actor、session、runner、surface、allowed_tools、allow_cron、working_directory。
- `mcp_bridge.rs` 在 tool list 和 tool call 被 allowlist 拒绝时记录 `PermissionEvent`；调用敏感工具前先问 broker。
- `ToolRegistry::execute_tool()` 在 `ToolExecutionGuard` 前后接入 broker：先生成 action/resource/risk，再执行现有 deny pattern；blocked 结果写 event。
- `skill_tool.rs` 在 `execute_script=true`、shell/script/path/artifact 相关路径接入 broker；官方内置低风险 script 可先走 allowlist，custom skill 默认 ask_admin 或 deny。
- `opencode_acp.rs` 的 `handle_opencode_permission_request()` 不再只硬编码 reject，而是把 params 转成 `PermissionRequest`，由 broker 返回 `deny` 或 `ask_user`。第一版仍默认 deny，但记录结构化原因。
- `codex_acp.rs` 保留启动参数约束，同时把 effective args 和 locked-down 状态写入 execution permission context，便于 trace 解释。

### 3. Policy 分层

建议策略从硬到软分层：

1. **Invariant hard deny**
   - 访问 repo checkout 外的敏感系统路径。
   - actor sandbox escape。
   - 读取 raw secret。
   - public web 直接执行 shell。
   - 群聊写个人私有状态。

2. **Surface default policy**
   - public web：上传文件只读、低风险 tools、无 shell。
   - admin web：可批准配置和 skill 权限，但敏感操作仍走 operator audit。
   - desktop bundled：可通过用户确认授予本机文件和官方 script。
   - desktop remote：本地路径权限默认不可用。
   - IM direct：短确认仅限低/中风险。
   - IM group：默认只读和 defer。

3. **Skill/package policy**
   - system skill 可按内置 manifest 配默认 grant。
   - custom skill 默认 disabled 或 ask_admin。
   - skill package 升级后 checksum 变化会使旧 grant 失效。

4. **Session grant**
   - allow once / allow for current session / allow for current skill version。
   - grant 必须绑定 actor、surface、resource scope 和过期时间。

### 4. API 与前端

新增只读优先 API：

- `GET /api/permissions/events?actor=&session_id=&decision=&action=`
- `GET /api/permissions/policy-preview?channel=&user_id=&surface=&runner=`
- `GET /api/permissions/grants?actor=`
- `POST /api/permissions/requests/{id}/approve`
- `POST /api/permissions/requests/{id}/deny`
- `POST /api/permissions/grants/{id}/revoke`

第一版可以只开放 admin approve；public 用户只允许处理自己当前 session 的低风险 request。

前端落点：

- `packages/app/src/pages/settings.tsx`：Agent permissions 区块。
- `packages/app/src/pages/skills.tsx` 和 `components/skill-detail.tsx`：runtime permission history。
- 未来 `run_trace` 页面：把 permission event 放入同一时间线。
- Public/desktop chat：渲染 pending permission card；没有卡片能力时由 final response 提供确认码。

### 5. 与已有系统协作

- 与 `Agent Mutation Ledger`：permission broker 管“能不能执行这个动作”；mutation ledger 管“执行后用户状态怎么变、能否撤销”。两者通过 `permission_event_id` / `mutation_record_id` 关联。
- 与 `Skill Trust Marketplace`：marketplace 管“安装/启用某 skill 是否可信”；permission broker 管“当前 session 里这个 skill 的具体动作是否允许”。
- 与 `Operator Access Audit`：operator audit 管管理员身份和高危 admin 操作；permission broker 的 admin approvals 应写入 operator id。
- 与 `Run Trace Workbench`：run trace 展示一个 run 内 permission request 与 decision；broker 存结构化事件供 trace 读取。
- 与 `Runtime Readiness Matrix`：readiness 可以读取 policy preview，解释某 runner/surface 为什么不能执行某类能力。

## 实施步骤

### Phase 1: 结构化拒绝与观测

- 定义 `PermissionRequest`、`PermissionDecision`、`PermissionEvent` 类型。
- 新增 SQLite 存储或先复用 runtime audit DB 建表。
- 接入 `opencode_acp` permission request：仍默认 deny，但记录 action、resource、runner、session、tool title。
- 接入 MCP allowlist miss、ToolGuard block、skill script request 的 event 记录。
- 在 admin 增加只读 permission event 列表。

### Phase 2: Policy preview 和低风险 grant

- 实现 `PermissionPolicyEngine`，覆盖 surface defaults 和 hard deny。
- 给 official low-risk skill scripts 建白名单，例如 chart PNG rendering 只写 `gen_images` / skill artifact allowed roots。
- Public/desktop chat 支持 pending permission card；IM 支持确认码。
- 增加 session-scoped grant，默认短 TTL。

### Phase 3: Runner permission integration

- `opencode_acp` 根据 broker decision 回复 `allow_once` 或 reject。
- `codex_acp` 将 sandbox/approval effective config 写入 trace，并在需要时对 dangerous config 提示 readiness warning。
- MCP bridge 对 high-risk tools 统一 ask/deny，不依赖各 tool 自己返回中文错误。
- Multi-agent answer stage 复用相同 broker，并把 search/answer 两阶段 permission events 分开标记。

### Phase 4: 管理和审计闭环

- Settings 增加 policy preview、grant revoke 和 blocked reason 聚合。
- Skill detail 展示运行期权限事件和 grants。
- Run Trace Workbench 读取 permission timeline。
- Redacted Support Bundle 只导出脱敏 permission summary，不导出原始路径和 secret。

## 验证方式

- 单元测试：
  - `PermissionPolicyEngine` 对 public/admin/desktop/IM surface 的默认决策。
  - actor sandbox 内外路径判断。
  - custom skill script 默认拒绝，official allowlist 按 checksum / path 生效。
  - session grant 过期、撤销和 actor mismatch。
- Rust 集成测试：
  - opencode `session/request_permission` payload 转成 `PermissionRequest` 并默认 deny。
  - MCP `HONE_MCP_ALLOWED_TOOLS` 拒绝 `cron_job` 时写入 permission event。
  - `ToolExecutionGuard` block 同时返回原错误并落 audit event。
- 前端测试：
  - permission event 列表渲染 blocked reason。
  - pending permission card 的 approve/deny 状态。
  - skill detail 能展示运行期 permission history。
- 手工验收：
  - Desktop bundled 中执行官方 chart skill，确认允许后生成图表。
  - Public Web 试图读取本地绝对路径，必须解释不可用并建议上传文件。
  - IM 群聊请求写 portfolio，必须拒绝或引导私聊。
  - Remote desktop backend 请求本地路径，必须展示远端执行边界。
- 指标：
  - permission denied count by action/tool/runner/surface。
  - ask_user -> approved / denied / timeout 转化率。
  - 因权限导致的 failed run 占比。
  - 高风险 grant 数量和平均有效期。

## 风险与取舍

- **复杂度增加。**
  权限 broker 会横跨 runner、tool、skill、Web 和 desktop。第一版必须只做结构化拒绝和观测，避免一开始就实现完整权限 UI。

- **过度打扰用户。**
  如果每个小动作都弹确认，agent 体验会变差。策略应优先通过 surface defaults 和 official allowlist 降低提示频率，只对真实风险 ask。

- **误授权风险。**
  Grant 必须短期、窄 scope、可撤销，并绑定 actor / session / skill version / resource。不要引入全局“永远允许 bash”。

- **与现有提案边界容易混淆。**
  本提案不负责 skill 安装信任、不负责状态变更撤销、不负责 operator 身份体系。它只裁决运行时动作是否可执行，并产生结构化事件。

- **不同 runner 的能力不对称。**
  OpenCode 有 `session/request_permission`；Codex 主要靠启动 sandbox/approval config；function-calling 走 Hone tool registry。Broker 需要抽象出统一事件，但不能假装每个 runner 都支持同样的实时批准能力。

- **不要扩大默认权限。**
  当前 opencode 默认拒绝 edit/bash/webfetch/websearch/external_directory 是合理安全基线。第一阶段只把拒绝变清楚，允许能力必须逐项落地。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 和 `docs/proposals/`。相邻但不重复的主题如下：

- `auto_p1_skill-trust-marketplace.md` 关注 skill 包的发现、安装、兼容性、权限声明审批和回滚；本提案关注 skill 或 runner 在某一轮执行中发出的具体动作请求如何被裁决和审计。
- `auto_p1_agent-mutation-ledger.md` 关注 portfolio、cron、notification prefs、company profile 等用户状态变更的 before/after、确认和撤销；本提案关注动作执行前的 read/write/shell/tool/network 权限，不负责状态 diff 和 revert。
- `auto_p0_operator-access-audit.md` 关注 Web 管理端 operator 身份、角色和高危 admin 操作审计；本提案的 admin approval 会引用 operator，但不是身份系统。
- `auto_p1_run_trace_workbench.md` 关注一次 run 的可观测时间线；本提案提供 permission events，供 trace 展示。
- `auto_p1_runtime_readiness_matrix.md` 关注 runner、凭证、渠道和环境是否 ready；本提案的 policy preview 可作为 readiness 输入，但不是环境诊断面。
- `docs/proposals/skill-runtime-multi-agent-alignment.md` 关注 skill 与 Claude Code / multi-agent 语义对齐；本提案只补运行时权限 broker，不改变 skill prompt 注入模型。

因此，本提案的新增主题是：**把分散在 runner、MCP、tool guard、skill script 和 sandbox 中的运行时权限请求统一产品化**，让 Hone 在不扩大默认权限的前提下，具备可解释、可批准、可审计的 agent action permission 层。
