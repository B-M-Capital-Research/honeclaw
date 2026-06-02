# Proposal: Durable Web Event Inbox for Browser Push Continuity

status: proposed
priority: P1
created_at: 2026-06-03 02:04:03 +0800
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
- `docs/proposal/auto_p1_public-pwa-notification-bridge.md`
- `docs/proposal/auto_p1_end-user-notification-control.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_interrupted-run-recovery-inbox.md`
- `docs/proposal/auto_p1_context-return-links.md`
- `crates/hone-web-api/src/routes/events.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/lib.rs`
- `crates/hone-web-api/src/state.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `memory/src/session.rs`
- `memory/src/cron_job/history.rs`
- `packages/app/src/context/sessions.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/lib/stream.ts`
- `packages/app/src/lib/api.ts`

## 背景与现状

Honeclaw 已经把 Web、Public Web、桌面壳和多渠道主动消息接到同一个后端运行面：

- `crates/hone-web-api/src/routes/events.rs` 为管理端会话页提供 `/api/events` SSE，接收 `scheduled_message`、`push_message` 和 iMessage 运行事件。
- `crates/hone-web-api/src/routes/public.rs` 为 public `/chat` 提供 `/api/public/events` SSE，按当前 `hone_web_session` 推导 `ActorIdentity(channel=web)` 后过滤 `PushEvent`。
- `crates/hone-web-api/src/lib.rs` 中的 `WebBroadcastSink` 把 event-engine Web delivery 写入 `state.push_tx`，再由 SSE 广播给当前在线浏览器。
- Web cron 结果在 `routes/events.rs` 中先写入 session，再通过 SSE 让打开的浏览器实时追加；event-engine Web push 目前主要走 `push_message` 广播。
- `packages/app/src/context/sessions.tsx` 和 `packages/app/src/pages/chat.tsx` 都使用 `EventSource` 监听 `scheduled_message` / `push_message`，收到后直接用前端临时 `messageId()` 追加到当前视图。
- 当 broadcast receiver lagged 时，后端只发 `events_lagged`；前端目前没有 cursor、补拉窗口、事件去重或 ack 语义。

这套设计对“页面打开时实时看到新消息”有效，但它不是一个 durable inbox。浏览器断线、标签页休眠、手机切后台、桌面切换用户、broadcast 缓冲溢出、SSE 重连时，在线事件可能只存在于内存广播里。对于投资助理，主动提醒、定时任务结果和后台事件不是普通 toast；用户需要相信“我离开浏览器一会儿，回来仍能看到 Hone 说过什么”。

## 问题或机会

### 主要问题

1. **Broadcast SSE 不是消息真相源。**  
   `state.push_tx` 是进程内广播通道。新连接只能收到连接后的事件，无法按 actor 从上次 cursor 补拉。`events_lagged` 只告诉客户端跳过了 N 条，不能告诉跳过的是哪几条。

2. **Public chat 的主动消息可能只停留在前端临时状态。**  
   Public `/chat` 收到 `push_message` 后直接 append assistant message。若该 push 没有同步写入 session 或 notification log，刷新后可能只能靠其它页面推断；即使有些 cron 消息已入 session，前端也没有基于 event id 的去重。

3. **管理端与 public 端实时语义不一致。**  
   管理端会话页、public chat、logs stream 都用了 broadcast SSE，但各自处理 lag/reconnect 的方式不同。用户看到的“实时更新”、管理员看到的“推送记录”、后端记录的“已送达”之间缺少统一的 Web event delivery receipt。

4. **桌面和移动浏览器更容易断线。**  
   Desktop bundled/remote、mobile Safari/Chrome、系统休眠都会导致 `EventSource` 断开。当前前端能重连，但不能告诉后端“我已经看到 event E123”，也不能要求“把 E123 之后的事件补给我”。

5. **后续 PWA/Web Push 仍需要前台 inbox。**  
   `auto_p1_public-pwa-notification-bridge.md` 解决页面关闭或锁屏后的系统级通知，但用户点回应用后仍需要一个 durable in-app event inbox 来显示未读、已读、补拉和上下文跳转。不能把 PWA push 当作前台消息存储。

### 机会

新增 **Durable Web Event Inbox**：把发往 Web/public/admin 浏览器的 `scheduled_message`、`push_message`、recovery notice、未来 context link action 等在线事件写成 actor-scoped append-only inbox，再用 SSE 只负责低延迟传输。浏览器断线后按 cursor 补拉，收到后 ack，UI 以稳定 event id 去重。

这是 P1，因为它直接影响主动提醒可信度、public 用户留存、桌面/移动体验和运维排障；同时可以分阶段落地，不需要改变 event-engine 决策、Web Push、cron 存储或 session 主链路。

## 方案概述

新增一个 actor-scoped 的 `WebEventInbox`，作为浏览器可见实时事件的 durable 投影层。

核心对象：

- `WebEventItem`
  - `event_id`：稳定递增或 sortable id。
  - `actor`：沿用 `ActorIdentity`，不改变 actor/session 隔离。
  - `surface`：`admin_console`、`public_web`、`desktop_admin`、`desktop_public`、`channel_bridge`。
  - `event_type`：`scheduled_message`、`push_message`、`recovery_notice`、`run_status`、`notification_decision`、`context_action`。
  - `payload_json`：用户可见短 payload，不保存 secret、cookie、sandbox 外路径。
  - `source`：`cron`、`event_engine`、`imessage_bridge`、`recovery`、`manual_admin`、`system`。
  - `source_ref`：可选 `job_id`、`execution_id`、`event_id`、`session_id`、`notification_id`。
  - `created_at` / `expires_at`：控制补拉窗口和保留策略。
  - `delivery_state`：`pending`、`sent_live`、`acknowledged`、`expired`。
  - `first_sent_at` / `acknowledged_at` / `ack_client_id`：用于排障和未读计数。

第一版原则：

- SSE 仍保留，作为实时传输通道。
- 所有 Web-facing push 先写 inbox，再广播 event id + payload。
- 客户端初连时先 `GET /api/.../web-events?after=<cursor>` 补拉，再打开 SSE。
- 客户端收到 SSE 后按 `event_id` 去重，批量 ack。
- 不把完整 chat history 重复写进 inbox；inbox 只保存“需要浏览器实时/未读呈现的事件投影”。

## 用户体验变化

### Public 用户端

- `/chat` 打开时先恢复最近未确认事件，再连接 SSE。用户离开页面后回来，仍能看到期间的定时任务结果和主动推送。
- 如果发生 `events_lagged` 或网络断开，页面显示轻量状态“正在补齐最近消息”，然后通过 cursor 补拉，不让用户自己刷新猜测。
- 每条主动消息有稳定 id，不会因 SSE 重连重复出现两次。
- 未来 `/me` 可展示未读数量，例如“3 条 Hone 更新”，点击回到 chat、portfolio 或 notification detail。

### 管理端

- Sessions 页面不再只依赖 live broadcast 追加 scheduled/push 消息；切换 actor 或重连时按 actor cursor 补齐。
- Notifications / task-health / sessions 可以从 inbox event 打开 source ref，例如 cron execution、event-engine delivery、session message。
- 管理端可显示“live sent but not acknowledged”的浏览器事件，帮助区分后端没生成、SSE 没送达、用户浏览器没看到。

### 桌面端

- Desktop bundled/remote 下，后台 Web console 休眠再恢复时自动补齐最近 event，而不是只依赖当前进程内广播。
- 如果 desktop 进行了 channel process cleanup 或 backend restart，重启后的前端可以通过 inbox 看到之前未读的 recovery notice。
- 本提案不做原生通知；它为未来 desktop alert center 提供可读的 Web event source。

### 多渠道

- Feishu/Telegram/Discord/iMessage 不需要接入这个 inbox。它只处理 Web browser surfaces。
- 如果 channel bridge 把 iMessage 事件投影到 Web 管理端，投影事件也应有 id 和 ack，避免管理端重连后重复或遗漏。
- 群聊 actor 仍按 `SessionIdentity` 管历史；inbox 按可见 actor/surface 投影，不把私有 direct 事件推到 group surface。

## 技术方案

### 1. 存储与保留策略

在 `memory` 新增 `web_event_inbox.rs` 或在 `crates/hone-web-api` 先落一个窄存储模块。本地模式建议 SQLite，cloud 模式后续映射 PG：

```sql
CREATE TABLE web_event_inbox (
  event_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT NOT NULL DEFAULT '',
  surface TEXT NOT NULL,
  event_type TEXT NOT NULL,
  source TEXT NOT NULL,
  source_ref_json TEXT NOT NULL DEFAULT '{}',
  payload_json TEXT NOT NULL,
  delivery_state TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT,
  first_sent_at TEXT,
  acknowledged_at TEXT,
  ack_client_id TEXT
);

