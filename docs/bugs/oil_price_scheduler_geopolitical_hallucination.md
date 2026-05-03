# Bug: 原油定时播报把未核验地缘叙述当作油价事实送达用户

- **发现时间**: 2026-04-22 07:00 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 最新进展（2026-05-04 06:02 CST）

- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把预测口径和未核验价格差写成当前事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=15376`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-05-04T06:00:38.652250+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`布伦特原油 (Brent)：约 $108-111/桶（5月1日收盘价 $108.17，5月4日预测区间）`、`WTI 原油：数据未直接获取，通常较 Brent 贴水 $3-5/桶`、`过去一个月 Brent 累计下跌约 0.79%，但同比仍上涨 76%+`。
    - 这条回复虽然附带“市场参考数据、非实时交易所报价”的提示，但仍把 `5月4日预测区间`、`WTI 通常较 Brent 贴水 $3-5/桶` 这种推断性口径直接写进“最新原油价格”，没有降级成“未核验/仅供参考”，也没有给出同窗可追溯来源。
- `data/runtime/logs/sidecar.log`
    - `2026-05-04 06:00:36.245-06:00:36.246` 记录同一任务 `parse_kind=JsonTriggered` 并执行 `deliver`，`deliver_preview` 与 sqlite 中的用户可见正文一致。
- 结论：到 `2026-05-04 06:02` 为止，这条质量缺陷仍在真实整点播报里活跃；最新 06:00 窗口已从“地缘叙述当事实”进一步漂移到“预测区间/经验贴水当现价事实”，状态维持 `New`、严重等级维持 `P2`。

## 最新进展（2026-05-04 03:02 CST）

- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把未核验的宏观/供给归因写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=15240`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-05-04T03:01:11.632202+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.raw_preview` 与 `deliver_preview` 直接向用户发送：`近期变动背景：油价承压主要受OPEC+供应政策不确定性及全球经济增速担忧...`
    - 同条回复虽然明确承认“美国原油期货市场目前处于休市状态”“以上价格为最新可得数据（上一交易日5月1日收盘价）”，但仍把 `OPEC+供应政策不确定性` 与 `全球经济增速担忧` 组织成确定性主因，没有标注“原因待核验/仅供参考”。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-04 03:01:08.335-03:01:08.336` 记录同一任务 `parse_kind=JsonTriggered` 并执行 `deliver`，`deliver_preview` 与 sqlite 中的用户可见正文一致。
- 结论：到 `2026-05-04 03:02` 为止，这条质量缺陷仍在真实整点播报里活跃；最新 03:00 窗口虽然改写了原因话术，但仍把未经同窗来源核验的宏观/供给叙述包装成确定性油价主因，因此状态维持 `New`、严重等级维持 `P2`。

## 最新进展（2026-05-02 21:03 CST）

- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把未核验的贸易/地缘叙述写成确定性油价主因：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=13881`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-05-02T21:01:28.812662+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`近期变动因素：OPEC+产量决策影响供应预期，同时国际贸易关税政策不确定性持续影响市场情绪与伊朗原油相关供应风险。`
    - 同条回复虽然补了“数据存在数小时延迟”，但仍把 `国际贸易关税政策不确定性` 与 `伊朗原油相关供应风险` 组织成确定性主因，没有标注“原因待核验/仅供参考”。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-02 21:01:25.621-21:01:25.622` 记录同一任务 `parse_kind=JsonTriggered` 并执行 `deliver`，`deliver_preview` 与 sqlite 中的用户可见正文一致。
- 结论：到 `2026-05-02 21:03` 为止，这条质量缺陷仍在真实整点播报里活跃；最新 21:00 窗口虽然没有继续复用“标普上调油价预期”那套话术，但仍把未核验的贸易/地缘叙述包装成确定性油价归因，因此状态维持 `New`、严重等级维持 `P2`。

## 最新进展（2026-05-02 06:04 CST）

- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把未经核验的机构观点与供需叙事写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=13202`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-05-02T06:01:11.919294+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`标普最新预测2026年剩余时间原油价格：WTI 95美元/桶、布伦特100美元/桶`，并把 `标普于4月30日上调2026年剩余时间WTI和布伦特原油价格预期各15美元/桶`、`全球石油需求增加且供需趋于不平衡` 组织成确定性主因。
    - 同条回复没有标注“机构预测口径”与“原因待核验”，而是把供需解释包装成已确认事实；这仍不符合“搜索降级时只报已核验价格、未核验原因必须降级”的预期。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-02 06:01:11.919-06:01:11.920` 记录同一任务 `parse_kind=JsonTriggered` 并执行 `deliver`，`deliver_preview` 与 sqlite 中的用户可见正文一致。
- 结论：到 `2026-05-02 06:04` 为止，这条质量缺陷仍在真实整点播报里活跃；最新 06:00 窗口虽然不再复用“霍尔木兹海峡”叙事，但仍把未经核验的机构预测和供需解释组织成确定性油价主因，因此状态维持 `New`、严重等级维持 `P2`。

