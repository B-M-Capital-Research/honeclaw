# Proposal: Prompt Context Budget Inspector for Reliable Agent Turns

status: proposed
priority: P1
created_at: 2026-05-17 08:04:06 +0800
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
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_session-memory-correction.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-channels/src/turn_builder.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/prompt_audit.rs`
- `crates/hone-channels/src/agent_session/restore.rs`
- `crates/hone-channels/src/session_compactor.rs`
- `crates/hone-channels/src/attachments/ingest.rs`
- `crates/hone-tools/src/skill_runtime.rs`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `skills/company_portrait/SKILL.md`
- `skills/stock_research/SKILL.md`
- `skills/market_analysis/SKILL.md`
- `skills/scheduled_task/SKILL.md`
- `skills/pdf_understanding/SKILL.md`
- `packages/app/src/pages/llm-audit.tsx`
- `packages/app/src/pages/logs.tsx`
- `packages/app/src/pages/settings.tsx`

## 背景与现状

Hone 的一次回答已经不是“把用户输入交给模型”这么简单。当前主链路会在运行前组装多层上下文：

- `crates/hone-channels/src/prompt.rs` 构建 `PromptBundle`，把静态系统提示、金融领域约束、公司画像策略、渠道格式约束、管理员提示、session 当前时间和历史摘要分层放入 system prompt 与 runtime input。
- `crates/hone-channels/src/turn_builder.rs` 在每轮前根据 `allow_cron`、skill stage constraints、`build_skill_listing_for_stage(4_000, ...)` 和 related skill search，把技能索引与本轮相关技能提示注入 prompt。
- `crates/hone-channels/src/agent_session/restore.rs` 会从最近 compact boundary 之后恢复原始消息、assistant/tool transcript、invoked skill prompt 和 compact skill snapshot，供 ACP / OpenCode / Codex runner 继续上下文。
- `crates/hone-channels/src/session_compactor.rs` 在长会话中用 auxiliary LLM 生成 compact summary，并最多保留 4 个、每个 12k 字符的 compact skill snapshot。
- `crates/hone-channels/src/attachments/ingest.rs` 会把附件、PDF 预览、压缩包 manifest 和抽取片段变成本轮用户输入的一部分，并已有大小、数量和 preview 长度 gate。
- `crates/hone-channels/src/prompt_audit.rs` 会把最终 `system_prompt` 与 `runtime_input` 写入 `data/runtime/prompt-audit/<channel>/...json`，但这是运行后的审计文件，不是运行前的预算决策层。
- 多个 skill 的 `SKILL.md` 已经很长，特别是 `company_portrait`、`stock_research`、`scheduled_task`、`pdf_understanding`，同时 skill runtime 仍要求“默认只披露 listing，调用时完整注入”。

这些机制解决了很多具体问题：金融边界、技能发现、历史恢复、压缩、附件准入和 prompt audit。但它们还没有一个统一的 **Prompt Context Budget** 产品/架构层。现在每个上下文来源各自决定注入多少、何时注入、如何裁剪、如何解释优先级；系统缺少一个运行前可计算的“这一轮到底带了什么、为什么带、占多少预算、裁掉了什么、是否会影响回答质量”的对象。

## 问题或机会

这是 P1，因为上下文装配直接影响核心回答质量、成本、延迟、排障效率和多 runner 稳定性。它不是单点 bug，但会放大正在进行的 ACP runtime、skill runtime、company portraits、附件、session compaction 与 Hone Cloud API 等主线改造的风险。

当前缺口集中在六类：

1. **上下文大小没有统一预算。**
   `skill_listing` 有 4,000 字符上限，compact skill snapshot 有 12,000 字符上限，附件 preview 有自己的 byte/char 上限，session restore 又按 message window 恢复。它们没有汇总到同一个 turn budget，也没有按 runner/model/capability 动态调整。

2. **上下文来源不可解释。**
   Prompt audit 能事后看到完整 prompt，但管理员很难快速判断：本轮回答依赖的是 compact summary、原始消息、related skill hint、完整 skill body、PDF preview、附件 manifest、公司画像策略，还是用户当前输入。用户问“为什么你还记得/为什么没参考某资料”时，排查仍需要人工读长 prompt。

3. **裁剪策略分散且可能不一致。**
   长会话、附件、skill、skill snapshot、related skill hint 都会争夺上下文窗口。如果没有统一优先级，系统可能保留了低价值 skill listing，却裁掉了用户刚上传 PDF 的关键片段；也可能为了恢复旧 skill prompt 挤压当前 turn 的附件证据。

4. **成本和延迟难以前置控制。**
   `memory/src/llm_audit.rs` 已记录 token 结果，Usage Entitlement Ledger 提案也会处理跨运行成本，但这都是执行后或权益层。运行前没有“预计 prompt 规模过大 / 本轮将触发高成本路径 / 建议先 compact 或要求用户缩小范围”的判断。

5. **多 runner 能力差异没有进入 context budget。**
   `agent.runner` 可以是 `hone_cloud`、`opencode_acp`、`codex_acp`、`multi-agent` 或 function-calling。不同 runner 对本地文件、MCP、上下文长度、图片/附件、tool transcript 的处理不同。当前 prompt 装配主要按通用路径生成，而不是先产出一个可被 runner 消费的 context plan。

6. **后续提案需要共同前置层。**
   Run Trace Workbench 关注运行后证据，Session Memory Correction 关注错误摘要纠正，Investment Document Inbox 关注文档资产化，Research Artifact Library 关注研究交付物，Skill Runtime Alignment 关注技能注入语义。它们都会继续往 prompt 里放更多上下文；如果没有 context budget inspector，系统会越来越依赖 prompt audit 事后排障。

机会是：Hone 已经有明确 prompt layering、prompt audit、session compaction、skill stage constraints、附件 gate 和 LLM audit token 字段。第一版不需要改变 runner 协议，只要在 `PromptTurnBuilder` 和 `ExecutionService` 之间增加一个可序列化的 `PromptContextPlan`，就能把“上下文装配”从隐式字符串拼接升级为可测试、可预览、可调优的产品能力。

## 方案概述

新增 **Prompt Context Budget Inspector**，为每个 agent turn 生成一个结构化 `PromptContextPlan`：

- `sources`：本轮可用上下文来源，例如 static policy、session time、compact summary、recent messages、skill listing、related skill hints、invoked skill prompts、attachment previews、PDF extraction、cron policy、channel format guidance。
- `budget`：按字符数、估算 token、runner capability、硬上限、软上限记录每类上下文的预算。
- `priority`：明确当前输入、附件证据、用户 correction、结构化真相源、最近原文、compact summary、skill listing 等优先级。
- `included` / `trimmed` / `omitted`：记录哪些上下文被注入、哪些被裁剪、哪些因预算或 stage 不可用被跳过。
- `warnings`：例如 prompt 过大、附件预览被截断、skill snapshot 挤占预算、Hone Cloud route 不支持本地文件、auxiliary compact 不可用、当前 turn 建议先 `/compact`。
- `audit_ref`：和现有 prompt audit、未来 run trace、LLM audit 关联，但不默认暴露完整 prompt。

第一版目标是“可观测 + 可控裁剪”，不是重写所有 prompt。它应先保持现有最终 prompt 行为尽量不变，只新增计划、统计、预警和可测试裁剪边界；稳定后再逐步把各来源迁到统一 budget allocator。

## 用户体验变化

### 用户端

- Public `/chat` 在上下文过大或附件被截断时，可以给用户清晰提示，例如“这份 PDF 只读取了前 N 字符；如要分析具体页，请指定页码或上传截图”。
- 长会话临近预算上限时，用户看到的是“建议整理当前会话记忆”而不是突然回答变差或 runner 超时。
- 当用户问“你这次参考了什么”时，Hone 可以用安全摘要说明“参考了最近对话、当前附件预览和已加载的 company_portrait skill”，不泄露完整系统 prompt。

### 管理端

- `/logs`、`/llm-audit` 或未来 `/traces` 增加 `Context` tab：显示本轮 context plan 的来源、长度、裁剪、warnings 和 prompt audit link。
- Settings / Runtime Readiness 可显示当前 runner 的 context capability：本地文件可见性、MCP/tool transcript、建议最大 prompt、是否支持图像/附件、多阶段 handoff。
- 管理员排查回答质量时，可以先看“本轮是否真的带入了用户上传文档 / compact summary / skill prompt / 最近消息”，再决定是否查完整 prompt audit。

### 桌面端

- Desktop bundled/remote 模式复用后端 context plan API。Remote mode 明确显示“预算与裁剪由远端 runner 决定”，避免本机设置误导用户。
- 当本地长会话或大附件会让 ACP runner 变慢时，桌面端可以在发送前提示用户先 compact、缩小文档范围或切换到更合适的工作流。

### 多渠道

- Feishu / Telegram / Discord / iMessage 不需要展示完整 inspector，但当附件、图片、PDF 或长群聊上下文被裁剪时，应返回短提示，避免用户以为系统完整读取了所有材料。
- 群聊场景中，pretrigger buffer、group compact summary 和当前触发文本应在 context plan 中区分，防止把旧群聊噪音误当成本轮强上下文。

## 技术方案

### 1. 定义 `PromptContextPlan`

建议在 `crates/hone-channels` 先定义内部类型，稳定后再考虑提升到 `hone-core`：

```rust
pub struct PromptContextPlan {
    pub generated_at: String,
    pub session_id: String,
    pub actor: ActorIdentity,
    pub runner: String,
    pub total_chars: usize,
    pub estimated_tokens: Option<usize>,
    pub soft_budget_tokens: Option<usize>,
    pub hard_budget_tokens: Option<usize>,
    pub sources: Vec<PromptContextSource>,
    pub warnings: Vec<PromptContextWarning>,
}

