# Bug: Heartbeat 已触发提醒偶发向用户投递原始 JSON 载荷

- **发现时间**: 2026-04-18 11:06 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New

## 最新进展

- `2026-07-17 23:00-2026-07-18 03:00 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-17`
    - 同窗有 55 条 heartbeat `deliver_preview`、196 条 raw `<think>` 与 25 条 fenced JSON 信号。
    - 2026-07-18 00:00 CST 前后，`AI与科技持仓观察关键事件心跳提醒` 的用户可见 preview 仍以 fenced JSON 开头，包含 `status`、`event`、`BE`、`STX`、`LITE`、`AAOI`、`TSLA` 等结构化字段和行情项，而不是产品化自然语言提醒。
    - 多条 heartbeat raw preview 仍以 `<think>` 后接 fenced JSON 或裸协议状态收口，例如 `JsonNoop` / `PlainTextTriggered` 路径继续依赖解析器从自由文本尾部提取状态。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 13 条 user / 12 条 assistant，未确认 JSON 载荷进入 ordinary direct assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-17 15:01-19:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-17`
    - 同窗有 49 条 heartbeat `deliver_preview`，其中 15:30 CST `小米30港元破位预警`、16:00 CST `AI与科技持仓观察关键事件心跳提醒`、17:00 CST `小米30港元破位预警`、18:00 CST `ORCL 大事件监控`、18:30 CST `RKLB异动监控`、19:00 CST `小米30港元破位预警` 的用户可见 preview 仍以 fenced JSON 开头。
    - 这些 preview 包含 `status`、`triggered`、`symbol`、`condition`、`current_price` / `price`、`prev_close`、`change_pct`、`volume` 等协议字段，而不是产品化自然语言提醒。
    - 同窗多条 heartbeat raw preview 仍以 `<think>` 加 fenced JSON 或裸协议状态收口，说明模型输出协议未稳定收敛到用户态正文。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 8 条 user / 9 条 assistant，未确认 JSON 载荷进入 ordinary direct assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-17 11:01-15:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-17`
    - 同窗有 67 条 heartbeat `deliver_preview`，`parse_kind` 分布包含 `PlainTextTriggered=134`、`JsonTriggered=5`、`JsonNoop=105`。
    - 15:00 CST `光模块板块关键事件心跳提醒`、`SIVE POET/Nokia/1.6T DFB 心跳检测`、`Cerebras IPO与业务进展心跳监控`、`RKLB异动监控`、`持仓重大事件心跳检测`、`中际旭创关键事件心跳提醒` 等多条 heartbeat 以 `PlainTextTriggered` deliver，自然语言正文里仍混有 `noop` 状态、结构化字段或协议化标题，而 raw preview 普遍以 `<think>` 开头。
    - 同窗 `JsonTriggered` / `JsonNoop` raw preview 继续出现 fenced JSON 或裸协议状态，说明模型输出协议仍未稳定收敛到用户态正文。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 16 条 user / 16 条 assistant，未确认 JSON 载荷进入普通 direct assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-17 07:01-11:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-16` / `web.log.2026-07-17`
    - 09:01 CST `AI与科技持仓观察关键事件心跳提醒` `deliver_preview` 继续以 fenced JSON 开头，包含 `status`、`event`、`data_time` 等协议字段。
    - 09:30 CST `RKLB异动监控` `deliver_preview` 以 fenced JSON 开头，包含 `triggered`、`symbol`、`condition`、`price`、`prev_close`、`change_pct`、`volume` 等协议字段。
    - 11:00 CST `FOTO 光子学ETF心跳检测` `deliver_preview` 在自然语言标题后继续拼入 fenced JSON，包含 `status`、`triggered`、`symbol`、`condition`、`price` 等协议字段。
    - 同窗多条 heartbeat raw preview 仍以 `<think>` 加 fenced JSON 或裸协议状态收口，说明模型输出协议未稳定收敛到用户态正文。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 10 条 user / 10 条 assistant，未确认 JSON 载荷进入普通 direct assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-17 03:01-07:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-16`
    - 同窗仍有 42 条 heartbeat `deliver_preview`，其中 06:00-07:00 CST RKLB、FOTO、CBRS 等触发提醒继续以 fenced JSON 开头。
    - 代表样本包括 06:00 CST `RKLB异动监控`、07:00 CST `FOTO 光子学ETF心跳检测`、07:00 CST `Cerebras IPO与业务进展心跳监控`，用户可见 preview 继续包含 `status`、`triggered`、`symbol`、`condition`、`price`、`prev_close`、`change_pct` 等协议字段。
    - 同窗多条 heartbeat raw preview 仍以 `<think>` 加 fenced JSON 或裸协议状态收口，说明模型输出协议未稳定收敛到用户态正文。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 5 条 user / 6 条 assistant，未确认 JSON 载荷进入 direct / 普通 scheduler assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-16 23:01-2026-07-17 03:03 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-16`
    - 同窗仍有 20 条 heartbeat `deliver_preview` 以 fenced JSON 开头。
    - 代表样本包括 00:30 CST `小米30港元破位预警`、00:30 / 01:00 / 01:30 CST `AAOI 1.6T 光模块心跳检测`、00:30 CST `RKLB异动监控`、01:00 CST `持仓重大事件心跳检测`、01:01 / 02:00 CST `存储板块关键事件心跳提醒`、02:00 CST `FOTO 光子学ETF心跳检测` 等，用户可见 preview 继续包含 `status`、`triggered`、`symbol`、`condition`、`price`、`prev_close`、`change_pct` 等协议字段。
    - 23:30-03:00 CST 多条 heartbeat raw preview 仍以 `<think>` 加 fenced JSON 或裸 JSON 收口，说明模型输出协议未稳定收敛到用户态正文。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 5 条 user / 5 条 assistant，未确认 JSON 载荷进入 direct / 普通 scheduler assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-16 19:02-23:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-16`
    - 20:00 / 21:00 CST `小米30港元破位预警` 的 `deliver_preview` 继续以 fenced JSON 开头，包含 `"status": "triggered"`、`"triggered"`、`"symbol": "1810.HK"`、`"condition"`、`"current_price"`、`"previous_close"`、`"change_pct"` 等结构化协议字段。
    - 19:30-23:00 CST 多条 heartbeat raw preview 仍以 `<think>` 加 fenced JSON 或裸 JSON 收口，例如 NBIS / NVDA / AAOI / 光模块 / 存储板块 heartbeat 以 `JsonNoop` 或 `JsonTriggered` 分类但 raw 内容仍是模型中间协议。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 29 条 user / 29 条 assistant，未确认 JSON 载荷进入 direct / 普通 scheduler assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-16 15:03-19:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-16`
    - 16:30 / 17:00 / 18:00 CST `小米30港元破位预警` 的 `deliver_preview` 多次以 fenced JSON 开头，包含 `"status": "triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price"`、`"previous_close"`、`"change_pct"` 等结构化协议字段。
    - 17:00 CST `Monitor_Watchlist_11` 的 `deliver_preview` 以 fenced JSON 开头，包含 `"triggered"`、`"ticker":"ASTS"`、`"current_price"`、`"trigger_price"`、`"logic"` 等结构化字段。
    - 17:30 / 19:00 CST 多条 heartbeat raw preview 仍以 `<think>` 加 fenced JSON 或裸 JSON 收口，说明模型输出协议未稳定收敛到用户态正文。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 6 条 user / 6 条 assistant，未确认 JSON 载荷进入 direct / 普通 scheduler assistant final；未见错投、全渠道不可用或数据安全证据。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有主功能链路阻断，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-16 07:02-11:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-16`
    - 同窗至少 9 条 heartbeat `deliver_preview` 以 fenced JSON 开头，说明用户可见提醒仍可能收到原始 JSON 载荷或 JSON 残片，而不是产品化自然语言提醒。
    - 08:00 / 09:00 / 09:30 / 10:00 / 10:30 / 11:00 CST `小米30港元破位预警` 的 deliver preview 多次以 fenced JSON 开头，包含 `"status": "triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price"` 等字段。
    - 08:30 / 10:30 CST `Monitor_Watchlist_11` 的 deliver preview 以 fenced JSON 开头，包含 `"triggered"`、`"ticker":"ASTS"`、`"current_price"`、`"trigger_price"`、`"logic"` 等结构化字段。
    - 10:31 CST `AI与科技持仓观察关键事件心跳提醒` 的 deliver preview 以 fenced JSON 开头，包含 `"status": "triggered"`、`"event"`、`"data_time"` 等协议字段。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 5 条 user / 5 条 assistant，未确认 JSON 载荷进入 direct / 普通 scheduler assistant final。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有错投、全渠道不可用或数据安全证据，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-16 03:02-07:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 同窗至少 5 条 heartbeat `deliver_preview` 以 fenced JSON 或协议 JSON 开头，说明用户可见提醒仍可能收到原始 JSON 载荷或 JSON 残片，而不是产品化自然语言提醒。
    - 03:30 CST `小米30港元破位预警` 的 deliver preview 以 fenced JSON 开头，包含 `"status": "triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"` 等字段。
    - 04:00 / 04:30 / 07:00 CST `Monitor_Watchlist_11` 的 deliver preview 多次以 fenced JSON 开头，包含 `"triggered"`、`"ticker":"ASTS"`、`"current_price"`、`"trigger_price"`、`"logic"` 等结构化字段。
    - 03:31 CST `AI与科技持仓观察关键事件心跳提醒` 的 deliver preview 以 fenced JSON 开头，包含 `"status": "triggered"`、`"event"`、`"data_time"` 等协议字段。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 10 条 user / 11 条 assistant，未确认 JSON 载荷进入 direct / 普通 scheduler assistant final。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有错投、全渠道不可用或数据安全证据，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-15 23:02-2026-07-16 03:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 同窗至少 11 条 `HeartbeatDiag deliver` 的 `deliver_preview` 仍以 fenced JSON 开头，说明用户可见提醒仍可能收到原始 JSON 载荷或 JSON 残片，而不是产品化自然语言提醒。
    - 23:30 CST `小米30港元破位预警` 的 deliver preview 以 fenced JSON 开头，包含 `"status": "triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"` 等字段。
    - 03:00 CST `AI与科技持仓观察关键事件心跳提醒` 的 deliver preview 也以 fenced JSON 开头，包含 `"triggered_tickers": ["AAOI", "DELL"]` 等结构化字段。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 6 条 user / 6 条 assistant，覆盖 3 个 session，均以 assistant 收口；未确认 JSON 载荷进入 direct / 普通 scheduler assistant final。
  - 判断：这些样本说明缺陷仍是 heartbeat 出站格式化退化；当前没有错投、全渠道不可用或数据安全证据，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-15 19:01-23:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 同窗 heartbeat `deliver_preview` 以 fenced JSON 开头命中 5 次。
    - 19:00 CST `小米30港元破位预警` `deliver_preview` 继续以 fenced JSON 开头，包含 `"status": "triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price"` 等结构化字段。
    - 23:00 CST `全天原油价格3小时播报` `deliver_preview` 以 fenced JSON 开头，包含 `"status": "triggered"`、`"triggered"`、`"alert_type": "crude_oil_price_broadcast"`、`"timestamp_beijing"`、`"wti"` 等协议字段。
  - 会话质量对照：同窗 `data/sessions.sqlite3` 新增 48 条 user / 55 条 assistant，近期 28 个 session 均以 assistant 收口；未确认 JSON 载荷进入 direct / 普通 scheduler assistant final。
  - 判断：最新样本仍是 heartbeat 出站格式化退化；当前没有错投、全渠道不可用或数据安全证据，主要影响提醒格式质量，因此维持质量性 `P3 / New`，非 P1。

