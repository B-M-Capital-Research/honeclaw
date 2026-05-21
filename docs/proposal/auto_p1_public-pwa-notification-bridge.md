# Proposal: Public PWA Notification Bridge for Mobile Retention

status: proposed
priority: P1
created_at: 2026-05-21 14:03:44 +0800
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
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_end-user-notification-control.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `packages/app/index.html`
- `packages/app/public/site.webmanifest`
- `packages/app/public/asset-recovery-sw.js`
- `packages/app/src/lib/asset-recovery.ts`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-me.tsx`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-event-engine/src/router/`
- `crates/hone-event-engine/src/sinks/`
- `memory/src/web_auth.rs`

## 背景与现状

Honeclaw 已经有一个面向公开用户的 Web surface：`packages/app/src/app.tsx` 在 `VITE_HONE_APP_SURFACE=public` 时暴露 `/`、`/roadmap`、`/chat`、`/me`、`/portfolio`、`/terms`、`/privacy`。`README.md` 也把 public chat、portfolio monitoring、scheduled tasks 和跨平台通知作为产品主线之一。

当前 public Web 已具备基础 PWA 形态：`packages/app/index.html` 引入 `/site.webmanifest`，`packages/app/public/site.webmanifest` 设置了 `display: "standalone"` 和 192/512 图标；`packages/app/src/lib/asset-recovery.ts` 会注册 `/asset-recovery-sw.js`，但这个 service worker 只拦截 `/assets/` 的 stale chunk，负责刷新恢复，不承担离线 shell、安装引导或通知投递。

后端方面，`crates/hone-web-api/src/routes/public.rs` 已经提供 public SMS 登录、cookie session、public chat、附件上传、history 和 actor-scoped public SSE；`routes/notifications.rs` 能聚合 cron 与 event-engine delivery log；`routes/notification_prefs.rs` 已有 actor 级通知偏好模型。事件引擎和渠道 sink 主要面向 Feishu / Telegram / Discord / iMessage 等外部 IM 或管理端排障。

也就是说，Hone 已经有公开移动入口和通知系统，但缺少一个低摩擦、用户自助、无需第三方 IM 绑定的 **浏览器原生到达渠道**。对很多投资用户来说，手机浏览器/PWA 是试用 Hone 的第一站；如果只能依赖“保持网页打开的 SSE”或另行绑定 IM，关键提醒和复访路径都会被切断。

## 问题或机会

1. **公开用户的留存链路过度依赖外部渠道。** Feishu/Telegram/Discord/iMessage 很适合重度用户和团队，但邀请制 public Web 用户可能尚未配置任何 IM channel。完成 SMS 登录后，如果没有浏览器通知授权和安装提示，Hone 很难把“财报提醒、主线变化、定时任务结果”带回用户。

2. **PWA 形态目前停在 manifest 层。** `site.webmanifest` 可以让浏览器识别安装能力，但产品没有解释何时建议安装、安装后有哪些移动工作台入口，也没有把 `/chat`、`/portfolio`、`/me` 组织成 installed app 的日常使用路径。

3. **SSE 不是通知渠道。** public SSE 适合前台网页实时刷新，不能可靠覆盖锁屏、后台、关闭页面后的移动提醒。把 SSE 当成通知替代，会让用户以为自己开启了提醒，实际却只能在页面活跃时收到。

4. **通知偏好和投递通道缺一条 Web Push 目标。** `NotificationPrefs` 可以控制事件类型、severity、quiet hours、digest slots，但没有浏览器 `PushSubscription` 这类 delivery target；event-engine sink 也没有把浏览器 push 作为一等目标纳入 delivery log。

5. **增长体验存在自然机会。** AI agent 产品近期普遍强调“从聊天窗口变成可持续陪伴的工作台”：移动安装、系统通知、快捷入口、分享/回流、任务完成唤醒，是从 demo 转成习惯的关键。本提案不需要依赖外部新闻事实；它是对当前 public Web 与多渠道架构缺口的直接补齐。

这值得列为 P1：它不改变核心投资分析逻辑，却直接影响公开用户激活、复访、提醒可信度和渠道配置门槛；同时可以分阶段落地，不需要先完成 linked workspace、支付或团队协作。

## 方案概述

新增 **Public PWA Notification Bridge**：把 Hone public Web 从“可安装网页”升级为“可授权通知、可解释偏好、可审计投递、可从通知回到上下文”的轻量移动工作台。

核心对象：

