# Bug: Web scheduler claims mobile notification delivery but only writes web session events

- 发现时间：2026-05-15 15:04 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：New
- 修复情况：未修复；当前证据显示任务执行与会话落库成功，但手机系统级通知未送达，且产品/assistant 表述会让用户误以为可依赖手机提醒。
- GitHub issue：无；当前不是 P1，未创建 issue。

## 证据来源

- `data/sessions.sqlite3`
  - `session_id=Actor_web__direct__web-user-ba50cb9401c0`
  - `2026-05-15T12:23:03+08:00` 起，用户追问 Web 定时任务如何在手机收到提醒；assistant 引导用户打开手机上的 Hone 网页/App、允许通知，并说明可用一次性任务测试手机提醒。
  - `2026-05-15T12:32:55+08:00` 用户要求 3 分钟后发测试通知；`2026-05-15T12:40:39+08:00` 用户反馈“没收到”。
  - `2026-05-15T12:45:33+08:00` 用户要求重新 1 分钟后发一条；`2026-05-15T12:48:44+08:00` 用户再次反馈“你还是没发”。
  - `2026-05-15T12:49:09+08:00` assistant 最终承认：任务创建并触发了，但没有变成手机系统通知；当前 web 通道不等于手机系统级 push。
- `cron_job_runs`
  - `run_id=21817`，`job_name=12:35 测试通知`，`executed_at=2026-05-15T12:35:11+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`，`detail_json.console_event_sent=false`，`delivery_channel=web`。
  - `run_id=21818`，`job_name=12:47 二次测试通知`，`executed_at=2026-05-15T12:47:13+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`，`detail_json.console_event_sent=false`，`delivery_channel=web`。
- 代码确认
  - `packages/app/src/context/sessions.tsx` 只监听 SSE `scheduled_message` / `push_message` 并 append 到当前会话。
  - `rg` 未发现 Web 端对 `Notification.requestPermission`、`PushManager` 或真正 Web Push 订阅的实现；现有 service worker 只用于 asset recovery。
  - `crates/hone-web-api/src/routes/events.rs` 的 Web scheduler 记录 `console_event_sent`，但 `web_scheduler_delivery_status(false)` 仍会把会话落库视为 `sent + delivered=1`。

## 端到端链路

Web 用户创建持仓新闻晚报 -> 用户询问如何在手机收到提醒 -> assistant 将 Web 会话通知解释为可通过手机网页/App 通知权限接收 -> 用户请求一次性测试通知 -> scheduler 到点触发并把 assistant final 写回 Web 会话 -> Web 投递层只尝试 SSE `scheduled_message`，且本轮 `console_event_sent=false` -> `cron_job_runs` 仍记为 `completed + sent + delivered=1` -> 用户手机系统通知中心没有收到提醒。

## 期望效果

- 如果 Web scheduler 只支持会话内消息，应明确告诉用户：当前不能保证手机系统通知，也不能把它当作可靠手机提醒。
- 如果产品要支持手机提醒，应有真正的 Web Push / App Push / 邮件 / 短信等可审计投递目标，并把送达结果与会话落库区分开。
- assistant 在创建或测试 Web 定时任务时，应基于可用 channel capability 给出准确承诺；无法系统级 push 时，不应引导用户反复排查手机通知权限。

## 当前实现效果

- 任务执行链路成功，测试通知正文也写入 Web 会话。
- 两条测试通知均 `console_event_sent=false`，说明实时控制台事件没有送到活跃 SSE 监听者。
- 后端仍把这类 Web scheduler 记录为 `sent + delivered=1`，这符合旧修复里的“会话落库即送达”定义，但不能代表手机系统通知已送达。
- assistant 前几轮把“当前 Hone 网页/会话通知通道”与手机系统级通知排查混在一起，直到用户两次反馈没收到后才明确承认当前没有真正打到手机通知中心。

## 用户影响

- 用户创建的 20:00 持仓新闻晚报可能只落在 Web 会话里，无法作为手机提醒使用。
- 台账显示 `delivered=1`，运维或后续 agent 容易误判为通知已送达，实际用户侧没有收到系统提醒。
- 用户被引导做手机权限排查和重复测试，增加信任损耗；这不是单次表达偏好问题，而是能力边界和送达语义不一致。

## 根因判断

- Web scheduler 的当前送达语义只覆盖“写入会话 / 尝试 SSE 事件”，没有覆盖手机系统级 push。
- 前端没有可见的 Web Push 订阅与浏览器通知授权链路；后端也没有独立的 push 订阅表或外部通知投递结果。
- assistant 缺少 channel capability 约束：当 `delivery_channel=web` 且没有 Web Push 能力时，仍按“手机通知权限”给出操作建议，导致用户预期与系统实际能力错位。

## 下一步建议

1. 产品层先明确 Web scheduler 的承诺：仅会话内消息，还是必须支持系统级 push。
2. 若保持仅会话内消息：在 cron job tool / system prompt / UI 文案里明确禁止承诺手机系统通知，并在回答中建议用户改用已支持的 Feishu / Telegram / Discord 等渠道。
3. 若要支持手机提醒：新增 Web Push/App Push capability、订阅状态检查与投递台账字段，区分 `session_persisted`、`sse_event_sent`、`system_push_sent`、`system_push_failed`。
4. 修复后补一条回归：当 `delivery_channel=web` 且无 push capability 时，assistant 创建测试通知的 final 必须说明“只能写入当前 Hone 会话，不保证手机系统通知”。
