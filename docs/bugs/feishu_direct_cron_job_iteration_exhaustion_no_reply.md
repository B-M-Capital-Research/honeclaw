# Bug: Feishu 直聊任务治理 / 定时汇总请求在搜索阶段耗尽迭代后整轮无回复

- **发现时间**: 2026-04-16 12:06 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
    - `2026-04-16T11:24:58.582499+08:00` 用户发送长文本任务配置请求，要求按“资深产业价值投资专家”口径重写和执行一组日常任务
    - 在 `timestamp >= 2026-04-16T11:24:00+08:00` 的真实时间窗里，该会话只新增了这条 `role=user` 消息，没有任何新的 `role=assistant` 落库
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-04-17T12:00:00.495769+08:00` 定时任务触发 `每日公司资讯与分析总结`，要求汇总 `TEM/CAI/NBIS/CRWV/NVDA/GOOGL/TSM` 的最新资讯、分析师总结与财报日期
    - 截至 `2026-04-17T12:01:26.486+08:00` 对应失败日志写出时，同一 session 仍只新增这条 `role=user` 消息，没有新的 `role=assistant` 落库
  - 最近一小时调度台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2138`，`job_id=j_7c688485`，`job_name=每日公司资讯与分析总结`
    - `executed_at=2026-04-17T12:01:28.608155+08:00`
    - `execution_status=execution_failed`，`message_send_status=sent`，`delivered=1`，`error_message=已达最大迭代次数 8`
    - 同一时间窗内 `web.log` 没有对应 `step=reply.send`，`session_messages` 也没有新增 assistant 文本，说明 scheduler 台账与真实会话可见结果已经出现不一致
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 11:24:58.565` `step=reply.placeholder`，说明 Feishu 侧已经开始处理该请求
    - `2026-04-16 11:25:06.434` 到 `11:26:02.804` 之间，search 阶段连续多次执行 `cron_job`
    - `2026-04-16 11:26:02.804` `stage=search.done success=false iterations=8 tool_calls=8 live_search_tool=false`
    - `2026-04-16 11:26:02.806` `ERROR [MsgFlow/feishu] failed ... error="已达最大迭代次数 8"`
    - 失败后同一 session 没有出现 `step=session.persist_assistant`、`done user=...` 或 `step=reply.send`
    - `2026-04-17 12:00:00.638` `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 进入 `runner.stage=multi_agent.search.start`
    - `2026-04-17 12:00:10.754` 到 `12:01:26.369` 之间，search 阶段依次完成 `data_fetch snapshot TEM/CAI/NBIS/CRWV/NVDA/GOOGL/TSM` 与 `data_fetch earnings_calendar`
    - `2026-04-17 12:01:26.380` 记录 `stage=search.done success=false iterations=8 tool_calls=8 live_search_tool=true`
    - `2026-04-17 12:01:26.486` `ERROR [MsgFlow/feishu] failed ... error="已达最大迭代次数 8"`
    - 失败后同一 session 同样没有出现 `step=session.persist_assistant`、`done user=...` 或 `step=reply.send`
  - 历史日志回溯：
    - `data/runtime/logs/web.log` 中长期存在 `已达最大迭代次数 8` 的 search 失败记录，但本轮证据确认它已直接落在 Feishu 直聊任务治理场景，并表现为“用户无最终回复”

## 端到端链路

1. 用户在 Feishu 直聊中提交任务治理类请求，或由直达定时任务触发复杂汇总请求。
2. Multi-agent search 阶段持续消耗迭代预算，但没有把已拿到的工具结果收敛成 answer 输入或用户可见结论。
3. 已观测到两种失败路径：
   - 任务治理请求里反复调用 `cron_job`，8 轮都没有收敛。
   - 定时汇总请求里连续完成 8 次 `data_fetch`，但仍在 search 阶段触顶，没有进入 answer。
4. runner 在达到 `max_iterations=8` 后直接以 `error="已达最大迭代次数 8"` 终止。
5. 失败后链路没有持久化 assistant 兜底文案，也没有发送最终回复，用户侧整轮静默结束。

## 期望效果

- 任务治理或多标的定时汇总类问题，应在有限轮数内收敛到明确结果，不能在 search 阶段耗尽迭代后仍没有结论。
- 即便 search 阶段耗尽迭代，也应向用户返回可见的失败说明或降级结论，而不是整轮无回复。
- 会话落库应能反映最终用户可见结果；如果没有正常回答，至少要有受控错误文案，而不是只留下用户消息。

## 当前实现效果