- `WebPushSubscription`：绑定到 public web actor，保存 endpoint、p256dh/auth key、user_agent、created_at、last_seen_at、revoked_at、permission_state 和 install/source metadata。
- `WebPushTarget`：event-engine / cron delivery 可使用的 actor target，不替代 Feishu/Telegram/Discord/iMessage，只作为 `channel=web_push` 的新 sink。
- `PwaInstallState`：前端本地状态，记录是否 standalone、是否已提示安装、用户是否暂不安装、最后一次安装 CTA 展示场景。
- `NotificationDeepLink`：把 push payload 链回 `/chat`、`/portfolio`、`/notifications` 或未来事件详情，避免用户点开后只落到首页。

一期目标只支持 public Web 登录用户，先做主动授权、订阅保存、测试通知、事件引擎 digest/direct delivery 和基础投递日志。管理端/桌面端只做观测与配置，不把 desktop bundled 本地通知一并塞进本阶段。

## 用户体验变化

### 用户端

- `/me` 增加移动工作台区块：显示当前是否已安装为 app、浏览器通知是否可用、当前设备是否已订阅。
- `/chat` 和 `/portfolio` 在用户完成关键动作后出现克制的安装/通知 CTA，例如首次成功对话、创建/导入持仓、开启投资提醒后，而不是刚进首页就索要权限。
- 用户点击“开启提醒”后先看到明确说明：提醒范围、quiet hours、可随时关闭、不会发送交易指令。确认后再触发浏览器 `Notification.requestPermission()`。
- 通知点击后回到相关上下文：财报/价格事件进入 portfolio 或 future notifications detail；定时任务结果进入 chat/history；账号或授权问题进入 `/me`。
- 如果浏览器不支持 Web Push、在 iOS 非 standalone 场景不可用、或用户拒绝权限，页面应显示替代路径：保持 IM channel、邮件/社区联系，或稍后再试。

### 管理端

- Users 或 Notifications 页面能看到某个 public actor 是否有 active web_push target、最近一次订阅心跳、最近一次成功/失败投递、失败原因分类。
- Notifications 聚合页将 `delivery_channel=web_push` 纳入过滤和 summary，不把它混成普通 web SSE。
- Settings 可配置 Web Push master switch、VAPID key 状态、payload 最大长度、默认是否允许 digest/direct 两类 push。

### 桌面端

- Desktop remote 模式访问 public service 时可以展示“此服务支持浏览器/PWA 通知”的状态，但不负责生成本机通知。
- Desktop bundled/local 模式本阶段不启用 Web Push，避免把本地 loopback、证书、service worker scope 和 native notification 混在一起。桌面本机通知可以作为后续独立 proposal。

### 多渠道

- Web Push 成为和 Feishu/Telegram/Discord/iMessage 并列的轻量 sink，但只面向 public web actor 的浏览器订阅。
- 同一 actor 如果同时绑定 IM 与 Web Push，第一版采用用户偏好里的 channel allow/block 决策；不自动双发所有 High 事件，避免噪音。
- IM 失败不自动 fallback 到 Web Push，除非未来 delivery decision loop 明确支持 fallback policy。第一版只保证各 channel 的投递日志清楚。

## 技术方案

### 1. 前端 PWA 与 service worker 分层

保留现有 `/asset-recovery-sw.js` 的 stale asset 恢复能力，但不要继续把所有 PWA 逻辑堆进一个隐式恢复脚本。建议新增：

- `packages/app/public/pwa-sw.js`：处理 `push`、`notificationclick`、订阅状态消息和必要的 cache version metadata。
- `packages/app/src/lib/pwa.ts`：封装 support detection、permission request、subscribe/unsubscribe、standalone detection、deep-link helpers。
- `packages/app/src/components/public-pwa-card.tsx`：在 `/me` 使用的设备/通知状态卡片。

`asset-recovery-sw.js` 可以保留为同一个 service worker 的子逻辑，也可以在合并前先维持现状；关键是不要让 push handler 破坏现有 asset recovery reload 行为。若最终只能注册一个 service worker，则应把 recovery fetch handler 和 push/click handler 合并到 `pwa-sw.js`，并增加回归测试覆盖 `/assets/` 404 仍会通知页面刷新。

### 2. Web Push subscription 存储

在 `memory` 或 `crates/hone-web-api` 下新增 public web push 存储，优先放入共享 SQLite，和 `web_auth.rs` 的 public user/session 模型保持接近：

```sql
CREATE TABLE web_push_subscriptions (
  id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT NOT NULL DEFAULT '',
  endpoint_hash TEXT NOT NULL UNIQUE,
  endpoint_ciphertext TEXT NOT NULL,
  p256dh_ciphertext TEXT NOT NULL,
  auth_ciphertext TEXT NOT NULL,
  user_agent TEXT NOT NULL DEFAULT '',
  permission_state TEXT NOT NULL,
  created_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  revoked_at TEXT,
  last_error TEXT,
  metadata_json TEXT NOT NULL DEFAULT '{}'
);
```

