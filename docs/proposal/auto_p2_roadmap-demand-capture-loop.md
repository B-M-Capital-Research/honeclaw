# Proposal: Roadmap Demand Capture Loop for Public Product Discovery

status: proposed
priority: P2
created_at: 2026-05-28 20:08:25 +0800
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
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_privacy-preserving-product-events.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `docs/proposal/auto_p2_shareable-investment-briefs.md`
- `packages/app/src/pages/public-home.tsx`
- `packages/app/src/pages/public-roadmap.tsx`
- `packages/app/src/pages/public-blog.tsx`
- `packages/app/src/pages/public-blog-post.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/lib/public-content.ts`
- `packages/app/src/components/public-nav.tsx`
- `packages/app/src/components/public-contact-menu.tsx`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `memory/src/web_auth.rs`

## 背景与现状

Honeclaw 已经具备一条完整的公开产品发现路径：

- `README.md` 和 public home 明确把 Hone 定位为严肃投资者的开源 AI agent，而不是普通聊天玩具。
- `packages/app/src/pages/public-home.tsx` 通过视频、案例轮播、Blog CTA 和 GitHub 链接解释产品价值。
- `packages/app/src/pages/public-roadmap.tsx` 是 content-driven 的路线图页面，展示 quick start、roadmap phases、capability matrix、channels、architecture、skills、开源边界、docs、contributing 和 FAQ。
- `packages/app/src/lib/public-content.ts` 维护中英文并行文案，包含 roadmap、docs、GitHub、安装命令、Blog、联系入口和公开 chat 的准入说明。
- `PublicContactMenu` 目前把用户引向 Bilibili、YouTube、邮件和微信社群；这适合人工沟通，但没有把用户对某个能力的兴趣结构化进入系统。
- `memory/src/web_auth.rs` 和 `crates/hone-web-api/src/routes/web_users.rs` 已经有 invite user、手机号、API key、last login、quota 和 session 统计；这说明系统能管理真实试用用户，但还不能记录“这个人为什么想试用”。
- 现有 proposal 已覆盖 invite activation、privacy-preserving product events、response feedback、shareable briefs、rollout kill switch 等能力，但仍缺少一个面向公开路线图和内容页的显式需求收集闭环。

也就是说，Hone 已经能告诉外部访问者“我们正在做什么”和“如何联系”，却还不能把访问者对具体 roadmap item、channel、deployment mode、skill、API、billing 或 desktop 能力的兴趣转成可执行的产品信号。

## 问题或机会

当前公开路线图承担了产品说明、开源文档导航和信任建设三类职责，但它还不是一个 demand surface。

主要缺口：

1. **需求信号只能流向非结构化渠道。**  
   用户看到 roadmap 后，如果对 `desktop`、`Hone Cloud API`、`Feishu/Telegram`、`company portraits`、`billing` 或某个 skill 感兴趣，只能发邮件、加微信、去 GitHub issue 或直接进 chat。后续运营和产品判断要靠人工记忆。

2. **路线图 item 没有可量化的兴趣优先级。**  
   Roadmap phases 和 capability matrix 能表达 maintainer 计划，但不能回答外部用户最想要什么、哪类用户想要、他们是否愿意试用或付费、是否已经拿到 invite。

3. **公开内容和 invite 管理断开。**  
   Admin 可以创建 invite、查看 quota、last login 和 API key，但不知道这个用户最初被哪个 Blog、视频、roadmap item 或能力吸引。`Invite Activation Funnel` 解决 invite 后的激活阶段，本提案解决 invite 前后的“需求来源和意图”。

4. **产品发布缺少需求闭环。**  
   当某个 proposal 或 roadmap item 落地，系统没有候选人列表可以通知，也没有办法比较“声称感兴趣的人是否真的试用”。这会削弱 P1/P2 需求排序和发布后的复盘。

5. **开源与商业化之间缺少清晰桥梁。**  
   GitHub stars、Blog 阅读、视频观看和社群联系都是有价值的弱信号；Hone 还需要一个明确、隐私克制的强信号：用户主动选择“我想要这个能力 / 我愿意试用这个场景 / 我需要被通知”。

这是 P2，而不是 P1：它不直接影响核心可用性或安全边界，也不应打断 Cloud PG/OSS、ACP runtime、skill runtime、通知可靠性等主线。但它能提高产品优先级判断、试用转化、人工跟进效率和路线图可信度，适合在产品增长与运营基础补齐后落地。

## 方案概述