- `2026-07-15 15:02-19:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 18:30 CST `Monitor_Watchlist_11`
      - `job_id=j_ab7e8fb1`
      - `target=+8618668080998`
      - `parse_kind=PlainTextTriggered`
      - `deliver_preview` 以 fenced JSON 开头，包含 `"triggered"`、`"ticker":"ASTS"`、`"current_price":68.82`、`"trigger_price":69.83`、`"logic"` 等结构化协议字段。
    - 19:00 CST `小米30港元破位预警`
      - `job_id=j_654aef9b`
      - `target=+8613871396421`
      - `parse_kind=PlainTextTriggered`
      - `deliver_preview` 继续以 fenced JSON 开头，包含 `"status": "triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price": 25.86` 等结构化字段。
  - 会话质量对照：
    - 同窗 `data/sessions.sqlite3` 按真实 `timestamp` 有 8 条 user / 9 条 assistant；assistant final 污染扫描未命中 `<think>`、本机路径、原始工具字段、`company_profiles/` 或 panic。
  - 判断：
    - 最新样本仍是 heartbeat 出站格式化退化；不是新的独立根因。
    - 当前没有错投、全渠道不可用或数据安全证据；主要伤害是出站预览和潜在用户可见提醒的结构 / 格式质量，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- `2026-07-15 11:01-15:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 11:00 / 14:30 CST `小米30港元破位预警`
      - `job_id=j_654aef9b`
      - `target=+8613871396421`
      - `parse_kind=PlainTextTriggered`
      - `deliver_preview` 继续以 fenced JSON 开头，包含 `"status": "triggered"`、`"triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price"`、`"currency"`、`"previous_close"`、`"change_pct"` 等结构化协议字段。
    - 11:00-14:00 CST 同一 job 多次又被 `安全执行器不可用` runner guard 拒绝，说明该格式退化与 runner guard 是并行问题；本单只记录已进入 deliver preview 的 JSON 载荷外泄。
  - 会话质量对照：
    - 同窗 `data/sessions.sqlite3` 没有新的真实 `timestamp` assistant final；未确认 JSON 载荷进入 direct 会话。
  - 判断：
    - 最新样本仍是 heartbeat 出站格式化退化；不是新的独立根因。
    - 当前没有错投、全渠道不可用或数据安全证据；主要伤害是出站预览和潜在用户可见提醒的结构 / 格式质量，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- `2026-07-15 07:04-11:02 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 08:00 / 08:30 / 10:00 / 10:30 / 11:00 CST `小米30港元破位预警`
      - `job_id=j_654aef9b`
      - `target=+8613871396421`
      - `parse_kind=PlainTextTriggered`
      - `deliver_preview` 多次以 fenced JSON 开头，包含 `"status": "triggered"`、`"triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price"`、`"currency"`、`"previous_close"` 等结构化协议字段。
    - 这些样本横跨 08:00、08:30、10:00、10:30、11:00 五个窗口，说明该格式退化不是单次偶发。
  - 会话质量对照：
    - 同窗 `data/sessions.sqlite3` 有 29 个 user turn / 29 条 assistant 记录，19 个近期 session 均以 assistant 收口；未见 JSON 或 fenced block 污染进入 direct / 普通 scheduler assistant final。
  - 判断：
    - 最新样本仍是 heartbeat 出站格式化退化；不是新的独立根因。
    - 当前没有错投、全渠道不可用或数据安全证据；主要伤害是出站预览和潜在用户可见提醒的结构 / 格式质量，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- `2026-07-13 11:04-15:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-13`
    - 12:00 / 12:30 / 13:30 / 14:00 / 14:30 / 15:00 CST `小米30港元破位预警` 多次生成以 fenced JSON 开头的 `deliver_preview`，包含 `"status": "triggered"`、`"triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price"` 等结构化协议字段。
    - 15:00 CST `全天原油价格3小时播报` `parse_kind=JsonTriggered`，自然语言价格播报后继续拼入 `",\n      "attribution_...` 结构化字段残片。
  - 会话质量对照：
    - 同窗 `data/sessions.sqlite3` 有 3 个 user turn / 3 条 assistant final，未见 JSON 或 fenced block 污染进入 direct / scheduler assistant final。
  - 判断：
    - 最新样本仍是 heartbeat 出站格式化退化；不是新的独立根因。
    - 当前没有错投、全渠道不可用或数据安全证据；主要伤害是出站预览和潜在用户可见提醒的结构 / 格式质量，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- `2026-07-13 07:00-11:01 CST` 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-13`
    - 10:30 CST `小米30港元破位预警`
      - `job_id=j_654aef9b`
      - `target=+8613871396421`
      - `parse_kind=PlainTextTriggered`
      - `deliver_preview` 以 fenced JSON 开头，包含 `"status": "triggered"`、`"triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"`、`"current_price": 26.48` 等结构化协议字段。
    - 11:00 CST 同一 `小米30港元破位预警` 再次生成 fenced JSON `deliver_preview`，本轮 `current_price` 变为 `26.06`，说明该格式退化不是单次偶发。
    - 11:00 CST `全天原油价格3小时播报` `deliver_preview` 也以 fenced JSON 开头，包含 `"status": "triggered"`、`"北京当前时间": "2026-07-13 15:18"`、`"triggered"`、`"symbol": "WTI"` 等结构化字段。
  - 会话质量对照：
    - `data/sessions.sqlite3` 在 07:00-10:30 CST 有 27 个 user turn / 27 条 assistant final，均成对收口；assistant final 污染扫描未命中空回复、`<think>`、本机路径、provider 原始错误或结构化 JSON 外泄。
  - 判断：
    - 该样本仍是 heartbeat 用户可见提醒格式化退化的同一链路；不是新的独立根因。
    - 当前没有错投、全渠道不可用或数据安全证据；主要伤害是出站预览和潜在用户可见提醒的结构 / 格式质量，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- `2026-07-12 03:02-07:02 CST` 真实运行态复发，状态从代码级 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-07-11`
    - 05:30 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒`
    - `job_id=j_218175e9`
    - `target=web-user-879a3b18fce2`
    - `parse_kind=PlainTextTriggered`
    - `deliver_preview` 以 fenced JSON 开头：包含 `"status": "triggered"`、`"scan_time": "2026-07-12T03:00+08:00"`、`"tickers_checked": ["TEM", "AAOI", "KRMN", "RKLB", "MRVL"]`、`"events": [` 等结构化协议字段。
    - 同条随后记录 `心跳任务未命中，跳过发送`，未确认正式投递；但 live 出站预览已经退化为用户不可读的结构化载荷，说明 2026-07-11 03:09 的代码级清理未覆盖当前 `PlainTextTriggered` + fenced JSON 形态。
  - 会话质量对照：
    - `data/sessions.sqlite3` 在 03:02-07:02 CST 新增 3 个 user turn / 3 条 assistant final，均为 scheduler 触发后正常收口。
    - assistant final 污染扫描未命中空回复、`<think>`、`reasoning_content`、本机路径、provider 原始错误、panic、quota、`data_fetch`、`quote_short`、`company_profiles/` 或原始工具 JSON。
  - 判断：
    - 该样本仍是 heartbeat 用户可见提醒格式化退化的同一链路；不是新的独立根因。
    - 当前没有错投、漏投、全渠道不可用或数据安全证据，且该条最终未发送；主要伤害是出站预览和潜在用户可见提醒的结构/格式质量，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- `2026-07-11 03:09 CST` 代码级修复并回归通过，状态更新为 `Fixed`：
  - `crates/hone-channels/src/scheduler.rs`
    - `trim_scheduler_trailing_json_field_residue(...)` 扩展了 heartbeat 尾随结构化字段裁剪范围，新增覆盖 `facts`、`actions_needed`、`action_items`、`catalyst/catalysts`、`event/events`、`summary`、`thesis`、`evidence`，并补 `:[` 数组残片形态，避免自然语言提醒后继续拼入数组或对象协议字段。
    - `heartbeat_message_trailing_field(...)` 同步扩展同一组字段，保证畸形 `JsonTriggered` 恢复路径也能把这些字段视作 `message` 之后的结构化尾巴，而不是正文内容。
  - 新增 / 复跑回归：
    - `cargo test -p hone-channels scheduler_delivery_text_trims_trailing_json_fact_residue --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_malformed_triggered_message_strips_trailing_data_object --lib -- --nocapture`
    - `cargo check -p hone-channels --tests`
  - 当前按代码与回归验证更新为 `Fixed`；本轮未重启 live runtime，待后续运行态复核是否已消除 `facts/actions_needed/catalyst` 尾巴污染。

