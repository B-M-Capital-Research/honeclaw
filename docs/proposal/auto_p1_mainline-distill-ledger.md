# Proposal: Mainline Distill Ledger and Review Gate

status: proposed
priority: P1
created_at: 2026-05-22 20:07:31 +0800
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
- `docs/proposal/auto_p1_company-portrait-health.md`
- `docs/proposal/auto_p1_cross-company-thesis-map.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `crates/hone-event-engine/src/prefs.rs`
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
- `crates/hone-event-engine/src/global_digest/mainline_cron.rs`
- `crates/hone-event-engine/src/unified_digest/scheduler.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/user-mainline-view.tsx`
- `packages/app/src/lib/mainline-context-model.ts`

## 背景与现状

Hone 已经把 company portraits 作为用户长期研究记忆的核心资产，并通过主线蒸馏把长文画像压缩成 digest 个性化所需的短上下文：

- `crates/hone-event-engine/src/global_digest/mainline_distill.rs` 读取 actor sandbox 下的 `company_profiles/*/profile.md`，为每个持仓 ticker 生成 1-2 句 `mainline_by_ticker`，并生成跨 ticker 的 `mainline_style`。
- `crates/hone-event-engine/src/prefs.rs` 把 `mainline_by_ticker`、`mainline_style`、`last_mainline_distilled_at` 和 `mainline_distill_skipped` 存在 `NotificationPrefs` 里，后续 `unified_digest::scheduler` 读取这些字段做 personalize。
- `public_digest.rs` 允许 public 用户在 `/portfolio` 查看当前蒸馏结果，也允许用户手动触发一次 refresh；`event_engine_admin.rs` 提供 admin 端按 actor 查看和触发主线蒸馏。
- 当前合并策略是：如果新一轮 `by_ticker` 非空，就覆盖整个 `mainline_by_ticker`；如果 style 有值就覆盖旧 style；如果某个 ticker 失败则记录 skipped，旧主线可能被整体保留或被新 map 替换。
- `docs/invariants.md` 要求长期投资判断不能被单条新闻或短期价格动作随意改写，也要求 company portraits 保留足够证据让后续复盘。

这说明主线蒸馏已经从一个后台辅助任务变成了影响用户体验和主动通知质量的关键派生状态。问题在于：这个状态目前只以“最新值”存在于 prefs 里，没有自己的版本、输入快照、diff、质量信号、审核状态和回滚入口。

对 Hone 来说，主线蒸馏不是普通摘要。它会决定每日 digest 中什么被认为是噪音，什么被认为可能改变投资主线。如果蒸馏错误、过度概括、遗漏证伪条件，后续通知和回答都会沿着错误的短上下文继续工作。

## 问题或机会

1. **蒸馏结果缺少可追溯版本。**
   当前 `NotificationPrefs` 只保留最新 `mainline_by_ticker` 和 `mainline_style`。当用户说“为什么今天 Hone 觉得这条新闻不重要”时，系统很难回答当时用的是哪一版主线、由哪些 profile 内容生成、和上一版差异是什么。

2. **直接覆盖让错误影响面过大。**
   `merge_into_prefs()` 的行为适合早期自动化，但当 public 用户可以手动 refresh、后台 cron 可以定期刷新、多个模型 route 可能参与蒸馏时，一次异常输出可能让 digest personalize 短期内全部偏移。现有 skipped 字段只能告诉用户哪些 ticker 失败，不能告诉用户哪些 ticker 被错误改写。

3. **用户和管理员没有审核 diff。**
   Public `/portfolio` 展示最新主线，但不展示“本轮改变了什么”。Admin `UserMainlineView` 能代 actor 触发蒸馏，但也缺少 before/after、source profile hash、模型、耗时、失败原因、是否已应用等操作语义。

4. **质量提案之间缺少最后一道派生状态门。**
   `company-portrait-health` 关注源画像质量，`evidence-review-queue` 关注事件是否需要复盘，`cross-company-thesis-map` 关注跨画像一致性。但即使这些输入都更健康，最终写入 `NotificationPrefs` 的派生短主线仍需要自己的验收层。

5. **商业化和留存需要解释个性化依据。**
   对投资助手而言，“这条推送为什么重要 / 为什么没推”是信任核心。一个可查看、可回滚的主线蒸馏台账，能把个性化从黑盒变成用户能理解的长期资产维护过程。

这是 P1：它不属于立即安全事故，但直接影响 digest 相关性、长期记忆可信度、用户留存和管理员排障效率。第一版可以在不重写 company profile、event router 或 unified digest 的前提下落地。

## 方案概述

新增 **Mainline Distill Ledger and Review Gate**：把每次主线蒸馏从“直接覆盖 prefs”升级为“生成候选版本、记录输入与 diff、通过自动门禁或人工确认后应用、可回滚”的轻量派生状态层。

核心对象：

1. `MainlineDistillRun`
   记录一次蒸馏运行：actor、trigger、model/profile、source profile hashes、portfolio holdings、started/completed 时间、status、errors、token/latency 摘要、产出的 candidate id。

2. `MainlineDistillCandidate`
   一版候选主线：per-ticker mainlines、global style、skipped tickers、quality flags、before/after diff、created_at、是否已应用。

3. `MainlineDistillDiff`
   面向 UI 和 agent 的结构化差异：新增、删除、变化过大、证伪条件消失、长度异常、ticker 覆盖减少、style 大幅漂移。

4. `ReviewGateDecision`
   应用决策：`auto_applied`、`needs_review`、`rejected`、`rolled_back`、`expired`。第一版可以默认只对高风险 diff 要求审核，低风险正常自动应用。

5. `ActiveMainlineVersion`
   当前 `NotificationPrefs` 中生效的主线版本引用。短期可以通过额外元数据文件维护，长期可在 prefs 中增加 `mainline_version_id`。

关键原则：

- `profile.md` 和 `events/*.md` 仍是长期研究记忆真相源。
- `NotificationPrefs` 仍是 digest runtime 的读取入口，避免重写现有 personalize 路径。
- Ledger 是派生与审计层，不直接改写 company portraits。
- Public 用户可以确认自己的主线候选；admin 可以替 actor 审核和回滚。
- 第一版不要求每次都人工审核，重点是让风险较高的覆盖可见、可解释、可撤回。

## 用户体验变化

### 用户端

- Public `/portfolio` 的“上次更新”区域增加“查看变化”入口：
  - 显示最近一次蒸馏是自动应用、待确认、失败还是回滚。
  - 展示每个 ticker 的 before/after diff，突出“主线变化较大”“持仓缺画像”“证伪条件疑似消失”等风险。
  - 对低风险自动更新，只显示折叠的更新记录；对高风险候选，提示用户确认后再应用。
- 用户手动点击“立即更新”时，第一步生成 candidate，不直接覆盖；如果 diff 低风险可自动应用并显示结果，如果高风险则显示确认界面。
- 当用户认为新主线不准确，可以选择：
  - `保持旧版`
  - `应用新版`
  - `让 Hone 复审画像后再蒸馏`
  - `回到 chat 说明哪里不对`
- 空状态仍保持简单：没有 portfolio 或没有画像时，不暴露 ledger 概念，只提示先建立投资上下文。

### 管理端

- Admin 用户详情或 mainline tab 增加 `Distill runs` 面板：
  - 最近运行列表：actor、trigger、model route、status、耗时、覆盖 ticker 数、skipped 数、gate decision。
  - Candidate 详情：source profile 文件 hash/mtime、before/after、quality flags、错误日志。
  - 操作：apply、reject、rollback、rerun with current profiles。
- Notifications 或 digest 排障页面可以引用 active mainline version：当管理员追查某条 digest 为什么被 personalized 时，可以看到当时生效的主线版本。
- 对配置缺失或模型不可用的失败，使用稳定 reason code，例如 `provider_unavailable`、`no_profiles`、`coverage_drop`、`empty_output`、`large_semantic_drift`。

### 桌面端

- Desktop bundled/remote 复用 Web API 和 Web UI，不新增 sidecar。
- Dashboard 可以显示一个轻量 badge：`主线待确认 2` 或 `主线蒸馏失败`，点击进入同一 portfolio/mainline 页面。
- 本地单用户模式默认允许低风险自动应用，高风险保留在本地 review queue，避免用户每日打开桌面时看到过多确认弹窗。

### 多渠道

- Feishu / Telegram / Discord 中用户说“更新我的投资主线”时，agent 不应直接让后台覆盖 prefs，而是生成 candidate 并返回短提示：“已生成主线更新候选，2 处变化较大，请到 Web/desktop 确认”。
- 对低风险更新，可在 IM 中回复摘要和 version id。
- 对主动 digest，消息本身不展示长 diff；只有当用户问“为什么推这个”时，agent 用 active version 摘要解释。

## 技术方案

### 1. 新增 ledger 存储

建议优先在 `memory` 新增 `mainline_distill_ledger` 模块，使用 SQLite 存元数据和候选 JSON。文件型 `NotificationPrefs` 继续作为 runtime 生效配置，避免第一版迁移风险。

表结构示意：

```text
mainline_distill_runs (
  run_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  trigger TEXT NOT NULL,
  model_ref TEXT,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  completed_at TEXT,
  error_code TEXT,
  error_message TEXT,
  candidate_id TEXT,
  source_snapshot_json TEXT NOT NULL
)

mainline_distill_candidates (
  candidate_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  before_json TEXT NOT NULL,
  after_json TEXT NOT NULL,
  diff_json TEXT NOT NULL,
  quality_flags_json TEXT NOT NULL,
  gate_decision TEXT NOT NULL,
  applied_at TEXT,
  rejected_at TEXT,
  rolled_back_from_version_id TEXT,
  created_at TEXT NOT NULL
)

mainline_distill_versions (
  version_id TEXT PRIMARY KEY,
  candidate_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  applied_at TEXT NOT NULL,
  active BOOLEAN NOT NULL,
  prefs_sha256 TEXT NOT NULL
)
```

`source_snapshot_json` 记录：

- holdings 列表与 portfolio mtime/hash。
- 每个参与 profile 的 path、mtime、sha256、parsed ticker。
- distill model/profile/ref。
- 上一版 active mainline version。

### 2. 拆分 distill 与 apply

在 `mainline_distill.rs` 中把当前 `distill_and_persist_one()` 拆成两层能力：

- `distill_candidate_for_actor(...) -> DistilledMainlineCandidate`
- `apply_distill_candidate(...) -> NotificationPrefs`

保留旧 `distill_and_persist_one()` 作为兼容 façade，但内部改为：

1. 生成 run。
2. 生成 candidate 和 diff。
3. 运行 gate。
4. 低风险自动 apply，高风险只保存 candidate。
5. 返回当前 active prefs 与 gate 状态。

为了减少改动面，第一阶段可以只让 public/admin 手动 refresh 走 ledger；后台 cron 仍自动应用但记录 run/candidate。第二阶段再让 cron 高风险 diff 进入待审。

### 3. 自动 review gate 规则

第一版 gate 用确定性规则，不引入额外 LLM：

- `coverage_drop`：本轮 after 覆盖 ticker 数少于 before，且不是用户 portfolio 移除。
- `empty_or_too_short`：主线为空或过短。
- `too_long`：单 ticker 主线超过阈值，可能把整段 profile 摘要塞入 prefs。
- `large_text_change`：旧主线存在且新主线差异过大。
- `risk_terms_removed`：旧文本含“风险 / 证伪 / 如果 / 除非 / 关注”，新文本完全丢失类似约束词。
- `style_drift`：global style 从明确风格变成泛化描述。
- `skipped_active_holding`：仍在 portfolio 的 ticker 被 skipped。

Gate 结果：

- 无 flags 或只有低风险 skipped：`auto_applied`
- 覆盖下降、约束消失、大幅漂移：`needs_review`
- 全空、provider error、无法保存：`failed`

这些规则只判断派生状态风险，不判断投资内容对错。

### 4. API 设计

Admin API：

- `GET /api/mainline-distill/runs?channel=&user_id=&channel_scope=`
- `GET /api/mainline-distill/candidates/:candidate_id`
- `POST /api/mainline-distill/refresh`
- `POST /api/mainline-distill/candidates/:candidate_id/apply`
- `POST /api/mainline-distill/candidates/:candidate_id/reject`
- `POST /api/mainline-distill/versions/:version_id/rollback`

Public API：

- `GET /api/public/mainline-distill/runs`
- `GET /api/public/mainline-distill/candidates/:candidate_id`
- `POST /api/public/digest-context/refresh`
- `POST /api/public/mainline-distill/candidates/:candidate_id/apply`
- `POST /api/public/mainline-distill/candidates/:candidate_id/reject`

兼容策略：

- 现有 `GET /api/public/digest-context` 继续返回当前 active prefs 字段。
- `POST /api/public/digest-context/refresh` 可继续返回 `ok/mainline_count/skipped_tickers`，新增 `candidate_id`、`gate_decision`、`applied`、`review_required`。
- 前端旧逻辑看到 `ok=true` 仍能工作；新逻辑使用 candidate 字段展示 diff。

### 5. 前端落地

- `packages/app/src/pages/public-portfolio.tsx`：
  - 在 refresh 后根据 `review_required` 展示 diff panel。
  - 在“上次更新”旁增加最近 run 状态。
  - 对待确认 candidate 提供 apply/reject。
- `packages/app/src/components/user-mainline-view.tsx`：
  - Admin mainline 视图增加 runs/candidates 列表。
  - 展示 source profile coverage、quality flags 和 rollback 操作。
- `packages/app/src/lib/mainline-context-model.ts`：
  - 增加 diff 派生 helper，保证 UI 不直接依赖后端 JSON 细节。

### 6. 与 unified digest 的关系

`unified_digest::scheduler` 仍读取 `NotificationPrefs.mainline_by_ticker` 和 `mainline_style`。第一版不改变 runtime 读路径，只新增 version metadata：

- digest run metadata 可以记录 active version id。
- 若 active version 缺失，则行为和现在一致。
- 如果某 actor 有 `needs_review` candidate，digest 不自动使用 candidate；仍使用旧 active prefs，并可在 admin 端显示“主线有待确认更新”。

## 实施步骤

### Phase 1: Ledger schema and candidate generation

- 新增 `memory/src/mainline_distill_ledger.rs` 与 SQLite 表。
- 增加 run/candidate/version 类型和单元测试。
- 在 `mainline_distill.rs` 中拆出 candidate 生成，不改变现有 cron 行为。
- 为 public/admin 手动 refresh 写入 run 与 candidate。

### Phase 2: Diff and deterministic gate

- 实现 before/after diff、quality flags 和 gate decision。
- 低风险 candidate 自动 apply，高风险保留待审。
- 给 `POST /api/public/digest-context/refresh` 和 admin refresh 响应增加 candidate/gate 字段。
- 保留旧响应字段兼容已有前端。

### Phase 3: Public and admin review UI

- Public `/portfolio` 展示待确认 diff、apply/reject 和最近 run 状态。
- Admin mainline 视图展示 run/candidate 列表、source snapshots、quality flags。
- 添加 rollback 操作，回滚只恢复 mainline/style/skipped/version metadata，不碰其它 notification prefs。

### Phase 4: Digest trace integration

- unified digest 写入 active mainline version id 到运行或投递 metadata。
- Notifications/detail 或未来 run trace 能显示“本次 personalize 使用了哪版主线”。
- Cron 对高风险 diff 也进入 `needs_review`，避免后台静默覆盖。

## 验证方式

### 自动化测试

- Rust 单元测试：
  - `distill_candidate_for_actor` 生成 source snapshot，包含 profile hash、portfolio holdings 和 before prefs。
  - diff 能识别新增、修改、删除、coverage drop、skipped active holding。
  - gate 对低风险更新返回 `auto_applied`，对 coverage drop / empty output / large drift 返回 `needs_review` 或 `failed`。
  - rollback 只恢复 mainline 相关字段，不覆盖 `quiet_hours`、`digest_slots`、source allow/block 等其它 prefs。
- API 测试：
  - public refresh 对当前 session actor 生效，不能访问其它 actor candidate。
  - admin refresh 可以指定 actor，candidate apply 后 `GET digest-context` 返回新版。
  - reject 后 candidate 不再可 apply，除非显式 rerun。

### 前端测试

- `public-portfolio`：
  - refresh 返回 `auto_applied` 时显示成功摘要。
  - refresh 返回 `needs_review` 时显示 diff 和 apply/reject。
  - 移动端 diff panel 不遮挡主线卡片。
- `user-mainline-view`：
  - runs 列表能渲染 success/failed/needs_review。
  - rollback 按钮只在 active 旧版本可用。

### 手工验收

- 构造一个 actor：portfolio 有 `MU/RKLB`，两个 profile 都可蒸馏。首次 refresh 自动应用。
- 修改一个 profile，让新主线删除关键风险约束；refresh 应生成 `needs_review` candidate，不覆盖 active prefs。
- 点击 apply 后，public `/portfolio` 和 admin mainline 看到新版。
- 点击 rollback 后，digest context 恢复旧主线，version id 更新。
- 删除一个 profile 后 refresh，不应静默清空所有旧主线；应标记 skipped/coverage risk。

### 指标

- mainline refresh 成功率。
- candidate auto-apply 比例。
- needs-review candidate 数量和平均处理时长。
- rollback 次数。
- digest 个性化投诉或用户反馈中“主线不准”的占比。

## 风险与取舍

- 风险：把简单 refresh 变复杂，用户不想审核文本 diff。取舍：只对高风险 diff 打断流程，低风险仍自动应用，UI 默认折叠更新记录。
- 风险：ledger 与 prefs 双写可能出现不一致。取舍：runtime 仍以 prefs 为准；version 只是审计引用。apply 时保存 prefs sha256，后续发现不匹配时提示 manual recovery。
- 风险：确定性 gate 无法真正理解投资语义。取舍：第一版只拦结构和大幅漂移风险，不判断投资观点正确性；语义复审仍走 company portrait / evidence review。
- 风险：SQLite 表增加维护面。取舍：只存候选 JSON 与小型 source snapshot，不复制完整 profile 正文；profile hash 用于追溯，原文仍在 actor sandbox。
- 风险：cron 积累大量候选。取舍：保留最近 N 次或 90 天 run；已应用/拒绝 candidate 可压缩存储，保留 version diff 和 hash。
- 不做：不直接编辑 `profile.md`，不替代 company portrait health，不新增交易建议逻辑，不改变 digest personalize 的核心 prompt，不把每条通知都附带长主线 diff。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 和 `docs/proposals/` 下全部现有提案，并重点检查了包含 `mainline`、`主线`、`distill`、`蒸馏`、`preview`、`rollback` 的主题。

- 不重复 `auto_p1_evidence_review_queue.md`：该提案把事件转成待复盘证据，解决“外部证据是否应更新画像”；本提案关注“画像已经被蒸馏后，派生短主线如何版本化、审核和回滚”。
- 不重复 `auto_p1_company-portrait-health.md`：该提案评估 `profile.md` 和 events 的结构健康；本提案不 lint 源画像，而是给最终写入 `NotificationPrefs` 的蒸馏结果加 ledger/gate。
- 不重复 `auto_p1_cross-company-thesis-map.md`：该提案生成跨公司共享主线地图；本提案不做跨公司比较，只管理 per-ticker mainline/style 的候选版本。
- 不重复 `auto_p1_investment_context_intake.md`：该提案解决 portfolio/profile/mainline 初始化缺口；本提案解决主线 refresh 之后的 diff、应用和回滚。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：该提案把 portfolio、主线和持仓规模转成组合暴露视图；本提案不生成风险雷达，只保护 digest personalize 的主线输入。
- 不重复 `auto_p1_source-provenance-freshness.md`：该提案追踪外部事实来源和新鲜度；本提案追踪内部长期记忆派生状态的版本与应用决策。
- 不重复 `auto_p1_agent-mutation-ledger.md`：该提案是泛化的 agent 状态变更台账；本提案针对 mainline distill 这个已存在、会影响 digest 个性化的派生状态，提出更窄、更快可落地的 schema、gate 和 UI。

差异结论：现有提案已经覆盖源画像质量、事件复盘、跨公司一致性、组合暴露和通用 mutation 审计，但还没有专门保护 `mainline_distill -> NotificationPrefs -> unified_digest personalize` 这条关键派生链路。本提案填补的是“长期记忆被压缩成推送个性化输入”这一最后一步的可追溯和可回滚能力。
