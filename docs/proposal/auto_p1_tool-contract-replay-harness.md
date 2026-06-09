# Proposal: Tool Contract Registry and Replay Harness

status: proposed
priority: P1
created_at: 2026-06-09 14:04:12 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/`
- `docs/proposals/`
- `crates/hone-tools/src/base.rs`
- `crates/hone-tools/src/registry.rs`
- `crates/hone-tools/src/lib.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-tools/src/data_fetch.rs`
- `crates/hone-tools/src/web_search.rs`
- `crates/hone-tools/src/deep_research.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-tools/src/notification_prefs_tool.rs`
- `crates/hone-tools/src/cron_job_tool.rs`
- `crates/hone-channels/src/core/bot_core.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `agents/function_calling/src/lib.rs`
- `tests/regression/ci/test_skill_runtime_stage_consistency.sh`
- `tests/regression/manual/test_hone_mcp_cron_visibility.sh`
- `tests/regression/manual/test_hone_mcp_skill_dir_env.sh`

## 背景与现状

Honeclaw 的 agent 能力已经不只是聊天回答。当前运行时会在 `HoneBotCore::create_tool_registry()` 中为一次执行构建工具表，并按 actor、channel target、cron 权限、cloud/local store 注入不同能力。这个工具表会被多条路径消费：

- `agents/function_calling/src/lib.rs` 直接把 `ToolRegistry::get_tools_schema()` 传给 OpenAI-compatible function calling。
- `crates/hone-channels/src/mcp_bridge.rs` 把同一份 OpenAI-style schema 转成 MCP `tools/list` 的 `inputSchema`，再通过 `tools/call` 返回 `structuredContent`。
- `crates/hone-channels/src/execution.rs` 为 persistent conversation 和 transient task 统一准备 runner、actor sandbox、tool registry、prompt audit 和 allowed-tools / max-tool-calls 限制。
- `crates/hone-tools/src/skill_tool.rs` 在工具层内又承载 skill script 执行、artifact 校验、stderr 脱敏和 invoked skill metadata 持久化。

这说明 tool contract 已经是 Hone 产品架构的核心连接层：Web 对话、多渠道 IM、桌面 sidecar、ACP/MCP runner、event-engine transient task、skill runtime、投资画像和组合管理都依赖它。

但当前 `Tool` trait 只定义了 name、description、参数列表和 `execute(args) -> Value`。输入 schema 可被导出，输出则主要靠每个 tool 自己返回自由 JSON。有些工具用 `{ "error": ... }` 表示业务失败，有些直接返回 `HoneError::Tool`，有些返回 `success: false`，有些允许 partial result 加 `errors`。MCP bridge 目前用 `value.get("error").is_some()` 推导 `isError`，这对自由 JSON 是可用但脆弱的约定。已有测试覆盖了许多局部逻辑和 skill stage consistency，但还没有一个面向“所有内置工具 contract 是否稳定、不同 runner 是否看到同一工具语义、工具结果是否可回放”的产品/架构面。

## 问题或机会

### 1. Tool 已经成为产品 API，但没有版本化 contract

对用户而言，`portfolio_management`、`notification_preferences`、`cron_job`、`data_fetch`、`web_search`、`skill_tool` 等不是内部实现细节，而是 agent 能力的实际边界。它们决定了用户能不能建立持仓、创建提醒、调节通知、读取行情、生成图表和沉淀公司画像。

如果某个工具悄悄改了参数名、枚举值、错误 shape 或成功字段，影响会跨越：

- function-calling runner 的工具选择；
- MCP/ACP runner 的 `tools/list` 与 `tools/call`；
- skill 的 slash/direct invoke；
- multi-agent 两阶段执行；
- Web / IM 渠道用户可见错误；
- LLM audit、prompt audit、support bundle 和后续 replay 工具。

### 2. 错误语义和 partial success 难以自动判断

`web_search`、`data_fetch` 和 `deep_research` 都做了错误脱敏；`data_fetch` 还允许 snapshot 局部失败但整体可用；`restart_hone` 需要确认语义；`cron_job` 受 allow_cron 和 actor/admin 权限影响；`notification_prefs` 有 actor-scoped side effect。今天这些差异藏在各工具实现里，缺少统一分类：

