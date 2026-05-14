# Proposal: Invite Activation Funnel and Success Milestones

status: proposed
priority: P1
created_at: 2026-05-10 17:04:03 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `memory/src/web_auth.rs`
- `memory/src/quota.rs`
- `memory/src/portfolio.rs`
- `memory/src/company_profile/storage.rs`
- `memory/src/cron_job/storage.rs`
- `memory/src/session.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/cron.rs`
- `crates/hone-web-api/src/types.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/users.tsx`
- `packages/app/src/lib/types.ts`
- `packages/app/src/lib/admin-content/users.ts`

## 背景与现状

Hone 已经从本地开源工具演进出一条真实的 public trial / invite 产品链路：

- `memory/src/web_auth.rs` 用 SQLite 保存 Web invite user、手机号、邀请码、密码 hash、TOS 接受记录、HttpOnly session、API key hash / prefix / last used。
- `crates/hone-web-api/src/routes/public.rs` 把 public 登录用户固定映射为 `ActorIdentity::new("web", user_id, None)`，并为 `/chat`、`/history`、文件上传和 OpenAI-compatible `/api/public/v1/chat/completions` 提供入口。
- `crates/hone-web-api/src/routes/web_users.rs` 的 invite 列表已经返回 `active_session_count`、`daily_limit`、`success_count`、`in_flight`、`remaining_today`、`last_login_at` 和 API key 信息。
- `packages/app/src/pages/settings.tsx` 的 Web invite 管理表可以创建邀请码、复制 invite code、重置 invite、启停用户、获取或重置 API key，并展示会话数、今日剩余额度和最后登录时间。
- Public `/me` 主要展示账户、密码、TOS、今日剩余额度、总使用次数和登录信息；public `/portfolio` 展示当前 Web actor 的投资上下文、主线、画像和空状态引导。
- 管理端 `/users/:actorKey/*` 已经能查看单个 actor 的 portfolio、company profiles、mainline、sessions 和 research，但它是资产视图，不是用户旅程视图。
- `memory/src/quota.rs`、`memory/src/session.rs`、`memory/src/portfolio.rs`、`memory/src/company_profile/storage.rs`、`memory/src/cron_job/storage.rs` 已经分别保留了对话、会话、投资资产、长期画像和自动化任务的证据。

也就是说，Hone 已经知道一个 invite 用户是否登录、是否有 API key、今天用了多少次、有哪些会话和资产；但这些信号还没有被组织成“这个用户是否真正激活、卡在哪一步、下一步应该引导什么”的产品模型。

当前 public invite 管理更像技术控制台：它回答“这个 invite 是否可用”和“额度还剩多少”。对一个以严肃投资研究、长期记忆、多渠道自动化和商业转化为目标的产品来说，还需要回答：

- 用户是否完成了第一次有效对话？
- 用户是否给过持仓、关注标的或投资风格？
- 用户是否已经有至少一个可复用的公司画像或研究资产？
- 用户是否启动了任何会在未来主动创造价值的任务、提醒或 digest？
- 用户是否用过 API key / Hone Cloud 客户端，或者只停留在 Web 页面？
- 用户卡在了 invite 登录、首次对话、投资上下文、自动化、留存复访中的哪一步？

## 问题或机会

这是 P1 级机会，因为 Hone 的 public / invite 链路已经足够接近真实增长漏斗，但缺少里程碑状态会让产品和运营判断停留在猜测。

### 问题

1. **邀请用户的成功状态过于技术化。**
   Settings invite table 展示 code、phone、status、API key、sessions、remaining、last login。这些字段对排障有用，但无法判断用户是否感受到 Hone 的核心价值。一个用户可能登录 3 次、用完 12 次额度，却没有建立任何 portfolio、画像、任务或复盘资产。

2. **Public 用户缺少下一步成功路径。**
   `/me` 展示额度和账户资料，`/portfolio` 在空状态下提示去 `/chat`，但没有根据用户当前阶段给出一条清晰的下一步。例如：先补持仓、再建立第一个公司画像、再开一个周复盘任务、再绑定 API key 或桌面端。

