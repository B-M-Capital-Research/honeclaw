# Proposal: Desktop Native Alert Center for Local Attention and Recovery

status: proposed
priority: P2
created_at: 2026-05-31 08:07:03 +0800
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
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposal/auto_p1_end-user-notification-control.md`
- `docs/proposal/auto_p1_public-pwa-notification-bridge.md`
- `docs/proposal/auto_p1_temporal-operations-calendar.md`
- `docs/proposal/auto_p1_interrupted-run-recovery-inbox.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p2_desktop-quick-capture-inbox.md`
- `bins/hone-desktop/src/{main.rs,commands.rs,tray.rs}`
- `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs`
- `crates/hone-web-api/src/routes/{notifications,notification_prefs,logs,meta}.rs`
- `crates/hone-event-engine/src/sinks/`
- `crates/hone-event-engine/src/router/`
- `memory/src/cron_job/{history,mod,types}.rs`
- `packages/app/src/pages/{notifications,task-health,logs,settings,public-me}.tsx`

## 背景与现状

Hone 已经把桌面端从一个简单壳子推进到可管理本机 runtime 的 Tauri host：

- `bins/hone-desktop/src/commands.rs` 暴露了 backend 连接、bundled backend 启停、渠道设置、agent/model/FMP/Tavily 设置、CLI probe、channel cleanup 等命令。
- `bins/hone-desktop/src/sidecar/processes.rs` 负责 bundled runtime 锁预检、子进程启动、channel heartbeat 清理、sidecar 日志写入和重复进程清理。
- `docs/repo-map.md` 记录了桌面 bundled/remote 两种模式：bundled 由 Tauri 启动 `hone-console-page` 和启用的 channel sidecars；remote 直接连远端 HTTP backend。
- 管理端已有 `/notifications`、`/task-health`、`/logs`：分别查看 cron/event-engine 投递日志、任务运行健康和 runtime/channel 日志。
- `routes/notifications.rs` 已能把 cron 定时任务执行记录和 event-engine `delivery_log` 合并成统一通知审计视图。
- `NotificationPrefs`、`notification_prefs` API 和相关 skill 已提供 per-actor quiet hours、digest slot、severity、kind/source allow/block 等偏好。

但桌面端仍缺一个非常基础的产品层：**本机注意力面**。`bins/hone-desktop/src/tray.rs` 目前只是空的 extension point，桌面应用能启动和管理 backend，却不能像一个长期投资助理那样在本机系统层：

- 告诉用户“有一条重要提醒需要看”。
- 告诉用户“后台任务失败、渠道掉线、配置缺口导致提醒不会送达”。
- 让用户从菜单快速进入最近提醒、任务健康、日志或修复动作。
- 在不打开 Web 管理页的情况下临时暂停本机提醒、稍后提醒、只看 High 级别。

这造成一个产品断层：Hone 的通知和自动化链路越来越强，但桌面 app 对用户来说仍更像 runtime 管理器，而不是一个常驻、可信、可恢复的工作台。

## 问题或机会

### 问题

1. **桌面没有系统级提醒入口。**  
   event-engine 和 cron 可以把消息送到 Feishu、Telegram、Discord、iMessage，也可以在管理页里看到日志，但桌面 bundled 用户本机没有 native notification 或 tray badge。用户必须主动打开 Web UI 才知道有没有重要提醒或失败。

2. **桌面状态只在页面内可见，不在 OS attention 面可见。**  
   `/api/channels`、`/api/logs`、`/api/admin/task-runs` 和 `/api/admin/notifications` 都是 Web/API 视角。对桌面用户来说，channel 掉线、backend 恢复中、旧进程冲突、API key 缺失等状态应该先在 tray/status menu 暴露，而不是等用户排障时才翻页面。

3. **通知偏好和本机勿扰没有分层。**  
   `NotificationPrefs` 解决 actor 收什么、什么时候收、是否 quiet hold；但桌面 native notification 还需要一层 device-local preference，例如本机弹窗是否启用、只弹 High、工作时间内静音、今天暂停、是否在锁屏显示正文。这个偏好不应污染 event-engine 的跨渠道业务规则。

4. **PWA/Web Push 与 Desktop Native 是不同问题。**  
   `auto_p1_public-pwa-notification-bridge.md` 解决 public Web/mobile 用户没有 IM 绑定时的浏览器推送；桌面 bundled/local 用户需要的是 Tauri native notification、tray menu、local runtime recovery 和本机-only state。把两者混在一起会让 service worker、VAPID、loopback backend 和 macOS 权限边界变复杂。

5. **桌面恢复动作缺少轻量入口。**  
   已有 desktop startup UX 提案关注启动接管；当前代码也已有 `cleanup_channel_processes` 命令。但用户日常遇到“Telegram listener 掉了”“Feishu 没推”“backend 正在重启”时，理想入口应该是 tray 菜单里的 `Retry channel`、`Open logs`、`Open notifications`、`Pause alerts`，而不是先进入设置页。

### 机会

AI agent 产品的桌面形态正在从“打开一个聊天窗口”转向“常驻工作台”：低打扰地监控任务、在重要时刻唤醒用户、失败时给出可执行恢复动作、把长任务结果放进 inbox。Hone 的投资助理定位天然适合这个方向，因为它的价值不只是回答一次问题，而是持续守护纪律和提醒关键变化。

本提案定为 P2：它不会替代当前 P0/P1 的数据安全、投递可靠性、权限、云化和核心通知控制；但它可以显著提升桌面留存、任务可信度和本地体验，并且能以小步方式接入现有通知/任务健康/日志 API，不需要重构 event-engine。

## 方案概述

新增 **Desktop Native Alert Center**：在 Tauri 桌面端增加本机通知、tray 状态菜单和本机提醒 inbox 投影。它不是新的业务通知系统，而是把现有 cron/event-engine/task-health/channel-status 的重要状态，以桌面本机方式呈现和恢复。

核心原则：

- 不改变 `NotificationPrefs` 的业务语义；新增的是 device-local desktop alert preference。
- 不把所有 IM 推送复制成桌面弹窗；只对用户明确启用的 actor/事件级别弹出。
- 不绕过现有 delivery log；native alert 的展示、点击、忽略、失败也要有最小审计。
- 不把 desktop native notification 和 Web Push/PWA 混为同一 sink。Web Push 面向 browser subscription，Desktop Native 面向 Tauri app + local OS permission。
- bundled 模式优先，remote 模式只在后端明确支持并且用户登录/授权后展示。

第一版建议只做三个能力：

1. **Tray status menu**
   - backend 状态、channel live count、最近失败任务、最近未读重要通知。
   - 快捷入口：Open Dashboard、Open Notifications、Open Task Health、Open Logs、Pause Desktop Alerts、Cleanup duplicate channel processes。

2. **Desktop alert projection**
   - 后台轮询或订阅 backend 的通知摘要与任务健康摘要。
   - 仅对 High/failed/recovery-needed 等有限类别触发 OS notification。
   - 点击通知打开对应 Web route，而不是在通知里展示长内容。

3. **Local alert inbox**
   - 在桌面 app 内记录最近本机 alert：source、actor、title、summary、deep link、shown_at、clicked_at、dismissed_at。
   - 默认短 retention，例如 7 天或 200 条。
   - 这不是业务真相源，只是本机 attention history。

## 用户体验变化

### 用户端 / 桌面端

- macOS 菜单栏出现 Hone tray icon，常驻显示 backend/channel 简要状态。
- 有重要事件或失败时，桌面弹出 native notification：
  - `NVDA earnings update moved into digest`
  - `Feishu channel disconnected`
  - `Daily brief failed: missing FMP key`
  - `Scheduled report completed`
- 点击通知进入相关页面：
  - 投资事件 -> `/notifications` 对应记录或 `/portfolio`。
  - 任务失败 -> `/task-health`。
  - channel/配置问题 -> `/settings` 或 `/logs`。
- Tray 菜单提供轻量操作：
  - `Pause alerts for 1h`
  - `Quiet until tomorrow`
  - `Only High alerts`
  - `Open latest notification`
  - `Clean duplicate channel processes`
- 用户可以在 Desktop Settings 里看到本机提醒权限和偏好，不需要理解 event-engine 全量配置。

### 管理端

- `/notifications` 仍是业务投递审计，不被桌面本机 alert 取代。
- 可在通知详情中看到“desktop alert projected/shown/clicked”这种本机投影状态，但第一版可以只在本地 desktop inbox 保留，不强行写入服务端。
- `/task-health` 的失败摘要可以被 Desktop Alert Center 消费，帮助 operator 第一时间看到失败。

### 多渠道

- Feishu/Telegram/Discord/iMessage 继续作为独立 channel sink；Desktop Native 不自动复制所有外部 channel 消息。
- 如果同一 actor 已通过 IM 收到 High 事件，本机是否再弹由 desktop local preference 决定；第一版建议默认只弹“运行失败/恢复动作”和用户显式开启的投资 High。
- 群聊事件默认不弹出个人桌面通知，除非后续 workspace/room 权限明确。

### Public Web / PWA

- Public PWA 通知桥继续独立处理 browser push subscription。
- Desktop remote 访问 public service 时，可以显示“Web Push 可用”和“Desktop Native 仅在桌面 host 可用”的清晰区别。

## 技术方案

### 1. Tauri tray 与 notification 权限

将 `bins/hone-desktop/src/tray.rs` 从空 extension point 扩展为桌面本机注意力入口：

- 初始化 tray icon 和 menu。
- 注册菜单事件：
  - open admin/public route
  - pause/resume desktop alerts
  - cleanup channel processes
  - reconnect/start bundled backend
  - open logs directory or logs page
- 集成 Tauri notification plugin 或原生通知能力，启动时检查 permission 状态。

建议新增 command：

- `get_desktop_alert_settings()`
- `set_desktop_alert_settings(settings)`
- `list_desktop_alerts(limit)`
- `mark_desktop_alert_clicked(alert_id)`
- `dismiss_desktop_alert(alert_id)`
- `test_desktop_notification()`

这些 command 的状态存放在 desktop app data dir，不写入 `config.yaml`，因为它们是 device-local preference，不是业务 runtime config。

### 2. Desktop alert settings

建议模型：

```rust
pub struct DesktopAlertSettings {
    pub enabled: bool,
    pub min_severity: String,
    pub include_runtime_failures: bool,
    pub include_channel_status: bool,
    pub include_investment_events: bool,
    pub quiet_until: Option<String>,
    pub show_sensitive_body: bool,
    pub max_alerts_per_hour: u32,
}
```

默认值应保守：

- native alert 默认不开启或首次启动只提示用户选择。
- runtime failures 可以在 app 内 badge 显示，但 OS notification 需要用户同意。
- `show_sensitive_body=false`，锁屏通知只显示短标题，例如 `Hone has an investment alert`。

### 3. 后端投影 API

第一版不需要新增复杂服务端 store，可复用现有 API：

- `/api/meta`：backend readiness/capabilities。
- `/api/channels`：channel live registrations + OS process scan。
- `/api/admin/notifications`：最近投递记录与 summary。
- `/api/admin/task-runs`：任务运行健康。
- `/api/logs`：打开日志排障入口。

为了减少前端轮询拼接，建议新增一个轻量聚合 API：

- `GET /api/desktop/alert-summary`

返回：

```json
{
  "backend": { "status": "ok" },
  "channels": [
    { "channel": "feishu", "status": "running", "duplicate_count": 0 }
  ],
  "recent_failures": [
    { "kind": "task_run", "id": "...", "title": "...", "route": "/task-health" }
  ],
  "recent_high_notifications": [
    { "record_source": "event_engine", "id": "...", "title": "...", "route": "/notifications" }
  ]
}
```

该 API 只做只读聚合，不改变 cron/event-engine/channel 状态。

### 4. 本机 alert inbox

在 desktop data dir 下维护本机 SQLite 或 JSONL：

```text
<desktop_data_dir>/alerts/desktop_alerts.sqlite3
```

字段：

```text
desktop_alerts (
  alert_id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  source_id TEXT,
  severity TEXT NOT NULL,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  route TEXT NOT NULL,
  actor_key TEXT,
  shown_at TEXT NOT NULL,
  clicked_at TEXT,
  dismissed_at TEXT,
  metadata_json TEXT NOT NULL
)
```

Inbox 用途：

- 去重：同一 `source + source_id` 不重复弹。
- 点击恢复：用户错过 native notification 后仍可在 tray 看最近 alerts。
- 排障：本机 alert 为什么弹/没弹可查。

它不是跨设备业务记录；远端同步属于后续协作/用户数据层，不在第一版。

### 5. Deep link 与打开页面

Desktop host 已经知道 backend base URL。点击 alert 时：

- bundled mode：打开内嵌 WebView 对应 route，或聚焦当前窗口后导航。
- remote mode：打开 remote base URL 对应 route。
- backend 不可用：打开本机 status/recovery screen，并保留目标 route，恢复后跳转。

Route 要稳定：

- `/notifications?record=<id>`
- `/task-health?task=<name>&run=<id>`
- `/logs?source=<channel>`
- `/settings?section=channels`

如果现有页面暂不支持 query 参数，第一版可以先打开对应页面，再在后续 UI polish 中支持定位具体记录。

### 6. 敏感内容边界

- Native notification 默认不展示持仓数量、成本价、完整公司画像、用户消息原文、文件路径或 API key 错误细节。
- 锁屏/通知中心正文只放 summary 级别信息。
- 详细内容必须在已登录/已打开的 Web UI 中查看。
- 本机 alert inbox 不存完整投资报告或 prompt，只存标题、短摘要和 route。

## 实施步骤

### Phase 1: Tray status foundation

1. 在 `tray.rs` 初始化 tray icon/menu。
2. 增加菜单动作：Open Dashboard、Open Notifications、Open Task Health、Open Logs、Pause/Resume Alerts。
3. 复用 `backend_status`、`cleanup_channel_processes` 等已有 command。
4. 保存 device-local alert settings。

### Phase 2: Alert summary projection

1. 新增 `/api/desktop/alert-summary` 或前端聚合现有 `/api/meta`、`/api/channels`、`/api/admin/notifications`、`/api/admin/task-runs`。
2. Desktop shell 每 30-60 秒刷新 summary。
3. 实现本机 alert 去重与 inbox。
4. 不触发 OS notification，只在 tray badge/menu 中显示。

### Phase 3: Native notification opt-in

1. 增加 notification permission 检查和测试通知。
2. 用户 opt-in 后，对 runtime failure/channel disconnected/high investment events 触发 OS notification。
3. 点击通知聚焦窗口并打开 deep link。
4. 增加 quiet_until、max_alerts_per_hour、show_sensitive_body 设置。

### Phase 4: Recovery actions

1. 针对 channel duplicated/disconnected 提供 `Clean duplicates`、`Restart bundled channel`。
2. 针对 missing key/configured false 提供设置页 deep link。
3. 针对 backend unavailable 显示 local recovery panel，而不是静默失败。

## 验证方式

- 单元测试：
  - desktop alert settings 默认值、quiet_until、生效优先级。
  - alert dedupe：同一 source/source_id 不重复弹。
  - sensitive body redaction：`show_sensitive_body=false` 时不包含 actor user id、成本价、路径、完整错误。
  - alert summary model：failed task/channel down/high notification 正确转成 route。

- 前端/桌面 smoke：
  - `HONE_DESKTOP_SMOKE_SERVER=1` 下仍能启动 backend smoke server。
  - tray 初始化失败不能阻断 core desktop startup。
  - notification permission denied 时 UI 显示清楚状态，不反复请求权限。

- 回归脚本：
  - 新增 CI-safe model test 或 desktop smoke test，验证 alert summary API 在空 store、无 events.sqlite3、无 cron history 时返回空摘要而非 500。
  - 不依赖真实 macOS notification 权限、不依赖 Feishu/Telegram/Discord 账号。

- 手工验收：
  - bundled 模式启动后 tray 能打开 dashboard/notifications/logs。
  - 模拟 channel sidecar 退出，tray 显示 channel down，并能进入 logs。
  - 模拟一条失败 cron run，Desktop Alert Center 生成本机 inbox item。
  - 用户开启 native notification 后，测试通知可以弹出并点击回到 app。

## 风险与取舍

- **风险：重复打扰。**  
  取舍：默认只做 tray badge/menu；OS notification 需要 opt-in，且默认只弹 runtime failure 和 High 级别摘要。

- **风险：与 `NotificationPrefs` 产生两套规则。**  
  取舍：`NotificationPrefs` 仍决定业务事件是否应该送达；DesktopAlertSettings 只决定本机是否额外弹窗/展示 badge。

- **风险：泄露敏感投资信息到锁屏。**  
  取舍：默认隐藏敏感正文，只显示短标题；详细内容必须打开 app 后查看。

- **风险：Tauri tray/notification 跨平台差异。**  
  取舍：第一版优先 macOS，Windows/Linux 显示为 capability degraded；核心 backend 不依赖 native notification。

- **风险：远端 backend 和本机 desktop 的身份不一致。**  
  取舍：remote mode 第一版只显示 backend status 和打开 route，不默认拉取任意 actor 的投资通知，除非已有登录/权限状态可证明当前用户。

- **不做：**  
  不新增 Web Push，不替代 Feishu/Telegram/Discord/iMessage sink，不做移动 push，不做全量跨设备 notification sync，不把桌面 alert inbox 当作业务投递真相源。

## 与已有提案的差异

本轮查重覆盖 `docs/proposal/` 和历史 `docs/proposals/`，重点对比了通知、桌面、投递、恢复和 PWA 相关主题：

- 不重复 `auto_p1_end-user-notification-control.md`：该提案解决终端用户如何查看和修改 `NotificationPrefs`；本提案解决桌面设备本机如何显示 tray/native alert 和恢复入口。前者是业务偏好，后者是 device-local attention layer。
- 不重复 `auto_p1_public-pwa-notification-bridge.md`：该提案是 browser Web Push/PWA subscription；本提案是 Tauri desktop native notification/tray，不涉及 service worker、VAPID 或 browser push。
- 不重复 `docs/proposals/desktop-bundled-runtime-startup-ux.md`：该历史提案关注启动时锁冲突和 bundled runtime 接管；本提案关注应用启动后的日常状态、提醒、tray 菜单和恢复动作。
- 不重复 `auto_p2_desktop-quick-capture-inbox.md`：quick capture 解决从桌面采集证据进入 Hone；本提案解决 Hone 运行结果和异常如何回到桌面用户注意力面。
- 不重复 `auto_p1_temporal-operations-calendar.md`：calendar 预测未来任务/推送节奏；本提案只消费已发生或近期需要用户关注的摘要并映射为本机提醒。
- 不重复 `auto_p1_interrupted-run-recovery-inbox.md`：recovery inbox 关注 agent run 中断后的业务恢复；本提案覆盖桌面本机 alert inbox，其中包括但不限于 run failure，也包含 channel/runtime/task health。
- 不重复 `auto_p1_delivery_decision_loop.md`：delivery decision 解释某条事件为什么发送/过滤/digest；本提案不改变 router 决策，只决定已产生的重要状态是否在桌面本机呈现。

## 文档同步说明

本轮只新增 proposal，不开始实现，不改变模块边界、入口、长期约束、测试规则或运行配置，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。若后续进入实现，应新增或复用 `docs/current-plans/desktop-native-alert-center.md`，并在引入 Tauri tray/native notification、desktop alert settings store、`/api/desktop/alert-summary` 或 notification click deep link 时同步更新 `docs/repo-map.md`；若确定 device-local alert preference 与业务 `NotificationPrefs` 的长期边界，应补充 `docs/decisions.md` 或 ADR。