- `input_validation_error`
- `permission_or_stage_unavailable`
- `external_dependency_error`
- `partial_success`
- `side_effect_applied`
- `needs_confirmation`
- `artifact_generated`
- `no_data`

缺少分类后，管理端、桌面端和多渠道无法用一致方式告诉用户“这是配置问题、权限问题、外部服务问题，还是工具按设计没有数据”。

### 3. 工具回归仍偏代码单元，缺少产品级 replay fixture

仓库已经有 `tests/regression/ci/test_skill_runtime_stage_consistency.sh`，证明 MCP stage 对 skill/cron 可见性的一部分一致性。也有 manual scripts 覆盖真实 MCP 和 live finance paths。但这些测试没有回答：

- 每个注册工具的 OpenAI schema 与 MCP schema 是否稳定且可比对；
- 每个工具最小 success/error fixture 是否仍返回兼容 JSON；
- 同一 fixture 在 function-calling registry 和 MCP bridge 下是否得到同等错误分类；
- actor-scoped side effect 工具是否能在 dry-run/sandbox 下安全回放；
- 新增工具是否有最低 contract 文档和回归样本。

### 4. 这是 AI agent 产品的基础设施机会

行业里 agent 产品正在从“模型会调用工具”走向“工具能力可观测、可测试、可授权、可迁移”。Hone 已经有 skill runtime、MCP bridge、actor sandbox、prompt audit、LLM audit、run trace 方向和 multi-channel delivery；下一步需要把 tool contract 变成一等资产，让能力扩展不再依赖人工读代码确认。

这会直接提升：

- 核心体验：agent 工具失败能被更清楚地解释和恢复；
- 研发效率：新增/修改工具有 contract gate，降低跨 runner 破坏；
- 运维效率：管理员能按工具名、错误类别、actor、runner 回放问题；
- 商业化：public API / Hone Cloud / desktop remote backend 可公布稳定能力清单；
- 安全性：高风险 side-effect 工具可以按 contract 声明 confirmation、scope 和 dry-run 支持。

## 方案概述

新增 **Tool Contract Registry and Replay Harness**，把内置工具从“只有输入 schema 的动态 registry”提升为“版本化 contract + fixture replay + 管理端可见能力面”。

第一版不需要重写所有工具。它应以兼容方式新增一层 metadata 和验证工具：

1. 在 `hone-tools` 中定义 `ToolContract`，描述输入 schema、输出分类、side effect、权限、dry-run 能力、错误 code、artifact 类型、fixture 路径和 contract version。
2. 让现有 `Tool` trait 可以选择性返回 contract metadata；没有显式 metadata 的旧工具由 wrapper 自动生成 legacy contract。
3. 增加 `hone-cli tools contract` 或 CI helper，导出当前 config/actor/channel/allow_cron 下可见工具的 contract manifest。
4. 建立 `tests/fixtures/tool-contracts/`，为关键工具保留最小 success/error/permission fixture。
5. 在 CI-safe regression 中校验 schema、MCP 转换、错误分类和 fixture replay。
6. 在管理端 Skills/Runtime/Logs 相关页面逐步显示 Tool Contract 状态：可见、可调用、需要 actor、需要外部依赖、支持 dry-run、最近错误类别。

## 用户体验变化

### 用户端

- Public `/chat` 和多渠道对话在工具失败时可以得到更具体的恢复提示，例如“行情源暂时不可用，稍后重试”而不是泛化的工具错误。
- 用户通过自然语言修改持仓、通知偏好或 cron 任务时，agent 可以基于 contract 知道该工具是否有 side effect、是否需要确认、是否支持 dry-run preview。
- 生成图片、文件或公司画像事件时，输出中可以明确区分“artifact 已生成”“artifact 生成失败但文本回答可用”“artifact 路径不可公开”等状态。

### 管理端

- Skills 页面旁新增或合并一个 Tools tab，列出当前部署下实际注册的工具、contract version、输入参数、输出类别、依赖、stage 限制和最近 replay 状态。
- Logs / LLM Audit / Run Trace 可以按 `tool_name + contract_version + outcome_code` 过滤，而不是只靠自然语言错误。
- Task Health 可以显示 event-engine / scheduler transient task 依赖的工具 contract 是否健康。

### 桌面端

