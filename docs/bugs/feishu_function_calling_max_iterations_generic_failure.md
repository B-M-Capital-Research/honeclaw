# Bug: Function-calling 直聊 / 普通 scheduler 耗尽迭代后只返回通用失败

- **发现时间**: 2026-07-20 11:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `2026-07-21 03:02-07:03 CST` 修复后运行态复核，状态维持 `Fixed`：
    - 最近非文档提交 `39fe6e59 fix strict actor iteration budget` 发生在 `2026-07-21T03:09:07+08:00`，与本单 2026-07-20 代码级修复一致。
    - 同窗 `data/sessions.sqlite3` 新增 15 条 user / 9 条 assistant / 4 条 system compact，覆盖 7 个更新 session；采样点 07:00 Feishu scheduler 后续已在 07:02 assistant 收口。
    - assistant final 污染扫描未命中 `max_iterations`；`data/runtime/logs/web.log.2026-07-20` 同窗未再出现 `max_iterations_exceeded:10`。05:00 Web scheduler 的用户可见“执行出错”经 runtime 观察更接近实体 guard / 任务词误抽问题，已归入 `scheduler_finance_entity_guard_misclassifies_instruction_words.md`，不作为本单回退证据。
    - 结论：本轮未见 strict runner 10 次迭代上限复发，继续按代码级 / 运行态初步止血 `Fixed` 记录；后续若出现 `max_iterations_exceeded:18`，应另按恢复策略不足评估，不直接等同于本单的 `10` 次预算回退。
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

- 直接根因为普通用户 fallback 到 strict `function_calling` runner 时，被单独压到 `STRICT_ACTOR_MAX_ITERATIONS=10`；这低于当前仓库其它 agent 路径默认使用的 `18`，导致复杂直聊 / 普通 scheduler 在已有多轮有效 `data_fetch` / `web_search` 结果时仍被过早截断成 `max_iterations_exceeded:10`。
- 现有 heartbeat 有 `BudgetRecovery { reason: MaxIterationsExceeded }` 相关分支，但本轮 Feishu / Web direct 与普通 scheduler 样本仍直接以 strict runner 的 `10` 次上限失败收口，说明至少有一层问题是普通 strict runner 的预算明显偏紧，而不是 heartbeat 恢复逻辑本身。
- 这不同于既有 `scheduler_heartbeat_iteration_exhaustion_skips_alert.md`：本单影响的是普通 direct / 普通 scheduler 回复生成，不是 heartbeat 专用触发提醒解析。

## 修复记录

- 2026-07-20 代码级修复：
  - `crates/hone-channels/src/core/bot_core.rs`
    - 将普通用户 strict fallback `function_calling` runner 的 `STRICT_ACTOR_MAX_ITERATIONS` 从 `10` 对齐到仓库当前标准预算 `18`，避免 Web / Feishu direct 与普通 scheduler 在已取得多轮有效工具结果后被过早截断。
  - `crates/hone-channels/src/core/tests.rs`
    - 新增 `strict_actor_runner_uses_the_standard_iteration_budget`，锁定 strict fallback runner 使用标准迭代预算，避免后续再次无意回落到更低上限。

## 验证

- `cargo test -p hone-channels strict_actor_runner_uses_the_standard_iteration_budget --lib -- --nocapture`
- `cargo test -p hone-channels effective_context_owner_follows_actor_runner_route --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 下一步建议

1. 后续巡检重点看 2026-07-20 之后的新样本是否还出现 `max_iterations_exceeded:10`；若 strict runner 已不再报 `10`，说明这次预算对齐已至少止血一层活跃坏态。
2. 如果后续仍在 `18` 次上限触顶，则再继续补 function-calling 的“基于已有 tool result 直接总结”恢复策略，而不是再次单纯抬高上限。
