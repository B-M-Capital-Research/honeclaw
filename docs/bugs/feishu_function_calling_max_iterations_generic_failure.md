# Bug: Function-calling 直聊 / 普通 scheduler 耗尽迭代后只返回通用失败

- **发现时间**: 2026-07-20 11:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `2026-07-21 19:01-23:01 CST` 运行态复核补充同根 Web direct 连续失败样本，状态维持 `New/P2`：
    - `session_id=Actor_web__direct__web-user-0545ade83537`
      - `2026-07-21T21:13:18.115453+08:00` 用户请求“交换机 概念A股和美股的核心标的”，`2026-07-21T21:16:17.176346+08:00` assistant final 仅为通用失败提示。
      - `2026-07-21T21:21:14.813446+08:00` 用户同题重试，`2026-07-21T21:23:49.051338+08:00` assistant final 再次仅为通用失败提示。
      - `2026-07-21T22:33:16.454049+08:00` 用户第三次同题重试，`2026-07-21T22:36:23.777791+08:00` assistant final 仍仅为通用失败提示。
    - 同窗 runtime 日志显示第三次请求在 22:33-22:36 已成功执行多轮 `data_fetch search`、`data_fetch quote`、`data_fetch snapshot` 和 `web_search`，覆盖 688702.SS、002396.SZ、000063.SZ、000938.SZ、JNPR 等交换机 / 网络设备相关标的；`2026-07-21 22:36:23` 仍以 `error="max_iterations_exceeded:10"` 失败并持久化 failure assistant。
    - 同一简单主题连续三次失败，说明当前 live Web direct strict function-calling 路径仍会在已有工具结果时耗尽 10 次迭代并丢弃答案；与本缺陷同根，不新建重复缺陷。
    - 严重等级维持 `P2`：它阻断用户明确提出的投研任务，但同窗其它 Web / Feishu direct 与 scheduler 仍有正常 assistant 收口，未见跨用户错投、数据破坏、敏感信息泄露或全渠道停摆，因此不是 `P1`。
  - `2026-07-21 11:02-15:02 CST` 运行态复核补充同根普通 scheduler 样本，状态维持 `New/P2`：
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
      - `2026-07-21T12:00:01.666687+08:00` Feishu scheduler `每日公司资讯与分析总结` 触发，要求汇总 TEM / CAI / NBIS / CRWV / NVDA / GOOGL / TSM 的最新资讯、分析师摘要和下次财报日期。
      - `2026-07-21T12:01:37.421899+08:00` assistant final 仅返回通用失败提示；随后同 session 追加 `scheduler_failure=true` 的失败补偿消息。
    - 同窗 runtime 日志显示本轮已成功执行多轮 `data_fetch quote`（TSM / NBIS / CRWV / RKLB / TEM / CAI / NVDA）和 `web_search`，最后仍在 `2026-07-21 12:01:37` 以 `error="max_iterations_exceeded:10"` 失败，`SchedulerDiag` 将其压成 `internal_error_suppressed` 并跳过 Feishu 外发。
    - 该样本晚于 `2026-07-21T03:09:07+08:00` 的 `39fe6e59 fix strict actor iteration budget` 与 10:31 复发样本，说明当前 live 普通 scheduler 路由仍可能保留 10 次 strict function-calling 上限或修复未生效；不新建重复缺陷。
  - `2026-07-21 07:03-11:02 CST` 运行态复核回退为 `New/P2`：
    - `session_id=Actor_feishu__direct__ou_5f49e2e252460a05eee0ff98f685cf9f16`
      - `2026-07-21T10:29:06.193184+08:00` Feishu direct 用户要求从 PE 估值和增长潜力角度推荐本轮暴跌后错杀的 A 股 AI 产业链股票。
      - `2026-07-21T10:31:41.614577+08:00` assistant final 仅返回通用失败提示，用户没有拿到推荐、排序、估值框架或降级摘要。
    - 同窗 runtime 日志显示本轮已成功执行多轮 `web_search`、`data_fetch search` 和 `data_fetch quote`，覆盖光模块 / AI 算力 / 相关 A 股 ticker 的搜索与报价；`2026-07-21 10:31:41` 最终仍以 `error="max_iterations_exceeded:10"` 失败并发送 failure fallback。
    - 这晚于 `2026-07-21T03:09:07+08:00` 的 `39fe6e59 fix strict actor iteration budget`，说明 live 路径仍存在 `10` 次预算复发或修复未在当前运行态生效；状态从 `Fixed` 回退为 `New`。
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
- Runtime logs:
  - `data/runtime/logs/web.log.2026-07-21`
  - `12:01:37` Feishu scheduler `每日公司资讯与分析总结` 在多轮 `data_fetch` / `web_search` 成功后，`MsgFlow/feishu failed ... error="max_iterations_exceeded:10"`；随后记录 `suppressed internal failure fallback`，Feishu 本轮不发送业务正文。
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

