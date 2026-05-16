# Proposal: Company Portrait Health Contract and Review Cadence

status: proposed
priority: P1
created_at: 2026-05-16 14:05:23 +0800
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
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_cross-company-thesis-map.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `memory/src/company_profile/types.rs`
- `memory/src/company_profile/storage.rs`
- `memory/src/company_profile/markdown.rs`
- `memory/src/company_profile/transfer.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `packages/app/src/context/company-profiles.tsx`
- `packages/app/src/components/company-profile-detail.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `skills/company_portrait/SKILL.md`

## 背景与现状

Hone 已经明确把 company portraits 作为用户可见长期研究记忆的核心资产，而不是普通缓存：

- `docs/invariants.md` 规定用户可见长期研究记忆目前只保留 company portraits，且画像应保留投资主线、关键经营指标、估值框架、风险账本、证伪条件和事件证据。
- `memory/src/company_profile/types.rs` 中 `ProfileMetadata` 已有 `status`、`tracking.enabled`、`tracking.cadence`、`tracking.focus_metrics`、`updated_at`、`last_reviewed_at` 等字段，说明画像天然支持“跟踪与复审”语义。
- `CompanyProfileStorage` 能按 actor sandbox 读取、列出、导入、导出、删除画像，并通过 relaxed parser 兼容缺少 frontmatter 的旧 Markdown。
- `crates/hone-web-api/src/routes/company_profiles.rs` 已提供 admin 端 actor-scoped 画像列表、详情、导入预览、导入应用、导出和删除。
- Public `/portfolio` 通过 `public_digest.rs` 读取当前 web actor 的画像摘要、持仓、投资主线蒸馏结果和 skipped ticker，并允许用户查看只读画像。
- `skills/company_portrait/SKILL.md` 已定义画像维护工作流，但当前质量约束主要停留在 prompt 层，由 agent 自觉执行。

这套基础说明 Hone 已有长期记忆的文件真相源和展示入口。但系统还没有一个一等的“画像健康度”契约：一个 profile 是否结构完整、是否过期、是否能被 mainline distill 可靠识别、事件时间线是否有证据、关键 metrics 是否按 cadence 复审，目前需要人打开 Markdown 自己判断。

对投资助手来说，这会让长期记忆逐渐变成“看起来存在但可信度不明”的资产。尤其当用户依赖画像驱动 digest、公司主线、跨公司地图、研究报告 handoff 和多渠道回答时，画像本身的健康状态应成为产品可见对象。

## 问题或机会

1. **画像存在不等于画像可用。**  
   `list_profiles_raw()` 可以显示 legacy Markdown，public 端也能展示只读内容，但缺 ticker、缺 frontmatter、缺证伪条件、缺风险、缺 metrics、事件 refs 为空或 `last_reviewed_at` 长期为空时，系统仍把它当作正常画像展示。

2. **`tracking` 和 `last_reviewed_at` 没有形成复审闭环。**  
   metadata 已有 cadence 和 focus metrics，但 UI、API、public portfolio 和 agent workflow 尚未把它们转成“本周到期复审 / 已逾期 / 无需跟踪 / 已完成复审”的状态。

3. **导入导出只解决迁移，不解决质量门槛。**  
   `preview_import_bundle()` 能识别冲突并支持 replace/skip，但不会告诉用户导入包中哪些画像缺关键字段、哪些 events 缺 refs、哪些 profile 无法被 ticker/mainline 识别。

4. **Agent 更新画像后缺少可机读验收。**  
   `company_portrait` skill 要求保存研究路径和事件影响，但当前没有 lint/check API 验证本次更新是否真正补了 `last_reviewed_at`、新增 event、写明证伪条件或保留 evidence refs。

5. **Admin 和用户端缺少维护优先级。**  
   当一个 actor 有几十个 profile 时，管理员或用户不知道先修哪个：是持仓但无 ticker 的画像、超过 cadence 未复审的画像、事件很多但 mainline 为空的画像，还是缺 refs 的高价值标的。

这是 P1。它不如输出安全门或 operator access 那样属于立即安全风险，但会显著影响核心记忆可信度、digest 个性化质量、用户留存、客服排障和后续商业化交付质量。

## 方案概述

新增 **Company Portrait Health Contract and Review Cadence**：一层 actor-scoped 的只读质量评估与复审编排能力，不替代 `profile.md` 真相源，不让 UI 直接编辑画像正文。

核心对象：