## 最新进展（2026-05-02 03:03 CST）
- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把未核验的宏观/地缘归因写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=13070`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-05-02T03:00:54.340876+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`近期油价承压，主要因市场对2026年需求前景的担忧——摩根大通预测2026年布伦特均价约60美元/桶，尽管汇丰上调预测至80美元/桶，但整体预期仍偏谨慎。地缘风险（特别是霍尔木兹海峡局势）为价格提供一定支撑，但基本面走弱压力更大。`
    - 同条回复还把 `布伦特原油约 108.02 美元/桶（5月1日数据），较前一日下跌约2.15%` 与上述主因一起组织成确定性结论，没有标注“原因未核验/暂不归因”。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-02 03:00:52.513-03:00:52.514` 记录同一任务 `parse_kind=JsonTriggered` 并执行 `deliver`，`deliver_preview` 与 sqlite 中的用户可见正文一致。
- 结论：到 `2026-05-02 03:03` 为止，这条质量缺陷仍在真实整点播报里活跃；即使搜索与价格链路继续给出可用片段，最终播报仍把 `2026 年需求前景` 与 `霍尔木兹海峡局势` 组织成确定性因果，没有降级为“已核验价格 + 原因待确认”，因此状态维持 `New`、严重等级维持 `P2`。

## 最新进展（2026-05-02 00:02 CST）

- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把未核验的地缘归因写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=12935`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-05-02T00:01:22.448715+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`近期变动原因：价格主要受地缘政治紧张局势驱动，特别是霍尔木兹海峡供应风险持续支撑油价。`
    - 同条回复继续把 `地缘政治因素已超越供需基本面成为主导力量`、`油价此前从极端高位$126回调后，目前在高位区间震荡运行` 包装成确定性叙事，没有降级成“待核验/仅供参考”。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-02 00:01:19.115-00:01:19.116` 记录同一任务 `parse_kind=JsonTriggered` 并执行 `deliver`，`deliver_preview` 与 sqlite 中的用户可见正文一致。
- 结论：到 `2026-05-02 00:02` 为止，这条质量缺陷仍在真实整点播报里活跃；即使 `web_search` 的空成功边界已经加固，原油 heartbeat 仍会在没有可追溯同窗来源的情况下，把霍尔木兹海峡相关叙述写成确定性油价主因，因此状态维持 `New`、严重等级维持 `P2`。

## 修复进展（2026-05-01 19:07 CST）

- `crates/hone-tools/src/web_search.rs` 将 Tavily 未配置 key、全 key 因额度 / 鉴权 / 临时故障不可用时的返回从 `Ok({"status":"unavailable","results":[]})` 改为 `HoneError::Tool(...)`。
- 这会让 `ToolRegistry` 与 function-calling runner 把搜索降级识别为工具失败，而不是继续记录 `tool_execute_success` 并给后续模型一个看似成功的空搜索证据；错误文案保持脱敏，不包含 Tavily 原始支持邮箱、升级提示或 key 细节。
- 这不是针对单次 Tavily 波动的特判，而是通用错误边界修复：外部搜索不可用时必须显式失败，原油 / 宏观 / 地缘归因链路不能再把“空成功搜索”当作已核验来源。
- 验证：
  - `cargo test -p hone-tools web_search --lib -- --nocapture`
  - `cargo check -p hone-tools --tests`
- 状态调整为 `Fixed`。后续若已部署当前代码后仍出现“全 key 不可用但 `web_search` 被记为成功，并据此输出确定性高风险归因”，应重新打开并优先排查是否还有其它搜索工具路径返回空成功。

## 最新进展（2026-05-01 15:18 CST）

- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把未核验的地缘/宏观归因写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=12509`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-05-01T15:01:09.785404+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`近期变动原因（据 EIA）：2026 年一季度原油及成品油价格大幅攀升，主因 2 月 28 日中东军事行动及后续霍尔木兹海峡实质关闭。`
    - 同条回复继续把 `Brent 涨幅超过 WTI`、`美国页岩油产量韧性有限` 组织成确定性叙事；正文没有把这些高风险原因降级成“待核验/仅供参考”。
  - `data/runtime/logs/web.log.2026-05-01`
    - `2026-05-01 15:01:07.598` 记录同一任务 `parse_kind=JsonTriggered` 并执行 `deliver`，说明这不是中间草稿，而是已外发的最终可见播报。
- 结论：到 `2026-05-01 15:18` 为止，这条质量缺陷继续活跃；最新整点播报仍把未核验的中东军事行动/霍尔木兹海峡叙述包装成确定性油价主因，因此状态维持 `New`、严重等级维持 `P2`。

## 最新进展（2026-05-01 04:01 CST）

- `Oil_Price_Monitor_Closing` 在最近一小时真实窗口再次成功送达，且正文继续把未核验的地缘/宏观归因写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=11974`
    - `job_name=Oil_Price_Monitor_Closing`
    - `executed_at=2026-05-01T04:01:02.613973+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`结论是：油价今天从中东风险高位回落，未形成尾盘继续压制科技股的主线。`
    - 同条回复继续把 `CNBC口径显示，Brent在4月30日盘中曾因美伊冲突风险冲至约126美元高位，随后回落至约114.22美元`、`Trading Economics口径显示，WTI相关原油报价约105.74美元，较前一日回落` 组织成“共同结论一致”的确定性叙事，并据此下结论 `科技股风险偏好恢复`；正文没有把这些原因归因降级成“待核验/仅供参考”。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-01 04:00:11.651-04:00:12.385` 与 `04:00:40.078-04:00:40.796` 同窗连续记录 Tavily 4 个 key 全部因 `usage limit` / `鉴权被拒绝` 失败，但 `web_search` 仍回写 `tool_execute_success`
    - `2026-05-01 04:01:00.018` 同一会话仍落成 `step=session.persist_assistant detail=done -> done ... success=true reply.chars=1302`，说明这不是中间草稿，而是已持久化并成功外发的最终可见回复