- 任务治理样本里，search 阶段 8 次工具调用全部落在 `cron_job`，`live_search_tool=false`，说明 agent 在任务编排问题上陷入了工具循环，而不是完成分析或进入 answer 阶段。
- 定时汇总样本里，search 阶段 8 次工具调用全部成功返回，`live_search_tool=true`，但仍在 `data_fetch` 收集阶段耗尽预算，说明问题不只限于 `cron_job` 工具循环，而是“search 触顶后的失败分支仍会静默”这一公共收口缺陷。
- 两个样本的日志都结束于 `failed ... error="已达最大迭代次数 8"`，之后没有 `session.persist_assistant`、没有 `reply.send`、也没有 `done user=...` 收尾日志。
- `session_messages` 里按真实 `timestamp` 过滤后，两条会话在失败窗口都只剩用户输入，没有任何新的 assistant 文本，说明这不是“回答发出但落库丢失”，而是整轮确实没有产出用户可见最终回复。
- `2026-04-17 12:01` 的最新 scheduler 台账又暴露出第二层症状：`cron_job_runs` 已记成 `execution_failed + sent + delivered=1`，但真实会话与运行日志都没有 `reply.send` 或 assistant 落库，说明“失败已送达”的账本口径也不可靠。
- 这条事故不是单纯回答质量浅或格式不佳，而是用户提出的核心任务根本没有完成。

## 用户影响

- 这是功能性缺陷。用户已经明确要求系统重写和整理日常任务，但本轮没有收到任何最终回复，任务链路被直接中断。
- 由于问题发生在 Feishu 直聊主链路，且没有可见错误兜底，用户无法判断是请求还在处理中还是系统已经失败，因此定级为 `P1`。
- 之所以不是 `P0`，是因为当前证据仍集中在单条会话和单类任务治理请求，没有证明所有 Feishu 直聊都不可用。

## 根因判断

- 已确认存在两层根因叠加：
  - 上游 search 策略在不同任务类型上都可能失控，要么反复调用 `cron_job`，要么在高基数 `data_fetch` 收集里直接耗尽 8 轮。
  - 下游 `max_iterations` 触顶后的失败分支仍没有稳定接入用户可见降级文案，因此一旦触顶就演化成“无回复”。
- 同时还存在链路台账失真：scheduler 侧已经把本轮记成 `sent`，但消息日志与会话落库都不支持“用户实际收到了失败提示”这一结论；因此排查时不能只看 `cron_job_runs.message_send_status`。
- 日志里 `tool_execute_success name=cron_job` 与多次 `data_fetch ... status=done` 都连续出现，说明不是单次工具报错，而是 agent orchestration 缺少“有进展但仍未收敛”与“重复同工具无进展”的统一中止/收口机制。

## 下一步建议

- 优先排查 multi-agent search 在任务治理/任务重写类请求上的工具选择与停止条件，避免重复调用 `cron_job` 却不产出中间结论。
- 同时排查多标的定时汇总 prompt 的 search 预算与收敛条件，避免仅因标的较多就把 8 轮全部消耗在 `data_fetch`。
- 为“达到最大迭代次数”失败分支补稳定的用户态错误兜底，至少保证 Feishu 直聊和直达定时任务不会再次整轮无回复。
- 增加回归用例：当同一 search 会话连续多轮只调用 `cron_job` 且未形成答案时，应返回明确失败文案或提前终止，而不是耗尽 8 轮后静默结束。
- 再补一条回归：当 search 阶段已完成多次 `data_fetch` 但未能在 8 轮内进入 answer 时，也必须返回受控失败文案或降级摘要，而不是静默终止。

## 当前修复进展（2026-04-17 10:40 CST）

- 本轮先修“耗尽迭代后整轮无回复”的下游症状，而不是直接修改 search 策略：
  - `bins/hone-feishu/src/handler.rs` 已为每条消息处理增加 join/panic 兜底，并补 `handler.session_run=dispatch/completed` 边界日志。
  - `bins/hone-feishu/src/outbound.rs` 已为 placeholder 更新失败补 standalone send 回退；即使错误阶段无法更新已有 placeholder，也应继续尝试发出最终失败文案。
- 但 `2026-04-17 12:01:26` 的最新定时汇总样本说明，这个“失败后至少不再静默”的目标尚未达成：本轮仍直接结束于 `failed ... error="已达最大迭代次数 8"`，且没有任何 assistant 落库或 `reply.send`。
- 自动化验证已通过：
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-channels`
- 由于“search 触顶后仍静默失败”的公共收口缺陷尚未消除，而“反复调用 `cron_job` / 多次 `data_fetch` 后不收敛”这两类上游触发形态也都仍在，本单维持 `Fixing`，待下一轮真实样本确认“至少不再无回复”后再决定是否拆出更细的策略单。
- `2026-04-17 12:01` 的最新定时汇总样本还表明，当前巡检不能把 `cron_job_runs` 的 `sent/delivered=1` 视为修复迹象；在真实会话仍无 assistant 落库、`web.log` 仍无 `reply.send` 的前提下，本单继续保持 `Fixing`。
