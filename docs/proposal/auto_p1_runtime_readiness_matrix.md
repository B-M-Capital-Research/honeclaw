# Proposal: Runtime Readiness Matrix for Model Routes and Capabilities

status: proposed
priority: P1
created_at: 2026-05-08 17:03:10 +0800
owner: automation
related_files:
- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/current-plan.md`
- `config.example.yaml`
- `bins/hone-cli/src/reports.rs`
- `bins/hone-cli/src/onboard.rs`
- `bins/hone-cli/src/main.rs`
- `bins/hone-desktop/src/sidecar.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/settings-model.ts`
- `packages/app/src/lib/backend.ts`
- `crates/hone-web-api/src/routes/meta.rs`
- `crates/hone-channels/src/runners/types.rs`
- `crates/hone-channels/src/runners/hone_cloud.rs`
- `crates/hone-channels/src/runners/opencode_acp.rs`
- `crates/hone-channels/src/runners/codex_acp.rs`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Honeclaw 现在已经不是单一聊天进程，而是一组可以本地运行、桌面托管、公共 Web 使用、IM 多渠道接入、定时任务主动推送的 agent runtime。`config.example.yaml` 中的 `agent.runner` 默认推荐 `hone_cloud`，同时保留 `codex_acp`、`opencode_acp`、`multi-agent` 等路线；后台压缩和 heartbeat 又走独立的 `llm.auxiliary`；事件引擎、FMP、Tavily、渠道 auth、desktop sidecar、Hone Cloud API key 也各有自己的前置条件。

仓库已经有不少分散的健康检查能力：

- `bins/hone-cli/src/reports.rs` 中 `status` 汇总 model route、channel auth、API key 和 runtime binary；`doctor` 检查 canonical config、effective config、runtime/data/skills 目录、runner binary、channel auth。
- `bins/hone-cli/src/onboard.rs` 能引导 runner、channels、admins、providers，并在结束时可选运行 doctor。
- `packages/app/src/pages/settings.tsx` 的 Agent 设置页有 Hone Cloud、OpenAI/opencode、auxiliary、Codex ACP、Gemini CLI 等卡片，提供局部测试按钮。
- `packages/app/src/lib/backend.ts` 暴露 desktop-only 的 `check_agent_cli`、`test_openai_channel` 和 backend meta probing。
- `crates/hone-web-api/src/routes/meta.rs` 返回 capability 列表，例如 `chat`、`skills`、`cron_jobs`、`company_profiles`、`llm_audit`、`web_invites`，但这些 capability 表示 API/部署能力，不表示背后的模型、数据源、渠道是否可实际运行。
- `hone_cloud` runner 在 API key 为空时会在运行期失败；`codex_acp` 在 runner 启动时做版本矩阵校验；`opencode_acp` 支持继承本机 OpenCode 配置，也支持显式 Hone-side override。

这些能力说明基础已经存在，但产品层缺少一个统一的 readiness 对象。用户看到的是多个局部状态：设置页某个按钮测试成功、CLI status 某个 key configured、channel 进程 running、`/api/meta` 某个 capability 存在。它们没有被汇总成“当前这个部署能不能完成一次普通聊天、一次定时任务、一次 heartbeat 压缩、一次多渠道推送、一次公司画像更新”的可解释答案。

## 问题或机会

1. **新用户和桌面用户容易在首次真实消息时才发现阻塞。**  
   例如 `agent.runner=hone_cloud` 但 API key 为空，或选择 `opencode_acp` 但本机 OpenCode 继承配置不可用，当前往往要等 chat run 失败才知道。局部测试按钮也不会自动说明这会影响 public chat、cron、heartbeat 或多渠道任务。

2. **管理员缺少部署级 capability health。**  
   `/api/meta` 只能说 backend 支持 `cron_jobs`、`skills`、`llm_audit`，但不能说“cron 可创建但辅助模型未配置，所以 heartbeat/session compression 会退化”，“Feishu 进程 alive 但 auth 缺字段”，“Hone Cloud 可选中但凭证缺失”。

3. **CLI、Web、Desktop 对同一配置的解释分散。**  
   `hone-cli status`、`doctor`、desktop settings、backend meta、runner runtime error 各自有一套口径。活跃任务中还有 canonical config、desktop bundled apply、ACP runtime、skill runtime 等重构；没有统一 readiness schema，后续每加一个 runner 或 capability 都会继续扩散。

4. **商业化和支持效率会被“配置能否跑起来”卡住。**  
   Public/Hone Cloud、open-source local、desktop bundled、remote backend 的目标用户不同。一个用户问“为什么我的定时任务没发”“为什么桌面里能保存但聊天失败”，维护者现在需要串配置、日志、LLM audit、channel status。Runtime readiness 可以在问题发生前给出下一步动作。

这值得列为 P1：它不改变 agent 主链路，也不要求先完成 trace workbench 或 entitlement ledger，但能显著降低首次激活失败、桌面配置困惑和远程部署支持成本。

## 方案概述

新增 **Runtime Readiness Matrix**：一个统一的、可序列化的 readiness 评估层，把当前部署的关键 runtime route 和产品 capability 转成结构化 verdict。它不替代 `doctor`、`status`、`/api/meta` 或各个设置页测试，而是把它们的结果纳入同一个口径：

- `route readiness`：primary dialogue runner、auxiliary route、multi-agent search/answer、Hone Cloud、local ACP CLI。
- `capability readiness`：chat、public chat、cron/scheduler、heartbeat/session compression、skills/MCP、company portraits、event digest、channel delivery。
- `deployment readiness`：desktop bundled、desktop remote、CLI start、public web service。
- `next action`：每个 block/warn 给出用户能执行的最短修复动作，例如 `hone-cli models set ...`、打开 desktop settings、配置 Hone Cloud API key、安装/升级 `codex-acp`、配置 FMP/Tavily、启用 channel auth。

第一版只做本地推导和轻量 probe，不引入外部 SaaS，不改变 runner fallback 语义，不自动切换用户选择的 runner。

## 用户体验变化

### 用户端

- Public `/chat` 在登录后看到简洁状态：`Ready`、`Limited`、`Action needed`。普通用户不看内部 prompt、runner binary、API key，只看到能理解的说明，例如“聊天服务暂不可用，请联系管理员”或“服务可用，部分定时监控功能稍后开放”。
- API-key 型 Hone Cloud 客户端如果服务端当前 runner/data source 不 ready，错误响应可以携带稳定 reason code，而不是只返回底层 HTTP/runner 错误文本。

### 管理端

- Dashboard 增加 `Runtime Readiness` 区块，按功能显示：Chat、Scheduled tasks、Heartbeat/compression、Skills, Market data、Channels。
- Settings Agent 页面不再只有卡片内测试按钮，而是显示“当前选中 runner 是否满足 Chat/Task/Background route”。测试结果写入 readiness cache，保存配置后自动刷新。
- `/logs`、`/llm-audit`、未来 `/traces` 可以反向链接 readiness verdict：一次失败是运行中异常，还是运行前已知配置阻塞。

### 桌面端

- Desktop 首屏或 settings 顶部显示本地 readiness：bundled backend connected、sidecar binaries present、runner CLI present、primary route configured、auxiliary route configured、enabled channels healthy。
- Remote mode 明确显示“远端 readiness 由远端 backend 返回”，避免用户误以为本地 desktop 的 CLI 检查能代表远端运行环境。
- 对 `opencode_acp` 的“继承本机 OpenCode 配置”给出可验证状态：Hone-side override 为空不再被误判为缺配置，而是标为 `inherits_local_config`，并提示需要用一次 no-op probe 验证本机 OpenCode 是否可用。

### 多渠道

- Channel status 不只显示进程和 heartbeat，还显示该渠道当前依赖的核心 route 是否 ready。比如 Feishu running 但 primary runner blocked，应显示“进程在线，回复链路不可用”。
- 定时任务创建页或任务详情可显示“当前 task 可执行但无法投递到 Telegram，因为 bot token missing / process stopped / chat_scope mismatch”。

## 技术方案

### 1. 新增 readiness 类型和 evaluator

建议在 `hone-core` 或 `hone-web-api` 的共享层定义只含数据的类型，优先从 `hone-core` 开始，方便 CLI、desktop 和 Web API 共用：

```rust
pub enum ReadinessStatus {
    Ready,
    Degraded,
    Blocked,
    Unknown,
}

