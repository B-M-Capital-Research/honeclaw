# Bug: Feishu 直聊存储股最新价格回复在行情工具未完成时输出未充分校验数值

- **发现时间**: 2026-06-06 23:04 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-06 19:03-23:03 CST。
  - 本窗有 5 个 user turn 与 5 个 assistant final，Feishu direct 均成对收口；`cron_job_runs` 同窗没有新增记录。
  - assistant final 污染扫描未命中空回复、`company_profiles/...`、`公司画像公司画像`、本机绝对路径、`data/agent-sandboxes`、`hone-mcp binary not found`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic 或 `index out of bounds`。
  - `session_id=Actor_feishu__direct__ou_5f58ff884640e647a1792f618f45209251` 在 2026-06-06 20:53 CST 收到用户输入摘要：`对，看下最新的价格`。上一轮上下文是 MU / SNDK 存储股回调区间。
  - 20:54 CST assistant final 回复“最新可用价格是6月5日美股收盘和盘后口径”，并给出 MU `收盘价 864.01` / `盘后价 857.20`，SNDK `收盘价 1559.32` / `盘后价 1529.50`，随后继续按这些数值判断 MU 接近安全垫区间、SNDK 仍只是观察区。
- `data/runtime/logs/acp-events.log`
  - 同轮 20:54:25 CST 先发起 `finance: MU` 搜索。
  - 20:54:32 CST 发起 `MU stock price June 6 2026 latest close Micron` 与 `SNDK stock price June 6 2026 latest close Sandisk` 搜索。
  - 20:54:33 CST 打开 Yahoo Finance 的 MU 页面。
  - 20:54:36 CST 打开 `https://stockanalysis.com/stocks/mu/`。
  - 20:54:38-20:54:39 CST 已经开始向用户流式输出 MU `收盘价：864.01` 与 `盘后价：857.20` 等精确行情数字。
  - 20:54:50 CST `stockanalysis.com/stocks/mu/` 对应 tool call 才标记 `completed`，即用户可见精确价格在至少一个行情页面读取完成前已经生成。
  - 本轮没有看到同等明确的 SNDK 行情页面打开记录；但 final 同样输出了 SNDK 精确收盘价、盘后价、日内区间、52 周区间和 Forward PE。
- 最近四小时非文档提交仅有 `26d4aa57 docs: record web direct image attachment bug`，不涉及行情校验链路修复。

## 端到端链路

1. Feishu 用户在已有 MU / SNDK 存储股讨论上下文中要求“看下最新的价格”。
2. runner 发起 Web 搜索和页面打开来核行情。
3. assistant 在行情页面读取完成前已经流式输出精确价格和盘后价格。
4. assistant 后续继续基于这些数值给出加仓观察区、安全垫区和等待建议。
5. 会话最终正常 `end_turn` 收口，用户没有看到内部错误或工具原文。

## 期望效果

- 对“最新价格”这类强时效行情请求，assistant 应等行情工具返回并消费完成后再输出精确数字。
- 如果只拿到搜索摘要、页面未读完或某标的未完成独立核验，应明确写“未完成稳定校验”，并避免给出精确盘后价、日内区间、Forward PE 或加仓判断。
- MU 与 SNDK 应分别完成实体和行情核验，不能只打开一个标的页面后对两个标的都输出完整精确行情。

## 当前实现效果

- assistant final 结构完整、投递正常，也没有外露工具协议或原始错误。
- 但精确行情数字在至少一个行情工具完成前已经进入用户可见流式回复。
- 对 SNDK 没有看到明确页面读取完成证据，final 仍给出了完整行情字段和交易节奏判断。
- 这会让用户把未充分校验的价格当成最新行情锚点。

## 用户影响

- 这是质量性 bug，不是链路级功能故障。
- 用户的问题已经得到一条可读回复，Feishu direct 没有未回复、重复回复、发错对象、投递失败、数据写坏或内部错误泄露。
- 但用户要求的是最新行情，assistant 却在工具链未完全收口时给出精确价格并据此分析买点，可能降低投资判断质量。
- 因此本项不影响主功能链路，按规则定级为 `P3`，不是 `P1/P2`。

## 根因判断

- 初步判断是强时效金融回答没有严格等待行情工具完成，也没有把“工具仍在读取 / 某标的未完成独立核验”的状态转化为保守输出。
- 该问题与 `feishu_direct_futu_premarket_stale_price_advice.md` 不同：FUTU 缺陷是盘前大跌场景把常规交易旧价当决策锚；本轮是周末行情查询中，精确价格输出早于工具链完成，并且 SNDK 独立核验证据不足。
- 该问题与 `feishu_direct_partial_reply_before_tool_completion.md` 也不同：本轮不是半成品短答提前持久化，而是完整 final 在行情核验边界上过早输出精确数值。

## 下一步建议

- 在强时效行情 prompt / runner 汇总层增加约束：存在未完成行情工具时，不得输出精确价格或交易区间结论。
- 对多标的行情请求，要求每个标的都有独立核验证据；缺少某标的时只输出“该标的未完成稳定校验”。
- 增加回归：用户问“最新价格”且工具调用仍未完成时，final 不应包含精确收盘价、盘后价、日内区间或 Forward PE。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码，未运行代码测试。
- 已验证范围：SQLite 会话收口、assistant final 污染扫描、ACP tool call / final chunk 时序、`cron_job_runs` 同窗无新增、最近四小时提交检查。