- `2026-07-10 03:02 CST` 真实运行态复发，状态从 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=47777`
    - `job_name=DRAM 心跳监控`
    - `executed_at=2026-07-10T03:01:15.498268+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 前半段已经是自然语言提醒：`DRAM现价$65.25，已较昨收$62.04上涨+5.17%，突破$60触发位...`
    - 但自然语言正文后继续拼入结构化字段残片：`","facts":[...]`、`"actions_needed":[...]`、`{"level":"catalyst"...`
    - `detail_json.scheduler.deliver_preview` 同步保留 `","facts":[...]` 字段尾巴，说明不是单纯台账展示截断，而是准备投递的用户可见正文已经被结构化字段污染。
  - 查重结论：
    - 该样本与本文档既有 `JsonTriggered` 成功送达分支的“自然语言 + JSON 字段尾巴”同根；不是新的独立根因，因此不新建重复文档。
    - 最新污染字段扩展到 `facts`、`actions_needed` 和 catalyst 对象，说明 2026-06-22 的字段尾巴裁剪没有覆盖当前 JSON 形态。
  - 用户影响：
    - heartbeat 触发提醒已执行、已投递，也没有错投、漏投或全链路不可用证据。
    - 但用户会收到混有结构化协议字段的提醒正文，阅读体验和产品可信度下降，并暴露内部输出协议形态；这不影响主功能链路，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-06-22 03:28 CST）

