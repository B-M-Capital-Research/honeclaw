# Proposal: Research Artifact Library and Handoff Loop

status: proposed
priority: P1
created_at: 2026-05-05 17:04:14 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `config.example.yaml`
- `crates/hone-core/src/config/server.rs`
- `crates/hone-web-api/src/routes/research.rs`
- `crates/hone-web-api/src/routes/mod.rs`
- `packages/app/src/context/research.tsx`
- `packages/app/src/components/research-detail.tsx`
- `packages/app/src/components/research-preview.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/persist.ts`
- `packages/app/src/lib/types.ts`
- `memory/src/company_profile/{types,storage,transfer}.rs`
- `skills/company_portrait/SKILL.md`

## 背景与现状

Hone 当前已经有两类研究资产：

- 长期记忆资产：公司画像以 `company_profiles/<profile_id>/profile.md` 和 `events/*.md` 存在 actor sandbox 中，属于可持续维护的投资主线、风险、证据和事件时间线。
- 深度研究报告：管理端 `ResearchPage` 通过 `crates/hone-web-api/src/routes/research.rs` 代理外部 `research_api_base`，支持启动任务、轮询状态、生成 / 下载 PDF。前端 `packages/app/src/context/research.tsx` 把任务列表和完成后的 `answer_markdown` 写入 browser `localStorage`，`ResearchDetail` / `ResearchPreview` 负责展示和导出体验。

这说明 Hone 已经在产品上出现了“长文研究交付物”入口，但它还没有成为 Hone 自己的长期资产层。当前实现有几个明显边界：

- 研究任务是 admin console 的本地浏览器状态，不按 `ActorIdentity` 持久化；换浏览器、换设备或清 localStorage 后，报告索引会丢失。
- 后端只是安全代理外部 API，并校验 `research_api_base` 只允许 HTTPS 远端或 HTTP loopback；没有本地 `ResearchArtifact` 元数据、状态机、权限边界和审计。
- 完成报告的 Markdown / PDF 没有和公司画像、public `/portfolio`、会话历史、scheduled task、digest thesis 蒸馏形成交接关系。
- `config.example.yaml` 有 `web.research_api_key`，但当前研究代理路径没有把它产品化成能力状态、密钥缺失提示、成本/权益消耗或可诊断错误。
- 多渠道 `/report` shortcut 和 Web research 页在产品语义上接近，但仓库文档只说明 `/report` 是 local private workflow runner bridge，还没有统一“研究报告交付物”对象。

对投资研究助手来说，深度研究报告不应该只是一次性 PDF 预览。它应该成为用户长期研究系统里的可引用交付物：能回到公司画像，能被会话继续追问，能被管理员审阅和复用，能被用户知道哪些结论已经沉淀、哪些仍只是临时报告。

## 问题或机会

### 问题

1. 深度研究结果没有服务端真相源。
   `ResearchTask.answer_markdown` 当前由前端轮询完成后写入 localStorage。它适合 UI 恢复，但不是可审计、可迁移、可共享、可被 agent 引用的产品资产。

2. 报告与长期公司画像断开。
   公司画像强调 thesis、证据、证伪条件和事件时间线；深度研究报告通常包含大量基础事实、行业结构、财务分析和风险，但完成后没有一个明确动作把可沉淀内容交给 `company_portrait` skill。

3. 用户端无法消费报告价值。
   Public `/portfolio` 能展示只读画像和蒸馏投资主线，但深度研究报告只在管理端 research 模块可见。对邀请制 Web 用户来说，这降低了“我让 Hone 做了一份研究，之后还能在哪里找回来”的确定性。

4. 管理端无法运营研究交付物。
   管理员只能看到当前浏览器中的研究任务列表，无法按 actor、ticker、状态、耗时、失败原因、来源入口、是否已沉淀到画像来管理。

5. 桌面与多渠道缺少统一交接。
   桌面 bundled/remote 只是承载 Web console；Feishu / Telegram / Discord 的 `/report` 或自然语言研究需求即使完成，也没有统一 artifact id 可回到 Web/desktop 查看、继续追问或写入画像。

### 机会

新增 Research Artifact Library 可以直接增强 Hone 的核心定位：

- 用户留存：研究报告从“一次生成”变成“可回看、可追问、可沉淀”的长期资产。
- 研究质量：报告中的 thesis-changing 结论进入公司画像前有明确 review / handoff，而不是让模型下次凭聊天记忆碰运气。
- 商业化：深度研究天然适合成为高价值权益项，可和 usage entitlement proposal 后续衔接。
- 运维：外部研究 API、PDF 生成、Markdown 渲染、actor 归属、失败任务都有可诊断对象。
- 增长体验：public 用户可在 `/portfolio` 或 `/me` 看到“你的研究库”，更容易理解 Hone 不是普通聊天窗口。

