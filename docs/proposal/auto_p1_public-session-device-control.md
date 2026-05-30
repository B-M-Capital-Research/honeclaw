# Proposal: Public Session Device Control Center

status: proposed
priority: P1
created_at: 2026-05-30 14:05:10 +0800 CST
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
- `docs/proposal/auto_p0_operator-access-audit.md`
- `docs/proposal/auto_p0_public-edge-abuse-guard.md`
- `docs/proposal/auto_p1_policy-consent-ledger.md`
- `docs/proposal/auto_p1_public-pwa-notification-bridge.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p2_self-serve-billing-checkout.md`
- `memory/src/web_auth.rs`
- `crates/hone-core/src/cloud_runtime.rs`
- `crates/hone-web-api/src/public_auth.rs`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/types.ts`
- `packages/app/src/lib/public-chat.ts`

## 背景与现状

Honeclaw 的 public Web 面已经从展示页扩展成真实用户入口：

- Public app 暴露 `/chat`、`/me`、`/portfolio`、blog、roadmap、terms 和 privacy，登录后能使用公开聊天、历史、上传附件、portfolio 视图和 OpenAI-compatible `/api/public/v1/chat/completions`。
- `memory/src/web_auth.rs` 保存 invite user、手机号、TOS 接受版本、hashed API key、public login cookie sessions，并在 local SQLite 与 cloud PG 之间提供同一存储抽象。
- `WebInviteSession` 目前只有 `session_token`、`user_id`、`created_at`、`expires_at`、`last_seen_at`。SQLite 表 `web_auth_sessions` 也是这几个字段；cloud 模式把 session record 放进 `cloud_web_auth_sessions`。
- `routes/public.rs` 通过 SMS + TOS 创建 `hone_web_session` HttpOnly cookie，支持短期 1 天和长期 30 天 TTL；`/api/public/auth/me` 用 cookie 恢复当前用户；`/api/public/auth/logout` 只删除当前 cookie 对应 session。
- `routes/web_users.rs` 管理端 invite 表能看到 `active_session_count`，也能 disable/reset invite 并清理该用户所有 session；但不能列出具体设备、不能只撤销某个设备，也不能区分“用户自己手机”“桌面远端浏览器”“异常 UA”。
- `public_auth.rs` 已有登录失败限流，`public-edge-abuse-guard` 提案覆盖登录前滥用防线；但登录成功之后的设备可见性、会话治理和用户自助安全操作还没有成为产品面。
- `packages/app/src/pages/public-me.tsx` 目前展示账号信息、额度/入口和 logout；它没有“当前设备”“其它登录设备”“退出所有其它设备”“最近登录风险”这类账号安全体验。

这意味着 Hone 已有最小可用的 public account/session 层，但还没有面向真实用户的 **device/session control**。当 public Web、Hone Cloud API、desktop remote backend 和未来 PWA notification 同时存在时，用户会自然期待能知道自己在哪些设备登录过，并能撤销不认识的登录态。

## 问题或机会

这是 P1。它不如 P0 的 public edge abuse 或 operator access 那样直接挡住系统级滥用，但会显著提升账号信任、云端可用性、支持排障和商业化准备度。

1. **用户只能退出当前设备，不能管理其它设备。**  
   当前 `/api/public/auth/logout` 只删除当前 cookie session。用户如果在共享电脑、旧手机、临时浏览器或远端桌面登录过，无法自助撤销其它 session，只能等 TTL 过期或请求管理员 disable/reset 整个 invite。

2. **管理端只有数量，没有具体风险上下文。**  
   `active_session_count` 能告诉管理员“这个用户有几个 session”，但不能解释 session 是什么时候创建的、最近是否使用、来自 public chat 还是 API/desktop、是否长期 remember、是否可能是重复登录或异常登录。

3. **登录成功后的信任体验弱。**  
   Hone 是投资研究助理，用户会输入持仓、画像、附件和长期记忆。即使系统已经使用 HttpOnly cookie 和 hashed session token，用户侧看不到这些安全边界，会降低公开 Web 与云端工作台的信任感。

4. **Cloud/API/desktop remote 会放大会话治理缺口。**  
   `hone_cloud` runner、public API key、自助 billing、PWA notification、desktop remote backend 都会让同一 public user 出现在更多设备和客户端。没有 session/device 对象，后续很难做设备级 revoke、风险提示、支持定位和账号恢复。

5. **现有安全提案覆盖的是相邻层，不是 public user session 体验。**  
   `Operator Access Audit` 聚焦管理员/operator；`Public Edge Abuse Guard` 聚焦登录前滥用和限流；`User Data Trust Center` 聚焦导出/删除；`Policy Consent Ledger` 聚焦协议同意。它们都需要一个更好的 public session substrate，但没有定义用户可见的设备控制中心。

机会是新增 **Public Session Device Control Center**：把 public login session 从“只有 token 和 TTL 的隐藏记录”升级为“可解释、可撤销、可审计、可面向用户展示的设备级登录态”。

## 方案概述

新增 actor-scoped 的 `PublicSessionDevice` 产品层，仍复用现有 `web_auth_sessions` 作为登录态真相源，但扩展 session metadata、API 和 UI。

核心对象：

- `PublicSessionDevice`：一条 public login session 的设备视图，包含 session id、current flag、created_at、last_seen_at、expires_at、remember flag、client label、user-agent hash/summary、IP hash/last prefix、source surface、revoked_at。
- `PublicSessionEvent`：登录、恢复、退出、撤销、过期清理、管理员清理、异常检测等事件。第一版可只保留最近事件或写入轻量 audit table。
- `SessionRiskSignal`：确定性风险标记，例如 `new_device`、`many_active_sessions`、`stale_but_valid`、`login_after_invite_reset_attempt`、`api_key_recently_reset`。第一版不做设备指纹或风控评分。
- `SessionRevokeAction`：用户撤销单个 session、退出其它设备、管理员撤销单个 session、disable/reset invite 清理全部 session。

第一版目标：

1. 保持 session token 只存 hash，不暴露原始 token。
2. 在创建 session 时记录非敏感设备 metadata。
3. Public `/me` 展示当前设备和其它活跃设备。
4. 用户可以 revoke 非当前 session 或 “logout other devices”。
5. 管理端 invite 用户详情能列出 session devices，并支持单个 revoke。
6. disable/reset invite 继续清理全部 session，但返回更具体的清理结果。

明确不做：

- 不做浏览器指纹、Canvas 指纹、ASN 自动封禁或完整 fraud scoring。
- 不把 IP/user-agent 原文长期持久化。
- 不替代 API key 管理；API key rotation 仍属于 `web_auth` / Hone Cloud API contract。
- 不要求 IM 渠道登录态合并；第一版只治理 public Web session。

## 用户体验变化

### 用户端

- `/me` 增加 `登录设备` 区块：
  - 当前设备：浏览器/平台摘要、登录时间、最近使用、到期时间、是否“保持登录”。
  - 其它设备：按最近使用排序，显示粗粒度 label，例如 `Safari on macOS`、`Chrome on iPhone`、`Desktop remote browser`。
  - 操作：撤销单个设备、退出所有其它设备。
- 新设备登录后，`/me` 顶部显示简短提示：
  - “这是一个新登录设备；如果不是你，请退出其它设备并联系管理员。”
  - 提示只基于 session metadata，不声称完成复杂风控。
- 当前 session 被其它设备或管理员撤销时：
  - `/chat`、`/portfolio`、`/me` 收到 401/403 后跳转登录，并显示 “登录态已失效，请重新验证手机号”。
  - 如果只是 TTL 过期，文案和“被撤销”区分开，避免用户误判账号异常。
- 用户执行敏感操作前，例如重置 API key、提交删除请求、未来 billing 变更，可以提示“建议先确认活跃设备”，但第一版不强制 MFA。

### 管理端

- Settings / Users 的 Web invite 表不再只显示 `active_session_count`：
  - 展开某个 public user，可查看活跃 session 列表、最近 login/revoke event、长期 session 数、即将过期 session 数。
  - 管理员可撤销某个 session，而不是只能 disable/reset 整个 invite。
  - disable/reset invite 的结果展示清理了哪些 session 类型：current/long/short/stale。
- 支持排障问题：
  - 用户说“手机还在线但电脑掉了”，管理员可以看到对应 session 是否过期、撤销或最近未见。
  - 用户说“我没登录过这个设备”，管理员可以撤销指定 session 并建议重置 API key。
- 管理端不显示手机号以外的新敏感原文；IP 只显示掩码或 hash 前缀，不展示完整地址。

### 桌面端

- Desktop remote backend 使用 public Web 或 Hone Cloud API 时，可以在设置页显示当前 remote account 的 session/device health：
  - 当前桌面 shell 是否绑定了可用 public session 或 API key。
  - 如果 remote backend 返回 session revoked / expired，桌面展示明确原因并引导重新登录。
- Bundled local mode 不需要 public session device center；只在连接 remote backend 或打开 public Web surface 时使用。
- 后续如果 desktop 引入内置 public login，可复用同一 device label 和 revoke API。

### 多渠道

- Feishu / Telegram / Discord 不共享 public Web session，第一版不把它们列为 device sessions。
- 如果未来 `linked-user-workspace` 落地，可以把“你的 Web 账号有新设备登录”作为私聊安全提醒，但不能在群聊暴露设备信息。
- PWA notification bridge 可复用 device/session 概念，但两者仍分开：
  - session device 代表登录态和账号访问。
  - web push subscription 代表某个浏览器通知 endpoint。

## 技术方案

### 1. 扩展 session metadata

在 `memory/src/web_auth.rs` 中把 `WebInviteSession` 拆成内部 token record 与外部 device view。SQLite 表可渐进增加列：

```text
web_auth_sessions (
  session_token TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  remember INTEGER NOT NULL DEFAULT 0,
  source_surface TEXT NOT NULL DEFAULT 'public_web',
  client_label TEXT,
  user_agent_hash TEXT,
  user_agent_summary TEXT,
  ip_hash TEXT,
  ip_hint TEXT,
  revoked_at TEXT,
  revoked_by TEXT,
  revoke_reason TEXT
)
```

Cloud 模式继续用 `cloud_web_auth_sessions.record` 存 JSON，并把 `expires_at` 列用于清理/索引。新增字段必须兼容旧 record：缺失时推导为 `remember = expires_at - created_at > 1 day`，`source_surface = public_web`。

隐私边界：

- `session_token` 仍只存 hash；SQLite legacy plaintext token 兼容路径只用于过渡验证，不用于新写入。
- `user_agent_hash` 用稳定 hash 做去重；`user_agent_summary` 只保留浏览器/平台大类。
- `ip_hash` 用带 server salt 的 hash；`ip_hint` 只保留粗粒度掩码，例如 `203.0.113.*` 或 `2001:db8:...`。
- 不保存原始 `X-Forwarded-For` 链；只使用 `public_client_key` 已选择的可信来源策略。

### 2. 新增存储 API

`WebAuthStorage` 增加：

```rust
pub struct PublicSessionDeviceInfo {
    pub session_id: String,
    pub user_id: String,
    pub current: bool,
    pub created_at: String,
    pub expires_at: String,
    pub last_seen_at: String,
    pub remember: bool,
    pub source_surface: String,
    pub client_label: Option<String>,
    pub user_agent_summary: Option<String>,
    pub ip_hint: Option<String>,
    pub revoked_at: Option<String>,
    pub revoked_by: Option<String>,
    pub revoke_reason: Option<String>,
}