- Desktop bundled 模式能在启动诊断里显示本地工具能力是否可用：例如 `web_search` 缺 Tavily、`data_fetch` 缺 FMP、`cron_job` 在当前 stage 不可用、`skill_tool` script artifact 目录不可写。
- Remote backend 模式可以通过 meta/capability 接口知道远端支持哪些工具 contract，避免桌面 UI 暗示一个远端不可用能力。

### 多渠道

- Feishu / Telegram / Discord / iMessage 输出适配器可以拿到统一错误分类，决定是否需要短消息提示、是否适合建议用户回 Web 管理端修复、是否应静默降级。
- MCP/ACP runner 看到的工具列表与 in-process function calling 的工具列表可以被同一 manifest 比对，减少“一个 runner 看得到但另一个 runner 调不通”的问题。

## 技术方案

### 1. Contract 类型

在 `crates/hone-tools` 增加类似以下结构：

```rust
pub struct ToolContract {
    pub name: String,
    pub contract_version: u32,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
    pub outcomes: Vec<ToolOutcomeContract>,
    pub side_effect: ToolSideEffect,
    pub actor_scope: ToolActorScope,
    pub stage_requirements: Vec<ToolStageRequirement>,
    pub external_dependencies: Vec<ToolDependency>,
    pub supports_dry_run: bool,
    pub fixtures: Vec<ToolFixtureRef>,
}
```

重点不是第一版就写完整 JSON Schema，而是先把最有用的 product contract 固化：

- outcome code：`ok`、`partial_success`、`needs_confirmation`、`input_invalid`、`permission_denied`、`stage_unavailable`、`external_unavailable`、`rate_limited`、`no_data`、`internal_error`。
- side effect：`none`、`actor_state_write`、`runtime_config_write`、`external_delivery`、`process_control`。
- actor scope：`none`、`required`、`admin_bypass_possible`、`channel_target_required`。
- dependency：`fmp`、`tavily`、`postgres`、`oss`、`cron_store`、`skill_registry`、`gen_images_dir`、`desktop_sidecar`。

### 2. Backward-compatible trait extension

不要一次性破坏所有工具实现。可以给 `Tool` 增加默认方法：

```rust
fn contract(&self) -> ToolContract {
    ToolContract::legacy_from_tool(self)
}
```

旧工具自动继承 legacy contract。关键工具逐个补充显式 contract：

- `DataFetchTool`：external dependency、partial success、FMP key missing、snapshot/query modes。
- `WebSearchTool`：Tavily dependency、key rejected/rate limited/temporary failure。
- `PortfolioTool`：actor-scoped write、dry-run preview、holding/watchlist mutation outcomes。
- `NotificationPrefsTool`：actor-scoped write、schedule overview dependency。
- `CronJobTool`：stage requirement `allow_cron`、actor/channel target、admin bypass、cloud/local persistence。
- `SkillTool`：script opt-in、artifact contract、stderr redaction、invoked skill metadata side effect。
- `RestartHoneTool`：process_control、needs_confirmation、desktop/install constraints。

### 3. Registry manifest export

`ToolRegistry` 增加：

- `get_tool_contracts() -> Vec<ToolContractManifest>`
- `get_tool_contract(name) -> Option<ToolContractManifest>`

MCP bridge 在 `tools/list` 仍保持 MCP 标准 shape，但可以在 Hone 自有诊断命令或 `resources/list` 未来扩展中暴露只读 contract manifest。第一版不要污染标准 MCP tool schema，避免 runner 兼容风险。

### 4. Replay fixture

新增 fixture 目录：

```text
tests/fixtures/tool-contracts/
  data_fetch/
    missing_key.input.json
    missing_key.expected.json
    snapshot_partial.expected.json
  web_search/
    missing_key.input.json
    missing_key.expected.json
  notification_prefs/
    get.input.json
    set_quiet_hours.input.json
  cron_job/
    stage_unavailable.input.json
  skill_tool/
    script_success.expected.json
```

对于会写状态的工具，fixture 必须运行在临时 data dir、临时 actor、临时 skill dir 下。对于外部依赖工具，CI-safe fixture 默认只覆盖 missing-key、mock transport 或 pure parser；真实 live 调用仍留在 `tests/regression/manual/`。

### 5. Outcome classifier

在 `ToolRegistry::execute_tool()` 后增加只读 classifier，不改变原始返回值：