- 本轮在 `sanitize_scheduler_delivery_text(...)` 增加 heartbeat / scheduler 正文尾随结构化字段残片裁剪：
  - 当用户可见正文已经形成自然语言提醒，但尾部继续拼入 `","data":{...}`、`"direction":...`、`"ticker":...`、`"exchange":...`、`"threshold":...` 等结构化字段时，现在会在第一段可疑 JSON 字段标记前截断。
  - 清理同时兼容未转义和 `\"...\"` 转义残片，避免 `deliver_preview` / 最终投递正文继续暴露协议字段尾巴。
  - 不会影响正常引号文本；新增回归专门覆盖“正常中文引号说明”不被误裁剪。
- 验证：
  - `cargo test -p hone-channels scheduler_delivery_text_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- 当前按代码与回归验证更新为 `Fixed`；若后续在最新代码运行态仍看到 heartbeat final 拼入新的结构化字段尾巴，再用新样本重新打开。

## 修复记录（2026-06-22 03:08 CST）

- heartbeat 畸形 `triggered` JSON 恢复逻辑已把 `data`、`direction`、`beat_threshold`、`threshold` 识别为 `message` 后续结构化字段，遇到自然语言提醒后拼入这些字段尾巴时会在出站前截断，避免 `","data":...` 或阈值字段残片进入用户可见提醒。
- 验证：
  - `cargo test -p hone-channels heartbeat_malformed_triggered_message_strips --lib -- --nocapture`