新增 **Roadmap Demand Capture Loop**：把 public roadmap、Blog、public home 和 contact 入口升级为轻量的显式需求收集面。用户可以对具体能力表达兴趣、留下联系或绑定已有 invite，管理员可以按 feature、persona、source 和状态管理需求，并在功能发布时回流到 invite / activation / rollout 流程。

核心对象：

- `RoadmapItem`
  稳定 feature id，例如 `desktop.bundled_runtime`、`public_chat.attachments`、`hone_cloud.api`、`company_profiles.transfer`、`event_engine.global_digest`、`telegram_channel`、`billing.self_serve`。

- `DemandSignal`
  用户主动表达的兴趣记录，包含 feature id、surface、source path、locale、intent、contact method、optional web user、created_at、status。

- `DemandPersona`
  轻量分类：`individual_investor`、`power_user`、`open_source_builder`、`investment_team`、`api_integrator`、`channel_operator`、`unknown`。

- `DemandIntent`
  `want_invite`、`want_notify`、`want_docs`、`want_api_access`、`want_desktop`、`want_channel_setup`、`want_paid_plan`、`contribute`、`other`。

- `DemandFollowup`
  管理端处理状态：`new`、`triaged`、`invite_sent`、`contacted`、`joined_trial`、`converted`、`closed_no_fit`。

第一版应非常克制：不做公开投票排行榜，不收集投资正文，不引入第三方 analytics，不自动给用户发送营销邮件。它只记录用户主动提交的明确需求，并把需求和已有 invite / product event / activation 系统解耦但可关联。

## 用户体验变化

### 用户端

- Public roadmap 的每个 phase / capability row 可以有一个轻量动作：`I want this`、`Notify me`、`Use this in my workflow`。
- 未登录用户点击后看到简短表单：
  - 你关心哪个使用场景；
  - 邮箱或微信/手机号等联系方式；
  - 是否想申请试用 invite；
  - 可选一句需求说明，限制长度并明确不要填写持仓或隐私信息。
- 已登录 public 用户可以把需求直接绑定到当前 web user，不需要重复填写联系方式。
- Blog 文章底部可以显示与文章主题相关的 roadmap item，例如 Rust/桌面架构文章链接到 `desktop`、`local runtime`、`open-source contribution`。
- 提交后用户看到的是可预期的结果：`已记录需求`、`会在该能力开放试用时通知`、`如果需要邀请码请等待人工处理`。

### 管理端

- Settings 或 Users 下新增 `Demand` tab：
  - 按 feature id、intent、persona、locale、source path、status、created_at 过滤。
  - 查看某条 demand 是否关联现有 `web_invite_user`、是否已经登录、是否完成 invite activation milestones。
  - 从需求记录一键创建 invite、标记 contacted、复制跟进话术或关闭为 no fit。
- Roadmap 管理视图显示每个 item 的 demand count、trial conversion、active users 和 linked proposals。
- 当某个 feature 通过 rollout registry 或 release note 标记为可用时，管理员可以筛出相关需求用户，手工或半自动发送通知。

### 桌面端

- Desktop remote/bundled 模式不需要独立需求系统。
- 桌面首次配置页若发现用户缺少 API key、channel auth 或 runner，可链接到对应 roadmap item 的需求/文档，而不是只显示技术错误。
- 如果用户以 public account 登录桌面，`/me` 可以展示“你关注的路线图能力”。

### 多渠道

- Feishu / Telegram / Discord 不需要直接暴露 roadmap form。
- 当 IM 用户问“什么时候支持 X”时，agent 可以回答当前 roadmap 状态，并提示“可以在 Web roadmap 留下需求”，避免在群聊里收集个人联系方式。
- 对已绑定 workspace 的未来用户，可让 agent 创建 demand signal，但必须先确认，不要把普通闲聊自动变成需求记录。

## 技术方案

### 1. Roadmap item registry

把当前 `public-content.ts` 中纯文案 roadmap/capability matrix 增加稳定 id，但不把 public copy 当数据库真相源。

建议新增一个小型 manifest，例如：

```text
packages/app/src/lib/roadmap-items.ts
```

或后端 shared manifest：

```rust
pub struct RoadmapItemDefinition {
    pub id: &'static str,
    pub title_key: &'static str,
    pub status: RoadmapItemStatus,
    pub phase: RoadmapPhase,
    pub related_routes: &'static [&'static str],
    pub related_proposals: &'static [&'static str],
}
```

第一版可先前端 manifest + 后端 allowlist 双写，避免用户提交任意 feature id。后续若 rollout registry 落地，`FeatureKey` 可以统一。