- 结论：到 `2026-05-01 04:01` 为止，这条质量缺陷继续活跃；搜索链路在同窗明显降级后，播报正文仍把“中东风险高位回落 -> 科技股风险偏好恢复”包装成确定性结论，因此状态维持 `New`、严重等级维持 `P2`。

## 最新进展（2026-04-30 15:01 CST）

- `全天原油价格3小时播报` 在最近一小时真实窗口再次成功送达，且正文继续把未核验的宏观/地缘归因写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=11311`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-30T15:01:15+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`近期变动主因：中东地缘风险溢价持续消退，OPEC+延续增产节奏，全球原油库存处于季节性累库阶段，需求端关注中美贸易磋商进展及夏季驾驶旺季消费预期。`
    - 同条消息同时写出 `注：因数据链路暂时受限，以上价格为行情参考区间，非交易所实时报价`，但仍把上面的归因作为确定性“主因”输出，没有降级成“原因未核验/仅供参考”。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-30 15:01:12.849-15:01:12.850` 记录同一任务 `parse_kind=JsonTriggered` 与成功 `deliver`，说明这不是中间草稿，而是已外发的最终可见播报。
    - 同窗 `15:00:41.563-15:00:43.326` 连续记录 Tavily `usage limit` / `鉴权被拒绝`，且 `web_search` 最终仍回写 `tool_execute_success`；外部搜索已明显降级，但播报正文没有把原因归因同步降级。
- 结论：这说明 `2026-04-28` 标记为 `Later` 的止血在 `2026-04-30 15:01` 真实窗口再次失效；同一根因已继续 live 复现，应维持 `New` 并回到活跃待修复队列。

## 修复进展（2026-04-28）

- 已在 `crates/hone-channels/src/scheduler.rs` 的 heartbeat prompt 增加“来源归因约束”：
  - 对 Reuters、WSJ、Bloomberg、官方机构或交易所等来源的归因，必须先确认本轮工具结果里确实出现该来源及其当前发布时间。
  - 若当前工具结果只给出二手转述、旧新闻、社媒传闻、模型记忆或无法交叉核验的地缘叙述，只能标注为“未核验/待确认/市场传闻”，不得包装成确定性油价主因。
  - 对原油、宏观、地缘政治、战争、外交谈判、航运和库存等高风险主题，若来源不足，优先报告已核验价格与时间口径，原因归因必须降级。
- 已补 `heartbeat_prompt_requires_source_grounding_for_geopolitics` 单元测试，锁住 prompt 对外部来源归因的约束。
- 状态调整为 `Later`：本轮已把 2026-04-26 的共享金融策略约束进一步补到 heartbeat 调度层；若后续真实原油播报窗口继续把未核验地缘叙述写成 Reuters/WSJ 等确定性事实，再改回 `New`。

## 最新进展（2026-04-28 04:01 CST）

