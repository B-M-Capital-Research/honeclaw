# Bug: Feishu 出站消息请求传输失败导致已生成回复无法送达

- **发现时间**: 2026-04-21 13:03 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=3947`
    - `job_name=每日公司资讯与分析总结`
    - `executed_at=2026-04-21T12:03:14.874759+08:00`
    - `execution_status=completed`
    - `message_send_status=send_failed`
    - `should_deliver=1`
    - `delivered=0`
    - `error_message=集成错误: Feishu send message request failed: error sending request for url (https://open.feishu.cn/open-apis/im/v1/messages?receive_id_type=open_id)`
    - `detail_json.receive_id=ou_39103ac18cf70a98afc6cfc7529120e5`
    - `response_preview` 已是完整公司资讯与财报日总结开头，说明模型执行和会话落库已完成，失败发生在最终 Feishu 出站请求阶段。
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `ordinal=17` 的 user turn 是 `[定时任务触发] 任务名称：每日公司资讯与分析总结`
    - `ordinal=18` 的 assistant turn 在 `2026-04-21T12:03:09.867260+08:00` 已写入 3175 字左右最终答复。
  - `data/runtime/logs/web.log`
    - `2026-04-21 12:03:14.873` 记录 `[Feishu] 定时任务投递失败: job=每日公司资讯与分析总结 ... Feishu send message request failed: error sending request for url (https://open.feishu.cn/open-apis/im/v1/messages?receive_id_type=open_id)`
    - `2026-04-21 12:03:50.549` 另一个真实直聊会话 `Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c` 已完成生成并落库，`reply.chars=1649`
    - `2026-04-21 12:03:55.551` 随后同一 Feishu 出站发送请求失败：`[Feishu] 发送回复失败: 集成错误: Feishu send message request failed: error sending request for url (https://open.feishu.cn/open-apis/im/v1/messages?receive_id_type=open_id)`
  - 相关但不同的已有缺陷：
    - [`feishu_scheduler_send_failed_http_400_after_generation.md`](./feishu_scheduler_send_failed_http_400_after_generation.md) 跟踪的是 Feishu API 返回 `HTTP 400 / code=99992361 / open_id cross app`。
    - [`feishu_scheduler_tenant_access_token_request_failure.md`](./feishu_scheduler_tenant_access_token_request_failure.md) 跟踪的是 `tenant_access_token/internal` 请求失败。
    - 本次样本发生在 `im/v1/messages?receive_id_type=open_id` 发送请求阶段，错误没有 Feishu 业务响应体，属于新的出站消息请求传输失败形态。

## 端到端链路

1. Feishu 定时任务或直聊消息进入正常 agent 执行链路。
2. Agent 完成工具调用、生成最终回复，并把 assistant 消息持久化到 `session_messages`。
3. 出站层调用 Feishu `im/v1/messages?receive_id_type=open_id` 发送最终文本。
4. 发送请求在传输层失败，返回 `error sending request for url (...)`。
5. 调度任务落成 `completed + send_failed + delivered=0`，直聊则记录 `发送回复失败`；用户侧无法收到已经生成的回复。

## 期望效果

- 最终回复已生成并持久化后，Feishu 出站发送应可靠送达，或至少自动重试短暂的传输失败。
- 若发送仍失败，应有可恢复的补偿机制，例如稍后重发、明确记录待补发状态，避免“数据库有答案但用户看不到”。
- 定时任务与直聊回复应共享出站请求失败的重试/补偿策略。

## 当前实现效果

- `2026-04-21 12:03` 同一时间窗里，一条定时任务和一条直聊回复都已经完成生成，但最终 Feishu 发送请求在同一个 `im/v1/messages` 出站端点失败。
- 定时任务有明确台账：`should_deliver=1` 但 `delivered=0`。
- 直聊会话有 `session.persist_assistant` 与 `handler.session_run completed success=true`，随后才出现 `发送回复失败`，说明用户可能看到 placeholder 或 busy 提示，但收不到正式答案。
- 这不是 AI 回答质量问题，而是消息投递链路功能失败。

## 用户影响

- 用户会认为任务没有回复或定时报告没有发送，即使系统内部已经生成了答案。
- 该问题直接影响直聊问答和定时任务两条核心 Feishu 出站链路，因此定级为 `P1`。
- 之所以不是 `P0`，是因为当前证据没有显示跨用户误投、数据泄露或全渠道完全不可用；影响是“已生成内容无法送达”。

## 根因判断

- 直接失败点是 Feishu `im/v1/messages` 出站 HTTP 请求传输层失败。
- 当前没有看到出站层对该类传输失败做自动重试、延迟补发或台账化待重送。
- 由于同一时间窗内定时任务和直聊都命中相同端点失败，初步判断根因更接近 Feishu 出站发送公共链路的瞬时传输失败缺少吸震，而不是某个任务模板或某个 answer 内容异常。

## 下一步建议

- 为 Feishu 出站 `send message request failed: error sending request for url (...)` 补有限重试，优先覆盖定时任务和直聊最终回复。
- 对已经生成且落库但发送失败的消息，增加可补偿状态或重发队列，避免用户侧永久丢失答案。
- 后续巡检重点观察 `data/runtime/logs/web.log` 与 `cron_job_runs` 中 `im/v1/messages?receive_id_type=open_id` 的同类传输失败是否继续扩散。