## 方案概述

新增一个 actor-scoped 的 `ResearchArtifact` 层，把外部深度研究任务、报告 Markdown/PDF、画像交接状态和用户可见入口统一起来。

核心对象建议：

1. `ResearchArtifact`
   服务端持久化对象，记录 artifact id、actor、company/ticker、source、status、external task id、created/updated/completed 时间、Markdown 路径、PDF 路径、错误、hash、大小和权限。

2. `ResearchHandoff`
   报告到长期记忆的交接记录，说明哪些结论建议写入公司画像、是否已交给 agent、是否已创建 profile event、是否被管理员/用户标记为“只保留报告，不沉淀”。

3. `ResearchArtifactIndex`
   Admin/public/desktop 共享的只读列表 API，按 actor、ticker、status、source、created_at 查询。

4. `ResearchFollowupPrompt`
   从报告生成继续追问或画像更新的结构化 prompt，不直接 UI 编辑 `profile.md`，而是把报告摘要、引用位置、目标 ticker 和约束交给现有 `company_portrait` skill。

关键原则：

- 不替换外部 research API；第一版仍可代理 `research_api_base`。
- 不把深度研究报告直接当成公司画像。画像仍是长期投资记忆的真相源，报告是交付物和证据输入。
- 不让 UI 直接改画像正文。沉淀动作仍走 agent-mediated file operation。
- 不打破 actor 隔离。第一版严格按当前 actor 存储和读取；跨渠道复用等待 linked workspace 提案。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加“研究报告”区块：按 ticker 展示最近完成的深度研究、状态、完成时间、是否已沉淀到画像。
- 用户点击报告可以在 Web 中阅读 Markdown 或下载 PDF；如果报告尚未完成，看到清晰进度和失败原因。
- 报告详情提供两个主动作：
  - `继续追问这份报告`：进入 `/chat`，预填带 artifact id 和报告摘要的上下文。
  - `让 Hone 更新公司画像`：生成一条明确的 `company_portrait` 交接 prompt，要求只沉淀长期 thesis、证据、风险和证伪条件，不把全文塞进画像。
- 空状态不鼓励用户生成泛泛报告，而是结合持仓 / 画像缺口提示：“这些持仓还没有深度研究交付物”。

### 管理端

- `ResearchPage` 从纯前端 localStorage 列表升级为服务端 artifact library。
- 列表支持 actor、ticker、status、source、has_handoff、created_at 过滤。
- 详情页展示：
  - 外部 task id、耗时、报告 hash、Markdown/PDF 是否存在。
  - 关联公司画像、最近 profile updated time、是否有 handoff 待处理。
  - 失败原因：外部 API 不可达、base URL 配置错误、PDF 生成失败、report 文件缺失、权限不匹配。
- 管理员可以把一份报告转为“画像更新任务草稿”，但仍需要 agent 执行，不在 UI 里直接合并 Markdown。

### 桌面端

- Desktop bundled/remote 模式复用同一 Web API。桌面只需要显示 backend capability：research configured / missing key / external API unreachable。
- Dashboard 或 Research 模块可以显示“最近完成报告”和“待沉淀报告”badge。
- 本地 packaged 用户如果没有配置 `research_api_base` 或 API key，看到的是可操作配置缺口，而不是研究页静默失败。

### 多渠道

- `/report 公司名` 或用户在 IM 中请求“做一份深度研究”时，回复里返回短 artifact reference，例如 `research:MU:20260505`，并提示可在 Web/desktop 查看完整报告。
- 报告完成后可推送摘要和链接；长 Markdown/PDF 不直接塞进 IM。
- 用户在 IM 中说“把刚才那份报告写入画像”，agent 通过 artifact id 读取受控摘要，再调用 `company_portrait` skill 处理。

## 技术方案

### 1. 新增 research artifact 存储

建议在 `memory` 增加 `research_artifact` 模块，使用 SQLite 存元数据，文件内容仍落在 actor-scoped runtime/artifact 目录：

```text
research_artifacts (
  artifact_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  company_name TEXT NOT NULL,
  ticker TEXT,
  source TEXT NOT NULL,
  external_task_id TEXT,
  external_task_name TEXT,
  status TEXT NOT NULL,
  markdown_path TEXT,
  pdf_path TEXT,
  markdown_sha256 TEXT,
  pdf_sha256 TEXT,
  error_message TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT
)

research_handoffs (
  handoff_id TEXT PRIMARY KEY,
  artifact_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  profile_id TEXT,
  status TEXT NOT NULL,
  agent_session_id TEXT,
  profile_event_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

文件路径建议在 data root 下使用：

```text
data/research_artifacts/<channel>/<scope__user>/<artifact_id>/
  report.md
  report.pdf
  metadata.json
