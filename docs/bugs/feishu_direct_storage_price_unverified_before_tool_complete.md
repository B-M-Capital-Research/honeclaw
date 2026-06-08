# Bug: Feishu 直聊存储股最新价格回复在行情工具未完成时输出未充分校验数值

- **发现时间**: 2026-06-06 23:04 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
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
- `data/sessions.sqlite3`
  - 2026-06-07 11:02-15:02 CST 复核窗口有 2 个 Feishu user turn 与 2 个 assistant final，均成对收口；`cron_job_runs` 同窗无新增记录。
  - assistant final 污染扫描未命中空回复、`company_profiles/...`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic 或 `index out of bounds`。
  - `session_id=Actor_feishu__direct__ou_5f175714e91a60d34339460cdd1268f8fb` 在 2026-06-07 12:31 CST 收到用户输入摘要：`存储美光，闪迪，dram基金做下对比`。
  - 12:33 CST assistant final 再次输出 MU `6月5日收盘 864.01` / `盘后 857.20`、SNDK `6月5日收盘 1559.32` / `盘后 1529.50`、DRAM ETF `6月5日收盘 55.79`，并给出 Forward PE、AUM、持仓、以及 5000 美元配置框架。
- `data/runtime/logs/acp-events.log`
  - 同轮 12:33 CST tool update 显示 runner 把上述 MU、SNDK、DRAM ETF 精确行情与估值数值写入 sandbox `company_profiles/MU.md`、`company_profiles/SNDK.md`、`company_profiles/DRAM_ETF.md`。
  - 同轮最终以 `stopReason=end_turn` 收口，未见 `stream disconnected before completion`、runner error、quota、panic 或用户可见内部错误。
- `data/sessions.sqlite3`
  - 2026-06-07 15:03-19:03 CST 复核窗口有 8 个 Feishu user turn 与 8 个 assistant final，4 个 Feishu direct 会话最新均以 assistant 收口；`cron_job_runs` 同窗无新增记录。
  - assistant final 污染扫描未命中空回复、`company_profiles/...`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic、`index out of bounds`、`stream disconnected`、`hone-mcp binary not found`、`技能未加载` 或 `当前运行器`。
  - `session_id=Actor_feishu__direct__ou_5f58ff884640e647a1792f618f45209251` 在 2026-06-07 15:55 CST 收到用户输入摘要：`周五跌得很可怕，什么时候可以抄底？`。上一轮上下文仍是 MU / SNDK 存储股回调与配置建议。
  - 15:58 CST assistant final 再次输出 MU `周五收盘 864.01` / `盘后参考约 857.2 到 857.4`、SNDK `周五收盘 1,559.32` / `盘后参考 1,528.87` / `周五日内低点 1,514.36`，并据此给出 MU `800-850` 试探、`720-780` 高值博、SNDK `1,250-1,350` 与 `1,050-1,180` 等抄底区间。
- `data/runtime/logs/acp-events.log`
  - 同轮 15:56 CST 完成 `finance: MU`、`finance: MU`、`June 5 2026 MU stock close after hours price Micron, June 5 2026 SNDK stock close after hours price Sandisk` 搜索，以及 MarketBeat SNDK chart 页面读取。
  - 15:56:52 CST assistant 已开始流式说明“MU 盘后约 857，SNDK 盘后约 1529”，随后 15:57 CST 读取本地 `MU.md` 与 `SNDK.md` 公司画像，并在 15:58 CST 输出完整抄底区间。
  - 该轮最终以 `stopReason=end_turn` 收口，未见 response error、runner error、stream disconnect、quota、panic 或用户可见内部错误。
- 最近四小时无非文档代码提交，不涉及行情校验链路修复。

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
- 2026-06-07 12:31 CST 同根复发时，assistant 不只把同一组精确行情数字用于用户可见投资对比，还把这些数字沉淀进 sandbox 公司画像，后续会话可能继续复用该画像中的未充分校验行情。
- 2026-06-07 15:55 CST 同根复发时，assistant 在用户询问抄底节奏时继续复用同一组 MU / SNDK 异常精确行情锚，并把它转化为具体分档抄底区间；这说明画像沉淀后的未充分校验行情已进入后续操作型建议链路。