```rust
pub struct ToolExecutionEnvelope {
    pub tool_name: String,
    pub contract_version: u32,
    pub raw: serde_json::Value,
    pub outcome: ToolOutcome,
    pub side_effect_applied: bool,
    pub retryable: bool,
    pub user_action: Option<ToolUserAction>,
}
```

第一版可以只在 replay harness 和 logs/audit 中使用 envelope，不改变 runner 看到的工具返回。等 contract 稳定后，再考虑让 MCP `structuredContent` 附加 Hone metadata，或让 LLM prompt 使用更明确的 outcome code。

### 6. 管理端集成

后端增加只读 API：

- `GET /api/tools/contracts`
- `GET /api/tools/contracts/{name}`
- `POST /api/tools/contracts/replay` 仅 admin，本地临时 data dir，默认 dry-run，禁止真实外部调用。

前端优先放在 Settings 或 Skills 附近，不新开大型页面。展示重点：

- 当前 actor/channel/stage 下 visible tools。
- 哪些工具缺外部依赖。
- 哪些工具有 side effect。
- 最近一次 contract replay 是否通过。
- tool schema 与 MCP schema 转换是否一致。

## 实施步骤

### Phase 0: Inventory and manifest-only

- 在 `hone-tools` 增加 `ToolContract` 类型和 legacy default。
- 给 `ToolRegistry` 增加 manifest export。
- 增加 CLI 或测试 helper，能在固定 actor/channel/allow_cron 下导出 JSON manifest。
- CI 检查所有 tool name 唯一、input schema 可序列化、contract version 存在、side effect 默认明确。

### Phase 1: Critical tool explicit contracts

- 给 `data_fetch`、`web_search`、`portfolio_tool`、`notification_prefs_tool`、`cron_job_tool`、`skill_tool` 补显式 contract。
- 写最小 fixture，覆盖 missing dependency、permission/stage unavailable、actor-scoped write dry-run、partial success 和 artifact success/error。
- 新增 CI-safe regression：`tests/regression/ci/test_tool_contract_replay.sh`。

### Phase 2: MCP/function-calling equivalence

- 在测试中启动 `hone-mcp`，比较 MCP `tools/list` 的 `inputSchema` 与 registry manifest 的 input schema。
- 用同一 fixture 分别走 registry direct call 和 MCP `tools/call`，检查 outcome classifier 一致。
- 给 `FunctionCallingAgent` 的工具执行路径记录 `tool_contract_version` 和 `outcome_code` 到 LLM audit metadata。

### Phase 3: Admin diagnostics

- 增加只读 API 和前端 Tools diagnostics surface。
- 在 Runtime Readiness / Task Health / Logs 中引用 tool contract 状态。
- 对高风险工具展示 side effect 和 confirmation requirement。

### Phase 4: Product hardening

- 让 public chat / IM adapters 使用 outcome category 生成更稳定的用户提示。
- 将 contract manifest 纳入 support bundle、run trace 和 future developer docs。
- 对 public Hone Cloud API 暴露 capability subset 时，引用同一 contract registry 生成只读能力摘要。

## 验证方式

- 静态验证：
  - `cargo test -p hone-tools tool_contract`
  - `cargo test -p hone-channels mcp_tool_contract`
  - 所有 registry tool 都有 contract version、side effect、actor scope 和 outcome 列表。
- CI-safe 回归：
  - `bash tests/regression/ci/test_tool_contract_replay.sh`
  - `bash tests/regression/ci/test_skill_runtime_stage_consistency.sh`
  - fixture 不依赖真实外部账号，不读取用户真实 `config.yaml`。
- MCP 等价验证：
  - 启动 `hone-mcp`，执行 `initialize`、`tools/list`、fixture `tools/call`。
  - 比对 MCP `inputSchema` 与 registry manifest。
- 管理端验收：
  - Settings/Skills 附近能看到工具能力列表、依赖状态、side effect、最近 replay verdict。
  - 缺 Tavily/FMP、cron stage unavailable、skill script artifact invalid 等错误能显示稳定 outcome code。
- 手工验证：
  - `tests/regression/manual/test_hone_mcp_cron_visibility.sh`
  - `tests/regression/manual/test_hone_mcp_skill_dir_env.sh`
  - live finance/search scripts 仍只在显式 `RUN_*_LIVE_SMOKES=1` 下执行。

