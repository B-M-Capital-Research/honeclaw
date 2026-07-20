# Bug: Function-calling 直聊 / 普通 scheduler 耗尽迭代后只返回通用失败

- **发现时间**: 2026-07-20 11:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `2026-07-20 15:05-19:02 CST` 复核新增 1 条同根样本，状态维持 `New/P2`：
    - `session_id=Actor_web__direct__web-user-400794904801`
      - `2026-07-20T17:09:01.711385+08:00` Web direct 用户在 TSLA 走势讨论后追问“人形机器人会不会是突破口啊”。
      - `2026-07-20T17:11:33.085813+08:00` assistant final 仅为通用失败提示。
      - `2026-07-20T17:12:18.630836+08:00` 用户同题补全为“特斯拉的人形机器人会不会是突破口啊”，`2026-07-20T17:13:19.843380+08:00` assistant 成功输出 2642 字回答，说明链路可恢复，但首轮仍丢弃了已取得的工具结果。
  - `2026-07-20 11:01-15:05 CST` 复核新增 3 条同根样本，状态维持 `New/P2`：
    - `session_id=Actor_web__direct__web-user-5bb05078acd4`
      - `2026-07-20T11:38:48.738262+08:00` Web direct 用户要求判断 A 股沪电股份走势、亏损时是否割肉或补仓。
      - `2026-07-20T11:41:02.051850+08:00` assistant final 仅为通用失败提示。
    - `session_id=Actor_feishu__direct__ou_5fce891d255ae588dde3bd7b1494a28d1e`
      - `2026-07-20T12:35:53.553414+08:00` Feishu direct 用户追加中国铝业回购、霍尔木兹海峡、超跌逻辑下买入胜率判断。
      - `2026-07-20T12:39:19.587033+08:00` assistant final 仅为通用失败提示。
    - `session_id=Actor_web__direct__web-user-f40ae1caa720`
      - `2026-07-20T13:31:19.805085+08:00` Web direct 用户问 510880 ETF 成本价 3.039 是否应止盈。
      - `2026-07-20T13:32:53.439495+08:00` assistant final 仅为通用失败提示；用户 13:33 同题重试后 13:34 成功得到 1206 字回答，说明链路可恢复，但首轮仍丢弃了已取得的工具结果。
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
  - `17:11:33` Web direct 特斯拉人形机器人追问在多轮工具执行后，`MsgFlow/web failed ... error="max_iterations_exceeded:10"`，最终持久化 failure assistant；同一会话 17:12 重试成功收口。
  - `11:41:02` Web direct 沪电股份走势判断在多轮 `data_fetch` / `web_search` 成功后，`MsgFlow/web failed ... error="max_iterations_exceeded:10"`，最终持久化 failure assistant。
  - `12:39:19` Feishu direct 中国铝业胜率判断在多轮 `data_fetch` / `web_search` 成功后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`，最终发送 failure fallback。
  - `13:32:53` Web direct 510880 ETF 止盈判断在多轮 `data_fetch` / `web_search` 成功后，`MsgFlow/web failed ... error="max_iterations_exceeded:10"`；同题 13:33 重试以 5 次迭代 / 6 个工具调用成功收口。
  - `08:00:50` Feishu scheduler 多次执行 `data_fetch quote ...` 后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`，随后记录 `suppressed internal failure fallback` 和 `定时任务执行失败，本轮不发送`。
  - `09:08:57` Feishu direct 多轮 `web_search` / `data_fetch` 成功后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`，最终持久化失败 assistant。
  - `10:54:18` Feishu direct 多轮 `web_search` / `data_fetch` 成功后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`，最终发送 failure fallback。

## 端到端链路

1. Feishu / Web direct 或普通 scheduler 收到复杂投研 / 历史数据请求。
2. function-calling runner 进入 agent loop，并成功执行多次 `web_search` / `data_fetch`。
3. 模型继续追加工具调用或搜索查询，直到达到 `max_iterations_exceeded:10`。
4. 上层把该内部错误净化成通用失败文案；普通 scheduler 记录失败并跳过本轮发送，直聊则向用户发送通用失败。
5. 用户没有拿到请求的分析结果，也没有拿到可操作的降级摘要或“信息过多请缩小范围”的解释。

## 期望效果

- function-calling 达到迭代上限前，应基于已成功获取的工具结果生成可用的部分答案，或主动停止继续调用工具。
- 达到迭代上限时，应进入受控恢复：限制后续工具调用、要求直接总结已得证据、或返回明确的用户可操作降级提示。
- 普通 scheduler 不应在已取得多轮数据后整轮无正文；直聊也不应只给“稍后再试”的通用失败。

## 当前实现效果

- 多个 Feishu / Web direct 与普通 scheduler 样本都已经成功执行多轮工具，但最终没有消费这些结果生成用户可见回答。
- Feishu / Web direct 用户只看到通用失败提示；普通 scheduler 仅写入失败台账 / failure fallback，未发送业务正文。
- 这是功能性缺陷：用户明确提出的分析任务没有完成，且已获取的中间证据被丢弃。

## 用户影响

- 复杂投研、历史分红复盘、产业链排序等高价值请求会在耗尽迭代后直接失败。
- 用户只能重试或改写问题；系统不会告诉用户需要缩小范围，也不会给出已有证据的部分结论。
- 定级为 `P2`：它会阻断普通直聊 / 普通 scheduler 的当前任务，但没有观察到跨用户错投、数据破坏、敏感信息泄露或渠道整体停摆，因此不定为 `P1`。

## 根因判断

- 直接根因为 function-calling agent loop 缺少普通直聊 / 普通 scheduler 的 max-iteration 恢复策略。
- 现有 heartbeat 有 `BudgetRecovery { reason: MaxIterationsExceeded }` 相关分支，但本轮 Feishu / Web direct 与普通 scheduler 样本仍直接以 `max_iterations_exceeded:10` 失败收口，说明该恢复没有覆盖这些链路或没有强制模型在预算耗尽前停止工具调用。
- 这不同于既有 `scheduler_heartbeat_iteration_exhaustion_skips_alert.md`：本单影响的是普通 direct / 普通 scheduler 回复生成，不是 heartbeat 专用触发提醒解析。

## 下一步建议

1. 给 function-calling direct / ordinary scheduler 增加 max-iteration recovery：二次运行时禁止或强限制工具调用，并要求只基于已有 tool result 生成回答。
2. 在达到工具 / 迭代预算前插入“必须总结”的停止条件，避免模型继续发起低收益搜索。
3. 将 `max_iterations_exceeded` 的失败文案从通用失败改成可操作提示，同时保留内部错误不外泄。
4. 补回归：模拟多次工具结果已可回答但模型继续请求工具，验证最终能生成部分答案而不是 failure fallback。