## 用户影响

- 这是质量性 bug，不是链路级功能故障。
- 用户的问题已经得到一条可读回复，Feishu direct 没有未回复、重复回复、发错对象、投递失败、数据写坏或内部错误泄露。
- 但用户要求的是最新行情，assistant 却在工具链未完全收口时给出精确价格并据此分析买点，可能降低投资判断质量。
- 因此本项不影响主功能链路，按规则定级为 `P3`，不是 `P1/P2`。

## 根因判断

- 初步判断是强时效金融回答没有严格等待行情工具完成，也没有把“工具仍在读取 / 某标的未完成独立核验”的状态转化为保守输出。
- 该问题与 `feishu_direct_futu_premarket_stale_price_advice.md` 不同：FUTU 缺陷是盘前大跌场景把常规交易旧价当决策锚；本轮是周末行情查询中，精确价格输出早于工具链完成，并且 SNDK 独立核验证据不足。
- 该问题与 `feishu_direct_partial_reply_before_tool_completion.md` 也不同：本轮不是半成品短答提前持久化，而是完整 final 在行情核验边界上过早输出精确数值。
- 2026-06-07 12:31 CST 复发说明问题不只发生在“用户明确问最新价格”的单轮，还会在多标的对比/配置建议中复用上次未充分校验的精确行情，并进一步写入公司画像；根因仍是强时效行情与操作建议缺少“必须重新校验或降级为未验证框架”的硬边界。
- 2026-06-07 15:55 CST 复发进一步说明，该根因会沿本地公司画像延续到后续抄底/买点建议：即使本轮有行情搜索与页面读取，assistant 仍把此前未充分校验的价格锚作为可操作区间基础，没有显式降级为“需重新核价后再定档”。

## 下一步建议

- 在强时效行情 prompt / runner 汇总层增加约束：存在未完成行情工具时，不得输出精确价格或交易区间结论。
- 对多标的行情请求，要求每个标的都有独立核验证据；缺少某标的时只输出“该标的未完成稳定校验”。
- 增加回归：用户问“最新价格”且工具调用仍未完成时，final 不应包含精确收盘价、盘后价、日内区间或 Forward PE。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码，未运行代码测试。
- 已验证范围：SQLite 会话收口、assistant final 污染扫描、ACP tool call / final chunk 时序、`cron_job_runs` 同窗无新增、最近四小时提交检查。
- 2026-06-07 15:02 CST 复核为缺陷台账维护任务，未修改业务代码，未运行代码测试；已验证范围：SQLite 会话收口、assistant final 污染扫描、ACP `end_turn`、本轮 `cron_job_runs` 无新增、最近四小时无非文档代码提交。
- 2026-06-07 19:03 CST 复核为缺陷台账维护任务，未修改业务代码，未运行代码测试；已验证范围：SQLite 会话收口、assistant final 污染扫描、ACP prompt / `end_turn` 对齐、相关 tool call 时序、本轮 `cron_job_runs` 无新增、最近四小时无非文档代码提交。

## 修复记录

- 2026-06-09 00:12 CST 进入 `Fixing`：`DEFAULT_FINANCE_DOMAIN_POLICY` 已新增“多标的最新行情约束”，要求多个股票 / ETF / 基金的最新价格、盘后价、日内区间、估值倍数或配置/抄底区间必须逐一具备本轮独立核验的来源、时间戳和交易时段口径；不得复用其它标的搜索结果、历史公司画像或未完成工具读取中的数字作为精确行情锚点；未完成稳定校验时不得输出精确价格、Forward PE 或操作区间。`build_prompt_bundle_always_includes_finance_domain_policy` 已补断言。
- 验证阻塞：本机 Rust toolchain 当前 `cargo` / `rustc` 均悬挂，本轮仅完成 `git diff --check`，不能标记 `Fixed`。下一轮需运行 `cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture` 与 `cargo check -p hone-channels --tests`。
- 2026-06-09 04:43 CST 状态更新为 `Fixed`：`cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture` 与 `cargo check -p hone-channels --tests` 通过。