```

路径只在服务端使用。前端通过 `/api/research/artifacts/:id/markdown` 和 `/api/research/artifacts/:id/pdf` 读取，不暴露本地绝对路径。

### 2. 改造现有 research proxy 为 artifact-aware

现有 API 保持兼容：

- `POST /api/research/start`
- `GET /api/research/status/:task_id`
- `POST /api/research/generate-pdf`
- `GET /api/research/download-pdf`

新增或扩展：

- `POST /api/research/artifacts/start`
  - admin 可指定 actor；public 端从 session 推导 actor。
  - 创建 `ResearchArtifact(status=running)` 后再调用外部 research API。
- `GET /api/research/artifacts`
  - admin 查询指定 actor 或全部。
- `GET /api/research/artifacts/:artifact_id`
- `GET /api/research/artifacts/:artifact_id/markdown`
- `GET /api/research/artifacts/:artifact_id/pdf`
- `POST /api/research/artifacts/:artifact_id/refresh-status`
- `POST /api/research/artifacts/:artifact_id/handoff`

兼容策略：

- 旧前端仍可用 task id 轮询；新前端优先 artifact id。
- 外部 API 返回 `answer_markdown` 时，后端保存到 artifact 目录并记录 hash。
- 旧 localStorage 任务可以在前端提供一次性“导入到研究库”动作，但不要自动上传未知来源的大段 Markdown。

### 3. 权限与配置状态

- Admin API 沿用 `/api` bearer token 和 actor query。
- Public API 从 `hone_web_session` 推导当前 `web` actor，不接受 query actor。
- IM `/report` 只为当前 channel actor 创建 artifact；群聊需要明确 group/session 规则，默认不把群聊报告写入个人 actor。
- `research_api_base` 继续执行当前 URL 安全校验；`research_api_key` 应通过服务端注入外部请求 header，不返回前端。
- `/api/meta` 或 `/api/research/capabilities` 返回：
  - `configured`: base URL 是否非默认占位、key 是否存在或当前外部服务是否允许匿名。
  - `reachable`: 最近一次健康检查。
  - `supports_pdf`: 外部 API 是否可生成 PDF。

### 4. 与公司画像的交接

新增 handoff 不直接改 `profile.md`。建议流程：

1. 用户或管理员在报告详情点击 `handoff_to_profile`。
2. 后端生成 `ResearchHandoff(status=draft)`，包含 artifact id、目标 ticker/profile_id、报告摘要、关键章节索引。
3. Chat prompt 或 tool 返回：
   - artifact 摘要
   - report markdown 的受控 excerpt
   - 当前公司画像摘要
   - 明确指令：只沉淀长期 thesis / risk / evidence / disconfirming conditions；若报告没有改变长期判断，只追加 research trail 或标记不更新。
4. Agent 通过 `company_portrait` skill 执行文件更新。
5. 更新成功后，工具或后端根据 profile mtime / event id 把 handoff 标记为 `profile_updated`。

这与 evidence review queue 的关系是互补的：evidence queue 处理单条事件是否改变 thesis；research artifact handoff 处理一份完整研究报告如何进入长期记忆。

### 5. 前端改造

前端新增：

- `packages/app/src/lib/research-artifacts.ts`
- `packages/app/src/context/research-artifacts.tsx`
- `packages/app/src/pages/research.tsx` 继续承载页面，但数据源从 localStorage-first 改为 API-first。

保留 localStorage 的用途：

- 只缓存 UI 选择、临时排序、旧任务迁移提示。
- 不再把 `answer_markdown` 作为唯一恢复来源。

Public 端可先只在 `/portfolio` 中展示最近报告列表，完整报告详情复用现有 Markdown preview 组件。

## 实施步骤

### Phase 1: Artifact 真相源

- 在 `memory` 新增 research artifact SQLite 存储、类型和单元测试。
- 新增 artifact-aware admin API：start、list、detail、status refresh、markdown read。
- `handle_research_status` 在完成时保存 `answer_markdown` 到 artifact 目录。
- Research 页面改为优先读取服务端 artifacts，旧 localStorage 只作为迁移提示。

### Phase 2: Public/desktop 可见性

- 新增 public artifact list/detail API，限定当前 web actor。
- Public `/portfolio` 展示当前用户相关报告和状态。
- Desktop dashboard 显示 research capability 状态和最近报告 badge。
- 配置缺失时给出明确提示：`research_api_base` 仍是占位、API key 缺失、外部服务不可达。

### Phase 3: Handoff loop

- 新增 `ResearchHandoff` 表和 action API。
- 报告详情支持 `继续追问` 与 `更新公司画像` 两个动作。
- 增加 agent/tool 读取 artifact 摘要的能力，确保不把本地绝对路径泄露给用户。
- company portrait 更新后允许 handoff 标记为 `profile_updated`，并在报告库展示。

### Phase 4: 指标与运营

- 管理端增加研究库筛选、失败原因聚合、待沉淀报告计数。
- 记录报告耗时、成功率、PDF 生成率、handoff 转化率、profile_updated 率。
- 后续和 usage entitlement ledger 对接，把深度研究作为单独权益项或高成本能力项。

## 验证方式

- Rust 单元测试：
  - artifact id 幂等 / 唯一，actor 字段完整保存。
  - public actor 不能读取其他 actor 的 artifact。
  - 外部任务完成后 Markdown 写入 actor-scoped artifact 目录并记录 sha256。
  - handoff 状态机拒绝非法跳转，例如 `draft -> profile_updated` 必须有关联 profile 或明确确认。
- Web API 测试：
  - admin 可按 actor/ticker/status 查询。
  - public API 忽略 query actor，只读取当前登录用户。
  - `research_api_base` 非 HTTPS 远端 / 非 loopback HTTP 继续被拒绝。
  - 外部 API 不可达时 artifact 进入 `error`，错误可读，不丢失任务对象。
- 前端验证：
  - `bun run test:web` 覆盖 research artifact 数据转换、状态展示、旧 localStorage 迁移提示。
  - 手工检查 admin Research 页面、public `/portfolio` 报告区、桌面 bundled/remote 模式下不溢出。
- 产品指标：
  - 完成报告可在换浏览器后恢复。
  - 报告到画像 handoff 的转化率可观测。
  - 管理员能在一个页面看到失败研究任务原因和待沉淀报告。

## 风险与取舍

- 风险：把外部研究报告变成第二套长期记忆，和公司画像冲突。取舍：报告库只做交付物和证据输入，长期投资主线仍以公司画像为源。
- 风险：报告 Markdown/PDF 占用本地磁盘。取舍：记录 hash/size，增加保留策略；默认保留最近 N 份或按 actor 配额，删除时只删 artifact，不删画像。
- 风险：外部 research API schema 变化导致状态解析脆弱。取舍：artifact 状态机允许 degraded external payload，并把原始错误保存到 artifact metadata。
- 风险：public 用户误以为报告是投资建议。取舍：报告详情沿用 Hone 的投资纪律约束，强调研究材料和风险框架，不生成买卖指令。
- 风险：handoff prompt 把全文塞进上下文导致成本过高。取舍：后端生成章节摘要和可控 excerpt；完整 Markdown 只在必要时按段读取。
- 不做：不内置新的深度研究模型、不绕过现有 `company_portrait` skill、不做跨 actor 共享、不接支付、不把 PDF 作为默认 IM 投递内容。

## 与已有提案的差异

查重范围：

- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 与 `auto_p1_evidence_review_queue.md` 不重复：该提案处理单条事件 / digest item 是否需要复盘；本提案处理完整深度研究报告的服务端留存、阅读、继续追问和画像交接。
- 与 `auto_p1_investment_context_intake.md` 不重复：该提案解决新用户持仓、画像、偏好和任务的初始化缺口；本提案解决研究交付物完成后的持久化和后续沉淀。
- 与 `auto_p1_run_trace_workbench.md` 不重复：该提案面向 agent 执行诊断；本提案面向用户可见研究资产，不追踪 runner 内部事件。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：该提案建立权益和成本账本；本提案可在后续接入权益，但第一版聚焦报告资产与 handoff。
- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案解决跨渠道真实用户归属；本提案第一版严格 actor-scoped。
- 与历史 `desktop-bundled-runtime-startup-ux.md` 和 `skill-runtime-multi-agent-alignment.md` 不重复：本提案不处理桌面 sidecar 启动治理，也不改变 skill runtime 调用语义。

本轮选择该主题，是因为当前仓库已经有可运行的深度研究代理和前端报告预览，但报告仍停留在浏览器本地状态，尚未成为 Hone 的长期研究资产。把它补成 artifact library 能直接提升用户留存、管理端运营和公司画像质量闭环。