1. `CompanyProfileHealth`
   单个 profile 的健康评估，包含 identity、schema、content、event trail、review cadence、mainline readiness 和 import compatibility。

2. `CompanyProfileHealthIssue`
   可执行问题项，例如 `missing_stock_code`、`legacy_no_frontmatter`、`missing_mainline_section`、`missing_disconfirming_conditions`、`event_without_refs`、`review_overdue`、`tracking_focus_metric_missing`、`distill_unmatched_ticker`。

3. `ReviewCadenceState`
   从 `tracking.enabled`、`tracking.cadence`、`last_reviewed_at`、`updated_at`、profile mtime 和 event mtime 推导：`not_tracked`、`due_soon`、`overdue`、`reviewed_recently`、`unknown`.

4. `HealthContractVersion`
   画像健康规则的版本号。第一版只做结构与 freshness 检查；后续可升级规则，但旧画像不能因为规则升级被误删或静默改写。

5. `ProfileReviewPrompt`
   当用户或管理员点击“复审画像”时生成的受控 agent prompt，要求 agent 读取当前 profile 和 events，按 focus metrics 复核，必要时通过 `company_portrait` skill 更新，而不是 UI 直接改 Markdown。

原则：

- `profile.md` 与 `events/*.md` 仍是长期记忆真相源。
- 健康评估是派生读模型，可删除重建。
- 第一版只评估和编排，不自动重写画像。
- 对 legacy Markdown 保持兼容：允许读取，但明确标记需要补 metadata。
- 健康分数不能伪装成投资建议，只表示文档维护质量和可用性。

## 用户体验变化

### 用户端

- Public `/portfolio` 在公司画像列表旁增加健康状态：
  - `健康`：结构完整，近期复审，主线可蒸馏。
  - `待复审`：超过 cadence 或存在新的事件但未更新 `last_reviewed_at`。
  - `缺信息`：缺 ticker、缺主线、缺证伪条件或缺关键 metrics。
  - `旧格式`：legacy Markdown 可读，但建议补 frontmatter。
- 用户点击某个状态后看到最多 3 个关键问题和下一步动作，例如“让 Hone 复审 NVDA 画像”。
- 空泛提示从“请通过 chat 修改画像”升级为具体 prompt：“请按我的 weekly tracking metrics 复审 NVDA，重点检查毛利率、HBM 供需和资本开支证据；如果长期主线未变，只追加事件说明。”
- 用户不会在 public 端直接编辑 Markdown；所有修改仍进入 chat/agent。

### 管理端

- Admin company profile 视图增加 `Health` 总览：
  - actor 画像总数、健康数、逾期复审数、legacy 格式数、缺 ticker 数、events 缺 refs 数。
  - 按 severity 排序的问题列表，可筛选 `portfolio holdings only`、`tracking enabled only`、`legacy only`。
- 导入画像包的 preview 中增加质量摘要：即将导入的画像是否缺 frontmatter、ticker、主线、事件 refs，以及导入后是否会让某些既有画像健康度下降。
- 详情页右侧显示当前 profile 的 health issues、复审 cadence、focus metrics 和建议 agent prompt。
- 管理员可以批量生成 review prompts，但不批量自动改写 profile。

### 桌面端

- Desktop bundled/remote 复用 Web console，不新增本地存储。
- Dashboard 可以显示“画像健康：12 个健康 / 3 个待复审 / 2 个旧格式”，点击进入 company profiles。
- 本地单用户模式下，逾期复审可以成为桌面工作台的轻量待办，而不是隐藏在 Markdown mtime 里。

### 多渠道

- Feishu / Telegram / Discord 私聊中，用户问“哪些公司画像该更新”时，agent 调用 health summary，只返回最重要的 3 项。
- 用户说“复审 MU 画像”时，agent 先读取 health issues 和 tracking focus metrics，再执行 `company_portrait` workflow。
- 群聊默认不暴露个人 actor 的画像健康，除非当前会话明确是共享 group `SessionIdentity` 且已有相应 actor scope。

## 技术方案

### 1. 画像健康评估器

在 `memory/src/company_profile/` 增加纯读评估模块，例如 `health.rs`：

```rust
pub struct CompanyProfileHealth {
    pub profile_id: String,
    pub stock_code: String,
    pub generated_at: String,
    pub contract_version: String,
    pub status: CompanyProfileHealthStatus,
    pub score: u8,
    pub review: ReviewCadenceState,
    pub issues: Vec<CompanyProfileHealthIssue>,
    pub source_updated_at: String,
    pub last_reviewed_at: Option<String>,
}
```

