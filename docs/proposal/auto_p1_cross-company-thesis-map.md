# Proposal: Cross-Company Thesis Map for Consistent Investment Memory

status: proposed
priority: P1
created_at: 2026-05-08 05:03:02 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `memory/src/company_profile/types.rs`
- `memory/src/company_profile/storage.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/company-profile-detail.tsx`
- `skills/company_portrait/SKILL.md`

## 背景与现状

Hone 当前已经把长期投资记忆收敛到 actor-scoped company portraits：

- `memory/src/company_profile/types.rs` 中的 `CompanyProfileDocument` 以单家公司为单位保存 `profile.md` 与 `events/*.md`，并在 metadata 里保留 `sector`、`industry_template`、`tracking`、`last_reviewed_at` 等字段。
- `memory/src/company_profile/storage.rs` 能按 actor 列出画像空间、按公司加载画像、导入/导出画像包，并保持 `company_profiles/<profile_id>/profile.md` 是文档型真相源。
- `skills/company_portrait/SKILL.md` 要求系统性公司研究默认沉淀到长期画像，且事件只记录净新增事实与长期判断变化。
- `docs/invariants.md` 已经明确约束：分析公司前，如果当前 actor sandbox 里已有相似公司画像，Hone 应先检查相关画像并复用同一宏观 / 行业叙事；跨公司差异应由公司级事实解释，而不是静默重写共享主线。
- Public `/portfolio` 会展示每个 ticker 的投资主线、全局投资风格和只读公司画像；Admin `/users` 下的 company profile 视图可以查看、导入、导出、删除 actor 空间里的画像。

这些设计让单家公司画像逐步成熟，但当前系统还缺一层“跨公司主线地图”。实际运行中，用户的投资判断往往不是孤立 ticker：半导体设备、GPU 供应链、SaaS 席位扩张、国防工业预算、消费品牌渠道库存，都会在多个公司画像之间共享同一组行业假设。现在这些共享假设只能散落在各个 `profile.md` 里，靠 agent 每次临场 `rg --files company_profiles` 后自行发现。

结果是：仓库已经把“必须检查相似画像”写成长期契约，但产品与数据结构还没有给 agent、用户和管理员一个可见、可验证、可复用的跨公司一致性层。

## 问题或机会

1. **同一行业主线容易在不同画像里漂移。**
   例如一个 actor 同时持有或关注 NVDA、AMD、TSM、ASML、AVGO。若每次只围绕单家公司更新画像，宏观需求、资本开支周期、先进制程瓶颈、AI 训练/推理迁移等共享假设可能在不同文件里出现不一致版本。后续回答会显得“每问一家公司就换一套世界观”。

2. **跨公司差异缺少显式解释。**
   Hone 的理想状态不是把同一行业里的公司说成一样，而是说明差异来自哪些公司级变量：毛利结构、客户集中度、供应链位置、资本强度、估值敏感性、管理层执行、监管或地缘敞口。当前这些变量没有被抽成稳定比较面。

3. **Public 与 Admin 只能看单 ticker 资产。**
   `/portfolio` 与公司画像详情能让用户看到每个 ticker 的主线，但不能回答“我的半导体持仓共享哪些假设，哪些是互相矛盾的？”管理员也难以排查某个 actor 的画像是不是已经长期未统一。

4. **Digest 和主动通知缺少共享主线上下文。**
   evidence review queue 与 delivery decision loop 关注事件是否应推送、是否要复盘，但如果同一条行业事件会同时影响多个公司，系统缺少一个“行业主线已受影响，哪些公司需要后续检查”的中间判断面。

5. **这直接影响核心可信度。**
   Hone 的产品定位是投资纪律与长期判断维护，而不是一次性问答。跨公司主线不一致会削弱用户对长期记忆的信任，也会让后续 safety gate、反馈闭环和 playbook 得到的上下文质量下降。

## 方案概述

新增 **Cross-Company Thesis Map**：一个 actor-scoped、派生型、可刷新、可审计的跨公司主线地图。它不替代单家公司 `profile.md`，而是从现有 company portraits 读取并生成一份按行业 / 主题组织的共享判断层。

核心产物：

- `thesis_maps/<map_id>.md`：面向人和 agent 的 Markdown 地图，按 sector、industry template、theme 或用户自定义 watch cluster 组织。
- `thesis_maps/index.json`：轻量索引，记录 map id、覆盖的 profile ids、生成时间、输入画像更新时间、冲突数量、stale 数量。
- `ThesisMapIssue`：结构化问题项，用来标出跨画像矛盾、缺失变量、过期假设和需要 agent 复盘的主题。

第一版聚焦三个场景：

1. **同一行业共享主线**
   从同一 `sector` / `industry_template` 下的画像提取共同假设、关键驱动和证伪条件。

2. **公司级差异表**
   对每家公司列出“为什么它和同组公司不同”，要求差异来自 profile 中已有证据，而不是模型自由发挥。