- 无关联 GitHub Issue；本轮按代码级修复关闭，不依赖生产日志、线上渠道状态或 live 重启。
- **证据来源**:
  - `2026-06-16 03:03 CST` 巡检补充复发证据：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=43281`
    - `job_id=j_9ee85d42`
    - `job_name=Cerebras IPO与业务进展心跳监控`
    - `executed_at=2026-06-16T00:31:07.317015+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 前半段已经是自然语言提醒，但尾部仍拼入 JSON 字段残片：`","data":{"ticker":"CBRS","exchange":"NASDAQ Global Market`
    - `detail_json.scheduler.deliver_preview` 同步保留该残片，说明不是单纯台账截断，而是准备投递的用户可见正文已经被结构化字段污染
    - 同窗另一条 heartbeat `TSLA 正负触发条件心跳监控` `run_id=43290` 正常触发并送达，无 JSON 残片；其余 heartbeat 失败主要是结构化 JSON / context window 既有形态，说明该问题仍是 `JsonTriggered` 成功送达分支的格式化抖动，而不是整批 scheduler 不可用
  - `2026-06-13 03:01 CST` 巡检补充复发证据：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=41301`
    - `job_id=j_4756be4d`
    - `job_name=伦敦金跌破4500提醒`
    - `executed_at=2026-06-13T01:30:14.803841+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 前半段已经是自然语言提醒，但尾部仍拼入 JSON 字段残片：`"direction":"below_threshold","beat_threshold":"281.83`
    - `detail_json.scheduler.deliver_preview` 同步保留该残片，说明不是单纯台账截断，而是准备投递的用户可见正文已经被结构化字段污染
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2398`
    - `job_id=j_818f0150`
    - `job_name=TEM大事件心跳监控`
    - `executed_at=2026-04-18T10:31:30.506141+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接等于原始 JSON 对象字符串：
      - `{"trigger":"标的: TEM (Tempus AI)\n触发条件: 利好类事件 - 重要学术会议重磅数据发布\n当前价格: $55.87 ..."}`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `detail_json.scheduler.deliver_preview` 同样记录为原始 JSON 对象字符串，而不是自然语言提醒
  - 最近运行日志：
    - `data/runtime/logs/web.log`
      - `2026-04-18 10:31:26.888` `job_id=j_818f0150` 记录 `parse_kind=JsonTriggered`
      - 同一行 `deliver_preview="{"trigger":"标的: TEM (Tempus AI)\n触发条件: 利好类事件 - 重要学术会议重磅数据发布 ..."}"`
    - `data/runtime/logs/hone-feishu.release-restart.log`
      - `2026-04-18T02:31:26.888655Z` 同一任务同样记录 `deliver_preview="{"trigger":"标的: TEM (Tempus AI)\n触发条件: 利好类事件 - 重要学术会议重磅数据发布 ..."}"`
  - 同任务前后对照样本：
    - `run_id=2366`，`executed_at=2026-04-18T09:01:32.710632+08:00`，同一 `TEM大事件心跳监控` 已能投递自然语言提醒
    - `run_id=2408`，`executed_at=2026-04-18T11:01:27.592766+08:00`，同一任务再次恢复为自然语言提醒
    - 说明问题不是用户配置或任务语义变化，而是同一 heartbeat 触发链路在相邻窗口间出现“有时正常格式化、有时直接投递 JSON”的不稳定行为