CREATE INDEX idx_web_event_actor_created
  ON web_event_inbox(actor_channel, actor_user_id, actor_scope, created_at);
```

保留策略：

- 默认保留 7 天或最近 500 条/actor，可配置。
- `run_status` 可短保留，例如 24 小时；`scheduled_message` / `push_message` 可保留更久。
- payload 必须是用户可见摘要，不保存完整 prompt、raw token、API key、cookie、未脱敏路径。

### 2. 写入路径

新增 `WebEventInboxService`：

```rust
pub struct WebEventInput {
    pub actor: ActorIdentity,
    pub surface: WebEventSurface,
    pub event_type: String,
    pub source: String,
    pub source_ref: serde_json::Value,
    pub payload: serde_json::Value,
}

pub trait WebEventInboxStore {
    fn append_event(&self, input: WebEventInput) -> HoneResult<WebEventItem>;
    fn list_after(&self, actor: &ActorIdentity, cursor: Option<&str>, limit: usize) -> HoneResult<Vec<WebEventItem>>;
    fn ack_events(&self, actor: &ActorIdentity, ids: &[String], client_id: &str) -> HoneResult<usize>;
}
```

改造顺序：

1. `WebBroadcastSink::send()`：先 append `push_message`，再 broadcast `{ event_id, text, source_ref }`。
2. `handle_scheduler_events()` 的 Web scheduled result：在写 session 后 append `scheduled_message`，再 broadcast。
3. `routes/imessage.rs` 中投影到 Web 的 iMessage 事件：对需要前端补拉的事件追加 inbox；纯瞬时 typing/progress 可以保持 volatile。
4. `logs.rs` 不进入此 inbox。日志流是运维日志，不是用户可见消息；它已有 log buffer 和 warning 语义。

### 3. API 与 SSE cursor

Public API：

- `GET /api/public/web-events?after=<event_id>&limit=100`
  - 从 cookie session 推导 actor。
  - 返回当前 actor 未过期事件。
- `POST /api/public/web-events/ack`
  - body: `{ ids: string[], client_id?: string }`
  - 只能 ack 当前 actor 的事件。

Admin API：

- `GET /api/web-events?channel=&user_id=&channel_scope=&after=&limit=`
- `POST /api/web-events/ack`

SSE 改造：

- `Event::id(event_id)` 设置标准 SSE id。
- 每个事件 payload 也带 `event_id`，兼容当前前端解析。
- 可读取 `Last-Event-ID` header 作为补拉 hint，但不要只依赖浏览器自动重放；前端仍应显式调用 list API。
- `events_lagged` 触发时，前端立即调用 `GET web-events?after=lastSeenEventId`。

### 4. 前端状态模型

新增 `packages/app/src/lib/web-events.ts` 与 shared helper：

- `loadWebEvents(after)`
- `ackWebEvents(ids)`
- `mergeEventById(existing, incoming)`
- `webEventToChatMessage(event)`

Public `/chat`：

1. 从 localStorage 读取 `hone_web_event_cursor:<user>`。
2. 调用补拉 API，按 `event_id` 去重追加。
3. 打开 `/api/public/events` SSE。
4. 收到 SSE 后 merge，更新 cursor，延迟批量 ack。
5. 如果 SSE error 或 lagged，显示补拉状态并重跑 step 2。

Admin `sessions.tsx`：

- 当前用户切换时使用 actor-scoped cursor，不复用不同 actor 的 cursor。
- 对当前打开的 direct session 自动 ack；未打开的 actor event 只增加未读计数。
- 保留 `refreshHistoryForKey()` 作为 session truth fallback，inbox 只负责实时/未读投影。

### 5. 与 session/history 的关系

为了避免两个真相源打架：

- `SessionStorage` 仍是聊天历史真相源。
- Cron scheduled Web result 如果已经写入 session，inbox payload 只做快速投影；刷新历史后可按 `source_ref.session_message_id` 或 content/time heuristic 去重。
- Event-engine Web push 如果未来也写入 session，应使用同一个 `event_id` / `source_ref`，让前端识别“这是同一条消息的 live projection”。
- 不要求老 session retroactively 生成 inbox event；只对新 Web-facing events 生效。

## 实施步骤

### Phase 1: Inbox store and read API

- 新增 `WebEventItem` 类型、SQLite store、append/list/ack 单元测试。
- 增加 admin/public list/ack API，但暂不接管所有写入。
- 对 payload 做大小限制和脱敏测试。

### Phase 2: Scheduled and push write-through

- 改造 `WebBroadcastSink::send()` 和 Web scheduler delivery：写 inbox 后再 broadcast。
- SSE event 设置 id，payload 带 `event_id`。
- 保持现有 `push_tx` 作为 low-latency path，不改变 event-engine 路由决策。

### Phase 3: Frontend cursor and dedupe

- Public `/chat` 接入补拉、cursor、ack、dedupe。
- Admin `sessions.tsx` 接入 actor cursor 和 lagged 补拉。
- 对 SSE reconnect、lagged、页面切后台恢复做前端测试。

### Phase 4: Observability and cleanup

- 增加 `/api/web-events/summary` 或在 dashboard 显示未 ack、expired、lagged recovery 数量。
- 加入定期 purge。
- 把 event id/source ref 接入 notifications/task-health/session deep link。

## 验证方式

- Rust 单元测试：
  - `append_event` 生成稳定 sortable id，并按 actor 隔离。
  - `list_after` 对空 cursor、指定 cursor、limit、expired event 返回正确。
  - `ack_events` 不能 ack 其它 actor 的 event。
  - payload 超限、包含不允许字段或无效 actor 时失败。
- Web API 测试：
  - public list/ack 只能读取当前 cookie actor。
  - admin list 可按 actor 查询。
  - SSE 输出包含 `id:` 和 payload `event_id`。
- Frontend tests：
  - `mergeEventById` 对重连重复事件不重复追加。
  - `events_lagged` 后触发补拉并保持消息顺序。
  - actor 切换不会复用错误 cursor。
- Regression：
  - 新增 CI-safe 脚本模拟 append 两条事件、断开 SSE、按 cursor 补拉、ack 后 summary 变更。
  - 不依赖真实短信、外部 IM、真实 Web Push 或模型。
- 手工验收：
  - Public `/chat` 打开后收到 event-engine Web push；刷新页面不重复。
  - 断网期间触发 Web scheduled task；恢复后补拉到同一条消息。
  - 管理端切换 actor 后只看到对应 actor 的未读 event。

## 风险与取舍

- **风险：inbox 与 session history 出现重复显示。**  
  取舍：第一版所有 inbox event 必须有 `event_id` 和 `source_ref`，前端按 id 去重；刷新历史后以 session 为准，inbox 只作为 live projection。

- **风险：新增存储增加清理和隐私面。**  
  取舍：payload 只保存短摘要，默认短保留，纳入未来 user data trust/export/delete；不保存 raw session token、完整 prompt 或 secret。

- **风险：ack 语义被误解为用户已读投资信息。**  
  取舍：ack 只表示浏览器客户端已接收或当前页面已展示，不表示用户投资判断已复盘。业务复盘仍由 evidence review、notification decision 或 company portrait 流程处理。

- **风险：过早把所有 SSE 都 durable 化。**  
  取舍：logs、typing、短进度 tick 可以保持 volatile；只把用户可见且需要补拉的 `scheduled_message` / `push_message` / recovery 类事件放入 inbox。

- **风险：cloud/local 双路径实现成本。**  
  取舍：本地先 SQLite，cloud 后续接 PG；接口从第一版就保持 store trait，避免把 SQLite 细节泄漏到 route。

## 与已有提案的差异

查重范围：

- `docs/proposal/` 下所有 `auto_p*.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 不重复 `auto_p1_public-pwa-notification-bridge.md`：该提案解决浏览器系统通知、service worker push subscription 和锁屏/后台触达；本提案解决前台 Web 应用内的 durable event inbox、SSE cursor、补拉、去重和 ack。
- 不重复 `auto_p1_end-user-notification-control.md`：该提案让用户配置通知偏好、quiet hours 和 digest；本提案不改变通知策略，只保证已生成的 Web-facing event 可可靠呈现。
- 不重复 `auto_p1_delivery_decision_loop.md`：该提案解释事件为什么 sent/queued/filtered；本提案处理 sent 到 Web browser surface 后的传输连续性和用户端未读状态。
- 不重复 `auto_p1_interrupted-run-recovery-inbox.md`：该提案处理 agent/session/cron run 中断后的恢复对象；本提案处理正常生成的浏览器事件在 SSE 断线、lagged 或重连时不丢失。
- 不重复 `auto_p1_context-return-links.md`：context link 解决点回正确对象；本提案可携带 `source_ref` 或 future context link，但核心是 event delivery cursor 和 durable inbox。

本轮选择该主题，是因为当前代码已经显示 Web/public/admin 实时消息依赖进程内 broadcast，且前端有重连但缺少 cursor/ack。它是主动投资助理可信度的窄而关键缺口，可以独立实施，也能支撑未来 PWA、桌面通知和通知决策产品面。
