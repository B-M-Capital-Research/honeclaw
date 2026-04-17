# Bug: Feishu 直聊任务配置请求在搜索阶段反复调用 `cron_job` 后耗尽迭代并整轮无回复

- **发现时间**: 2026-04-16 12:06 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
    - `2026-04-16T11:24:58.582499+08:00` 用户发送长文本任务配置请求，要求按“资深产业价值投资专家”口径重写和执行一组日常任务
    - 在 `timestamp >= 2026-04-16T11:24:00+08:00` 的真实时间窗里，该会话只新增了这条 `role=user` 消息，没有任何新的 `role=assistant` 落库
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 11:24:58.565` `step=reply.placeholder`，说明 Feishu 侧已经开始处理该请求
    - `2026-04-16 11:25:06.434` 到 `11:26:02.804` 之间，search 阶段连续多次执行 `cron_job`
    - `2026-04-16 11:26:02.804` `stage=search.done success=false iterations=8 tool_calls=8 live_search_tool=false`
    - `2026-04-16 11:26:02.806` `ERROR [MsgFlow/feishu] failed ... error="已达最大迭代次数 8"`
    - 失败后同一 session 没有出现 `step=session.persist_assistant`、`done user=...` 或 `step=reply.send`
  - 历史日志回溯：
    - `data/runtime/logs/web.log` 中长期存在 `已达最大迭代次数 8` 的 search 失败记录，但本轮证据确认它已直接落在 Feishu 直聊任务治理场景，并表现为“用户无最终回复”

## 端到端链路

1. 用户在 Feishu 直聊中发送任务配置重写请求，希望系统按新角色与任务清单重新组织日常任务。
2. Multi-agent search 阶段没有进入信息检索或收敛总结，而是把问题持续路由到 `cron_job` 工具。
3. search 阶段在 64 秒内连续执行了 8 次 `cron_job`，每次工具层都成功返回，但都没有形成最终结论。
4. runner 在达到 `max_iterations=8` 后直接以 `error="已达最大迭代次数 8"` 终止。
5. 失败后链路没有持久化 assistant 兜底文案，也没有发送最终回复，用户侧只看到处理中占位，随后整轮静默结束。

## 期望效果

- 任务配置或定时任务治理类问题，应在有限轮数内收敛到明确结果，不能无意义地重复调用同一个 `cron_job` 工具。
- 即便 search 阶段耗尽迭代，也应向用户返回可见的失败说明或降级结论，而不是整轮无回复。
- 会话落库应能反映最终用户可见结果；如果没有正常回答，至少要有受控错误文案，而不是只留下用户消息。

## 当前实现效果

- 本轮会话中，search 阶段 8 次工具调用全部落在 `cron_job`，`live_search_tool=false`，说明 agent 在任务治理问题上陷入了工具循环，而不是完成分析或进入 answer 阶段。
- 日志在 `11:26:02.806` 明确结束于 `failed ... error="已达最大迭代次数 8"`，之后没有 `session.persist_assistant`、没有 `reply.send`、也没有 `done user=...` 收尾日志。
- `session_messages` 里按真实 `timestamp` 过滤后，`11:24` 之后只剩用户输入，没有任何新的 assistant 文本，说明这不是“回答发出但落库丢失”，而是整轮确实没有产出用户可见最终回复。
- 这条事故不是单纯回答质量浅或格式不佳，而是用户提出的核心任务根本没有完成。

## 用户影响

- 这是功能性缺陷。用户已经明确要求系统重写和整理日常任务，但本轮没有收到任何最终回复，任务链路被直接中断。
- 由于问题发生在 Feishu 直聊主链路，且没有可见错误兜底，用户无法判断是请求还在处理中还是系统已经失败，因此定级为 `P1`。
- 之所以不是 `P0`，是因为当前证据仍集中在单条会话和单类任务治理请求，没有证明所有 Feishu 直聊都不可用。

## 根因判断

- search 阶段对“重写任务配置”这类请求的策略错误，持续把问题路由到 `cron_job` 工具，而没有在若干次工具返回后收敛为总结或进入 answer 阶段。
- `max_iterations` 触顶后的失败分支没有像 timeout/空回复场景那样接入统一的用户可见降级文案，因此直接演化成“无回复”。
- 日志里 `tool_execute_success name=cron_job` 连续出现，说明不是单次工具报错，而是 agent orchestration 缺少“重复同工具但无进展”的中止与降级机制。

## 下一步建议

- 优先排查 multi-agent search 在任务治理/任务重写类请求上的工具选择与停止条件，避免重复调用 `cron_job` 却不产出中间结论。
- 为“达到最大迭代次数”失败分支补统一用户态错误兜底，至少保证 Feishu 直聊不会再次整轮无回复。
- 增加回归用例：当同一 search 会话连续多轮只调用 `cron_job` 且未形成答案时，应返回明确失败文案或提前终止，而不是耗尽 8 轮后静默结束。

## 当前修复进展（2026-04-17 10:40 CST）

- 本轮先修“耗尽迭代后整轮无回复”的下游症状，而不是直接修改 search 策略：
  - `bins/hone-feishu/src/handler.rs` 已为每条消息处理增加 join/panic 兜底，并补 `handler.session_run=dispatch/completed` 边界日志。
  - `bins/hone-feishu/src/outbound.rs` 已为 placeholder 更新失败补 standalone send 回退；即使错误阶段无法更新已有 placeholder，也应继续尝试发出最终失败文案。
- 这意味着下一次再出现 `已达最大迭代次数 8` 时，用户侧更应收到受控失败提示，而不是只看到 placeholder 后静默结束。
- 自动化验证已通过：
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-channels`
- 由于“反复调用 `cron_job` 不收敛”的根因仍未直接消除，本单状态更新为 `Fixing`，待真实任务治理样本确认“至少不再无回复”后，再决定是否拆出独立的策略优化单。