pub struct ReadinessFinding {
    pub code: String,
    pub severity: ReadinessStatus,
    pub subject: String,
    pub message: String,
    pub next_action: Option<String>,
    pub source: Option<String>,
}

pub struct RuntimeReadinessMatrix {
    pub generated_at: String,
    pub deployment_mode: String,
    pub active_runner: String,
    pub routes: Vec<RouteReadiness>,
    pub capabilities: Vec<CapabilityReadiness>,
    pub findings: Vec<ReadinessFinding>,
}
```

Evaluator 输入应只依赖当前 config、runtime paths、deployment mode、optional probe cache 和 heartbeat/channel status，避免直接读取 session 或用户私有内容。

### 2. Route readiness 口径

建议第一版覆盖这些 route：

- `primary_chat`：由 `agent.runner` 决定。`hone_cloud` 要求 base_url/model 合法、API key configured、可选 HTTP probe；`opencode_acp` 要求 binary 可用，若 override 为空则标记 `inherits_local_config` 并建议 probe；`codex_acp` 要求 codex/codex-acp 版本满足矩阵；`multi-agent` 要求 search/answer base_url/model/api_key 配置。
- `auxiliary_background`：由 `llm.auxiliary` 优先，legacy `llm.openrouter.sub_model` 只作为 fallback 描述。用于 heartbeat、session compression、safety judge 等后台路径。
- `market_data`：FMP key pool、Tavily/search key、event engine LLM classifier/global digest model 是否配置。
- `skill_execution`：skills dir 存在、skill registry 可读、`hone-mcp` binary 可定位、当前 runner/stage 是否支持所需工具。
- `channel_delivery`：每个启用 channel 的 auth、process heartbeat、chat_scope、平台限制。

### 3. Capability readiness 口径

Capability 不等于 API 是否存在，而是“用户能否完成该产品动作”。例如：

- `chat`: `primary_chat=Ready` 且 backend connected。
- `public_chat`: `chat=Ready` 且 public auth/session/entitlement 基础可用。
- `scheduled_tasks`: cron store 可读写、primary route ready、目标 channel 或 web delivery ready。
- `heartbeat`: heartbeat jobs 可见，auxiliary route ready 或明确 fallback。
- `company_profiles`: actor sandbox root 可写、company profile storage 可读、primary route 支持文件操作或通过 Hone Cloud 的能力声明。
- `multichannel_delivery`: outbound adapter/channel process ready，render contract 另由 `auto_p1_multichannel-render-preview.md` 处理。

### 4. API 与 CLI

新增只读 API：

- `GET /api/runtime-readiness`
- `POST /api/runtime-readiness/probe`：可选触发轻量 probe，要求 admin/desktop 权限；不在 public 端开放。

CLI 侧：

- `hone-cli status --json` 增加 `readiness` 字段，兼容旧字段。
- `hone-cli doctor` 继续输出 check 列表，但附加 summary：`runtime_readiness=ready/degraded/blocked`。
- 可选新增 `hone-cli doctor --readiness` 只打印矩阵和 next actions。

### 5. Desktop 与 Web 前端

前端建议新增：

- `packages/app/src/lib/runtime-readiness.ts`
- `packages/app/src/context/runtime-readiness.tsx`
- Dashboard readiness panel
- Settings Agent page 顶部 readiness strip
- Channel status row 增加 core route blocked/degraded badge

Desktop command 可以复用 backend API；只有需要本机 CLI probe 时才走 Tauri command。Remote mode 下不运行本地 CLI probe，避免误判。

### 6. 兼容与迁移

- 不改变现有 `agent.runner`、`llm.auxiliary`、`AgentRunner` trait 和 runner 执行语义。
- 不自动 fallback 到另一个 runner。Readiness 只提示“当前配置不可用”和“可选修复动作”。
- 旧 `/api/meta` 的 `capabilities` 保持不变，新 readiness API 独立返回动态健康状态。
- Probe 结果可先保存在内存中，带 TTL；不需要第一版新增持久表。

## 实施步骤

### Phase 1: 只读矩阵与 CLI 对齐

- 定义 readiness 类型与 deterministic evaluator。
- 覆盖 config/path/binary/auth/key 的静态判断，不做外部网络 probe。
- 在 `hone-cli status --json` 与 `doctor` summary 中输出 readiness。
- 给 `hone_cloud` missing API key、`opencode_acp` local inheritance、`codex_acp` version matrix、multi-agent search/answer missing key 写单元测试。

### Phase 2: Web API 与管理端

- 新增 `/api/runtime-readiness`。
- Dashboard 增加 readiness panel，Settings Agent 页显示当前 runner 对 chat/background 的影响。
- Channel status 结合 readiness 显示“process online but core route blocked”。
- 增加前端数据转换测试，确保旧后端没有 readiness API 时页面 graceful degrade。

### Phase 3: 轻量 probe 与桌面远端区分

- 增加 admin-only `probe`：Hone Cloud/OpenAI-compatible 发送最小请求，ACP runner 只跑 version/no-op initialization，不执行用户 prompt。
- Desktop bundled mode 允许本机 CLI probe；remote mode 只展示远端返回的 probe 状态。
- Probe 结果带 `last_checked_at`、`ttl_seconds`、`error_code` 和 redacted detail。

### Phase 4: 产品闭环

- Public chat blocked 时返回用户友好的 reason code。
- Task create/detail 页面显示 task readiness。
- Onboard 结束页根据 readiness 给出下一步，而不是只列 `status`/`doctor` 命令。
- 未来如果 `auto_p1_run_trace_workbench.md` 落地，trace detail 可引用运行前 readiness snapshot。

## 验证方式

- 单元测试：
  - `hone_cloud` runner selected + empty API key => `primary_chat=Blocked`，next action 指向配置 API key。
  - `opencode_acp` selected + override empty => `primary_chat=Degraded/Unknown` with `inherits_local_config` finding，而不是错误地判定 missing key。
  - `codex_acp` 版本低于要求 => blocked finding 带 upgrade command。
  - `multi-agent` search 或 answer key 缺失 => 对应 route blocked。
  - auxiliary route 为空但 legacy openrouter sub_model/key 存在 => degraded fallback finding。
- CLI 回归：
  - `hone-cli status --json` 输出可反序列化 readiness。
  - `hone-cli doctor` 在缺 runner binary、缺 channel auth、缺 Hone Cloud key 时 summary 与原有 check 一致。
- Web/API 测试：
  - `GET /api/runtime-readiness` 不泄露 raw API key、本地敏感路径或完整 OpenCode config。
  - admin dashboard 能渲染 ready/degraded/blocked/unknown 四种状态。
  - remote desktop 模式不调用本地 CLI probe。
- 手工验收：
  - 新 clone 跑 `hone-cli onboard`，跳过 Hone Cloud key 后，readiness 清楚提示 chat route blocked。
  - 设置 `opencode_acp` 且本机 OpenCode 已配置时，probe 后显示 ready；未配置时显示可执行的修复说明。
  - 启用 Telegram 但缺 token，channel readiness blocked，不影响 Web chat readiness。
- 指标：
  - 首次消息前可发现的配置阻塞占比提升。
  - “保存设置成功但聊天失败”的支持问题下降。
  - Readiness API p95 在无 probe 模式低于 50ms。

## 风险与取舍

- 风险：readiness 可能被误当成强保证。取舍：状态必须区分 deterministic check、cached probe、runtime observation，并显示 `generated_at`/`last_checked_at`。
- 风险：probe 调用外部模型增加成本或触发风控。取舍：第一版默认只做静态判断，网络 probe 必须用户触发或 admin 显式开启，且使用最小请求。
- 风险：暴露太多内部配置。取舍：public 端只返回 coarse reason code；完整 findings 仅 admin/desktop 可见，所有 key 和敏感路径默认脱敏。
- 风险：和 `doctor`、`status` 重复。取舍：保留 `doctor` 作为低层检查列表，readiness 作为产品动作级聚合 verdict。
- 风险：Hone Cloud 能力和本地 ACP 能力不完全一致。取舍：readiness route 增加 capability tags，不假设所有 runner 都支持文件写入、MCP、长上下文或本地图像。
- 不做：不自动切 runner，不做成本/权益账本，不做单次 run trace 聚合，不做投资上下文 gap resolver，不改变现有 config source of truth。

## 与已有提案的差异

查重范围：

- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `docs/proposal/auto_p1_cross-company-thesis-map.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 与 `auto_p1_investment_context_intake.md` 不重复：该提案处理用户投资资料、组合、画像、偏好和 digest 前置上下文是否齐；本提案处理 runtime/model/provider/channel 是否能支撑产品能力。
- 与 `auto_p1_run_trace_workbench.md` 不重复：trace workbench 聚合一次已经发生的 run 证据；本提案在运行前给出配置和能力 readiness verdict。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：entitlement 管理额度、成本和商业权限；本提案管理技术运行前置条件，不判断用户是否有付费权益。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该提案关注 desktop sidecar 启动、锁和进程接管；本提案关注 sidecar 已连接后，各功能 route 是否可实际运行。
- 与 `auto_p1_multichannel-render-preview.md` 不重复：该提案处理内容渲染到不同平台的预览和降级；本提案只判断 channel/process/auth/core route 是否 ready。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该提案处理 skill schema 和 runner stage 对齐；本提案只把 skill/MCP/runner availability 纳入 readiness 矩阵。
