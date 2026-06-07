# Bug: Event-engine FMP price/news poller 持续请求失败导致行情与新闻增量退化

## 发现时间

- 2026-06-07 03:02 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- New

## 证据来源

- `data/runtime/task_runs.2026-06-06.jsonl`
  - 2026-06-06 23:01-2026-06-07 03:01 CST 最近四小时内，`poller.fmp.price` 出现 48 次 `failed + items=0`，`poller.fmp.news` 出现 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 有 8 次 `ok + items=0`，说明 runtime 仍在调度 poller tick，失败集中在 FMP quote/news 请求链路。
  - 失败信息均为脱敏后的 `FMP 请求失败: error sending request for url (https://financialmodelingprep.com/...apikey=<redacted>)`，覆盖 `/api/v3/quote/...` 与 `/api/v3/stock_news?limit=50`。
- `data/runtime/task_runs.2026-06-06.jsonl`
  - 2026-06-07 03:01-07:01 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，`internal.unified_digest_scheduler` 与 `internal.daily_report` 仍按分钟 tick 记录 `skipped`，说明 runtime tick 未整体停摆。
  - 失败形态仍是 FMP quote/news 请求发送失败，没有观察到恢复样本。
- `data/runtime/task_runs.2026-06-06.jsonl` 与 `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 07:00-11:04 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.earnings` 与 `poller.fmp.macro` 各新增 2 次 `failed + items=0`，错误同为 FMP 请求发送失败；`poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，`poller.fmp.corp_action`、`poller.fmp.sec_filings`、`poller.fmp.analyst_grade` 各 2 次 `ok + items=0`。
  - `internal.unified_digest_scheduler` 同窗有 2 次 `ok + items=46`，说明 event-engine runtime 仍在运行，失败集中在 FMP 部分 API 请求链路。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 11:02-15:02 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，说明 runtime tick 未整体停摆，失败继续集中在 FMP price/news 请求发送链路。
  - `internal.unified_digest_scheduler` 与 `internal.daily_report` 同窗仅记录周期性 `skipped`，未观察到用户可见 FMP 原始错误外泄。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 15:03-19:03 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`；`internal.unified_digest_scheduler` 与 `internal.daily_report` 仅记录周期性 `skipped`。
  - 错误仍为脱敏后的 FMP quote/news 请求发送失败，尚未观察到恢复样本或用户可见原始 FMP 错误。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 2026-06-07 19:02-23:02 CST 后续四小时继续复现：`poller.fmp.price` 再新增 48 次 `failed + items=0`，`poller.fmp.news` 再新增 16 次 `failed + items=0`。
  - 同窗 `poller.fmp.extended_hours` 仍有 8 次 `ok + items=0`，`internal.daily_report` 有 1 次 `ok + items=1`，`internal.unified_digest_scheduler` 有 2 次 `ok`；说明 runtime 未整体停摆。
  - 失败仍集中在 FMP quote/news 请求发送链路，尚无本轮用户可见 FMP 原始错误外泄。
- 当天更早记录显示：
  - 2026-06-06 08:04-09:24 CST `poller.fmp.price` 曾连续成功 18 次，`poller.fmp.news` 曾成功 5 次。
  - 2026-06-06 09:29 CST 起，price/news poller 开始持续失败；截至 2026-06-07 03:01 CST 最近四小时仍未恢复。
- `data/sessions.sqlite3`
  - 2026-06-06 23:01-2026-06-07 03:01 CST 有 3 个 Feishu user turn 与 3 个 assistant final，均成对收口；本轮没有直接用户可见投递失败或原始 FMP 错误外泄。
  - 2026-06-07 03:01-07:01 CST 没有新增可判定直聊质量的新消息；SQLite 最新消息仍停在 2026-06-07 00:41 CST。
  - 2026-06-07 07:00-11:04 CST 有 5 个 user turn 与 5 个 assistant final，Feishu direct 与 Discord scheduler 均有 assistant 记录收口；assistant final 污染扫描未命中空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误或 panic。
  - 2026-06-07 15:03-19:03 CST 有 8 个 Feishu user turn 与 8 个 assistant final，4 个 Feishu direct 会话最新均以 assistant 收口；`cron_job_runs` 同窗无新增记录，assistant final 污染扫描未命中 FMP 原始错误、空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或 stream disconnect。
  - 2026-06-07 19:02-23:02 CST 有 14 个 Feishu user turn 与 15 个 assistant 记录，7 个 Feishu direct 活跃会话最新均以 assistant 收口；多出的 1 条 assistant 是 daily-limit final/text 双记录，另立 P3 跟踪。`cron_job_runs` 同窗无新增记录，assistant final 污染扫描未命中 FMP 原始错误、空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或 stream disconnect。