## 端到端链路

1. Feishu heartbeat 任务 `TEM大事件心跳监控` 在 `2026-04-18 10:31` 命中触发条件，scheduler 进入已触发投递分支。
2. 模型原始输出依旧带有 `<think>` 分析段，但解析器成功识别出 `JsonTriggered`。
3. 当前投递链路没有把这次解析结果稳定格式化成自然语言提醒，而是直接把提取出的 JSON 对象字符串作为最终投递正文。
4. 调度台账把本轮记为 `completed + sent + delivered=1`，但用户实际拿到的是结构化对象文本，而不是面向人类阅读的提醒文案。

## 期望效果

- heartbeat 在命中 `JsonTriggered` 后，应始终输出稳定、可直接阅读的自然语言提醒。
- 无论模型内部返回中文、英文，或不同字段顺序的 JSON，scheduler 最终投递都不应把原始对象字符串直接发给用户。
- `cron_job_runs.response_preview` 应反映用户最终看到的提醒文案，而不是格式化前的结构化对象。

## 当前实现效果

- `2026-06-16 00:31` 的 `Cerebras IPO与业务进展心跳监控` 已成功触发并送达，正文主体是自然语言提醒，但后面继续拼入 `data.ticker` / `data.exchange` 字段残片。该样本与 `2026-06-13` 的金价样本同属“自然语言 + 结构化字段尾巴”混合输出形态，说明尾随 JSON 字段清理仍未覆盖非金价 heartbeat 任务。
- `2026-06-13 01:30` 的 `伦敦金跌破4500提醒` 已经成功触发并送达，正文主体是自然语言提醒，但末尾仍外露 JSON 字段残片 `direction` / `beat_threshold`。这晚于 2026-04-20 `unwrap_nested_json_message` 修复记录，说明修复只覆盖了完整 `{"trigger": ...}` 对象直出，未覆盖“自然语言 + 结构化字段尾巴”的混合输出形态。
- `2026-04-18 10:31` 的 `TEM大事件心跳监控` 已经成功命中触发并送达，但送达内容退化为原始 JSON 对象字符串。
- 这一轮不是简单的“记录脏了但用户侧正常”：`detail_json.scheduler.deliver_preview` 已直接等于 JSON 字符串，说明调度器准备发送的正文本身就是未格式化对象。
- 同一个任务在 `09:01` 和 `11:01` 又都恢复为自然语言提醒，进一步说明这是格式化链路的不稳定抖动。
- 同时间窗里其它 heartbeat 任务仍持续保留 `<think>` 污染的 `raw_preview`，说明当前 `JsonTriggered` 的投递格式化也仍建立在脆弱的协议解析之上。