3. **一致性问题队列**
   标出明显冲突，例如一份画像认为需求周期上行，另一份同组画像仍以需求下行为核心假设；或某家公司画像长期未审阅，已经不能支撑当前 map。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加“行业主线地图”区块，显示用户当前持仓 / 关注列表里的主要 cluster。
- 每个 cluster 展示共享主线、关键变量、覆盖公司、最后刷新时间和待复盘问题数。
- 用户点开后看到一个简洁比较面：同一行业下各公司的共同假设、差异变量、主要风险和需要补证据的位置。
- 当用户在 `/chat` 问“为什么我同时持有这些半导体公司？”或“这条 AI capex 新闻影响我的哪些标的？”时，agent 可以优先读取 thesis map，再下钻单家公司画像。

### 管理端

- Admin company profile 视图增加 actor 级一致性摘要：画像数量、行业 cluster 数、冲突数、stale map 数。
- 管理员可以手动触发某个 actor 的 map refresh，查看生成日志和冲突项，但不直接编辑 company portraits。
- `/users/:actor/profiles` 的导入后提示可以补充“建议刷新 thesis map”，避免画像包导入后跨公司主线仍停留在旧状态。

### 桌面端

- Desktop 复用同一 Web surface 和 API，不需要独立数据模型。
- 本地单用户场景可以把 thesis map 作为 portfolio workspace 的默认概览，降低用户每次从单 ticker 进入的认知负担。

### 多渠道

- Feishu / Telegram / Discord 中，用户提到多个 ticker 或行业问题时，agent 可以用 thesis map 快速判断是否需要进入跨公司回答。
- 渠道回复只返回紧凑摘要；若存在多个冲突项，引导用户到 Web / desktop 查看完整 map，而不是在聊天窗口里塞长表格。

## 技术方案

### 1. 新增派生存储，不改变 company profile 真相源

在 actor sandbox 下新增：

```text
thesis_maps/
  index.json
  semiconductor_hardware.md
  saas.md
  custom_ai_infra.md
```

`profile.md` 与 `events/*.md` 仍是长期记忆真相源。Thesis map 是派生读模型，可以删除重建；删除 map 不应删除任何公司画像。

建议结构：

```rust
struct ThesisMapSummary {
    map_id: String,
    title: String,
    actor: ActorIdentity,
    profile_ids: Vec<String>,
    sector: Option<String>,
    industry_template: Option<IndustryTemplate>,
    generated_at: String,
    source_updated_at_max: String,
    stale: bool,
    issue_count: usize,
}

struct ThesisMapIssue {
    issue_id: String,
    kind: ThesisMapIssueKind,
    profile_ids: Vec<String>,
    severity: String,
    summary: String,
    evidence_refs: Vec<String>,
    suggested_action: String,
}
```

### 2. 生成流程保持 agent-mediated

第一版不要让后端用规则硬写投资判断。后端只负责：

- 列出 actor 的 profiles、metadata、更新时间和事件数量。
- 提供 map 文件读写位置。
- 校验 map 输出是否包含覆盖列表、共享假设、差异变量、问题项和 source profile refs。

实际生成由 agent 通过新 skill 或 `company_portrait` 的扩展工作流完成：

1. 读取同组 `company_profiles/*/profile.md` 与最近事件。
2. 生成或刷新 `thesis_maps/<map_id>.md`。
3. 只在 map 中记录跨公司比较，不直接改写任何单家公司画像。
4. 如果发现某家公司画像本身需要更新，输出 `ThesisMapIssue`，由用户或后续 evidence review / playbook 决定是否进入 profile update。

### 3. API 与前端

新增只读优先的 API：

- `GET /api/company-profile-thesis-maps?channel=&user_id=&channel_scope=`
- `GET /api/company-profile-thesis-maps/:map_id?...`
- `POST /api/company-profile-thesis-maps/:map_id/refresh`：创建一条 agent prompt / task draft 或同步触发轻量 refresh，具体执行方式按当前 runner 能力灰度。

Public 端只开放当前 Web actor 的只读查询与 refresh 请求；Admin 端可以指定 actor 查询。

### 4. 与现有提案的关系

- evidence review queue 可以把“这条行业事件影响多个 map profiles”作为生成 review item 的输入。
- investment playbook launcher 可以加入“刷新行业主线地图”作为 Company Thesis Starter 的后续步骤。
- research artifact library 可以把深度研究报告 handoff 到 thesis map，但 map 本身不是报告库。
- linked user workspace 落地后，map 可以升级为 workspace-scoped；第一版仍按 actor 存储，避免提前改变 `ActorIdentity` 边界。

### 5. 兼容与迁移策略