3. **管理端无法按激活阶段运营。**
   管理员可以在 `/settings` 创建和启停 invite，也可以进 `/users` 看某个 actor 的资产，但缺少一张 cohort/activation 视图回答“哪些用户从未登录、哪些首聊失败、哪些没有投资上下文、哪些已经进入高价值留存阶段”。

4. **商业转化信号分散。**
   `usage entitlement` 可以解决额度和成本，`response feedback` 可以解决回答质量，`linked workspace` 可以解决跨渠道身份；但运营仍需要一个独立的 journey model，把“用户已经看到什么价值、还缺哪一步、是否值得人工跟进或升级”表达出来。

5. **产品改动难以验证增长效果。**
   新增 intake、playbook、public portfolio、API key、desktop handoff 或通知体验后，如果没有统一 activation milestones，就只能看总会话数、登录次数或主观反馈，无法知道某个改动是否缩短了 time-to-value。

### 机会

新增 **Invite Activation Funnel and Success Milestones**：一层只读优先、可派生、可审计的产品旅程模型，把已有技术信号转换为用户激活阶段、成功里程碑、阻塞原因和 next-best-action。

它不替代已有模块：

- 不替代 `Usage Entitlement Ledger`：权益账本回答“能不能用、消耗多少”；本提案回答“是否跨过价值门槛、该引导什么”。
- 不替代 `Investment Context Intake`：intake 负责收集投资上下文；本提案负责判断用户是否需要 intake，以及 intake 后是否继续前进。
- 不替代 `Investment Playbook Launcher`：playbook 负责启动标准工作流；本提案负责识别哪些用户适合推荐哪个 playbook。
- 不替代 `Linked User Workspace`：workspace 负责跨渠道身份；本提案在 workspace 落地前先基于 Web actor 运行，之后可汇总到 workspace 级别。

## 方案概述

为 public invite 用户增加一个派生型 `ActivationProfile`，按 actor 或未来 workspace 汇总激活状态。

建议里程碑从少到多，先覆盖 7 个阶段：

1. `invited`：已创建 invite user，尚未登录。
2. `account_ready`：已登录并完成 TOS / 密码要求，存在活跃 session。
3. `first_value_chat`：至少一次成功用户对话，并且不是纯登录/测试/寒暄。
4. `investment_context_added`：portfolio 或 watchlist 非空，或 global investment style / mainline prefs 非空。
5. `memory_seeded`：至少一个 company profile、research artifact、或可复用的长期研究记录存在。
6. `automation_enabled`：至少一个启用 cron、digest、价格提醒、周复盘或 portfolio guardrail 工作流存在。
7. `retained_or_integrated`：出现复访、API key 使用、桌面/远端客户端使用、或跨渠道绑定迹象。

每个阶段都应带：

- `status`: `missing` / `partial` / `done` / `stale`
- `evidence`: 触发阶段的原始证据引用，例如 invite row、session id、portfolio count、profile ids、cron ids、api_key_last_used_at
- `first_completed_at` / `last_seen_at`
- `blockers`: 例如 no_login、quota_exhausted_before_context、empty_portfolio、no_profile、no_enabled_automation、api_key_unused
- `next_actions`: 面向用户或管理员的下一步，例如 open intake、start thesis starter、create weekly review、send follow-up, copy API key instructions

第一版可以完全派生，不必写入复杂事件流：

- Web auth row 提供 invite/account/API key 状态。
- Quota snapshot 和 sessions 提供使用、成功对话、最近活跃。
- Portfolio storage 提供持仓/关注和基础投资上下文。
- Company profile storage 提供长期记忆资产数量与最近更新时间。
- Cron job storage 提供自动化任务数量、启用状态和最近执行。
- Public digest / notification prefs 提供推送偏好和 digest 是否具备个性化输入。

后续如果需要更细的渠道 attribution，再追加 `ActivationEvent` append-only ledger，记录每次 milestone 转换、admin follow-up、推荐动作和用户点击。

## 用户体验变化

### 用户端

