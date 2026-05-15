# Bug: Web scheduler claims mobile notification delivery but only writes web session events

- 发现时间：2026-05-15 15:04 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：Fixed
- 修复情况：2026-05-16 00:06 已收口 Web 渠道能力边界：Web cron 对话提示明确只保证写入 Hone 会话 / 在线 SSE，不承诺手机系统通知；Web scheduler 台账 detail 显式记录 `system_push_supported=false` / `system_push_sent=false`。
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

## 修复进展（2026-05-16 00:06 CST）

- 已在 Web 渠道且允许 cron 的对话提示中注入 `【Web 定时任务送达边界】`：
  - 当前 Web 定时任务结果只保证写入当前 Hone 会话；
  - 网页在线且 SSE 连接存在时会实时追加到页面；
  - 当前没有 Web Push / 手机系统通知能力，不允许承诺会出现在手机通知中心，也不再引导用户排查手机通知权限。
- 已在 Web scheduler 执行台账 detail 中补充：
  - `system_push_supported=false`
  - `system_push_sent=false`
  - 继续保留 `console_event_sent`，用于区分页面实时 SSE 是否送达。
- 保留既有“会话落库即 Web delivered”的语义，不把离线页面视为 send_failed；但台账不再让后续排障误读为手机系统 push 已送达。
- 新增回归：
  - `resolve_prompt_input_warns_web_cron_cannot_send_mobile_system_push`
  - `web_scheduler_detail_distinguishes_session_delivery_from_system_push`
- 验证：
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/prompt.rs crates/hone-channels/src/turn_builder.rs crates/hone-channels/src/agent_session/tests.rs crates/hone-web-api/src/routes/events.rs`
  - `cargo test -p hone-channels resolve_prompt_input_warns_web_cron_cannot_send_mobile_system_push -- --nocapture`
  - `cargo test -p hone-web-api web_scheduler_ -- --nocapture`
  - `cargo check -p hone-web-api --tests`
- 修复提交：`fbba5342`
- 状态更新为 `Fixed`。若后续产品要求真正手机提醒，应另开功能/缺陷补 Web Push/App Push 订阅、授权状态检查和系统级投递台账。

## 下一步建议

1. 若保持仅会话内消息：后续 UI 可继续补显式说明，但当前 prompt 与台账已先阻断错误承诺。
2. 若要支持手机提醒：新增 Web Push/App Push capability、订阅状态检查与投递台账字段，区分 `session_persisted`、`sse_event_sent`、`system_push_sent`、`system_push_failed`。