### 2. Demand signal store

在 `memory` 中新增 SQLite store，例如 `memory/src/roadmap_demand.rs`：

```text
roadmap_demand_signals (
  demand_id TEXT PRIMARY KEY,
  roadmap_item_id TEXT NOT NULL,
  intent TEXT NOT NULL,
  persona TEXT,
  surface TEXT NOT NULL,
  source_path TEXT NOT NULL,
  locale TEXT NOT NULL,
  contact_method TEXT,
  contact_value_hash TEXT,
  contact_display_hint TEXT,
  web_user_id TEXT,
  invite_user_id TEXT,
  message_preview TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

roadmap_demand_followups (
  followup_id TEXT PRIMARY KEY,
  demand_id TEXT NOT NULL,
  action TEXT NOT NULL,
  operator_id TEXT,
  note TEXT,
  created_at TEXT NOT NULL
)
```

隐私边界：

- 默认存 contact hash 和 display hint，例如邮箱域名或手机号后四位；明文联系方式是否保存应由配置决定。
- `message_preview` 必须限制长度，并在 UI 明确禁止填写持仓、账户、交易或其它敏感投资信息。
- 如果 `Privacy-Preserving Product Event Plane` 已实现，提交需求时可以记录 `roadmap.demand_submitted` 产品事件；但 demand signal 本身是显式用户提交，不依赖被动 telemetry。

### 3. Public API

新增：

- `GET /api/public/roadmap/items`
- `POST /api/public/roadmap/demand`
- `GET /api/public/me/roadmap-demand`

Admin API：

- `GET /api/admin/roadmap-demand?item=&status=&intent=&persona=&from=&to=`
- `POST /api/admin/roadmap-demand/:id/status`
- `POST /api/admin/roadmap-demand/:id/followup`
- `POST /api/admin/roadmap-demand/:id/create-invite`

Rate limit 与 abuse guard：

- 匿名提交必须有 IP / anonymous cookie 级别的低频限制。
- 同一 contact + item + intent 在短窗口内去重。
- `message_preview` 走长度限制、HTML escape 和基础垃圾内容过滤。

### 4. Frontend

Public surface：

- `public-roadmap.tsx` 的 phase item / capability row 增加需求 CTA。
- `public-home.tsx` 的 hero、feature carousel 和 Blog 卡片可以按相关 item 加一个低干扰 CTA。
- `public-blog-post.tsx` 在文章末尾展示相关 roadmap item 和 demand CTA。
- `public-me.tsx` 在登录后显示“关注的能力”和状态。

Admin surface：

- 新增 `packages/app/src/pages/roadmap-demand.tsx` 或并入 settings/users。
- `packages/app/src/lib/roadmap-demand.ts` 做数据转换和筛选测试。
- `users.tsx` actor 详情可展示该用户关联 demand 与 activation stage。

### 5. 与既有系统的关系

- 与 invite activation：demand 是 invite 前/早期意图；activation 是 invite 后价值进度。二者通过 `web_user_id` 或 `invite_user_id` 关联。
- 与 product events：product events 记录页面/功能行为；demand 记录用户主动提交的需求。两者可互相引用，但不互相替代。
- 与 response feedback：feedback 评价某次回答；demand 表达对未来产品能力的兴趣。
- 与 rollout kill switch：feature 进入 canary 或 release 后，demand 列表可以作为通知和试用候选池；demand 不决定是否开放功能。
- 与 shareable briefs：brief 可以带来试用兴趣；本提案提供统一落点，不负责 brief 内容生成或分享链路。

## 实施步骤

### Phase 1: Manifest and read-only UI hooks

- 定义稳定 roadmap item id，覆盖当前 public roadmap/capability matrix 中最重要的 10 到 20 个能力。
- 在 public roadmap 页面渲染 `I want this` CTA 和 modal，但先只连接 mock submit 或后端 disabled 状态。
- 给 `roadmap-items` manifest 写前端测试，确保中英文 copy 有对应 item id。

### Phase 2: Demand storage and public submit

- 新增 `roadmap_demand` SQLite store、类型和单元测试。
- 新增 public submit API，包含 allowlist、去重、rate limit、长度限制和隐私校验。
- Public roadmap / Blog / home 接入真实 submit。
- 已登录 public 用户自动关联当前 web user。

### Phase 3: Admin triage

- 增加 admin demand list、filters、status mutation 和 followup timeline。
- 支持从 demand 创建 invite，并把 demand id 写入 invite metadata 或 followup note。
- Settings invite table 增加 demand count / top interest badge。