- Public `/me` 从账户资料页升级为“账户 + 价值进度”页，显示 3 到 5 个里程碑，而不是只显示今日额度。
- 新用户看到明确下一步：补投资上下文、建立第一家公司画像、启动周复盘、保存 API key 或安装桌面端。
- Public `/portfolio` 空态可以读取 `ActivationProfile`，按阶段给出更具体的入口，而不是统一提示“去 chat 里告诉 agent”。
- Public `/chat` 在完成首次高价值回答后，可以轻量提示“把这次研究沉淀为公司画像”或“开启下次自动复盘”，但不能打断正常对话。
- API key 用户如果已经创建 key 但从未使用，`/me` 可显示简短接入说明和最后使用状态。

### 管理端

- Settings invite tab 增加一列 `Activation`，展示阶段 badge：未登录、已登录、首聊完成、已补上下文、已建画像、已启用自动化、已留存。
- 新增筛选：never logged in、first chat only、needs context、needs automation、high intent、stalled after quota、API key issued but unused。
- 点击某个 invite 可以看到 `ActivationProfile` 侧栏：证据、阻塞、next actions、最近会话、资产数量和建议跟进话术。
- `/users` actor 详情页可以显示同一套 milestone，用于判断一个用户的资产是否足以支撑高质量 digest。
- 管理员可以手动标记一次 follow-up，例如“已微信联系”“已发送桌面安装指引”，但第一版不做 CRM 复杂流程。

### 桌面端

- 本地 desktop/bundled 模式可以把 activation milestones 当成本机 onboarding checklist：配置 runner、完成首次 chat、导入/创建 portfolio、启动一个本地任务。
- remote backend / Hone Cloud 模式可以展示“你当前 public 账号已完成哪些步骤”，避免桌面安装后看起来像一个全新空账号。
- 不要求第一版解决跨设备同步；如果 `Linked User Workspace` 尚未落地，desktop 只显示当前连接的 Web actor 或本地 actor 状态。

### 多渠道

- Feishu / Telegram / Discord 中，如果用户首次提出持仓或关注标的，agent 可以在成功写入 portfolio 后返回一句“已完成投资上下文第一步，可在 Web `/portfolio` 查看”。
- 对未绑定 Web invite 的 IM actor，管理端可以看到它处于 `unlinked_active_actor` 状态，提示后续通过 workspace/linking 归并。
- 不在 IM 里塞完整 onboarding 流程；多渠道只需要产出 milestone evidence 和轻量提醒。

## 技术方案

### 数据模型

新增一个派生类型，建议放在 Web API 或 memory helper 层，第一版不必创建持久表：

```rust
pub struct ActivationProfile {
    pub subject: ActivationSubject,
    pub stage: ActivationStage,
    pub score: u8,
    pub milestones: Vec<ActivationMilestone>,
    pub blockers: Vec<ActivationBlocker>,
    pub next_actions: Vec<ActivationNextAction>,
    pub evidence: ActivationEvidenceSummary,
    pub computed_at: String,
}
```

其中：

- `ActivationSubject`: v1 为 `ActorIdentity`，未来支持 `WorkspaceIdentity`。
- `ActivationStage`: `invited` / `account_ready` / `first_value_chat` / `investment_context_added` / `memory_seeded` / `automation_enabled` / `retained_or_integrated`。
- `ActivationMilestone`: 带 `status`、`first_completed_at`、`last_seen_at`、`evidence_refs`。
- `ActivationNextAction`: `kind`、`label`、`target_route`、`admin_only`、`reason`。

### Evidence 读取

新增 `ActivationProfileBuilder`，只读组合现有存储：

- `WebAuthStorage::list_invite_users()` / `find_invite_user()`：invite、login、password、TOS、API key 状态。
- `ConversationQuotaStorage::snapshot_for_date()`：当日使用与额度阻塞。
- `SessionStorage` 或现有 history/users 投影：成功会话数、最近活跃、是否只有寒暄类消息。第一版可先用 message count 与 last_time，避免引入 LLM 分类。
- `PortfolioStorage`：holdings/watchlist 数量、是否只有 tracking-only。
- `CompanyProfileStorage`：profile count、recent profile ids、last updated。
- `CronJobStorage`：enabled job count、repeat/tag、last execution status。
- `NotificationPrefs` / public digest routes：是否有 thesis/global style 与 digest 个性化输入。

### API

新增或扩展：