- `2026-07-21 10:31` 新样本证明，至少当前 live function-calling strict runner 仍会把普通 Feishu direct 投研请求截断在 `max_iterations_exceeded:10`，即使日志已显示多轮工具成功且 `answer_preserved=true`。
- 当前无法仅凭台账判断是 03:09 代码修复未部署到 live、某条 Feishu direct 路由仍使用旧预算常量，还是 recovery 后又落回旧的 10 次限制；但对用户来说表现仍是同一功能链路失败，因此按同根因文档回退，不另建重复文档。
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

1. 优先确认当前 live Feishu direct strict runner 是否已经加载 `39fe6e59` 后的预算配置；如果未部署，先重启 / 部署后复核同类复杂投研请求。
2. 若已部署仍出现 `max_iterations_exceeded:10`，检查是否有其它 strict runner 路由、恢复分支或环境变量覆盖仍保留 10 次上限。
3. 补 function-calling 的“基于已有 tool result 直接总结”恢复策略，避免即便触顶也丢弃已成功取得的数据。

## 最新运行态复核（2026-07-22 11:03 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-22 07:03-11:03 CST。
  - 09:00 CST Feishu scheduler `核心观察池早间简报` 的用户任务要求“使用最新美股市场价格，不要用过时数据”，assistant final 仅返回“抱歉，这次处理失败了。请稍后再试。”，随后又写入 scheduler failure 补偿“本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。”
- `data/runtime/logs/web.log.2026-07-22`
  - 同窗未命中 `max_iterations_exceeded`，因此本轮不能确认该样本仍是 10 次预算上限复发。
- 本轮判断
  - 这条样本只证明普通 scheduler 仍会在复杂投研 / 观察池任务中退化成通用失败，用户没有拿到降级摘要或部分结果；但根因尚未确认为本单原始的 `max_iterations_exceeded:10`。
  - 暂按既有 function-calling 通用失败链路追加待查证据，不新建重复缺陷、不调整严重等级或状态。影响仍是单轮 scheduler 任务失败；同窗 direct 与多个 scheduler 正常收口，未见错投、数据破坏或全渠道不可用，维持功能性 `P2 / New`，非 P1。

## 最新运行态复核（2026-07-23 11:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-23 07:01-11:02 CST。
  - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
  - 08:30 CST Feishu scheduler `美股AI产业链盘后报告` 触发，任务要求生成美股盘后全产业链景气报告。
  - 08:31 CST assistant final 只返回“抱歉，这次处理失败了。请稍后再试。”，随后追加 scheduler failure 补偿“本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。”
- `data/runtime/logs/web.log.2026-07-23`
  - 同一 session 在 08:31 前已执行多轮 `web_search` / `data_fetch`，包括 Tesla / Alphabet 财报与 TSLA quote。
  - 08:31:22 CST 记录 `answer_preserved=true`，随后 `MsgFlow/feishu failed ... error="max_iterations_exceeded:18"`，`SchedulerDiag` 将其压成 `internal_error_suppressed` 并跳过本轮 Feishu 外发。
- 本轮判断
  - 这是同一 function-calling 在已取得工具结果后仍耗尽迭代、最终只给通用失败的链路；与旧样本的 `10` 次预算不同，本窗已触到 `18` 次上限，说明单纯提高预算不足以保证复杂 scheduler 收口。
  - 影响是单轮普通 scheduler 业务正文缺失；同窗其它 direct / scheduler 可正常收口，未见错投、数据破坏或全渠道不可用，维持功能性 `P2 / New`，非 P1。