- `Oil_Price_Monitor_Closing` 在最新真实窗口再次成功送达，但正文继续把未核验的地缘叙事写成确定性事实：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=8322`
    - `job_name=Oil_Price_Monitor_Closing`
    - `executed_at=2026-04-28T04:01:13.314715+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`Reuters和WSJ均指向同一条主线：美伊和平谈判停滞，霍尔木兹海峡运输仍受限制，导致市场继续给原油供应风险溢价。`
    - 同条回复还给出 `Brent接近109美元、WTI接近97美元` 并据此推导 `COHR` / `RKLB` 的尾盘防守判断，但没有说明上述地缘因果是否来自本轮可追溯、同时间窗的已核验来源。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-28 04:00:01.107` 记录 scheduler 真实收到了 `Oil_Price_Monitor_Closing` 触发输入。
    - `2026-04-28 04:01:08.855` 记录同一会话 `step=session.persist_assistant detail=done` 与 `done ... success=true elapsed_ms=67733`，说明这不是只出现在中间日志里的草稿，而是已经完成持久化并成功外发的最终可见回复。
  - `data/runtime/logs/acp-events.log`
    - `2026-04-27T20:01:06-20:01:08Z` 连续流出最终 answer chunk，正文仍把 `美伊和平谈判停滞`、`霍尔木兹海峡运输仍受限制` 作为“Reuters 和 WSJ 均指向”的确定性结论输出，并附外链 URL；没有出现“原因未核验/暂不归因”的降级表述。
- 结论：这说明 2026-04-26 标记为 `Later` 的 prompt 级止血没有在真实原油盘后播报里稳定生效；同一根因已在最新真实窗口复现，应从 `Later` 改回 `New` 并重新进入活跃待修复队列。

## 修复进展（2026-04-26）

- 已在 `crates/hone-channels/src/prompt.rs` 的共享金融领域策略中补充“原油与大宗商品归因约束”：
  - 地缘政治、供给、库存、航运、外交谈判、军事行动、OPEC 等原因归因，必须来自本轮工具明确返回的来源、发布时间和可追溯事实。
  - 若搜索/API 降级、来源不足、时间戳缺失或无法交叉核验，只能报告已核验价格与口径，并明确写“原因未核验/暂不归因”。
  - 禁止把传闻、推测或旧上下文中的冲突/谈判/封锁/供应恢复叙述包装成确定性事实。
- 已补 prompt 单元测试，确保默认金融策略持续包含该归因约束。
- 状态调整为 `Later`：这是生成质量约束，当前不再占活跃修复队列；若真实原油播报窗口继续把未核验地缘叙述写成确定性事实，再改回 `New`。

## 修复动作（2026-04-24）

- `crates/hone-channels/src/prompt.rs` `DEFAULT_FINANCE_DOMAIN_POLICY` 末尾追加「报价字段一致性约束」：同一条输出里引用的任何价格数字都必须来自同一合约标的、同一时间点、同一口径；禁止把现货价、不同合约月份（CLJ26 / CLK26 等）、不同时间窗口（现价与日内高点/低点）混写成同一个「现价」。
- 约束同时要求数学一致性：`日内低点 ≤ 最新价 ≤ 日内高点`；写出任何 `较昨收 +/-X%` 时，数值必须由 (最新价 − 昨收) / 昨收 反推可复算。
- 该策略通过 `build_prompt` 注入每次会话 system_prompt，因此既覆盖 `全天原油价格3小时播报` 的 heartbeat，也覆盖其它 scheduler 产出价格字段的任务。
- 价格一致性主要靠 system_prompt 生效，后续巡检监督 `response_preview` 是否仍出现「现价 > 日内高点」式互斥叙述；heartbeat JSON 契约 smoke (`heartbeat_prompt_llm_smoke`) 也同时验证 prompt 注入没有破坏结构化输出。
- **LLM e2e 验证（2026-04-24）**：`crates/hone-channels/examples/finance_consistency_llm_smoke.rs` 直接把 `DEFAULT_FINANCE_DOMAIN_POLICY` 作为 system prompt 喂给生产辅助模型（MiniMax `MiniMax-M2.7-highspeed`），构造两个诱导 case 实跑：
  - `wti_math_inconsistent`：人为注入现价 $62.48 > 日内最高 $61.90 的硬冲突数据。实测模型输出以「⚠️ 数据一致性异常，无法完成播报」开头，用表格逐字段列出矛盾（"最新成交价 $62.48 ❌ 高于日内最高价 $61.90"），并建议用户核实后重传，不输出任何虚构的"现价"数字。
  - `wti_contract_mix`：注入 WTI 连续合约/CLJ26/CLK26/Brent 四组不同口径价格。实测模型在最前加「⚠️ 数据说明：以下价格来自不同合约与数据源」警示，每个数字都显式标注合约代码 + 时间戳（"CLK26 $61.40（纽约时间 10:12 ET）"），而不是把所有价混成一个"现价"。
  - 启发式判据（`SANE_KEYWORDS` 命中 + 两个冲突数字都出现则视为 pass）：两个 case 全部 `pass=true`。这是首次在生产真实模型上实测「报价字段一致性约束」确实生效，不只是 prompt 写得好看。
  - 未做：OpenRouter 侧 `deepseek/deepseek-v4-pro` 对照跑返回 404（模型名在 OpenRouter 不存在），待确认正确的 deepseek 版本后补跑。
- **证据来源**:
  - 2026-04-26 03:00 最新巡检样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=6426`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-26T03:00:41.702388+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 仍向用户发送：`WTI原油：$94.40/桶，较昨收$95.85下跌$1.45（-1.51%）`、`布伦特原油：约$105/桶（4月25日收盘参考价）`，并继续把 `巴基斯坦方面披露美伊或开启第二轮和平谈判`、`市场预期伊朗原油重返全球供应`、`周末前避险情绪升温`、`地缘风险溢价面临收缩压力` 组织成确定性变动原因。
    - 这说明即使 4 月 25 日 21:01 已经确认“价格一致性约束无法约束原因归因”，到 4 月 26 日 03:00 的最新真实送达里，模型仍继续把未经明确来源核验的地缘叙事包装成确定性事实；问题没有自然收敛。
    - `data/runtime/logs/sidecar.log`
    - `2026-04-26 03:00:41.702` 对应 run 仍显示该任务成功送达；`detail_json` 记录 `parse_kind=JsonTriggered`、`starts_with_json=false`，`raw_preview` 先在 `<think>` 中整理 WTI/Brent 数据，再直接输出上述归因，没有给出可追溯来源、发布时间或“搜索降级时只报价格”的降级说明。
    - 同一时间窗 `03:00:11.xxx` 再次出现 Tavily `usage limit` 告警，但播报仍然成功投递完整地缘归因，说明当前不是链路失败，而是质量性错误继续稳定出站。
    - 结论：到 `2026-04-26 03:00` 为止，这条缺陷仍在真实用户可见播报中活跃；状态保持 `Fixing`、严重等级维持 `P2`。
  - 2026-04-25 21:01 最新巡检样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=6290`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-25T21:01:42.373480+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 仍向用户发送：`WTI原油：$95.53/桶，日跌幅-0.33%`、`布伦特原油：$106.10/桶，日涨幅+0.98%`，并把 `霍尔木兹海峡供应担忧缓解`、`美国-伊朗第二轮谈判预期`、`本周整体强势`、`全球库存偏低` 组织成确定性“近期变动原因”。
    - 同条消息明确写的是 `4月24日收盘数据`，但在 4 月 25 日 21:00 的定时播报里仍把跨日期的地缘事件链条和宏观库存判断直接写成已核验事实，说明 4 月 24 日新增的价格一致性约束并没有覆盖“原因归因必须有可追溯证据/无法核验时应降级只报价格”这一质量缺口。
    - `data/runtime/logs/sidecar.log`
    - `2026-04-25 21:01:42` 对应 run 仍显示该任务成功送达；同一小时 `21:31` 的下一轮未命中窗口时，日志又继续暴露 `<think>...</think>\n\n{"status":"noop"}` 形态，说明 heartbeat 通道本身仍依赖从自由文本尾部抽取状态，不能证明播报正文曾经过更严格的结构化事实校验。
    - 结论：到 `2026-04-25 21:01` 为止，这条缺陷仍在真实用户可见播报中活跃；现有修复只能约束“价格字段别自相矛盾”，还不能阻止模型把未充分核验的地缘因子写成确定性原因，因此状态保持 `Fixing`、严重等级维持 `P2`。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5602`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-24T15:01:39.275252+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$96.55/桶，日涨幅+0.73%`、`布伦特原油：$106.18/桶，日涨幅+1.06%`，并把 `中东局势持续紧张`、`美伊围绕霍尔木兹海峡的对峙升级`、`该战略通道保持关闭状态`、`以色列-黎巴嫩停火协议延长3周` 写成近期波动原因。
    - 这说明 12:01 的“价格字段自相矛盾”之后，15:00 整点播报仍继续把未经充分核验的地缘链条组织成确定性归因；问题没有因为上一轮时间窗或价格修正而自然消失。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-24 15:01:35.368-15:01:35.369` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`raw_preview` 明确先列出价格，再把 `霍尔木兹海峡关闭`、`市场担忧伊朗战争持续` 等叙述直接拼进触发正文。
    - 同批 `15:01:19-15:01:32` 多次记录 `tavily request failed ... usage limit`，说明外部搜索链路仍有配额降级噪声，但最终仍生成并投递了确定性地缘结论。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5537`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-24T12:01:51.692857+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 向用户发送：`WTI 原油：$98.32/桶，较昨收+2.27%（+$2.18），日内高点 $84.13`、`布伦特原油：~$106.89/桶...`，同一条播报内部就出现“现价高于日内高点”的自相矛盾数值；随后又把 `OPEC+ 延续减产约束`、`霍尔木兹海峡相关地缘紧张局势`、`StoneX Q2 展望`、`中美贸易摩擦` 与 `近月升水走阔` 组织成确定性主因。
    - 这说明问题已经不只是“未经充分核验的地缘叙述”，还包括同一条已送达播报内部的价格字段互相打架；当前链路会把不同来源、不同合约月份和不同时间口径的数值混写成一个看似精确的用户消息。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-24 12:01:49.827-12:01:49.828` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`raw_preview` 明确先把 `WTI Crude Oil: ~$98.32/barrel (Apr 2026 front month, CLJ26)` 与 `WTI Financial Futures from CME: May 2026 = $86.20, Jun 2026 = $83.08` 混在同一次总结里，再组织成最终出站正文。
    - 这与 sqlite 里的 `日内高点 $84.13` 相互印证：生成链路在价格字段层面已经发生来源/合约口径串线，而不是单纯措辞夸张。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5394`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-24T06:00:59.865125+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：95.85美元/桶，较昨收+3.11%`、`布伦特原油：105.33美元/桶`，并把 `美伊冲突持续升级`、`特朗普向伊朗设定最后期限要求开放霍尔木兹海峡`、`摩根大通警告若供应持续中断油价可能升破150美元` 写成近期涨幅原因。
    - 这轮发生在 04:31 的 `OWALERT_PostMarket` 之后，说明问题没有停留在一轮盘后总结里，而是在 06:00 又回到原油专用 heartbeat 继续稳定送达同一类未经充分核验的地缘叙事。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-24 06:00:57.103-06:00:57.104` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`deliver_preview` 与 sqlite 中的送达正文一致。
    - 同批 `06:00:13-06:00:46` 多次记录 `tavily request failed ... usage limit`，说明外部搜索链路继续带配额降级噪声，但最终仍把确定性地缘结论组织进用户播报。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5360`
    - `job_name=OWALERT_PostMarket`
    - `executed_at=2026-04-24T04:31:29.138158+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 继续向用户发送：`油价继续上冲，使高估值和亏损股的尾盘承压更明显`，并把 `USO收在134.73美元附近，单日涨约4.12%；WTI约95.7美元，Brent约102.6至103.4美元` 作为“今天盘后的宏观主线”写入盘后复盘。
    - 这说明问题已经不只限于原油专用播报，04:00 的未经充分核验油价叙事在 04:30 的盘后复盘里被继续复用和放大，影响面扩散到综合市场总结。
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
    - `2026-04-24T04:30:00.219160+08:00` 用户触发：`[定时任务触发] 任务名称：OWALERT_PostMarket`
    - `2026-04-24T04:31:26.850681+08:00` assistant 最终回复写明：`与此同时，油价继续上冲，使高估值和亏损股的尾盘承压更明显`，并把 `USO收在134.73美元附近...WTI约95.7美元，Brent约102.6至103.4美元` 作为当天最重要变化之一；真实会话中已把原油结论扩散到盘后总复盘。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-24 04:30:17-04:31:26` 同一 `OWALERT_PostMarket` 会话继续执行 `skill_tool`、`data_fetch` 与 `web_search` 共 15 次工具调用后成功送达，说明这不是“工具没跑完”的误发，而是综合复盘链路真的吸收了同一批原油叙事并作为宏观主线输出。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5348`
    - `job_name=Oil_Price_Monitor_Closing`
    - `executed_at=2026-04-24T04:01:08.155295+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 直接向用户发送：`油价今天不是“高位横盘”，而是重新加速上冲；USO已到134.73美元，WTI约95.7美元，Brent约102.6至103.4美元。对今晚科技股，尤其COHR、RKLB这类高估值/高beta标的，油价已从“估值上限约束”升级为“尾盘需要提高防守敏感度”的级别。`
    - 这轮虽然没有把完整 `<think>` 预览落进 `cron_job_runs.detail_json`，但最终正文仍把高风险价格口径与宏观判断写成确定性结论，没有给出可追溯来源、发布时间、置信度或“若搜索降级则只报价格”的降级说明。
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
    - `2026-04-24T04:00:00.200961+08:00` 用户触发：`[定时任务触发] 任务名称：Oil_Price_Monitor_Closing`
    - `2026-04-24T04:01:04.764772+08:00` assistant 最终回复与 `run_id=5348` 的送达预览一致，说明这不是单纯 cron 台账异常，而是真实会话中已落库、可见的播报正文。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5325`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-24T03:00:48.850550+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油价格 $96.67/桶，日内涨幅 +7.81%`、`布伦特原油价格 $105.67/桶，涨幅 +3.69%`，并把 `2026年4月初以色列与伊朗冲突升级导致油价暴涨至近 $120/桶`、`OPEC+虽然同意增产，但警告复苏缓慢` 等高风险叙述写成近期价格大幅波动的主要原因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，先写“根据搜索结果，原油价格在2026年4月大幅上涨，主要原因是：1. 以色列与伊朗冲突升级 2. 原油价格达到四年高点，接近$120/桶 3. OPEC+虽然同意增产，但警告复苏缓慢”，随后直接组织成 `JsonTriggered` 出站正文；同一轮仍没有可追溯新闻来源、发布时间、来源可信度或交叉验证结果。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-24 03:00:46.667-03:00:46.669` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`deliver_preview` 与 sqlite 中的送达正文一致。
    - `2026-04-24 03:00:24.435` 与 `03:00:35.760-03:00:36.151` 同批再次记录 `tavily request failed ... usage limit`，说明外部检索仍带额度降级噪声，但最终仍生成并投递了确定性地缘结论。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5108`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T18:00:50.267119+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$94.43/桶`、`布伦特原油：$103.12/桶`，并把 `霍尔木兹海峡自2月底以来持续受阻（美伊冲突升级）`、`主要产油国出口均受影响`、`摩根大通警告若海峡受阻持续至5月中旬油价可能突破$150`、`特朗普给伊朗设定开放海峡最后期限` 写成近期变动主因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，先承认自己在整理“最新原油价格和近期变动原因”，随后直接把上述地缘叙述组织成 `JsonTriggered` 出站正文；同一轮没有提供可追溯新闻来源、发布时间、来源可信度或交叉验证结果。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-23 18:00:47.665-18:00:47.666` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`deliver_preview` 与 sqlite 中的送达正文一致。
    - `2026-04-23 18:00:48.344` 与 `18:00:48.601` 同批再次记录 `tavily request failed ... usage limit`，说明外部检索仍带额度降级噪声，但最终仍生成并投递了确定性地缘结论。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4975`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T12:00:26.652887+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 继续向用户发送：`WTI原油：约$99-101/桶（4月22日收盘区间）`、`布伦特原油：$101.91/桶（4月22日收盘，较前日上涨超3%，连续第四日上涨）`，并把 `美国燃料库存意外下降`、`全球航运路线地缘政治风险升温`、`美国与伊朗外交谈判进展不明` 和 `分析师预计布伦特Q2峰值或达$115/桶` 写成近期上涨原因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，承认 `没有返回精确的实时价格`，但随后仍把搜索片段拼成 `JsonTriggered` 并送达；同一轮没有提供可追溯来源、发布时间、来源可信度或交叉验证结果。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4904`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T09:01:33.486116+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$93.20/桶，日内涨幅 +$3.53 (+3.94%)`、`布伦特原油：$102.27/桶`，并把 `伊朗停火协议扩展不确定性升温`、`市场担忧中东地缘政治风险及供应扰动` 写成近期变动原因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部只把价格数据与“伊朗停火协议扩展不确定性”拼接后直接构造 `JsonTriggered`；同一轮没有提供可追溯来源、发布时间、来源可信度或交叉验证结果。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4828`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T06:00:47.640940+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 继续向用户发送：`WTI原油：约$92.60/桶`、`布伦特原油：约$99-101/桶区间`，并把 `4月13日美国宣布封锁伊朗港口后油价飙涨`、`4月17日伊朗外长宣布霍尔木兹海峡恢复商业航运后单日暴跌逾10%`、`市场正关注美伊局势后续走向及谈判进展` 写成近期波动主因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部列出 `美伊战争导致霍尔木兹海峡关闭/动荡` 等原因后解析为 `JsonTriggered`；同一轮仍没有给出可追溯来源、发布时间、来源可信度或交叉验证结果。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-23 06:00:45.147` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`deliver_preview` 与 sqlite 中的送达正文一致；同批 `06:00:10-06:00:25` 多次记录 `tavily request failed ... usage limit`，说明外部检索仍有额度降级迹象。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4759`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T03:00:39.918824+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$92.99 | 日涨幅 +$3.32 (+3.70%)`、`布伦特原油：$102.02 | 日涨幅 +$3.54 (+3.59%)`，并把 `美伊冲突持续升级`、`霍尔木兹海峡实际关闭`、`特朗普表示美伊冲突可能持续数周`、`WTI期货4月初曾单日飙升超12%`、`市场库存缓冲正在被消耗` 写成近期价格大涨主因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，只列出价格与宏观叙述后直接组织 `JsonTriggered` 待投递正文；同一轮未提供可追溯新闻来源、发布时间、来源可信度或交叉验证结果。
  - `data/runtime/logs/web.log`
    - `2026-04-23 03:00:34-03:00:36` 同批多次记录 `tavily request failed ... usage limit`，说明外部搜索链路仍有额度降级；`2026-04-23 03:00:37.236` 随后记录同一原油任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，并把上述地缘/库存叙述纳入 `deliver_preview`。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4691`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T00:00:36.565619+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$92.13/桶（4月22日收盘）`、`布伦特原油：$100.58/桶（4月22日收盘）`，并把 `霍尔木兹海峡局势骤然升温`、`至少三艘集装箱船遭到枪击`、`美国扣押伊朗船只`、`伊朗称其为"战争行为"`、`美伊脆弱的两周停火协议即将于4月23日到期` 写成近期大涨原因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，并在没有可追溯新闻来源、发布时间、来源可信度或交叉验证结果的情况下，将上述地缘叙述直接组织为 `JsonTriggered` 待投递正文。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-23 00:00:09-00:00:31` 同批多次记录 `tavily request failed ... usage limit`，说明外部搜索链路仍有额度降级；`2026-04-23 00:00:34.429` 随后记录同一原油任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，并把上述地缘事件链纳入 `raw_preview`。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4622`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T21:00:41.235036+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$88.82/桶（日内约+1.25%）`、`布伦特原油：$93.59/桶（今日约+0.94%）`，并把 `OPEC+持续减产预期`、`市场预期 OPEC 及盟友将把创纪录减产延长至7月`、`区域性冲突持续扰动供应链` 和 `IEA 警告2026年可能出现每日400万桶以上的供应过剩` 写成近期价格变动原因。
    - 这一轮仍没有给出可追溯新闻来源、发布时间或置信度说明；正文虽附 `价格可能存在延迟`，但原因归因仍是确定性宏观叙事。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4562`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T18:01:40.568312+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$88.73/桶，较前一交易日下跌1.05%，月涨0.68%，年涨42.49%。布伦特原油（6月期货）：$98.57/桶`，并把 `全球原油产量下降`、`地缘政治紧张局势持续`、`伊朗停火协议未能有效安抚市场` 写成近期主要驱动因素。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部只列出价格、全球加工量预期和地缘叙述，随后直接生成带宏观归因的 `JsonTriggered` 消息；这一轮仍没有给出可追溯新闻来源、时间口径或置信度说明。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4498`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T15:01:28.307896+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$89.26/桶，日跌幅-0.45%；布伦特原油：$98.35/桶，日跌幅-0.14%`，并把 `近期价格波动主要受地缘政治紧张局势及市场需求预期变化影响` 作为确定性归因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部只列出两条价格数据，随后直接生成带宏观归因的 `JsonTriggered` 消息；这一轮没有附带可追溯新闻来源或置信度说明。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4440`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T12:00:29.317390+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`Trump于4月2日就伊朗问题发表讲话后布伦特一度飙升近8%至$109新高`、`伊朗停火谈判进展令供应忧虑缓解`、`OPEC+减产预期与地缘风险持续支撑价格底部`。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，把 WTI/Brent 区间价和地缘叙述拼成待投递正文，再被解析为 `JsonTriggered`；说明 09:01 之后该任务仍在同一根因下继续投递未经充分核验的地缘叙述。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4397`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T09:01:42.878090+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 已再次向用户发送：`Trump于4月2日就伊朗问题发表讲话后布伦特飙升近8%至$109新高`、`OPEC+减产预期与地缘风险持续支撑价格`、`摩根大通预测2026年布伦特均价约$60（偏空）`。
    - `detail_json.scheduler.raw_preview` 显示模型在内部把 WTI/Brent 价格和地缘叙述混在同一段 `<think>` 自由文本中，再解析为 `JsonTriggered` 并送达；与 06:02 样本同根因，说明市场播报仍缺少来源质量与置信度门禁。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4326`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T06:02:01.143394+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 已向用户发送：`美伊战争于2月28日爆发`、`4月17日伊朗宣布霍尔木兹海峡重新开放，油价暴跌逾11%`、`4月19日美伊海上对峙升级`、`美伊停火协议即将于4月22日到期`
    - `detail_json.scheduler.raw_preview` 显示模型在内部将这些高风险叙述当作“关键事件”和“背景”组织，随后解析为 `JsonTriggered` 并送达。
  - `data/runtime/logs/web.log`
    - `2026-04-22 06:01:58.639` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`raw_preview` 以 `<think>` 开头并把上述地缘叙述纳入触发正文。
    - 同一小时 `06:01:14-06:01:44` 多次记录 `tavily request failed ... usage limit`，说明本轮外部检索质量本身存在降级迹象，但最终仍生成并投递了确定性地缘结论。
  - 对照同一任务前序输出：
    - `run_id=4263`（`2026-04-22T03:00:52.079347+08:00`）也曾围绕“美伊核谈判进展”“地缘紧张局势”解释原油价格，但 06:02 样本进一步升级为更具体的战争、海峡开放、停火到期等确定性事件串，并已投递。

## 端到端链路

Feishu scheduler / heartbeat 触发 `Oil_Price_Monitor_Closing`、`全天原油价格3小时播报` 与 `OWALERT_PostMarket` -> runner 查询原油价格与近期原因 -> web search 多个 key 触发额度失败告警或来源质量不足 -> 模型仍把未核验的地缘冲突叙述和交易导向判断组织成“油价上冲”结论 -> scheduler 以 `completed + sent` 送达用户，并进一步在综合盘后复盘里复用。

## 期望效果

原油播报应只发送已核验的价格、时间口径与可追溯的近期驱动因素。外部检索降级或来源不足时，应明确降低置信度、避免编造成确定性地缘事件链，必要时只播报价格并说明“原因需等待更多可验证来源”。

## 当前实现效果

系统在外部搜索出现额度失败告警或缺少来源门禁的同一轮，仍将高风险地缘叙述当作事实发送给用户，并把它们作为油价变化的主因。09:01 和 12:00 复核显示，即使输出不再复述 06:02 的完整战争/停火叙事，也仍继续把单一政治讲话、布伦特冲高、OPEC+ 与地缘风险包装成确定性驱动并送达。15:01 复核进一步显示，即便正文缩短到价格播报，模型仍会在没有可追溯来源的情况下把“地缘政治紧张局势及市场需求预期变化”写成确定性原因。18:01 复核又把“全球产量下降”“伊朗停火协议未能安抚市场”等宏观叙事写成近期驱动因素并送达。21:00 复核继续把 OPEC+ 延长创纪录减产、区域冲突扰动供应链和 IEA 2026 供给过剩预测作为确定性原因送达。2026-04-23 00:00 复核又把霍尔木兹枪击、美国扣押伊朗船只和美伊停火到期写成确定性近期大涨原因；03:00 复核继续把美伊冲突升级、霍尔木兹实际关闭、WTI 单日飙升超 12% 等未核验叙述写成价格大涨主因；06:00 复核再次把美国封锁伊朗港口、霍尔木兹恢复商业航运和美伊局势当作价格波动主因送达；09:01 复核又把“伊朗停火协议扩展不确定性”和“中东供应扰动”写成涨价原因；12:00 复核在承认没有精确实时价格的情况下，仍把 WTI 约 99-101 美元、布伦特 101.91 美元和航运/伊朗谈判风险作为确定性播报送达；18:00 复核又把“霍尔木兹海峡持续受阻”“主要产油国出口受影响”“油价可能突破150美元”“特朗普设定海峡开放最后期限”写成近期主因并成功送达；2026-04-24 03:00 复核再次把“以色列与伊朗冲突升级导致油价暴涨至近 $120/桶”“OPEC+ 虽同意增产但警告复苏缓慢”等叙述写成近期主要原因并成功送达；04:01 的 `Oil_Price_Monitor_Closing` 又进一步把“油价重新加速上冲、需提高尾盘防守”写成面向操作的确定性结论；04:31 的 `OWALERT_PostMarket` 则把同一批油价数字和“油价继续上冲”判断升级成盘后复盘的宏观主线；06:00 的 `全天原油价格3小时播报` 再次把“美伊冲突升级 / 特朗普最后期限 / 霍尔木兹风险”写成近期涨幅主因并成功送达；12:00 的最新播报更进一步把不同合约月份与来源口径混成同一条用户消息，出现 `WTI $98.32/桶` 却同时声称 `日内高点 $84.13` 的内部矛盾。该问题已经在原油专用 heartbeat 与综合市场总结之间来回扩散，不再只是单一任务的措辞偏差，而是连价格字段本身都可能失真。

## 用户影响

用户会收到已经送达的市场播报，并可能据此判断原油、通胀、成长股和组合风险。现在连综合盘后复盘也开始复用同一未经充分核验的油价叙事，用户可能把它当作“今日最重要变化”来调整次日计划。由于这是投资相关定时提醒的事实正确性问题，影响的是用户对市场变量的判断，而不仅是表达风格或格式质量，因此按业务正确性缺陷定级为 P2。

## 根因判断

初步判断是 heartbeat 内容生成链路缺少事实置信度和来源质量门禁：外部检索出现配额失败时，模型仍可沿用历史上下文、旧摘要或低质量搜索片段生成确定性事件叙述；调度器只按 `JsonTriggered` 发送，不校验事实来源、时间口径和任务主题是否足够匹配。

## 下一步建议

- 为市场播报类 heartbeat 增加来源质量检查：搜索配额失败或来源不足时，不允许输出确定性宏观事件链。
- 原油价格任务应把“价格数据”和“原因解释”拆成结构化字段，并为原因附来源时间；缺少来源时降级为低置信度说明。
- 检查历史 compact summary 是否把“美伊战争 / 霍尔木兹”叙述长期带入市场类 prompt，避免旧上下文污染后续定时播报。