第一版规则只依赖现有文件和 parser：

- identity：company name、stock code、aliases 是否可识别。
- schema：frontmatter 是否存在；legacy fallback 是否触发；metadata 时间是否合理。
- content：是否包含投资主线、风险、证伪条件、估值框架、关键 metrics 等约定 section。
- event trail：events 数量、发生时间、captured_at、`mainline_impact`、`refs` 是否为空。
- review：`tracking.enabled` 为 true 时，根据 `cadence` 与 `last_reviewed_at` 推导 due/overdue。
- mainline readiness：stock code 或 ticker 能否与 portfolio/digest mainline 使用的 symbol 对齐。

健康评估不读取外部行情，不调用 LLM，不做公司好坏判断。

### 2. API

新增只读优先路由：

- `GET /api/company-profiles/health?channel=&user_id=&channel_scope=`
- `GET /api/company-profiles/{id}/health?channel=&user_id=&channel_scope=`
- `GET /api/public/company-profiles/health`
- `POST /api/company-profiles/{id}/review-prompt`
- `POST /api/public/company-profiles/{id}/review-prompt`

public 路由从 `hone_web_session` 推导 actor，不能跨 actor 查询。Admin 路由沿用现有 `require_actor`。

`review-prompt` 只返回 prompt draft 或创建 chat draft，不直接写 profile。它应包含：

- profile id / ticker
- top health issues
- tracking focus metrics
- last reviewed time
- 最近 events 摘要
- company_portrait skill 的约束：只写长期有用内容，保留证据，若 thesis 未变也要说明复审结论。

### 3. 导入导出与 legacy 兼容

扩展 `preview_import_bundle()` 的响应或另加 `health_preview`：

- 对每个 imported profile 运行 health evaluator。
- conflict preview 中显示“新包更旧 / 新包缺 ticker / 新包缺 refs / 新包会替换掉更健康画像”的风险。
- `replace_all` 前展示健康度下降数量，避免用户误导入低质量包覆盖长期资产。

导出 bundle 可在 manifest 中增加 health summary，但不阻断导出。

### 4. Agent 与 skill 集成

更新 `skills/company_portrait/SKILL.md` 或其 references：

- 明确每次复审应更新 `last_reviewed_at`，除非只是读取不改。
- 如果 `tracking.enabled=true`，输出必须覆盖 `tracking.focus_metrics`。
- 事件文档必须尽量保留 refs；没有 refs 时要说明来源缺口。
- 复审未改变 thesis 时，仍应追加简短 event 或在 profile 中记录 review note，避免 `last_reviewed_at` 与证据脱节。

新增 `company_profile_health` tool 可以让 agent 在对话里读取当前 actor 的健康摘要，而不需要直接解析所有 Markdown。

### 5. 前端落点

- `packages/app/src/context/company-profiles.tsx`：加载 profile 列表后并行加载 health summary，按 profile id 关联。
- `packages/app/src/components/company-profile-detail.tsx`：详情页显示 health panel、issues、review cadence、生成复审 prompt 按钮。
- `packages/app/src/pages/public-portfolio.tsx`：在只读画像入口旁显示健康 badge 和逾期复审提示。
- `packages/app/src/lib/api.ts` / types：增加 `CompanyProfileHealth` 类型和 API client。

UI 应保持克制：默认只显示 badge 和最重要问题，不把健康检查做成大而空的评分游戏。

## 实施步骤

### Phase 1: 纯读健康评估

- 增加 `company_profile::health` 类型和评估器。
- 为 metadata、legacy Markdown、section 缺失、event refs、review cadence 写单元测试。
- 增加 admin health API。
- 在 admin company profile detail 展示 health badge 和 issue 列表。

### Phase 2: Public 与导入预览

- 增加 public health API，只暴露当前用户 actor。
- Public `/portfolio` 显示画像健康状态和复审入口。
- 导入 preview 增加 health summary 和 replace 风险提示。
- 前端 model 测试覆盖 health issue 排序和移动端摘要裁剪。

### Phase 3: 复审 prompt 与 agent 工具

- 增加 review prompt API，生成受控 prompt draft。
- 增加 `company_profile_health` tool，允许多渠道查询健康摘要。
- 更新 `company_portrait` skill references，明确复审输出和 `last_reviewed_at` 约束。
- Agent 完成复审后，健康状态应从 `overdue` 或 `missing_*` 转为更具体的新状态，而不是只生成一段回答。