## 用户影响

- 这是质量类缺陷。任务已执行、已投递，也没有发生错投、漏投或系统级失败。
- 但用户收到的是原始结构化对象，而不是产品化提醒文案，阅读体验和可信度明显下降，也会暴露内部协议形态。
- 之所以定级为 `P3`，是因为它没有阻断 heartbeat 主功能链路，用户仍收到触发提醒和核心价格信息；当前伤害主要是格式与质量退化，而不是功能不可用。

## 根因判断

- heartbeat `JsonTriggered` 分支的结果规范化不稳定；同一任务有时会把提取出的对象渲染成自然语言，有时却直接把 JSON 字符串作为最终正文。
- `2026-06-16` 复发样本显示污染字段已扩展到通用 `data` 对象字段（如 `ticker` / `exchange`），不是金价阈值任务的专属字段清理遗漏。
- `2026-06-13` 复发样本显示，格式化入口还可能只剥离对象开头或主体字段，却没有完整截断尾随结构化字段，导致自然语言正文后拼接 `direction` / `beat_threshold`。
- 结合最近一小时其它 heartbeat 仍保留 `<think>` 污染输出，可以推断当前格式化逻辑仍依赖脆弱的“先解析结构，再拼装文案”路径，不同轮次对对象形态或字段内容的兼容不一致。
- 这与 [`scheduler_heartbeat_unknown_status_silent_skip.md`](./scheduler_heartbeat_unknown_status_silent_skip.md) 共享同一协议脆弱背景，但这里的直接症状已从“失败跳过”变成“成功送达但格式退化”。

