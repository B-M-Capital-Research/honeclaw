# Bug: Web 定时任务仅在活跃 SSE 控制台存在时才会被记为已送达

- **发现时间**: 2026-04-27 10:18 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=8573`
  - `job_id=j_183bee8d`
  - `job_name=09:00 美股AI与航空科技晨报`
  - `actor_channel=web`
  - `executed_at=2026-04-28T09:01:31.177909+08:00`
  - `execution_status=completed`
  - `message_send_status=send_failed`
  - `delivered=0`
  - `detail_json={"console_event_sent":false,"scheduler":null}`
  - `response_preview` 已包含完整晨报开头：`北京时间 2026 年 4 月 28 日 09:00，今天的主线很集中：AI 基础设施正在进入财报验证窗口`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=7445`
  - `job_id=j_183bee8d`
  - `job_name=09:00 美股AI与航空科技晨报`
  - `actor_channel=web`
  - `executed_at=2026-04-27T09:02:12.160196+08:00`
  - `execution_status=completed`
  - `message_send_status=send_failed`
  - `delivered=0`
  - `detail_json={"console_event_sent":false,"scheduler":null}`
- `data/sessions.sqlite3` -> `session_messages`
  - 同一 Web 会话 `session_id=Actor_web__direct__web-user-ba50cb9401c0`
  - `ordinal=19` 为 `2026-04-27T09:00:00.339864+08:00` 的 scheduler user turn：`[定时任务触发] 任务名称：09:00 美股AI与航空科技晨报`
  - `ordinal=20` 为 `2026-04-27T09:02:12.142577+08:00` 的 assistant final，正文已经成功生成并写入会话
- 代码证据
  - [`crates/hone-web-api/src/routes/events.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-web-api/src/routes/events.rs:129) 显示 Web scheduler 当前先把结果投到 `push_tx.send(PushEvent { event: "scheduled_message", ... })`
  - [`crates/hone-web-api/src/routes/events.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-web-api/src/routes/events.rs:142) 显示只要 `push_tx.send(...)` 失败，就直接把 `message_send_status` 记成 `send_failed`
  - 同一分支没有 Web 离线补投、通知队列或其它独立送达通道；[`crates/hone-web-api/src/lib.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-web-api/src/lib.rs:500) 也确认 scheduler 默认覆盖 `web` 渠道

## 端到端链路

Web 用户创建 `09:00 美股AI与航空科技晨报` -> scheduler 到点触发 -> agent 成功生成晨报正文并写入同一 Web 会话 -> Web 投递层尝试通过 SSE `scheduled_message` 实时推送 -> 当时没有活跃控制台订阅时 `push_tx.send(...)` 失败 -> `cron_job_runs` 记为 `completed + send_failed + delivered=0`

## 期望效果

- Web 定时任务即使用户当时不在线，也应具备可被视为“已送达”的稳定投递语义，至少不能把“正文已写入会话但没有实时 SSE 监听者”误记为发送失败。
- 调度台账里的 `message_send_status` 应能准确区分“实时推送失败”和“用户实际无法看到结果”。

## 当前实现效果

- Web scheduler 把“是否存在活跃 SSE 控制台监听者”当成唯一送达判断。
- 同一条晨报正文已经写进会话，后续打开会话仍可读到，但台账仍记为 `send_failed`，说明系统把“离线可追溯”和“实时推送”混成了一个成功条件。
- 该问题在 `2026-04-28 09:01` 对同一 job 再次原样复现：最新 `run_id=8573` 与前一天 `run_id=7445` 都是 `completed + send_failed + detail_json.console_event_sent=false`，说明它不是单次偶发，而是稳定的离线送达判定缺口。

## 用户影响

- 这是功能性 bug，不是纯体验偏好：用户要求的是 `09:00` 提醒/晨报推送，但在离线或控制台未保持订阅时，这类 Web 定时任务不会被系统视为真正送达。
- 影响面当前看限于 Web scheduler 渠道；正文没有丢失，但主动提醒语义失效，且后台会持续把任务标成失败，干扰后续巡检和用户信任。

## 根因判断

- 根因不是 agent 生成失败，也不是会话持久化失败；同一 run 已经写入完整 assistant final。
- 当前 Web scheduler 的送达判定把单次 SSE 推送结果直接等同于“消息已送达”，缺少离线用户的补投/通知机制，也缺少“已落库但未实时推送”的中间状态。

## 下一步建议

- 先把 Web scheduler 的送达语义拆开：
  - 会话已成功持久化
  - 实时 SSE 是否送达
- 若产品定义允许“打开会话即可看到定时任务结果”视为成功，则 `message_send_status` 不应再因为 `console_event_sent=false` 直接记为 `send_failed`。
- 若产品定义要求 Web 端必须主动提醒，则需要补离线通知/待领取事件队列，再把这类 run 记为真正已送达。