pub struct PromptContextSource {
    pub id: String,
    pub kind: String,
    pub priority: u8,
    pub included_chars: usize,
    pub original_chars: usize,
    pub status: String, // included | trimmed | omitted
    pub reason_code: Option<String>,
    pub audit_label: String,
}
```

Token 估算第一版可以采用保守字符换算，不依赖外部 tokenizer。重点是相对预算、裁剪记录和 reason code 稳定。

### 2. 在 `PromptTurnBuilder` 中先生成计划

当前 `resolve_prompt_input()` 直接拼出 `system_prompt` 和 `runtime_input`。建议拆成两步：

1. `collect_prompt_context_sources(user_input) -> Vec<PromptContextSourceDraft>`
2. `allocate_prompt_context(sources, runner_capability) -> PromptContextPlan + PromptTurnInput`

第一阶段先只包住现有来源：

- static system + finance policy + company profile policy
- channel format guidance / admin prompt / cron policy
- turn-0 skill listing
- related skill hint
- conversation context / compact summary
- session context time block
- recv_extra / attachment context
- current user input

为了降低风险，Phase 1 不改变拼接顺序，只计算每段长度、来源和 warning。

### 3. 建立统一优先级

建议默认优先级从高到低：

1. 当前用户输入、明确附件/PDF/图片抽取结果、用户本轮 correction。
2. 安全与领域边界、当前时间、渠道格式、管理员/权限边界。
3. 当前 turn 已显式调用的 skill prompt。
4. 最近原始消息、tool transcript、session restore window。
5. compact summary 和 compact skill snapshot。
6. related skill hints。
7. turn-0 skill listing。
8. 可重新发现的低价值提示或重复说明。

裁剪时不能裁掉 safety policy、当前输入、当前时间和权限边界。对可裁剪内容必须写 reason code，例如 `budget_trimmed_related_skill_hint`、`omitted_disabled_skill_snapshot`、`attachment_preview_truncated`。

### 4. Runner capability 与预算配置

新增轻量 capability 推导，不要求网络 probe：

- `runner_kind`
- `manages_own_context`
- `supports_local_files`
- `supports_tool_transcript_restore`
- `supports_image_or_attachment_context`
- `recommended_context_chars`
- `recommended_context_tokens`

这些可以先从现有 `agent.runner`、runner trait 名称和 config 静态推导。未来 Runtime Readiness Matrix 可以读取同一 capability，但本提案不依赖它。

配置建议：

```yaml
agent:
  prompt_context:
    warning_chars: 60000
    hard_chars: 120000
    skill_listing_chars: 4000
    compact_summary_chars: 12000
    attachment_preview_chars: 6000