- 不迁移现有画像文件。
- 没有 `sector` 或 `industry_template` 的画像先进入 `general` map，并在 issue 中提示 metadata gap。
- 已导入的 legacy plain Markdown profile 仍通过现有 relaxed parser 读取；map 生成只把 frontmatter 缺失作为 quality issue，不阻断读取。
- Map refresh 应可重复执行，输出覆盖旧 map 前保留 `generated_at` 与 source profile 更新时间，便于判断 stale。

## 实施步骤

### Phase 1: 只读 map 存储与手工生成契约

- 增加 `ThesisMapStorage` 或先在 `CompanyProfileStorage` 下加派生读写 helper。
- 定义 map Markdown 模板和 `index.json` schema。
- 增加 admin-only API 列出 / 读取 map。
- 扩展 `company_portrait` skill references，明确跨公司 map 的写入格式与非目标。

### Phase 2: Agent refresh 工作流

- 增加 refresh endpoint，生成一条带 actor、map id、profile ids 的 agent prompt。
- 让 runner 读取相关 profile 并写入 `thesis_maps/<map_id>.md`。
- 对生成结果做最小 lint：必须列出输入 profiles、共享假设、公司级差异、风险、issues。

### Phase 3: Public / Admin 体验

- Public `/portfolio` 展示 thesis map 摘要、stale 状态和完整 map modal。
- Admin company profile detail 增加 map refresh、issue count、最近生成状态。
- 导入画像包后提示刷新受影响 map。

### Phase 4: 与事件和任务联动

- Event engine 或 evidence review queue 在行业级事件出现时引用相关 map，标注 affected profiles。
- Scheduled task / playbook 可以定期刷新重点 map，但默认不自动改写单家公司画像。
- 添加质量指标：map stale 率、issue open 数、同组画像覆盖率、refresh 成功率。

## 验证方式

- 单元测试：
  - map id sanitization、actor scoped path、index 读写、删除重建不影响 `company_profiles/`。
  - legacy profile / 缺 frontmatter profile 被纳入 map quality issue，而不是读取失败。
- API 测试：
  - admin 指定 actor 可以读取 map；public 只能读取当前登录 actor。
  - refresh endpoint 对未知 actor、未知 map、空 profile set 给出明确错误。
- Skill / agent 回归：
  - 给定 3 个半导体画像 fixture，refresh 后 map 必须包含共享假设、三家公司差异变量、至少一个 source ref。
  - 给定两份相互冲突画像，生成结果必须产生 `ThesisMapIssue`，而不是静默合并。
- 前端验收：
  - `/portfolio` 有多家公司画像时显示 cluster；无 map 时显示可刷新空态；stale map 给出刷新入口。
  - Admin 导入画像包后能看到受影响 map stale 状态。
- 指标：
  - map refresh 成功率、平均生成耗时、stale map 数、issue 数、用户点击 map 后进入 chat 复盘的比例。

## 风险与取舍

- **风险：把派生 map 误当成真相源。**
  规避方式：所有 UI 和 prompt 都明确 `profile.md` 仍是源，map 只是跨公司读模型；单家公司结论变更必须回写 profile。

- **风险：生成 map 增加模型成本。**
  规避方式：第一版手动触发或 playbook 触发，不做每次画像读取都自动刷新；用 source profile 更新时间判断 stale。

- **风险：错误聚类导致错误比较。**
  规避方式：先使用显式 `sector` / `industry_template`，允许用户或 admin 后续定义 custom cluster；不要让模型自动把无关公司强行归组。

- **风险：与 evidence review queue 范围重叠。**
  取舍：map 只回答“跨公司主线是否一致”，不承担事件处理状态；事件是否复盘仍由 evidence review queue 或 agent 对话完成。

- **非目标：**
  - 不直接开放 UI 编辑 `profile.md`。
  - 不把 thesis map 做成自动交易建议。
  - 不在第一版实现 workspace-scoped 合并；linked workspace 落地前继续 actor-scoped。

## 与已有提案的差异

- 不重复 `auto_p1_evidence_review_queue.md`：该提案处理事件证据是否需要复盘；本提案处理多个公司画像之间的共享主线、差异变量和一致性问题。
- 不重复 `auto_p1_research_artifact_library.md`：artifact library 管理深度研究交付物；thesis map 是从公司画像派生出来的跨公司读模型，不保存报告正文库。
- 不重复 `auto_p1_investment_context_intake.md`：context intake 解决用户初始持仓、偏好和画像缺口；thesis map 在画像已有后提高长期记忆一致性。
- 不重复 `auto_p1_linked-user-workspace.md`：linked workspace 解决跨渠道同一用户的资产归并；本提案第一版严格 actor-scoped。
- 不重复 `auto_p1_investment_playbook_launcher.md`：playbook 是启动工作流入口；thesis map 是其中可以被刷新和复用的长期读模型。
- 不重复 `auto_p0_investment_output_safety_gate.md`：safety gate 拦截高风险输出；thesis map 改善回答前的长期上下文质量，降低前置漂移。
