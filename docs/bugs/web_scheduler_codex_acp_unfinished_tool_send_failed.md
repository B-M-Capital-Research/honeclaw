# Bug: Web 定时任务在 Codex ACP 提前 `end_turn` 且仍有未完成搜索工具时整轮失败，失败提示既未落库也未送达

- **发现时间**: 2026-04-27 20:06 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed

## 修复进展（2026-04-28）

- 已在 `crates/hone-web-api/src/routes/events.rs` 为 Web scheduler 失败链路补会话级失败提示落库：
  - 非 heartbeat 调度任务失败时，若 `scheduler::ScheduledTaskExecution` 带用户可见错误，会把 `定时任务「...」执行出错，请稍后重试。` 写入对应 Web 会话。
  - 即使底层错误被共享调度层标记为 `failure_kind=internal_error_suppressed`、不应向用户暴露内部细节，Web transcript 仍会留下产品化失败消息，避免“cron 台账有 run，但用户会话无痕迹”。
  - 写入前会检查最后一条 assistant 是否已是同一失败提示，避免重复落库。
- 已补 `scheduler_failure_trace_required_*` 单元测试，锁住内部错误抑制时仍需要用户可追溯失败痕迹、正常 noop 不应写失败提示。

- **证据来源**:
  - 最近一小时真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=7936`
    - `job_id=j_f42bfebd`
    - `job_name=英伟达每日消息`
    - `actor_channel=web`
    - `executed_at=2026-04-27T20:01:13.883268+08:00`
    - `execution_status=execution_failed`
    - `message_send_status=send_failed`
    - `should_deliver=1`
    - `delivered=0`
    - `response_preview=定时任务「英伟达每日消息」执行出错，请稍后重试。`
    - `error_message=抱歉，这次处理失败了。请稍后再试。`
    - `detail_json={"console_event_sent":false,"scheduler":null}`
  - 最近一小时真实会话落库状态：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - 对应 Web 会话 `session_id=Actor_web__direct__web-user-e05f5e5f74a3`
    - 到本轮巡检时，`sessions.updated_at`、`last_message_at`、`imported_at` 仍停在 `2026-04-26T23:01:55+08:00`
    - `session_messages` 最新 `ordinal=21` 仍是 `2026-04-26T23:01:55+08:00` 的普通直聊答复，没有新增 `2026-04-27 20:00` 的 scheduler user turn，也没有新增 assistant 失败提示
    - 这说明 `run_id=7936` 既没有把本轮 scheduler 注入写进会话，也没有把失败提示落库到用户可见 transcript
  - 最近一小时运行日志：`data/runtime/logs/acp-events.log`
    - `2026-04-27T12:00:01.161518Z` 记录向 `Actor_web__direct__web-user-e05f5e5f74a3` 发送 `session/prompt`，正文明确是 `[定时任务触发] 任务名称：英伟达每日消息`
    - `2026-04-27T12:01:10-12:01:12Z` 连续记录大量 `session/update agent_message_chunk`，说明 Codex ACP 已经产出大段中间正文流
    - `2026-04-27T12:01:13.861840Z` 最后一个可见更新仍是 `tool_call_update status=completed`
    - `2026-04-27T12:01:13.862519Z` 紧接着收到 `result.stopReason=end_turn`
    - 但同一时段没有对应的 `session.persist_assistant`、`done user=... success=true`、`failed user=...` 或 `reply.send`
  - 历史同会话对照：
    - `2026-04-26T22:06:58+08:00` 该用户曾成功创建同一任务 `英伟达每日消息`
    - `2026-04-26T22:07:34+08:00` 同一会话还能正常列出该任务，说明配置创建与普通 Web 直聊本身不是当前故障点

## 端到端链路

1. Web 用户创建 `英伟达每日消息`，期望每天北京时间 `20:00` 收到 NVDA 最新公司消息摘要。
2. `2026-04-27 20:00` scheduler 到点触发，向 `Actor_web__direct__web-user-e05f5e5f74a3` 注入 scheduler prompt。
3. Codex ACP 开始流式生成正文并执行搜索工具，`acp-events.log` 中可见持续的 `agent_message_chunk` 与工具完成事件。
4. 在没有会话落库成功、也没有形成最终用户态消息的情况下，runner 提前以 `stopReason=end_turn` 收口。
5. Web scheduler 最终把本轮记为 `execution_failed + send_failed + delivered=0`，但真实会话没有新增任何 scheduler 消息或失败提示。

## 期望效果

- Web 定时任务即使执行失败，也应至少保证两件事之一成立：
  - 把失败提示稳定写入会话并让用户后续可追溯
  - 或通过实时通道把失败提示稳定送达用户
- 当 Codex ACP 在仍有未完成搜索工具时提前 `end_turn`，链路应显式记录为“unfinished tools”类失败，并阻止把中间流式输出当成成功收口。
- Web scheduler 不应出现“台账显示执行过一次失败任务，但会话 transcript 完全没有本轮痕迹”的状态失真。

## 当前实现效果

- 本轮 `run_id=7936` 已经把失败写进 scheduler 台账，但 Web 会话仍停留在 `2026-04-26 23:01` 的旧直聊内容。
- `acp-events.log` 证明本轮并非完全没有生成内容，而是已经输出了大段中间 chunk 后直接 `end_turn`；但这些 chunk 没有沉淀成最终 assistant 消息。
- `detail_json.console_event_sent=false` 说明本轮也没有通过 Web 的实时控制台事件被视为已送达。
- 之所以定级为 `P2`，是因为这会直接导致用户订阅的日报/提醒在本轮缺失，属于调度主链路失败；但当前证据只覆盖单个 Web 任务，没有显示跨渠道或大面积扩散，因此暂不升到 `P1`。

## 用户影响

- 用户订阅的是 `20:00` NVDA 日报，但这轮既没收到摘要，也无法在后续打开会话时看到失败提示或任务痕迹。
- 对巡检和排障来说，这会制造“台账有 run，用户会话无痕迹”的双重口径，容易误判成单纯的 SSE 离线问题。
- 这不是单纯的内容质量问题，而是 Web scheduler 的功能性失败。

## 根因判断

- 从 `acp-events.log` 的时序看，本轮更接近“Codex ACP 在工具链未完全收敛时提前 `end_turn`”，而不是纯粹的 Web SSE 离线：
  - 已有连续正文 chunk
  - 最后一个可见事件仍是工具完成
  - 随后直接 `stopReason=end_turn`
  - 但没有最终会话持久化或失败文案落库
- 这与 [`docs/bugs/web_scheduler_sse_delivery_required_for_send_success.md`](./web_scheduler_sse_delivery_required_for_send_success.md) 不是同一根因：旧缺陷的前提是“正文已经成功写入会话，只因没有活跃 SSE 监听者而被记成 send_failed”；本轮则连会话都没有新增本次 scheduler 记录。
- 高概率是 Web scheduler 在 `codex_acp` 的 unfinished-tool 失败路径上缺少和 Feishu/Discord 对齐的失败收口，导致中间 chunk 被丢弃、最终失败提示也没有写入 transcript。

## 下一步建议

- 为 Web scheduler 补专门的失败收口审计：
  - `stopReason=end_turn`
  - 是否仍有 pending tools
  - 是否产生过最终 assistant
  - 是否落库失败提示
- 把 `unfinished tool` 类失败统一沉淀为可追溯的 assistant 失败消息，避免出现“台账失败但会话无痕迹”。
- 将本单与 `web_scheduler_sse_delivery_required_for_send_success.md` 分开跟踪：
  - 后者关注“正文已落库但只因 SSE 离线被记失败”
  - 本单关注“Codex ACP 未完成工具时直接失败，且失败结果未进入会话”