安全取舍：

- endpoint 和 keys 至少不要明文写入普通日志；若 secrets vault 未落地，先用 config-owned local encryption key 或 SQLite private domain 标记，避免导出/支持包误带。
- subscription 必须归属当前 `hone_web_session` 认证出的 `ActorIdentity(channel=web)`，不允许前端提交任意 actor。
- 同一 endpoint 重新订阅时更新 last_seen，不创建重复目标。

### 3. Public API

新增 public 认证 API：

- `GET /api/public/pwa/status`：返回 support hints、当前 actor 订阅数量、当前 device endpoint 是否已登记、server VAPID public key。
- `POST /api/public/pwa/subscribe`：保存 subscription，输入浏览器 PushSubscription JSON、permission state、client metadata。
- `POST /api/public/pwa/unsubscribe`：按 endpoint 或 subscription id revoke。
- `POST /api/public/pwa/test`：发送一条低风险测试通知，写 delivery log。

管理端 API：

- `GET /api/admin/web-push/subscriptions?actor=...`
- `POST /api/admin/web-push/:id/revoke`
- Settings meta 暴露 Web Push configured/readiness 状态，但不泄露 private key。

### 4. Event-engine / cron sink

在 `crates/hone-event-engine/src/sinks/` 增加 `web_push.rs`：

- 输入 `MarketEvent` / rendered digest / cron execution result。
- 根据 actor 查找 active web push subscriptions。
- 渲染短 payload：title、body、icon、badge、tag、timestamp、deep_link、event_id、urgency。
- 调用 Web Push library 或最小 VAPID HTTP client。
- 对 410/404 endpoint 自动 revoke，对 payload too large 记录 `payload_truncated`，对权限/配置缺失记录 `target_missing` 或 `provider_not_configured`。

delivery log 必须把 `delivery_channel=web_push`、subscription id、status、error code、deep_link 写入 detail，方便 `routes/notifications.rs` 和管理端排查。

cron 自定义任务的 Web Push 支持应谨慎：第一版只允许已有 `channel_target=web_push:<subscription-or-actor>` 或 actor-level web push enabled 的任务投递，不把所有 Web 用户的 cron 输出默认推送到浏览器。

### 5. NotificationPrefs 集成

复用现有 `NotificationPrefs`，不要新增第二套提醒偏好。建议增加或投影：

- channel-specific allow/block：`web_push_enabled` 或通用 channel preference map。
- per-kind 默认：High/direct 事件可 push，digest 可按 slot push，Low 单条不直接 push。
- quiet hours 必须对 web push 生效。
- test notification 不受 event filters 影响，但写 audit/detail。

如果 `auto_p1_end-user-notification-control.md` 后续落地，本提案的 `/me` PWA card 应链接到同一偏好编辑面，而不是独立编辑所有细节。

### 6. Deep link 与路由

Push payload 中只放稳定短链接：

- `/portfolio?symbol=NVDA&event=<event_id>`
- `/chat?session=<session_id>&run=<run_id>`
- `/me?tab=notifications`

前端如果当前未登录，应先走 public login，然后恢复目标路径。不要把敏感事件正文、完整持仓、公司画像片段直接塞进 push payload；锁屏通知默认只展示摘要级标题和简短正文。

## 实施步骤

### Phase 1: PWA readiness and local state

- 增加 `pwa.ts` support detection、standalone detection、install CTA state。
- 在 `/me` 增加只读 PWA/notification readiness card。
- 扩展 `site.webmanifest` 的 `start_url`、`scope`、`description`、`shortcuts`，但保持现有图标路径兼容。
- 验证现有 asset recovery service worker 行为未退化。

### Phase 2: Subscription API and test push

- 新增 Web Push subscription 存储和 public subscribe/unsubscribe/status API。
- 配置 VAPID public/private key 来源，缺失时返回 clear readiness error。
- 实现 `/api/public/pwa/test`，前端提供测试按钮。
- 管理端 Settings/Users 显示订阅状态和最近测试结果。

### Phase 3: Event-engine direct and digest delivery

- 新增 `web_push` sink，并把 delivery log 接入 notifications 聚合。
- 先支持 event-engine High direct 和 digest slot，cron 输出只做显式目标。
- 处理 404/410 自动 revoke、quiet hours、payload truncation 和 provider-not-configured。

### Phase 4: Product polish and activation funnel

- 在首次成功对话、首次 portfolio 上线、开启 reminder 后展示分场景 CTA。
- 通知点击 deep link 恢复上下文。
- 将安装/授权/测试/投递事件接入未来 privacy-preserving product event plane；未落地前只保留本地最小计数。