```

第一版保持现有默认行为，只在超过 warning/hard 时给出 warning；Phase 2 再启用 hard 裁剪。

### 5. Prompt audit 扩展

`prompt_audit.rs` 当前只写完整 `system_prompt` 和 `runtime_input`。建议增加：

- `context_plan`
- `context_plan_hash`
- `system_prompt_chars`
- `runtime_input_chars`
- `estimated_tokens`
- `warnings`

旧 prompt audit 文件保持可读；新字段可选。Run Trace Workbench 落地后可直接引用 `context_plan_hash` 和 warnings，而不是重复解析 prompt。

### 6. API 与 UI

新增只读 admin API：

- `GET /api/prompt-context/latest?session_id=...`
- `GET /api/prompt-context/{audit_id_or_path}`

如果不想引入新索引，第一版可以从 `prompt-audit/latest-*.json` 读取 `context_plan`。后续再迁到 trace 存储。

前端先做轻量组件：

- `ContextPlanSummary`
- `ContextSourceTable`
- `ContextWarnings`

挂载点可以先放在 `/llm-audit` 详情、`/logs` 记录抽屉或未来 `/traces`，避免新增独立页面。

## 实施步骤

### Phase 1: 观测与审计

- 定义 `PromptContextPlan` / `PromptContextSource` / `PromptContextWarning`。
- 在 `PromptTurnBuilder::resolve_prompt_input()` 中记录现有上下文段落来源、长度、顺序和 warnings，但不改变最终 prompt。
- 扩展 `prompt_audit.rs` 写入 `context_plan`、长度统计和 hash。
- 为 prompt plan 生成、path sanitization、老 audit 兼容写单元测试。

### Phase 2: 预算与预警

- 引入静态 runner capability 和默认预算配置。
- 对超预算 turn 生成 warning：长会话、过大附件、过长 skill snapshot、related skill hint 过多、Hone Cloud/local runner capability mismatch。
- 在管理端 LLM audit/logs 详情中展示 context plan。
- 在 public / IM 用户可见错误中加入安全、短文本的裁剪提示。

### Phase 3: 受控裁剪

- 将 low-priority 的 related skill hint、turn-0 listing、compact skill snapshot 迁到统一 allocator。
- 保持当前用户输入、当前附件证据、时间锚点、安全 policy 不可裁剪。
- 增加 regression：构造长会话 + 多附件 + skill hint，断言裁剪后当前输入仍最后出现，安全 policy 仍在 system prompt，warnings 写入 audit。

### Phase 4: 产品闭环

- 与 Session Memory Correction 联动：当 compact summary 质量或长度成为问题时，引导用户 review/recompact。
- 与 Investment Document Inbox 联动：大文档优先通过 document id / section excerpt 引用，而不是把全文塞进 prompt。
- 与 Run Trace Workbench 联动：trace detail 展示 context plan，不默认展示完整 prompt。
- 与 Usage Entitlement Ledger 联动：把 estimated tokens 作为运行前成本预警，不直接作为扣费真相。

## 验证方式

- Rust 单元测试：
  - context plan 能覆盖 static policy、session context、conversation summary、skill listing、related skill hint、recv_extra、current input。
  - 当前用户输入始终在 runtime input 最后，且 source priority 最高。
  - safety / finance policy、当前时间和权限边界不可裁剪。
  - 老 prompt audit 文件缺少 `context_plan` 时读取逻辑 graceful degrade。
  - runner capability 不泄露 API key、本地敏感路径或完整 OpenCode config。
- 回归脚本：
  - 新增 CI-safe 脚本构造长 session、invoked skill metadata、附件 preview，生成 prompt audit 后检查 `context_plan.warnings` 和 source 统计。
- 前端验证：
  - `bun run test:web` 覆盖 context plan 数据转换、warning label、老后端兼容。
  - 手工检查 `/llm-audit` 或 `/logs` 详情在桌面和移动视口不溢出。
- 产品验收：
  - 对一轮普通聊天，管理员能在 1 次点击内看到本轮上下文来源摘要。
  - 对一轮大 PDF / 长会话，系统能说明哪些内容被截断或建议 compact。
  - 对一轮 skill 调用，context plan 能区分 turn-0 listing、related hint 和完整 invoked skill prompt。

## 风险与取舍

- 风险：把 prompt 结构化会扩大改动面。取舍：Phase 1 只观测不裁剪，保持最终 prompt 字符串不变。
- 风险：token 估算不准。取舍：第一版只用粗略估算做 warning，不作为硬扣费或强拒绝依据。
- 风险：管理员 UI 展示过多 prompt 细节。取舍：默认展示 source 摘要、长度和 warnings；完整 prompt 仍走现有 admin-only prompt audit 权限。
- 风险：过早裁剪可能降低回答质量。取舍：先裁剪低优先级、可重新发现的提示，不裁剪当前输入、附件证据、安全边界和时间锚点。
- 风险：runner capability 可能和真实模型窗口不一致。取舍：capability 标记来源为 static / configured / probed，避免把静态建议当成强保证。
- 不做：不重写 runner 协议，不替代 Run Trace Workbench，不处理用户数据导出删除，不把 session correction 自动写入长期画像，不把 prompt budget 变成付费账单真相源。

## 与已有提案的差异

查重范围：

- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_session-memory-correction.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_skill-trust-marketplace.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`