### Phase 4: 运营指标与联动

- Admin 汇总 actor 级健康指标：健康率、逾期率、legacy 率、events 缺 refs 比例。
- 与 evidence review queue 联动：open counter evidence 可作为 `review_due_reason`，但不由 health evaluator 自己创建 evidence。
- 与 cross-company thesis map 联动：map refresh 前先提示哪些 source profile 健康度不足。
- 与 research artifact handoff 联动：报告沉淀后可以重新计算相关 profile health。

## 验证方式

- Rust 单元测试：
  - legacy plain Markdown 可读但生成 `legacy_no_frontmatter` issue。
  - 缺 stock code、缺主线 section、缺证伪条件、event refs 为空分别生成稳定 issue code。
  - `tracking.enabled=true` 且 `last_reviewed_at` 超过 weekly/monthly cadence 时生成 `review_overdue`。
  - `tracking.enabled=false` 时不生成逾期复审 issue。
  - 健康评估只读，不修改 `profile.md` 或 `events/*.md`。
- Web API 测试：
  - admin 可查询指定 actor health。
  - public 只能查询当前 session actor。
  - profile 不存在返回 404，不因单个坏 event 文件导致整个 actor health 500。
- 前端测试：
  - health issue 排序、badge 状态、导入预览风险文案有 model 测试。
  - public `/portfolio` 在无画像、legacy 画像、逾期复审画像三种状态下都有明确空态或提示。
- 手工验收：
  - 构造一个完整画像、一个 legacy 画像、一个 tracking overdue 画像、一个 events 缺 refs 画像，admin 和 public 都能显示不同状态。
  - 点击 review prompt 时生成的 prompt 包含 profile id、focus metrics、top issues 和 company_portrait 约束。

## 风险与取舍

- 风险：健康评分被用户误解为公司投资质量评分。取舍：文案必须写成“画像维护质量 / 可用性”，不使用“好公司 / 坏公司”语义。
- 风险：规则过严导致老画像全部红灯。取舍：legacy 与 missing issues 分级展示，默认不阻断读取和导入。
- 风险：agent 为了消除 issue 机械填充空话。取舍：review prompt 要求引用 evidence 或明确写出“缺证据”，而不是补模板句。
- 风险：导入 preview 变复杂。取舍：只显示会影响可用性的 top issues，不在导入弹窗展示完整 lint 报告。
- 风险：与 evidence review queue 重叠。取舍：health 只判断 profile 资产是否健康和是否到期复审；evidence queue 负责具体事件是否需要处理。
- 不做：不自动改写画像、不新增 KB 页面、不把 health evaluator 接外部行情、不把完整 profile markdown 公开分享、不跨 actor 合并健康状态。

## 与已有提案的差异

查重范围：

- `docs/proposal/` 全部现有自动提案，包括 safety gate、operator access、mutation ledger、delivery decision loop、evidence review queue、investment context intake、cross-company thesis map、portfolio exposure radar、research artifact library、source provenance、user data trust center、shareable briefs 等。
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 不重复 `auto_p1_evidence_review_queue.md`：该提案把事件转成待复盘证据；本提案评估单个 company portrait 文件和事件时间线本身是否结构完整、是否到期复审、是否可被蒸馏。
- 不重复 `auto_p1_cross-company-thesis-map.md`：该提案生成跨公司共享主线地图；本提案不做跨公司比较，只给每个 profile 一个维护质量契约和 review cadence。
- 不重复 `auto_p1_investment_context_intake.md`：该提案解决新用户如何建立 portfolio/profile/prefs/task；本提案假设 profile 已存在，解决存量画像长期健康维护。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：该提案从 portfolio 角度做组合暴露和数据质量；本提案从 company portrait 文档角度做结构、证据、复审和导入质量。
- 不重复 `auto_p1_research_artifact_library.md`：该提案让研究报告成为长期交付物并 handoff 到画像；本提案负责报告或 agent 更新后，画像资产是否达到可维护状态。
- 不重复 `auto_p1_source-provenance-freshness.md`：该提案跟踪外部数据源的新鲜度和血缘；本提案只判断画像文档中的 refs、事件和复审时间，不建立外部 source observation registry。
- 不重复历史 `skill-runtime-multi-agent-alignment.md`：该提案关注 skill runtime 与 runner/stage 对齐；本提案只把 company portrait 作为长期研究资产增加健康检查和复审编排。
