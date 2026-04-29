# Bug: Web 定时任务仅在活跃 SSE 控制台存在时才会被记为已送达

- **发现时间**: 2026-04-27 10:18 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: Fixed

## 证据来源

- `2026-04-29 20:01` 最近一小时真实窗口显示该缺陷仍在最新生产窗口活跃：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=10323`
    - `job_id=j_f42bfebd`
    - `job_name=英伟达每日消息`
    - `actor_channel=web`
    - `executed_at=2026-04-29T20:01:14.101352+08:00`
    - `execution_status=completed`
    - `message_send_status=send_failed`
    - `delivered=0`
    - `should_deliver=1`
    - `detail_json={"console_event_sent":false,"scheduler":null}`
    - `response_preview` 已包含完整 NVDA 摘要开头与结构化段落，说明正文已生成完成，但调度台账再次把离线 Web 任务记成 `send_failed`
  - 同表历史对照：
    - `run_id=9099`（`2026-04-28 20:01:19+08:00`）是同一 `job_id=j_f42bfebd` 的前一日复现，最新 `10323` 说明这个晚间 job 也已经连续两天落成相同坏态
  - 结论：
    - 到 `2026-04-29 20:01` 为止，这条缺陷不仅限于晨报 job，连 `20:00` 的 `英伟达每日消息` 也继续稳定落成 `completed + send_failed + console_event_sent=false`

- `2026-04-29 09:02` 最近一小时真实窗口显示该缺陷仍在最新生产窗口活跃：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=9796`
    - `job_id=j_183bee8d`
    - `job_name=09:00 美股AI与航空科技晨报`
    - `actor_channel=web`
    - `executed_at=2026-04-29T09:02:33.617869+08:00`
    - `execution_status=completed`
    - `message_send_status=send_failed`
    - `delivered=0`
    - `should_deliver=1`
    - `detail_json={"console_event_sent":false,"scheduler":null}`
    - `response_preview` 已包含完整晨报开头与 `**最重要的 5 条**` 结构，说明正文已生成完成，但调度台账仍把离线 Web 任务记成 `send_failed`
  - 同表历史对照：
    - `run_id=8573`（`2026-04-28 09:01:31+08:00`）与 `run_id=7445`（`2026-04-27 09:02:12+08:00`）是同一 job 的前两次复现，最新 `9796` 说明该回归不是单日偶发
  - `data/runtime/logs/acp-events.log`
    - `2026-04-29 09:02:13-09:02:15` 同一 Web 会话 `Actor_web__direct__web-user-ba50cb9401c0` 连续流出晨报正文 chunk，内容覆盖 AI 基础设施主线、`CDNS` 财报与 Big Tech 财报窗口，说明 agent 生成阶段没有中断
  - 结论：
    - 到 `2026-04-29 09:02` 为止，这条缺陷不仅没有停留在 `2026-04-28 20:01` 的回归点，而且同一 `09:00 美股AI与航空科技晨报` 已连续第三天落成 `completed + send_failed + console_event_sent=false`