## 下一步建议

- 检查 heartbeat `JsonTriggered` 结果的统一格式化入口，确认对象型结果何时会被直接 `to_string` 或原样透传。
- 为 `triggered` 分支补回归测试，至少覆盖：
  - 对象型 `{"trigger":"..."}` 返回
  - 中英文字段内容
  - 同时含 `<think>` 污染原文但已成功解析出触发态的情况
- 在台账里继续观察是否还有其它 heartbeat 任务把 `response_preview` / `deliver_preview` 记成原始 JSON；若扩散到多条任务，可考虑提升优先级。
## 最新运行态复核（2026-07-17 23:02 CST）

- `data/runtime/logs/web.log.2026-07-17`
  - 巡检窗口：2026-07-17 19:01-23:01 CST。
  - 22:30 CST `小米30港元破位预警` `parse_kind=PlainTextTriggered` 的 `deliver_preview` 仍以 fenced JSON 开头，包含 `"status": "triggered"`、`"symbol": "1810.HK"`、`"condition": "现价 ≤ 30 港元"` 等协议字段。
  - 22:30 CST `AI与科技持仓观察关键事件心跳提醒` 的 `deliver_preview` 同样以 fenced JSON 开头，包含 `"status": "triggered"` 和长 `event` 字段。
  - 同窗仍有 47 条 `deliver_preview` 与 3 条 `JsonTriggered`，说明 heartbeat 出站内容仍可能把协议载荷当作用户消息。
- 本轮判断
  - 这仍是既有 heartbeat JSON / 协议字段外露质量缺陷复发，不是新的链路根因。
  - 触发与投递链路本身仍可运行，问题主要是用户可见格式和产品感退化，因此维持质量性 `P3 / New`，非 P1。