- `GET /api/web-users/invites/activation`：返回所有 invite 的 lightweight activation profile summary，用于 Settings 表格和筛选。
- `GET /api/web-users/invites/{user_id}/activation`：返回单个用户完整 profile。
- `GET /api/public/me/activation`：返回当前 public 用户自己的 milestones 和 next actions。
- 可选：`POST /api/web-users/invites/{user_id}/activation/follow-up`，记录 admin follow-up note。第一版可以先不做。

兼容策略：

- 后端 capability 中新增 `invite_activation`，前端有字段则显示，无字段保持旧 invite table。
- 所有 activation 字段都是派生值，不影响登录、额度、聊天或 API key。
- 如果某个 evidence store 读取失败，profile 应降级为 `partial` 并带 `evidence_unavailable`，而不是让 invite 列表整体失败。

### 前端

- `packages/app/src/lib/types.ts` 增加 `ActivationProfileSummary` / `ActivationMilestone` / `ActivationNextAction`。
- `packages/app/src/pages/settings.tsx` 的 invite table 增加阶段 badge、筛选、详情侧栏。
- `packages/app/src/pages/public-me.tsx` 增加 milestone list 和 next action cards。
- `packages/app/src/pages/public-portfolio.tsx` 空态读取 activation next action；对已有 portfolio 但无 profile 的用户推荐画像启动。
- `packages/app/src/pages/users.tsx` 可在 actor header 显示 activation stage，减少管理员在多个 tab 间切换。

### 后续可选持久化

如果只读派生不足以分析历史转化，可追加：

