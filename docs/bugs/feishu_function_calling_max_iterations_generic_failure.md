# Bug: Feishu function-calling 直聊 / 普通 scheduler 耗尽迭代后只返回通用失败

- **发现时间**: 2026-07-20 11:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
    - `2026-07-20T08:00:01.151872+08:00` 普通 scheduler `AI硬件涨价环节挖掘与标的推荐` 触发。
    - `2026-07-20T08:00:50.431215+08:00` assistant final 仅为通用失败提示；随后同 session 另写入 scheduler failure 元数据消息。
  - `session_id=Actor_feishu__direct__ou_5f49e2e252460a05eee0ff98f685cf9f16`
    - `2026-07-20T09:06:15.520997+08:00` 用户要求单独深挖光模块上游公司替代逻辑与节奏。
    - `2026-07-20T09:08:57.890512+08:00` assistant final 仅为通用失败提示。
  - `session_id=Actor_feishu__direct__ou_5fce891d255ae588dde3bd7b1494a28d1e`
    - `2026-07-20T10:51:38.781641+08:00` 用户询问中国铝业历史大分红及分红后股价走势。
    - `2026-07-20T10:54:18.789500+08:00` assistant final 仅为通用失败提示。
- `data/runtime/logs/web.log.2026-07-20`
  - `08:00:50` Feishu scheduler 多次执行 `data_fetch quote ...` 后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`，随后记录 `suppressed internal failure fallback` 和 `定时任务执行失败，本轮不发送`。
  - `09:08:57` Feishu direct 多轮 `web_search` / `data_fetch` 成功后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`，最终持久化失败 assistant。
  - `10:54:18` Feishu direct 多轮 `web_search` / `data_fetch` 成功后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`，最终发送 failure fallback。

## 端到端链路

1. Feishu direct 或普通 scheduler 收到复杂投研 / 历史数据请求。
2. function-calling runner 进入 agent loop，并成功执行多次 `web_search` / `data_fetch`。
3. 模型继续追加工具调用或搜索查询，直到达到 `max_iterations_exceeded:10`。
4. 上层把该内部错误净化成通用失败文案；普通 scheduler 记录失败并跳过本轮发送，直聊则向用户发送通用失败。
5. 用户没有拿到请求的分析结果，也没有拿到可操作的降级摘要或“信息过多请缩小范围”的解释。

## 期望效果

- function-calling 达到迭代上限前，应基于已成功获取的工具结果生成可用的部分答案，或主动停止继续调用工具。
- 达到迭代上限时，应进入受控恢复：限制后续工具调用、要求直接总结已得证据、或返回明确的用户可操作降级提示。
- 普通 scheduler 不应在已取得多轮数据后整轮无正文；直聊也不应只给“稍后再试”的通用失败。

## 当前实现效果

- 三个样本都已经成功执行多轮工具，但最终没有消费这些结果生成用户可见回答。
- Feishu direct 用户只看到通用失败提示；普通 scheduler 仅写入失败台账 / failure fallback，未发送业务正文。
- 这是功能性缺陷：用户明确提出的分析任务没有完成，且已获取的中间证据被丢弃。

## 用户影响

- 复杂投研、历史分红复盘、产业链排序等高价值请求会在耗尽迭代后直接失败。
- 用户只能重试或改写问题；系统不会告诉用户需要缩小范围，也不会给出已有证据的部分结论。
- 定级为 `P2`：它会阻断普通直聊 / 普通 scheduler 的当前任务，但没有观察到跨用户错投、数据破坏、敏感信息泄露或渠道整体停摆，因此不定为 `P1`。

## 根因判断

- 直接根因为 function-calling agent loop 缺少普通直聊 / 普通 scheduler 的 max-iteration 恢复策略。
- 现有 heartbeat 有 `BudgetRecovery { reason: MaxIterationsExceeded }` 相关分支，但本轮 Feishu direct / 普通 scheduler 样本仍直接以 `max_iterations_exceeded:10` 失败收口，说明该恢复没有覆盖这些链路或没有强制模型在预算耗尽前停止工具调用。
- 这不同于既有 `scheduler_heartbeat_iteration_exhaustion_skips_alert.md`：本单影响的是普通 direct / 普通 scheduler 回复生成，不是 heartbeat 专用触发提醒解析。

## 下一步建议

1. 给 function-calling direct / ordinary scheduler 增加 max-iteration recovery：二次运行时禁止或强限制工具调用，并要求只基于已有 tool result 生成回答。
2. 在达到工具 / 迭代预算前插入“必须总结”的停止条件，避免模型继续发起低收益搜索。
3. 将 `max_iterations_exceeded` 的失败文案从通用失败改成可操作提示，同时保留内部错误不外泄。
4. 补回归：模拟多次工具结果已可回答但模型继续请求工具，验证最终能生成部分答案而不是 failure fallback。