## 验证方式

- Unit tests：
  - PWA support detection 在 unsupported、permission denied、standalone、normal browser 下返回稳定状态。
  - subscription endpoint hash 去重、revoke、last_seen 更新。
  - Web Push sink 对 404/410 自动 revoke，对 payload too large 截断并记录 detail。
  - quiet hours、kind filter、digest/direct 决策与 `NotificationPrefs` 一致。
- Frontend tests：
  - `/me` PWA card 在未登录、已登录、unsupported、denied、subscribed 状态下文案和按钮正确。
  - `/chat` 成功对话后 CTA 不遮挡 composer，不重复骚扰。
- Regression scripts：
  - `tests/regression/ci/test_public_pwa_manifest.sh`：校验 manifest 必含 name、short_name、start_url、scope、display、icons、shortcuts。
  - Web Push HTTP client 可以用本地 mock server 验证 VAPID header、payload shape、410 revoke。
- Manual verification：
  - Chrome/Edge desktop 安装 PWA、授权通知、收到 test push、点击回到 `/me`。
  - Android Chrome 安装后收到 event-engine test push。
  - iOS Safari 对不支持/需安装后授权的路径给出正确降级说明。
  - Public service HTTPS 部署下 service worker scope 与 API cookie session 正常工作。
- Metrics：
  - public 登录用户中 PWA installed/notification granted/subscribed 比例。
  - push send success、click-through、unsubscribe、permission denied。
  - 已订阅用户的 7 日复访率、portfolio 页面复访率、提醒点击后会话启动率。

## 风险与取舍

- 风险：浏览器通知可能造成金融焦虑或打扰。取舍：默认只在用户明确开启后发送；Low 事件不单条 push；quiet hours 强制生效；锁屏 payload 不包含交易建议。
- 风险：Web Push 在不同平台支持差异大，尤其 iOS/Safari 行为复杂。取舍：第一版明确 support detection 和降级，不承诺所有浏览器一致。
- 风险：subscription endpoint/key 属于敏感数据。取舍：不写日志、不进入普通 support bundle；导出/删除路径要按 actor 数据处理。
- 风险：与 IM channel 双发造成噪音。取舍：第一版由 NotificationPrefs 决定 channel，管理端可以看到重复投递，不做隐式 fallback。
- 风险：service worker 变复杂后影响 asset recovery。取舍：把 stale asset recovery 纳入 PWA worker 回归测试，必要时先维持两个文件但只注册一个合并入口。
- 风险：PWA 工作会拖入 desktop native notification。取舍：本提案只覆盖 public Web/browser push；desktop native notification 另行设计。

## 与已有提案的差异

- 与 `auto_p1_end-user-notification-control.md` 不重复：该提案解决用户如何编辑通知偏好；本提案解决浏览器/PWA 如何成为一个真实投递 target，并把授权、subscription、service worker、sink 和 deep link 接起来。
- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案解释事件为什么推送、过滤或降级；本提案新增 `web_push` 这个 delivery channel，并要求其写入同一类 delivery log。
- 与 `auto_p1_invite_activation_funnel.md` 不重复：该提案关注邀请用户激活里程碑；本提案提供激活后的移动留存能力，可作为 funnel 中一个 milestone。
- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案处理跨渠道真实用户资产归属；本提案第一版只绑定当前 public web actor，不合并 IM actor 身份。
- 与 `auto_p1_multichannel-render-preview.md` 不重复：该提案关注多渠道消息渲染预览；本提案关注浏览器 Push payload、service worker 和通知点击回流。
- 与 `auto_p1_product-rollout-kill-switch.md` 不重复：该提案提供功能开关和灰度控制；本提案可受其控制，但不定义全局 rollout 系统。
- 与 `auto_p1_privacy-preserving-product-events.md` 不重复：该提案定义产品事件采集平面；本提案只建议记录 PWA 安装/授权/点击指标，具体采集可等事件平面落地。

查重结论：`docs/proposal/` 与 `docs/proposals/` 已覆盖通知偏好、通知决策、多渠道渲染、邀请激活、跨渠道 workspace、产品事件和运行排障，但没有覆盖“public Web 作为 PWA 安装入口 + Web Push subscription + event-engine/cron browser sink + 通知 deep link 回流”的端到端产品架构。因此本主题是新的、可落地的 P1 提案。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或运行配置。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/public-pwa-notification-bridge.md`，并在新增 service worker、public API、subscription 存储、event-engine sink、manifest contract 或通知偏好字段时同步更新 repo map、invariants、相关 runbook 和必要的 decision/ADR。