## 风险与取舍

- **风险：一次性要求完整 output JSON Schema 会拖慢迭代。**  
  取舍：第一版只强制 outcome、side effect、actor scope、dependency 和 fixture，output schema 可选。

- **风险：改变工具返回会影响 LLM 和 MCP runner。**  
  取舍：第一版 envelope 只用于 replay/audit/admin，不改变 `execute()` 原始返回，也不改变 MCP 标准 `tools/list` shape。

- **风险：side-effect 工具 replay 误写真实用户数据。**  
  取舍：CI fixture 必须使用临时 data dir、临时 actor、临时 config；没有 dry-run 的写工具只允许测试 invalid/permission path，成功写 path 放入明确 sandbox。

- **风险：contract metadata 与实现漂移。**  
  取舍：fixture replay 是 gate；显式 contract 必须伴随最小 fixture。新增工具没有 fixture 时只能以 legacy contract 进入，不允许标记为 stable。

- **风险：与 skill runtime eval 重叠。**  
  取舍：skill eval 关注 `SKILL.md`、slash/direct invoke、stage visibility 和脚本作者体验；本提案关注所有内置 tool 的输入/输出/错误/side-effect contract。`skill_tool` 只是其中一个被纳入的工具。

- **不做边界：**
  - 不把 Hone 内置工具开放成任意第三方 marketplace。
  - 不在第一版让 public API 支持 tool calling。
  - 不要求所有外部 live provider 进入 CI。
  - 不替代权限 broker、mutation ledger 或 output safety gate，只为它们提供更稳定的 tool outcome 事实。

## 与已有提案的差异

查重范围覆盖了 `docs/proposal/` 和 `docs/proposals/` 下的现有提案，重点比对了以下方向：

- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `docs/proposal/auto_p1_skill-authoring-eval-lab.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_hone-cloud-api-contract.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `docs/proposal/auto_p1_prompt-context-budget-inspector.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `docs/proposal/auto_p1_privacy-preserving-product-events.md`
- `docs/proposal/auto_p2_external-mcp-workspace-gateway.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`

差异结论：

- 不重复 skill runtime 方向：那些提案关注 skill 的发现、调用、作者体验和多阶段可见性；本提案关注所有 built-in tool 的 contract、fixture replay 和 MCP/function-calling 等价性。
- 不重复 mutation ledger / permission broker：它们关注高风险操作如何确认、授权、审计；本提案先把每个工具是否有 side effect、是否需要确认、错误属于哪一类变成机器可读事实。
- 不重复 run trace：run trace 串联一次 agent run 的时间线；本提案定义工具能力本身的稳定 contract 和可回放 fixture，可作为 run trace 的输入维度。
- 不重复 runtime readiness：readiness 判断当前部署能不能跑；本提案判断每个工具 contract 是否稳定、schema 是否兼容、fixture 是否仍可回放。
- 不重复 Hone Cloud API contract：该提案面向外部 OpenAI-compatible API；本提案是内部工具能力 contract，未来可被 API capability 摘要引用，但不扩展 public tool calling。
- 不重复 product events / multichannel render preview：它们关注产品事件与输出渲染；本提案关注工具执行结果和错误分类的源头。

因此，本提案填补的是 “Hone 工具能力作为 agent 产品 API 的版本化、可测试、可回放 contract” 这一层，现有 proposal 尚未单独覆盖。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，不修改业务代码、测试代码、运行配置，也不更新 `docs/current-plan.md`。原因是该任务不进入活跃实施态，只是为后续人或 agent 留下一份可执行提案。

若后续实际落地，应新增或复用 `docs/current-plans/tool-contract-replay-harness.md`，并在以下情况同步长期文档：

- 修改 `Tool` trait、`ToolRegistry`、MCP bridge 或 tool manifest：更新 `docs/repo-map.md`。
- 新增 tool contract 作为长期约束：更新 `docs/invariants.md`。
- 决定将 outcome envelope 暴露给 runner、MCP 或 public API：更新 `docs/decisions.md` 或补 ADR。
- 新增 CI-safe replay 脚本：按 `AGENTS.md` 测试组织策略放入 `tests/regression/ci/` 并更新相关说明。