## 端到端链路

1. event-engine runtime 定期执行 `poller.fmp.price` 与 `poller.fmp.news`。
2. poller 调用 Financial Modeling Prep quote/news API 获取观察池行情与新闻增量。
3. poller 结果进入 event-engine 的事件候选、digest、告警或后续投研上下文。
4. 最近窗口内 price/news poller 持续请求失败并返回 `items=0`，导致对应增量数据不可用。

## 期望效果

- FMP price/news poller 应在正常网络与有效 key 下持续产出可用行情/新闻增量。
- 单次请求失败应有重试、分批、降级或明确分类；持续失败应被标记为可运维的上游/网络/配置异常，而不是只在 task_runs 中反复记录同一失败。
- event-engine 下游若依赖这类数据，应能感知数据新鲜度不足并避免把缺失增量误当作“无事件”。

## 当前实现效果

- 最近四小时内 quote/news poller 全部失败且 `items=0`，没有观察到恢复样本。
- 后续 2026-06-07 03:01-07:01 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`，持续失败时长继续扩大。
- 后续 2026-06-07 07:00-11:04 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；earnings / macro 也出现同类请求发送失败，说明影响面从 price/news 扩展到更多 FMP API。
- 后续 2026-06-07 11:02-15:02 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours 仍按节奏 `ok`，说明当前证据仍指向 FMP price/news 请求发送链路持续退化。
- 后续 2026-06-07 15:03-19:03 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours 仍按节奏 `ok`，说明持续失败尚未恢复。
- 后续 2026-06-07 19:02-23:02 CST 复核窗口内，quote/news poller 仍全部失败且 `items=0`；extended-hours、daily report 与 unified digest scheduler 均有 `ok` 样本，说明失败继续集中在 FMP price/news 请求链路。
- 同一 runtime 的 extended-hours poller 仍按节奏运行并返回 `ok`，说明不是调度器完全停止。
- 当前缺陷尚未在本轮直接表现为用户可见错误、错投或格式污染，但已构成事件引擎数据摄取链路退化。

## 用户影响

- 用户依赖的实时行情、新闻增量、digest 候选与监控触发可能变旧、变少或漏报。
- 若下游没有严格的新鲜度检查，系统可能把“FMP 数据抓取失败”误解释为“观察池没有新闻/价格变化”。
- 该问题影响功能链路的数据正确性与监控完整性，因此定级为 P2；它没有直接导致本轮用户请求失败、跨用户错投、数据破坏或大面积可见错误，因此不定为 P1。

## 根因判断

- 直接原因是 FMP quote/news HTTP 请求在 poller 层持续 `error sending request`。
- 当前证据不足以确认是本机网络、FMP 上游、key/plan 限制、请求 batch 形态或客户端超时配置导致。
- 该问题不同于已关闭的 `event_engine_price_poller_transient_fetch_failure.md`：本轮不是单 tick 抖动，而是从 2026-06-06 09:29 CST 起持续到最近四小时的 price/news poller 全失败。
- 也不同于历史 `event_engine_price_poller_unbounded_quote_batch.md` 的已修复 batch 拆分问题：本轮错误信息是请求发送失败，尚未证明为 URL path 过长或单 batch 丢弃其它成功 batch。

## 下一步建议

- 先检查 event-engine FMP client 对 `error sending request` 的错误分类、重试和超时设置，确认是否需要按网络/上游/配置分别记录 `failure_kind`。
- 对 price/news poller 增加连续失败阈值告警，避免长时间只写 `task_runs` 而没有运行态告警。
- 检查失败期间是否仍有其它行情源或缓存被下游使用；若没有，应在 digest / alert 生成前显式注入数据新鲜度缺口。
- 对比 FMP quote/news 与 extended-hours 的请求域名、batch 大小、timeout、key 使用路径，定位为何 extended-hours 仍 ok 而 quote/news 持续失败。