差异结论：

- 与 `auto_p1_run_trace_workbench.md` 不重复：Run Trace 聚合运行后的日志、LLM audit、prompt audit、session 和 outbound 证据；本提案在运行前和 prompt audit 写入点建立上下文来源、预算、裁剪与预警对象。
- 与 `auto_p1_session-memory-correction.md` 不重复：Session Memory Correction 解决 compact summary 里的错误 claim 如何被用户修正；本提案只治理每轮上下文装配和预算，不提供记忆纠错 UI。
- 与 `auto_p1_investment_context_intake.md` 不重复：Investment Context Intake 解决 portfolio/profile/prefs/cron 的初始化缺口；本提案解决这些资产被带入 prompt 时的来源、长度、优先级和裁剪解释。
- 与 `auto_p1_investment_document_inbox.md` 不重复：Document Inbox 把用户文档资产化并提供受控 handoff；本提案只规定文档 preview / excerpt 进入本轮 prompt 的预算和可解释性。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：Usage Entitlement 解决权益、额度和成本归集；本提案只提供运行前 token/字符预警，不定义套餐或扣费。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该历史提案关注 skill 状态、multi-agent 阶段传递与 prompt 注入语义；本提案不改变 skill 调用模型，只把 skill listing / related hint / invoked prompt 纳入统一 context budget。

本轮只新增 proposal，不开始实施方案，因此不更新 `docs/current-plan.md`、`docs/repo-map.md` 或 `docs/invariants.md`。如果后续实际落地该提案，才需要新增 current plan，并在改变 prompt assembly、prompt audit schema、runner capability 或管理端诊断入口时同步更新 repo map 与相关长期约束。