- `2026-04-28 20:01` 最近一小时真实窗口显示该缺陷已回归：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=9099`
    - `job_id=j_f42bfebd`
    - `job_name=英伟达每日消息`
    - `actor_channel=web`
    - `executed_at=2026-04-28T20:01:19.068865+08:00`
    - `execution_status=completed`
    - `message_send_status=send_failed`
    - `delivered=0`
    - `should_deliver=1`
    - `detail_json={"console_event_sent":false,"scheduler":null}`
    - `response_preview` 已包含完整 NVDA 摘要开头，说明正文已生成完成，但调度台账再次把离线 Web 任务记成 `send_failed`
  - 最近一小时运行日志：`data/runtime/logs/web.log.2026-04-28`
    - `20:00:01.048-20:01:19.067` 同一会话 `Actor_web__direct__web-user-e05f5e5f74a3` 依次记录 `session.persist_user -> agent.run -> session.persist_assistant detail=done`
    - `20:01:19.067` 同轮明确落成 `done ... success=true elapsed_ms=77984 ... reply.chars=1689`
    - `20:01:19.068` 随后记录 `⏰ [web-user-e05f5e5f74a3] 定时任务完成`
    - 但 `cron_job_runs` 最终仍是 `completed + send_failed + console_event_sent=false`，与本单历史坏态一致

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

- `2026-04-29 20:01` 的 `英伟达每日消息` 说明，这条缺陷在最新一小时窗口仍未退出活跃态：正文已完整生成，但 `cron_job_runs` 依旧再次记成 `completed + send_failed + console_event_sent=false`。
- 同一 `job_id=j_f42bfebd` 已连续两天（`2026-04-28`、`2026-04-29`）在 `20:00` 窗口复现，说明这不是单个晨报 job 的特例，而是 Web scheduler 的通用离线送达判定仍未收口。
- `2026-04-29 09:02` 的 `09:00 美股AI与航空科技晨报` 说明，这条缺陷在当前最新窗口仍未退出活跃态：正文已完整生成并落成带结构化小标题的晨报，但 `cron_job_runs` 依旧再次记成 `completed + send_failed + console_event_sent=false`。
- 同一 `job_id=j_183bee8d` 已连续三天（`2026-04-27`、`2026-04-28`、`2026-04-29`）都落成相同坏态，说明 `2026-04-28` 写入的修复结论并未在线上稳定生效。
- `2026-04-28 20:01` 的 `英伟达每日消息` 说明，这条缺陷已经从 `Fixed` 回退为在线复现：正文已成功生成并记录 `session.persist_assistant detail=done`，但 `cron_job_runs` 仍再次落成 `completed + send_failed + console_event_sent=false`。
- 这说明当前线上行为仍会把“没有活跃 SSE 控制台监听者”或等价离线状态直接当作发送失败，而不是稳定记成“正文已落库、实时事件未送达”。
- `2026-04-28` 先前的修复结论至少没有在当前生产窗口稳定生效；因此本单状态改回 `New`，需要重新进入活跃缺陷队列。

## 用户影响

- 这是功能性 bug，不是纯体验偏好：用户要求的是 `09:00` 提醒/晨报推送，但在离线或控制台未保持订阅时，这类 Web 定时任务不会被系统视为真正送达。
- 影响面当前看限于 Web scheduler 渠道；正文没有丢失，但主动提醒语义失效，且后台会持续把任务标成失败，干扰后续巡检和用户信任。

## 根因判断

- `2026-04-29 09:02` 的第三次连续晨报复现说明，当前问题不是某个单次会话写坏或单日 Web runtime 波动，而是同一 Web scheduler job 在“用户离线 / 无活跃 SSE 订阅者”这一条件下持续沿用旧的送达判定。
- `2026-04-28 20:01` 的回归样本说明，当前生产链路仍把 `console_event_sent=false` 与 `message_send_status=send_failed` 绑定在一起，至少对 `run_id=9099` 没有兑现“正文落库即视为送达成功”的修复语义。
- 根因不是 agent 生成失败，也不是会话持久化失败；同一 run 已经写入完整 assistant final。
- 当前 Web scheduler 的送达判定把单次 SSE 推送结果直接等同于“消息已送达”，缺少离线用户的补投/通知机制，也缺少“已落库但未实时推送”的中间状态。

## 下一步建议

- 先核对 `run_id=9099` 真实写入的 `cron_job_runs` 分支是否仍走旧的 `console_event_sent=false => send_failed` 路径，确认修复是否只覆盖了部分 Web scheduler 调用面。
- 若未来产品定义要求 Web 端必须主动弹出离线通知，可另补待领取事件队列；但在此之前至少要恢复“正文落库不等于 send_failed”的台账语义。

## 修复与验证

- 2026-04-28: `crates/hone-web-api/src/routes/events.rs` 将 Web scheduler 的 SSE 结果降为观测字段，不再决定 `message_send_status`。
- 2026-04-28: `cargo check -p hone-memory -p hone-scheduler -p hone-tools -p hone-web-api -p hone-event-engine -p hone-channels --tests`
- 2026-04-30: 本轮复核当前 `crates/hone-web-api/src/routes/events.rs` 代码，Web 渠道在 agent 成功生成并写入会话后默认 `message_send_status=sent`、`delivered=true`；`push_tx.send(...)` 的结果只写入 `detail.console_event_sent`。这与期望语义一致，活跃队列中的 `New` 状态属于文档滞后。