pub fn list_session_devices_for_user(
    &self,
    user_id: &str,
    current_session_token: Option<&str>,
) -> HoneResult<Vec<PublicSessionDeviceInfo>>;

pub fn revoke_session_for_user(
    &self,
    user_id: &str,
    session_id: &str,
    revoked_by: SessionRevokedBy,
    reason: &str,
) -> HoneResult<bool>;

pub fn revoke_other_sessions_for_user(
    &self,
    user_id: &str,
    current_session_token: &str,
    reason: &str,
) -> HoneResult<u32>;
```

`session_id` 不应等于 token hash 原文。建议用 `sha256("public-session-id:" + token_hash)` 的短前缀或单独生成 `session_id` 列，避免 API 参数成为可离线关联的 token hash。

### 3. Public API

在 `build_public_app` 增加：

- `GET /api/public/auth/sessions`
- `POST /api/public/auth/sessions/{session_id}/revoke`
- `POST /api/public/auth/sessions/logout-others`

这些路由必须先通过当前 cookie 认证，并只能操作当前 user 的 session。撤销当前 session 可以允许，但应等同 logout 并清 cookie；撤销其它 session 不影响当前 cookie。

`/api/public/auth/me` 可返回简版：

```json
{
  "user": { "...": "..." },
  "session": {
    "session_id": "sess_...",
    "expires_at": "...",
    "remember": true,
    "active_session_count": 3
  }
}
```

前端需要完整列表时再调用 `/auth/sessions`，避免每次恢复登录都读全列表。

### 4. Admin API

在 admin `/api/web-users/invites` 旁增加：

- `GET /api/web-users/invites/{user_id}/sessions`
- `POST /api/web-users/invites/{user_id}/sessions/{session_id}/revoke`
- `POST /api/web-users/invites/{user_id}/sessions/revoke-all`

admin revoke 需要沿用现有 admin API auth；后续 `Operator Access Audit` 落地后，这些操作应写 operator audit。第一版至少在返回值和 logs 里标注 `revoked_by = admin`.

### 5. 前端改造

Public:

- `packages/app/src/lib/types.ts` 增加 `PublicSessionDeviceInfo`。
- `packages/app/src/lib/api.ts` 增加 public session APIs。
- `packages/app/src/pages/public-me.tsx` 增加 `登录设备` 区块和撤销按钮。
- `packages/app/src/lib/public-chat.ts` / `chat.tsx` 处理 session revoked 与 expired 的不同错误文案。

Admin:

- `WebInviteInfo` 保留 `active_session_count`，但可新增 session drawer 数据模型。
- `packages/app/src/pages/settings.tsx` 或 `users.tsx` 中的 invite 管理面增加 session 展开区。
- 如果 `active_session_count > 1`，显示可操作提示，而不是只有数字。

### 6. 兼容与迁移

- SQLite `ensure_column` 增加新列，旧 session 不需要强制迁移。
- Cloud PG `cloud_web_auth_sessions.record` 读取缺字段时补默认值。
- 旧 session 没有 device label 时展示 `Unknown browser` 和创建/最近使用时间。
- disable/reset invite 的现有行为保持：仍清理所有 session；只是可以记录更细事件和返回更具体 count。
- 不改变 `SESSION_TTL_DAYS_LONG` / `SESSION_TTL_DAYS_SHORT` 的契约；remember 只是解释字段。

## 实施步骤

1. **Session metadata substrate**
   - 扩展 `WebInviteSession` / internal cloud record。
   - 在 SMS login 创建 session 时从 headers 生成 client summary、UA hash、IP hint/hash、remember 和 source surface。
   - 增加 local SQLite 与 cloud PG 读写兼容测试。

2. **Public session APIs**
   - 增加 list/revoke/logout-others 路由。
   - 区分 current session、other session、expired/revoked/missing。
   - 增加 public auth 单元测试和 API contract 测试。

3. **Public `/me` UI**
   - 新增 device list、current badge、expires/last seen、revoke action。
   - 处理 loading/error/empty/unknown metadata 状态。
   - 将 401/403 session revoked 文案接入 public chat/account 页面。

4. **Admin session operations**
   - 增加 admin list/revoke 路由。
   - invite 表或用户详情抽屉展示 session devices。
   - 保留 disable/reset 全清理行为，并显示更具体清理反馈。

5. **Risk hints and support hooks**
   - 增加简单确定性 risk signals：active sessions 过多、长期 session 长时间未用、新 device。
   - 日志中只写 session id、user id、reason，不写 raw token、raw IP 或完整 UA。
   - 为未来 support bundle 和 operator audit 预留事件字段。

## 验证方式

- Rust 单元测试：
  - `memory/src/web_auth.rs` 覆盖新 session metadata 默认值、token hash 不泄漏、legacy session 兼容、list devices、revoke one、logout others、admin revoke。
  - cloud record serialization/deserialization 覆盖缺字段兼容。
- Web API 测试：
  - 当前 cookie 能列出自己的 sessions。
  - 用户不能撤销其它 user 的 session。
  - revoke 当前 session 后 cookie 被清理或下一次 `auth/me` 返回 unauthorized。
  - logout others 保留当前 session，撤销其它 active sessions。
- 前端单元测试：
  - `/me` device list 对 current/expired/unknown label/risk hint 的渲染。
  - public chat 对 expired vs revoked 的错误文案分支。
- 手工验收：
  - 用两个浏览器分别登录同一手机号，在 `/me` 看到两个设备。
  - 在浏览器 A 撤销浏览器 B，B 刷新后要求重新登录，A 仍可用。
  - 管理端撤销某个 public session，用户端出现明确失效提示。
  - disable invite 仍清理全部 session，reset invite 仍旋转 invite code。
- 安全检查：
  - 搜索 logs/API response，确认没有 raw `hone_web_session`、完整 IP、完整 UA。
  - 确认 session id 不能用于认证，只能作为 revoke/list 的不可逆标识。

## 风险与取舍

- **隐私 vs 可排障性。** 设备列表越详细越好排障，但 Hone 不应持久化完整 IP/UA。第一版只保留摘要和 hash，牺牲精细定位。
- **误判风险。** 不做复杂设备指纹，因此 “new device” 只是提示，不应触发强制封禁。
- **UI 复杂度。** `/me` 已经承担账号、额度和入口功能，device center 需要保持紧凑，避免把 public account 页变成后台控制台。
- **Cloud/local 双实现成本。** Local SQLite 和 cloud PG record 都要支持字段兼容；但这正好能验证 public auth hot path 的云化质量。
- **旧 session 元数据缺失。** 已有登录态只能显示 unknown device。接受这个过渡状态，不强制所有用户重新登录。
- **不做边界。** 第一版不实现 MFA、地理位置风控、设备指纹、跨渠道身份合并、API key device binding，也不替代 operator session 管理。

## 与已有提案的差异

- `auto_p0_operator-access-audit.md` 聚焦管理员/operator 的角色、token、session 和审计。本提案聚焦 public Web 用户自己的登录设备和自助撤销，不处理 admin 权限模型。
- `auto_p0_public-edge-abuse-guard.md` 聚焦登录前和 public edge 的滥用防线，例如 IP/phone/device cookie 限流。本提案聚焦登录成功后的 session/device 可见性和撤销。
- `auto_p1_user-data-trust-center.md` 覆盖导出、删除、数据范围和账号信任大面。本提案是其下游依赖之一，只解决 active login sessions 的产品和存储缺口。
- `auto_p1_policy-consent-ledger.md` 记录协议接受/撤回及其影响。本提案可以在撤回同意时被调用清理 sessions，但不定义 consent 状态机。
- `auto_p1_public-pwa-notification-bridge.md` 定义 Web Push subscription 和设备通知 endpoint。本提案定义登录 session device；两者可在 UI 中相邻展示，但生命周期、权限和 revoke 对象不同。
- `auto_p2_self-serve-billing-checkout.md` 关注订阅、付款状态和 API key/额度联动。本提案不处理 plan 或 payment，只保证用户能看到并撤销自己的 public 登录态。

查重结论：`docs/proposal/` 与 `docs/proposals/` 已有 operator access、public edge abuse、PWA notification、user data trust、billing 和 API contract 相关提案，但没有一篇单独定义 public Web 用户的 session device metadata、用户自助撤销、admin 单 session revoke 与 `/me` 设备中心。因此本主题不重复，且能为公开 Web、云端 API 和 desktop remote 的信任体验补上必要底座。