### Phase 4: Release and rollout loop

- Roadmap item 可关联 proposal、release note、rollout feature key。
- 当 item 状态从 planned/beta 变为 available，admin 可筛出相关 demand 并标记 notified。
- 结合 activation profile 评估 demand -> invite -> first value -> retained 的转化。

## 验证方式

- 单元测试：
  - Roadmap item manifest id 稳定且不重复。
  - Demand store create/list/update/followup 幂等，contact + item 去重生效。
  - 匿名、登录用户、已有 invite 三种提交路径都能生成正确关联。
  - 明文联系方式配置关闭时只保存 hash 和 display hint。
- Web API 测试：
  - 未知 roadmap item id 被拒绝。
  - message 超长、HTML/script、空 intent、重复提交、匿名高频提交返回稳定错误。
  - public 用户只能看自己的 demand，admin 才能查全量。
- 前端测试：
  - `public-roadmap` capability row 能打开 demand modal。
  - `public-me` 能展示已关注能力。
  - admin filter/status model 能处理 `new`、`triaged`、`invite_sent`、`converted`、`closed_no_fit`。
- 手工验收：
  - 未登录用户在 `/roadmap` 对 `Hone Cloud API` 留下需求。
  - 已登录用户在 `/me` 看到该需求。
  - 管理员在 demand tab 创建 invite 或标记 contacted。
  - 提交中不出现 raw API key、持仓详情、聊天正文或本地路径。
- 指标：
  - 每个 roadmap item 的 demand count、invite conversion、first login、first value chat、retained_or_integrated。
  - 需求提交到人工跟进的中位时长。
  - 功能发布后相关 demand 用户的试用率。

## 风险与取舍

- **风险：变成公开投票榜，诱导错误优先级。**  
  取舍：第一版只做管理端可见需求池，不公开票数，不承诺按人数排序。

- **风险：收集联系方式带来隐私负担。**  
  取舍：默认 hash + display hint，明文保存需要显式配置；文案限制用户不要提交投资隐私。

- **风险：和 product events 重复。**  
  取舍：product events 是被动使用事实；demand 是用户主动提交的意图。两者记录不同对象。

- **风险：roadmap item id 与 rollout feature key 漂移。**  
  取舍：先用 allowlist + 测试锁住 id，后续与 rollout registry 合并或建立映射表。

- **风险：增加 public abuse 面。**  
  取舍：匿名 submit 必须限流、去重、长度限制、垃圾过滤；必要时只允许登录用户提交。

- **不做：**
  - 不做自动邮件营销系统。
  - 不做公开投票排序。
  - 不把需求提交自动变成产品承诺。
  - 不收集聊天正文、持仓、交易、API key 或本地文件路径。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与历史 `docs/proposals/`，并重点全文检索了 `roadmap`、`demand`、`waitlist`、`interest`、`feature request`、`invite`、`product event`、`feedback`、`shareable` 等相关主题。

- 不重复 `auto_p1_invite_activation_funnel.md`：该提案关注 invite 用户登录后的激活里程碑；本提案关注用户在 roadmap/blog/home 上显式表达的能力需求，以及这些需求如何进入 invite 前后的跟进池。
- 不重复 `auto_p1_privacy-preserving-product-events.md`：该提案定义隐私优先的产品事件平面，用于被动 adoption intelligence；本提案定义用户主动提交的需求对象、状态和 followup。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：该提案处理单条回答质量反馈；本提案处理未来能力和路线图需求。
- 不重复 `auto_p2_shareable-investment-briefs.md`：该提案关注 brief 分享带来的信任增长；本提案提供 roadmap / public content 的统一需求落点，可接收 brief 带来的兴趣但不生成 brief。
- 不重复 `auto_p1_product-rollout-kill-switch.md`：该提案控制功能对谁开放和如何回滚；本提案只收集和管理谁对某项能力感兴趣，不能替代 rollout 决策。
- 不重复 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 或 `docs/proposals/skill-runtime-multi-agent-alignment.md`：本提案不处理桌面启动接管、sidecar 生命周期、skill frontmatter 或 runner 阶段能力。

差异结论：当前公开 roadmap 是静态发现面，已有增长/观测提案多处理 invite 后激活、被动产品事件或回答反馈。本提案填补的是“公开内容 -> 显式需求 -> invite/跟进 -> 发布通知 -> 转化复盘”的闭环。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/roadmap-demand-capture-loop.md`，并在新增 demand store、public/admin API、roadmap item manifest、privacy policy copy 或 invite metadata 后同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 或 decision。