```sql
CREATE TABLE activation_events (
    event_id TEXT PRIMARY KEY,
    subject_key TEXT NOT NULL,
    subject_kind TEXT NOT NULL,
    event_type TEXT NOT NULL,
    stage_from TEXT,
    stage_to TEXT,
    source TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

但这不应是一期前置项。先用派生 profile 证明产品价值，再决定是否沉淀事件流。

## 实施步骤

### Phase 1: 只读 ActivationProfile

- 定义 activation stage、milestone、blocker、next action 类型。
- 实现只读 `ActivationProfileBuilder`，先覆盖 Web invite actor。
- 在单元测试中用临时 web_auth/quota/portfolio/profile/cron 存储构造 4 类用户：未登录、首聊完成、已补上下文、已启用自动化。
- 新增 `GET /api/web-users/invites/{user_id}/activation` 和 `GET /api/public/me/activation`。

### Phase 2: 管理端 invite funnel

- Settings invite table 增加 activation badge 和筛选。
- 增加详情侧栏，展示 evidence、blockers、next actions。
- 保持旧字段可见，避免运营失去技术排障能力。
- 对 API key created but unused、quota exhausted before context、no portfolio after N chats 等状态做高亮。

### Phase 3: Public next-best-action

- Public `/me` 增加进度条和下一步卡片。
- Public `/portfolio` 空态接入 activation next action。
- Public `/chat` 在完成关键里程碑后给出低打扰提示；提示必须可关闭，且不影响金融安全边界。

### Phase 4: Cohort metrics and workspace extension

- 增加 admin summary：每个 stage 的用户数、7 日内 stuck count、time-to-first-value、time-to-context、time-to-automation。
- 如果 `Linked User Workspace` 落地，把 subject 从 actor 扩展为 workspace 汇总，同时保留 actor 级 evidence。
- 评估是否需要 `activation_events` 持久化和 admin follow-up notes。

## 验证方式

### 自动化测试

- `memory` 或 `hone-web-api` 单元测试：
  - 新建 invite 但未登录 -> `stage=invited`，next action 是发送/复制 invite。
  - 已登录无成功对话 -> `stage=account_ready`，next action 是开始 first value chat。
  - 有成功会话但无 portfolio/profile/cron -> `stage=first_value_chat`，blocker 包含 `investment_context_missing`。
  - 有 watchlist/holding -> `investment_context_added`。
  - 有 company profile -> `memory_seeded`。
  - 有 enabled cron -> `automation_enabled`。
  - API key created but never used -> blocker 或 next action 包含 `api_key_unused`。
- API contract 测试：
  - activation API 在 portfolio/profile/cron 某个 store 读取失败时返回 partial，而不是 500。
  - 无 `invite_activation` capability 的旧前端路径不崩。

### 前端测试

- Settings invite table 能按 activation stage 筛选。
- Public `/me` 在 milestones 缺失、partial、done 三种状态下渲染稳定。
- 移动端下 milestone 卡片和 invite table 不溢出。

### 手工验收

- 创建一个新 invite，登录 public site，完成一次 chat，补一个 watchlist，建立一个 company profile，创建一个 weekly review cron；每一步刷新后 stage 单调前进。
- 对只拿了 API key 但没有使用的用户，admin 能看到明确的 `api_key_unused` 提醒。
- 对已经用完 quota 但没有 portfolio 的用户，admin 能看到“额度消耗未转化为投资上下文”的阻塞。

### 指标

- `invite_created -> first_login` 转化率。
- `first_login -> first_value_chat` 转化率与中位耗时。
- `first_value_chat -> investment_context_added` 转化率。
- `investment_context_added -> memory_seeded` 转化率。
- `memory_seeded -> automation_enabled` 转化率。
- 7 日复访率、API key 首次使用率、quota exhausted before context 占比。
- admin follow-up 后的阶段推进率。

## 风险与取舍

- **风险：把增长漏斗做成监控噪音。** 如果 stage 太多或定义太复杂，运营会无从下手。缓解：一期只保留 7 个阶段，每个阶段最多 2 到 3 个 next actions。
- **风险：误判首次价值对话。** 第一版不应上复杂 LLM 分类。先用成功 assistant turn、message count、非空上下文/资产变化作为粗粒度 evidence；需要精细判断时再接 `response_feedback` 或 run trace。
- **风险：过度打扰用户。** Public `/chat` 提示必须低频、可关闭，并且不应在投资敏感回答中插入营销文案。
- **风险：扩大隐私面。** Activation profile 只能展示当前 admin 已可访问的 actor 汇总，不新增跨 actor 读取。自由文本会话内容不进入 profile，只引用 session id / count / timestamps。
- **风险：和权益/工作区/CRM 边界混淆。** 本提案不处理付费 plan、不做跨渠道自动合并、不做完整 CRM。它只提供 milestone、blocker 和 next action。
- **取舍：先派生后持久化。** 派生 profile 可能无法回答历史路径变化，但实现简单、低风险。等指标被证明有用后再补 `activation_events`。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，重点核对了以下相近主题：

- `auto_p1_usage_entitlement_ledger.md`：关注 plan、quota、usage event、成本控制和商业权益。本提案不决定用户能用多少，而是判断用户是否完成价值里程碑和下一步引导。
- `auto_p1_investment_context_intake.md`：关注如何收集 portfolio、画像、偏好和任务缺口。本提案只识别用户是否需要 intake、intake 是否完成，以及完成后是否继续进入 memory/automation 阶段。
- `auto_p1_investment_playbook_launcher.md`：关注标准投资工作流模板如何启动。本提案只把 playbook 作为 next action 候选，不定义 playbook 执行语义。
- `auto_p1_linked-user-workspace.md`：关注跨渠道真实用户工作区。本提案 v1 基于 Web invite actor，未来可迁到 workspace subject，但不解决绑定/合并。
- `auto_p1_response-feedback-learning-loop.md`：关注单条 answer 的质量反馈。本提案关注用户旅程和激活状态，可把负反馈作为 blocker，但不替代 answer-level feedback。
- `auto_p1_runtime_readiness_matrix.md`：关注部署、模型、渠道和 capability 是否可运行。本提案关注某个 invite/user 是否已经跨过产品价值门槛。
- `auto_p1_run_trace_workbench.md`：关注一次 agent run 的运行证据和排障。本提案聚合多次运行和资产状态，只用于激活/留存判断。

查重结论：现有提案覆盖了权益、上下文收集、标准工作流、跨渠道身份、回答质量和运行排障，但没有覆盖“invite 用户从创建、登录、首聊、补投资上下文、沉淀长期记忆、启用自动化到留存/集成”的阶段化产品漏斗。因此本主题是新的、可落地的 P1 产品/架构提案，能直接提升 public trial 的激活、运营跟进和增长验证能力。
