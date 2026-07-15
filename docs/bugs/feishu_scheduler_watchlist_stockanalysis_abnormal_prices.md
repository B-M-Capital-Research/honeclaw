# Bug: Feishu scheduler 核心观察池使用 StockAnalysis 异常价格作为行情锚

## 发现时间

- 2026-06-20 11:02 CST

## Bug Type

- Business Error

## 严重等级

- P3

## 状态

- New

## GitHub Issue

- 无，非 P1

## 最新进展

- 本轮 2026-07-15 15:02-19:02 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 18:30 / 19:00 CST `Monitor_Watchlist_11` raw preview 继续使用 `MU $983.12` 对比 `MU <= $252.00` 判断未触发。
    - 18:30 CST `Cerebras IPO与业务进展心跳监控` deliver preview 使用 `CBRS $203.81`、`Market Cap ~$4617 亿` 作为行情锚。
    - 18:30 CST `持仓重大事件心跳提醒` deliver preview 使用 `MU $983.12` 与 `SPCX $136.08`，并把 `MU` 昨收写成 `$93...` 量级混杂。
    - 19:00 CST `中际旭创关键事件心跳提醒` deliver preview 使用 `¥1,169.31`、日内 `¥1,160-¥1,210` 作为锚点，同时承认 FMP snapshot / news 接口未返回今日有效数据。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗按真实 `timestamp` 有 8 条 user / 9 条 assistant，近期真实会话均以 assistant 收口；assistant final 未见错投、投递失败、原始工具 JSON、敏感信息外泄或空回复。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格继续进入 heartbeat 判断上下文和用户可见 preview。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-15 11:01-15:02 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-15`
    - 11:00 / 12:30 / 14:30 CST `Monitor_Watchlist_11` raw preview 继续使用 `MU $983.12` 对比 `MU <= $252.00` 判断未触发。
    - 11:00 / 12:30 CST `闪迪关键事件心跳提醒` raw / deliver preview 继续使用 `SNDK $1,757.82`、昨收 `$1,673.97`、日高 `$1,812.47` 等异常数量级价格。
    - 11:00 CST `RKLB异动监控` deliver preview 写 `Market Cap ~$4562 亿`；12:30 CST 同 job 写 `Market Cap ~$456 亿`，同一链路市值数量级不稳定。
    - 12:30 CST `中际旭创关键事件心跳提醒` deliver preview 继续锚定 `¥1,181`、昨收 `¥1,184.05`；14:30 CST `光迅科技关键事件心跳提醒` deliver preview 使用 `¥218.99`、昨收 `¥232.95`，均在主行情源未稳定推进时进入用户可见 preview。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗没有新的真实 `timestamp` assistant final，正式会话侧未新增可审计用户可见样本；本轮证据来自 heartbeat runtime preview。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格继续进入 heartbeat 判断上下文和用户可见 preview。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-15 07:04-11:02 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 08:30 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34` 正常收口，但用户可见 final 输出 `SNDK 昨夜（7月14日）收于 1,755.11 美元`、50 日均线 `1,688 美元`、日内区间 `1,689.50` 等异常数量级行情锚。
    - 08:30 CST Feishu scheduler `Hone AI 每日早报` 继续输出 `MU +4.92%`、`SNDK +5.01%`、`BE +4.24%` 与 `BE $250` 等高风险行情锚并用于持仓判断。
  - `data/runtime/logs/web.log.2026-07-15`
    - 08:00 / 11:00 CST `Monitor_Watchlist_11` raw preview 继续使用 `MU $983.12` 对比 `MU <= $252.00` 判断未触发。
    - 08:00 / 11:00 CST `存储板块关键事件心跳提醒` / `持仓财报与重大新闻心跳提醒` raw 或 deliver preview 继续使用 `SNDK $1,757.82`、`SNDK $1,755`、`AAOI $125.45` 等高风险行情锚进入 noop / triggered 判断。
    - 11:00 CST `RKLB异动监控` deliver preview 写 `Market Cap ~$4562 亿`，数量级明显高风险，并进入用户可见 heartbeat preview。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格继续进入正式 scheduler final、heartbeat 判断上下文和用户可见 preview。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-15 03:02-07:03 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-14`
    - 03:30-07:01 CST heartbeat 日志中 `$1,` 数量级美元价格命中 17 次，`SNDK $1,` 命中 1 次，`SPY $75x` 命中 2 次；这些价格继续进入 raw / deliver preview、触发判断或 duplicate suppression。
    - 03:30 CST `Monitor_Watchlist_11` raw preview 使用 `MU: Current $977.04` 对比 `MU <= $252.00` 判断未触发。
    - 07:00 CST `Monitor_Watchlist_11` raw preview 使用 `MU $983.12`、`RKLB $78.81`、`BE $250+` 等高风险行情锚进入阈值判断。
    - 07:00 CST `存储板块关键事件心跳提醒` raw / deliver preview 继续引用 `SNDK $1,767.48` 等异常数量级价格，并围绕是否有新增触发事实判断。
    - 07:00 CST `美股黄金坑信号心跳检测` raw preview 使用 `SPY 751.83`、`QQQ 719.69`，同条还把基准数据写成 `2026-05-16 盘中`。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗新增 9 个 user turn / 9 条 assistant 记录，8 个近期 session 均以 assistant 收口；assistant final 未见错投、投递失败、原始工具 JSON、敏感信息外泄或空回复。
    - 05:01 CST Web scheduler `盘后美股复盘与SNDK/MU存储产业链日报` final 继续输出 `SNDK 从昨日约 1,668 美元低位强势反弹至约 1,853 美元`，且该 assistant row 没有 `assistant.tool_calls`。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格继续进入 heartbeat 判断上下文和正式 scheduler final。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-14 23:02-2026-07-15 03:02 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-14`
    - 23:30-03:00 CST `持仓重大事件心跳提醒` 多条 deliver preview 继续使用 `MU $965-$987`、`SPCX $138-$140` 等高风险行情锚，并据此判断是否触发。
    - 00:00-02:30 CST `闪迪关键事件心跳提醒` / `存储板块关键事件心跳提醒` 继续使用 `SNDK $1,767-$1,772`、昨收 `$1,673.97` 等异常数量级行情锚，并进入 noop / triggered 判断。
    - 03:00 CST `美股黄金坑信号心跳检测` deliver preview 使用 `SPY 752.13`、`QQQ` 同类高风险价格，并把数据快照误写成 2026 年 5 月。
    - 00:00-03:01 CST `中际旭创关键事件心跳提醒` / `光迅科技关键事件心跳提醒` 多次用 `中际旭创 ¥1,184.05`、`光迅科技 ¥232.95` 作为锚点；部分文本同时承认主行情源未推送新时间戳，仍输出“已校验”口径。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗新增 7 个 user turn / 7 条 assistant 记录，3 个近期 session 均以 assistant 收口；assistant final 未见内部字段、原始工具 JSON、投递失败或敏感信息外泄。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格继续进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-14 19:02-23:02 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 21:35 CST Feishu scheduler `科技核心股池 · 晚间击球区快报` 正常收口，但用户可见 final 继续输出 `MU $980.82`、`SNDK $1,771.55`、`STX $895.00` 等高风险数量级行情锚，并据此计算击球区偏离。
    - 23:00 CST Feishu scheduler `核心观察股池晚间快报` 正常收口，继续输出 `MU $975.50`、`SNDK $1,771.55`、`STX $895.00` 等同类价格。
  - `data/runtime/logs/web.log.2026-07-14`
    - 22:30-23:01 CST heartbeat deliver / raw preview 继续出现 `MU: Current 964.21`、`SNDK $1,748`、`SNDK $1,734.875` 等异常数量级价格，并进入 noop / triggered 判断。
  - 主链路未阻断，问题主要影响行情可信度和后续投资判断质量，维持质量性 `P3 / New`，非 P1。

- 本轮 2026-07-14 15:01-19:02 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-14`
    - 15:30 CST `存储板块关键事件心跳提醒` deliver preview 继续使用 `SNDK $1,673.97`、盘后进一步下探 2.4%、52 周高点等异常数量级价格，并据此落成 `triggered`。
    - 15:31 CST `光模块板块关键事件心跳提醒` deliver preview 写 `SNDK 单日暴跌 12.6%，收于 $1,673.97；盘后继续跌 2.4%`，继续把同一异常行情锚带入板块级触发判断。
    - 16:00-18:30 CST `持仓重大事件心跳提醒` 多条 deliver preview 继续使用 `MU $937`、`SPCX $139.14` 等高风险行情锚，并围绕同一时间戳判断无新增。
    - 19:00 CST `SIVE POET/Nokia/1.6T DFB 心跳检测` deliver preview 写 `POET $8.00`、50 日均线 `$11.66`、200 日均线 `$7.86`；该类价格可能合理但仍和同窗异常行情源混杂，需要后续按数据源 sanity check 统一处理。
  - `data/sessions.sqlite3` / `session_messages`
    - 15:01-19:02 CST 新增 5 个 user turn / 5 条 assistant 记录，Web / Feishu direct 与 scheduler 均有 assistant 收口；assistant final 未见内部字段、原始工具 JSON、投递失败或敏感信息外泄。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格继续进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-14 07:01-11:01 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 07:01-11:01 CST 新增 32 个 user turn / 44 条 assistant 记录；Feishu / Web / Discord 均有 assistant 终态，未见错投、敏感信息泄露或原始工具 JSON。
    - 07:03 CST Feishu 持仓早报继续使用 `AMD 557.89`、`MU 979.30` 等高风险数量级价格作为组合贡献锚点。
    - 08:30 CST Feishu scheduler `闪迪(SNDK)每日行情与行业简报` 正式 final 写出 `SNDK 收盘价 1,673.97 美元`、前收 `1,915.92`、52 周高点 `2,354.39`、YTD `+756%`，并据此生成跌破 50 日均线、估值和行业催化判断。
    - 08:46 CST Feishu scheduler `A股盘前高景气产业链推演` 继续使用 `NVDA 203.53`、`SNDK 1673.97`、`MRVL 217.53`、`MU 跌破 960` 等高风险行情锚，推导 A 股 PCB / CPO / 存储链压力。
    - 08:31 / 08:36 / 08:38 CST NOK 直聊先输出 `NOK 7月11日收盘价 4.50 美元`，随后又核验为 `11.675 美元` 并推测“最大可能是股票拆分”，但未确认拆股事件即继续生成 PE / EPS / 估值判断。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或互相矛盾的价格进入正式 scheduler final 和 direct 投研判断。
    - 主体报告可读并完成，问题没有阻断调度 / 投递主功能链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-14 03:01-07:01 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 03:01-07:01 CST 新增 12 个 user turn / 13 条 assistant 记录；Web direct、Feishu direct/scheduler 与 Web scheduler 均有 assistant 记录。除两条 scheduler 产品化失败外，其余样本均正常收口；未见错投、敏感信息泄露或原始工具 JSON。
    - 05:04 CST Web scheduler `盘后美股复盘与SNDK/MU存储产业链日报` final 使用 `SNDK 当前价约 1,668.21 美元`、日高 `1,836.51`、日低 `1,658.08`、`MU 收盘约 979.30 美元` 等高风险数量级价格，并据此判断 SK Hynix / 存储链冲击。
    - 05:34 CST Feishu scheduler `美股收盘后跨市场复盘` final 使用 `MU 前收979.30美元，跌4.32%至937美元`、`SPY 前收754.95美元`、`QQQ 前收725.51美元`、`LITE 前收802.01美元` 等高风险数量级行情锚。
    - 06:31 CST Web scheduler `1亿美元AI科技组合每日跟踪` final 使用 `DELL 435.97`、`AMD 546.72`、`MRVL 243.27`、`MU 991.64`、`ARM 327.87` 等作为上一交易日基准，并据此计算组合市值和单日贡献。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入正式 scheduler final 和组合 / 投研判断。
    - 主体报告可读并完成，问题没有阻断调度 / 投递主功能链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-13 23:02-2026-07-14 03:01 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 23:02-03:01 CST 新增 10 个 user turn / 10 条 assistant final，均正常收口；assistant final 污染扫描未确认空回复、内部字段、原始工具 JSON、投递失败或敏感信息外泄。
    - 00:02 / 00:08 CST Feishu direct 对 DRAM / SKHY / MU 的强时效金融问答继续输出 `SKHY $152.75`、`MU $932.78` 等高风险数量级价格；因同时缺少可审计工具证据，主要补入强时效金融核验缺陷，本文档记录其行情 sanity check 侧信号。
  - `data/runtime/logs/web.log.2026-07-13`
    - 23:30 CST `闪迪关键事件心跳提醒` raw preview 使用 `SNDK 现价 $1,772.97`、previous close `$1,915.92`、日内区间 `$1,701.01-$1,800` 进入 `JsonTriggered` 判断。
    - 00:30 / 01:00 / 03:01 CST 多条 `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒` raw preview 继续使用 `SNDK $1,915.92`、`$1,706.62`、`$1,659.86` 等异常数量级价格。
    - 23:31 / 01:01 / 03:00 CST `持仓重大事件心跳提醒` 与 `Monitor_Watchlist_11` raw / deliver preview 使用 `MU $938.52`、`$929.16`、`$979-991`、`$923.31` 等高风险数量级价格进入判断。
    - 00:30 / 01:01 / 02:00 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒` deliver preview 使用 `AAOI $110-$119`、50 日均线 `$165.90`、52 周高点 `$233` 等行情锚。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 direct 金融答案、heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-13 19:00-23:02 CST 真实运行态继续出现同根异常 / 高风险价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 19:00-23:02 CST 新增 49 个 user turn / 60 条 assistant 记录，Feishu direct、Feishu scheduler、Web direct 与 Web scheduler 均有 assistant 终态；assistant final 污染扫描未确认空回复、内部字段、原始工具 JSON、投递失败或敏感信息外泄。
    - 20:01 / 20:46 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7` 在 A 股高景气产业链推演中继续把 `NVDA 210美元` 作为关键美股锚点进入判断。
    - 21:29 CST Feishu session `Actor_feishu__direct__ou_5f175714e91a60d34339460cdd1268f8fb` 写出 `INTC 最新价：109.84美元`，并说明此前 `28.71美元` 是旧数据；该数量级偏离常识，说明行情 sanity check 仍不稳定。
  - `data/runtime/logs/web.log.2026-07-13`
    - 23:00 CST `闪迪关键事件心跳提醒` raw / deliver preview 使用 `SNDK at $1,772.97`、previous close `$1,915.92` 等异常数量级价格并生成 `triggered` 提醒正文。
    - 同窗 heartbeat `deliver_preview` 共 32 条；异常或高风险价格仍进入 heartbeat 判断上下文、用户可见 preview 或 duplicate suppression 路径。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 scheduler final、heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有错投、投递失败、空回复、数据破坏或全渠道不可用；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-13 11:04-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 11:04-15:01 CST 新增 3 个 user turn / 3 条 assistant final，均正常成对收口；本窗未确认新的普通 direct / scheduler final 正式输出异常行情。
    - assistant final 污染扫描未确认空回复、内部字段、原始工具 JSON 或投递失败。
  - `data/runtime/logs/web.log.2026-07-13`
    - 11:30 / 13:00 CST `Monitor_Watchlist_11` deliver preview 继续使用 `MU $979.30` 等异常数量级价格作为阈值判断锚点。
    - 12:00 / 13:30 / 15:00 CST `闪迪关键事件心跳提醒` deliver preview 继续使用 `SNDK $1,915.92`、市值约 `$2,837亿` 等高风险行情锚。
    - 11:00-15:00 CST 多条 heartbeat preview 继续使用 `SPY $754.95`、`AAOI $119.92`、`MRVL $235.81`、`NBIS $219.65`、`STX $910.34` 等高风险行情锚进入判断。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-13 07:00-11:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 07:00-10:30 CST 新增 27 个 user turn / 27 条 assistant final，均正常成对收口；本窗新回退的 `cron_job_runs` 台账滞后问题独立于行情 sanity check。
    - assistant final 污染扫描未确认空回复、内部字段、原始工具 JSON 或投递失败。
  - `data/runtime/logs/web.log.2026-07-13`
    - 本窗 heartbeat preview 继续使用异常或高风险行情锚点进入判断。
    - 08:31 CST Feishu scheduler `闪迪(SNDK)每日行情与行业简报` final 继续写出 `SNDK 最新可核常规盘为7月9日收于 1,915.92 美元`，虽属于普通 scheduler final，但同根为异常行情 sanity check 缺口。
    - 10:30 CST `美股黄金坑信号心跳检测` deliver preview 继续使用 `SPY $754.95`、`QQQ $725.51`、`NVDA +8.18%` 等高风险行情锚点进入判断。
    - 10:30 / 11:00 CST `NVDA 关键事件心跳提醒` raw / deliver preview 使用 `NVDA $210.96`、PE `36.12x`、Market Cap `$5.11T`，并围绕错误周末 / 交易日口径进入判断。
    - 10:30 / 11:00 CST `光迅科技关键事件心跳提醒` deliver preview 使用 `¥232`、`¥226.69`、日内高低和异常数量级成交量 / PE 作为行情锚。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 scheduler final、heartbeat 判断上下文和部分 deliver preview。
    - 直聊、普通 scheduler 与 heartbeat runner 均有正常收口或受控 duplicate suppression；未见空回复、错投、投递失败或原始工具 JSON。该问题主要削弱投研质量和价格判断可信度，不影响主功能链路，因此维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 2026-07-13 03:00-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 03:00-07:02 CST 新增 9 个 user turn / 9 条 assistant final，均正常成对收口；本窗新登记的普通 scheduler 日期口径问题独立于行情 sanity check。
    - assistant final 污染扫描未确认空回复、内部字段、原始工具 JSON 或投递失败。
  - `data/runtime/logs/web.log.2026-07-12`
    - 本窗 heartbeat preview 继续使用异常或高风险行情锚点进入判断。
    - 07:00 CST `闪迪关键事件心跳提醒` deliver preview 继续使用 `SNDK $1,915.92`、Forward PE 和市值等高风险行情锚。
    - 07:00 CST `NVDA 关键事件心跳提醒` deliver preview 继续使用 `NVDA $210.96`、PE `36.12x`、Market Cap `$5.11T`，并围绕错误休市 / 交易日口径进入判断。
    - 03:00-07:02 CST 统计命中 `MU $979.30`、`NBIS $219.65` 等高风险锚点；多条随后进入 duplicate suppression 或 skipped/noop 路径。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-12 15:02-19:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 15:02-19:02 CST 新增 3 组 user / assistant，17:30 CST Feishu scheduler、17:49 CST Web direct、18:00 CST Web scheduler 均正常收口；本窗没有确认新的普通 direct / scheduler final 正式输出异常行情。
    - assistant final 未见空回复、内部字段、原始工具 JSON 或投递失败。
  - `data/runtime/logs/web.log.2026-07-12`
    - 本窗 32 条 heartbeat preview 继续使用异常或高风险行情锚点进入判断。
    - 15:30-19:00 CST `NVDA 关键事件心跳提醒` 多条 raw / deliver preview 使用 `NVDA $210.96`、PE `36.12x`、Market Cap `$5.11T` 等高风险行情锚，并在 `JsonMalformed` 或 duplicate suppression 路径中继续参与判断。
    - 16:00-19:00 CST `中际旭创关键事件心跳提醒` 多条 raw / deliver preview 使用 `¥1,093.98`、昨收 `¥1,194.9`、日跌幅 `-8.45%`、Forward PE `81.4x` 等高风险 A 股数量级数据。
    - 19:00 CST `Monitor_Watchlist_11` raw preview 继续使用 `MU $979.30` 对比 `MU <= $252.00` 判断未触发；`持仓重大事件心跳提醒` deliver preview 也继续使用 `MU $979.30` 作为行情锚。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-12 07:01-11:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 07:01-11:01 CST 新增 2 个 user turn / 2 条 assistant final，均为 Feishu scheduler 文章跟踪任务正常收口；本窗没有确认新的普通 direct / scheduler final 正式输出异常行情。
    - assistant final 未见空回复、内部字段、原始工具 JSON 或投递失败。
  - `data/runtime/logs/web.log.2026-07-12`
    - 08:00 / 11:00 CST `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒` 多条 raw / deliver preview 继续使用 `SNDK $1,915.92`，并围绕 50 日均线、Forward PE、市值或财报日期进入判断。
    - 11:00 CST `AAOI 1.6T 光模块心跳检测` raw preview 继续使用 `AAOI $119.92`、日高低和 timestamp 作为行情锚。
    - 08:00 / 11:00 CST `光迅科技关键事件心跳提醒` deliver preview 继续使用 `¥233.45`、`¥238.49`、`¥210-¥258.66` 等高风险 A 股数量级数据进入判断。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-12 03:02-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 03:02-07:02 CST 新增 3 个 user turn / 3 条 assistant final，均为 scheduler 触发后正常收口；本窗没有确认新的普通 direct / scheduler final 正式输出异常行情。
    - assistant final 污染扫描未命中空回复、内部字段、原始工具 JSON 或投递失败。
  - `data/runtime/logs/web.log.2026-07-11`
    - 06:00 CST `Monitor_Watchlist_11` raw preview 继续使用 `MU: $979.30` 对比 `MU <= $252.00` 判断未触发。
    - 06:00-07:01 CST `闪迪关键事件心跳提醒` / `持仓财报与重大新闻心跳提醒` / `存储板块关键事件心跳提醒` 多条 deliver preview 继续使用 `SNDK $1,915.92`、`AAOI $119.92` 作为行情锚。
    - 07:00 CST `中际旭创关键事件心跳提醒` deliver preview 继续使用 `¥1,093.98`、昨收 `¥1,194.9`、日跌幅 `-8.45%` 等高风险 A 股数量级数据。
    - 07:01 CST `美股黄金坑信号心跳检测` raw preview 使用 `SPY $754.95` 等异常或高风险数量级指数 ETF 价格进入判断。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-11 23:02-2026-07-12 03:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 23:02-03:02 CST 新增 3 个 user turn / 2 条 assistant final；本窗没有确认新的普通 direct / scheduler final 正式输出异常行情。
    - 本轮另新增 Web direct 连续 user turn 漏答 P2；该问题与行情 sanity check 缺口不同根因。
  - `data/runtime/logs/web.log.2026-07-11`
    - 03:00 CST `闪迪关键事件心跳提醒` deliver preview 继续使用 `SNDK $1,915.92`、日内 `+3.10%` 作为当前锚点。
    - 03:00 CST `AAOI 1.6T 光模块心跳检测` deliver preview 继续使用 `AAOI $119.92`、日高低 `$124.78/$113.10` 和成交量作为行情锚点。
    - 03:00 CST `NVDA 关键事件心跳提醒` deliver preview 使用 `NVDA $210.96`、`+$8.18 (+4.03%)`、PE `36.12x` 和市值 `$5.11T` 等高风险行情锚；`ORCL 大事件监控` deliver preview 使用 `ORCL $140.68`、`-2.45%` 与市值 `$4052 亿`。
    - 多条样本随后进入 duplicate suppression、skipped/noop 或普通完成路径，但异常或高风险价格已经参与 heartbeat 判断和用户可见 preview 生成。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-11 19:01-23:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 19:01-23:02 CST 新增 3 个 user turn / 3 条 assistant 记录；本窗未确认新的普通 direct / scheduler final 正式输出异常行情。
  - `data/runtime/logs/web.log.2026-07-11`
    - 19:30 / 20:00 / 20:30 / 21:00 / 21:30 / 22:00 / 23:00 CST `光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒` 多条 deliver preview 继续使用 `SNDK $1,915.92` 和 `AAOI $119.92` 作为行情锚。
    - 20:30 / 22:00 / 22:30 / 23:00 CST `光迅科技关键事件心跳提醒` 多条 deliver preview 使用 `¥233.45`、`¥238.49`、`¥210-¥258.66` 等高风险 A 股数量级数据进入判断。
    - 19:00 / 20:00 / 20:30 / 21:30 / 22:00 / 22:30 / 23:00 CST `中际旭创关键事件心跳提醒` deliver preview 使用 `¥1,093.98`、昨收 `¥1,194.9`、日跌幅 `-8.45%` 等明显高风险数量级数据进入判断。
    - 多条样本随后进入 duplicate suppression 或 skipped/noop 路径，但异常价格已经参与 heartbeat 判断和用户可见 preview 生成。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-11 15:01-19:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 15:01 CST 后没有新增本地 assistant final；本窗未确认新的普通 direct / scheduler final 正式输出异常行情。
  - `data/runtime/logs/web.log.2026-07-11`
    - 15:00-19:00 CST `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒` 多条 raw / deliver preview 继续使用 `SNDK $1,915.92` 和 `AAOI $119.92` 作为行情锚。
    - 15:00 / 15:30 / 16:00 / 17:00 / 17:30 / 18:30 CST `光迅科技关键事件心跳提醒` 多条 deliver preview 使用 `¥233.45`、`¥238.49`、`¥210-¥258.66`、Forward PE `181.3x` 等高风险 A 股数量级数据进入判断。
    - 16:00 / 18:30 / 19:00 CST `中际旭创关键事件心跳提醒` deliver preview 使用 `¥1,093.98`、昨收 `¥1,194.90`、日跌幅 `-8.45%` 等明显高风险数量级数据进入判断。
    - 18:00 CST `美股黄金坑信号心跳检测` deliver preview 把 `SPY $754.95`、`QQQ` 等指数 ETF 价格作为当前判断依据；同条随后被旧“无法创建监控”基线 duplicate suppressed。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常或高风险数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-11 11:01-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗只有 1 条 assistant final，为 12:00 CST Feishu 普通 scheduler `每日公司资讯与分析总结`，正常收口，未命中空回复、内部字段、原始工具 JSON 或投递失败；本窗未确认新的普通 direct / scheduler final 正式输出异常行情。
  - `data/runtime/logs/web.log.2026-07-11`
    - 12:30 CST `Monitor_Watchlist_11` raw preview 写出 `MU: $979.3` 并对比 `MU <= $252.00` 判断未触发；同条最终 `JsonMalformed + parse failure escalated`，说明异常价格仍进入阈值判断上下文。
    - 13:00 CST `存储板块关键事件心跳提醒` raw preview 写出 `SNDK: last verified ~$1,915.92`，并围绕 `2026-07-12` 错误时间口径进入 `JsonNoop` 判断。
    - 14:00 CST `持仓财报与重大新闻心跳提醒` deliver preview 继续使用 `SNDK $1,915.92`、50 日均线上方约 `+14.7%`、Forward PE 约 `59.2x`；随后被 duplicate suppression。
    - 15:00 CST `持仓财报与重大新闻心跳提醒` deliver preview 继续使用 `SNDK $1,915.92`、日内 `+3.10%`、50 日均线上方约 `+14.7%`、Forward PE `~59.2x`；同条未确认正式发送但已进入 deliver / duplicate suppression 判断链路。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-11 07:01-11:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗 3 条 assistant final 均正常收口，未命中空回复、内部字段、原始工具 JSON 或投递失败；本窗未确认新的普通 direct / scheduler final 正式输出异常行情。
  - `data/runtime/logs/web.log.2026-07-11`
    - 11:00 CST `Monitor_Watchlist_11` raw preview 写出 `MU: $979.30` 并对比 `MU <= $252.00` 判断未触发；同条还显示工具预算耗尽后直接用已有异常价格继续判断。
    - 11:00 CST `闪迪关键事件心跳提醒` deliver / duplicate preview 多次使用 `SNDK $1,915.92`、日内 `+3.10%` 等异常数量级行情作为新闻与行情锚点。
    - 11:00 CST `AI与科技持仓观察关键事件心跳提醒` deliver preview 使用 `STX $910.34`、`BE $244.61` 等高风险数量级行情，并声称其余多只标的均已扫描。
    - 11:00 CST `AAOI 1.6T 光模块心跳检测` 与 `持仓财报与重大新闻心跳提醒` deliver preview 使用 `AAOI $119.92`、`SNDK $1,915.92` 等异常或高风险价格进入 heartbeat 判断 / duplicate suppression 路径。
  - 判断：
    - 最新证据仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入 heartbeat 判断上下文和部分 deliver preview。
    - 本窗没有新的正式普通 final 异常价格样本，也未阻断直聊 / 调度 / 投递主链路；因此仍按质量性 `P3 / New`。该问题不影响主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-11 03:00-07:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 05:02 CST Web scheduler final `盘后美股复盘与SNDK/MU存储产业链日报` 正常收口，但围绕 SK Hynix ADR 首秀、SNDK/MU 和存储产业链继续输出强时效行情与估值判断；同窗未见空回复、内部字段或投递失败。
    - 06:01 CST Feishu scheduler final `每日美股盘后收盘复盘` 正常收口，但继续输出异常数量级指数：`纳斯达克综合指数 26,281.61`、`标普 500 7,575.39`、`道指 52,637.01`，并据此解释风险偏好、利率和科技股状态。
  - `data/runtime/logs/web.log.2026-07-10`
    - 06:00 CST `持仓重大事件心跳提醒` raw preview 使用 `MU $979.30`、`ARM $323.39` 等异常或高风险数量级价格作为判断上下文。
    - 06:00-07:00 CST `闪迪关键事件心跳提醒` deliver / duplicate preview 多次使用 `SNDK $1,915.92`、昨收 `$1,858.27`、日内区间 `$1,773.51-$1,946.84`、市值约 `$2,837 亿` 等异常数量级行情。
    - 07:00 CST `Monitor_Watchlist_11` raw preview 明确写 `MU $979.30` 并对比 `MU <= $252.00` 判断未触发；模型虽提示价格看起来过高，但异常价格仍进入 watchlist 判断上下文。
    - 07:00 CST `美股黄金坑信号心跳检测` raw preview 使用 `SPY 754.95`、`QQQ` 等异常数量级指数 ETF 价格，并进入 `JsonNoop` 判断。
  - 判断：
    - 最新样本仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入用户可见 scheduler final、heartbeat deliver preview 与阈值判断上下文。
    - 直聊、调度和 heartbeat runner 均有正常收口或受控 duplicate suppression；未见空回复、错投、投递失败或原始工具 JSON。该问题主要削弱投研质量和价格判断可信度，不影响主功能链路，因此维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 本轮 2026-07-10 19:02-23:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 19:42 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `核心观察池早间简报` assistant final 正常收口，但继续输出 `SNDK $1,844.97`、`MU $1,009.79`、`STX $917.41`、`WDC $590.13`、`GEV $1,079.18` 等异常或高风险数量级价格，并作为 25 支观察池击球区判断锚点。
    - 21:37 CST 同 actor 的 `科技核心股池 · 晚间击球区快报` assistant final 正常收口，但继续输出 `MU $1,009.79`、`SNDK $1,828.82`、`STX $917.41`、`WDC $590.13` 等异常数量级价格。
    - 23:01 CST 同 actor 的 `核心观察股池晚间快报` 已降级为“最新行情未完成稳定校验，不输出精确现价”，说明局部 guard 有止血表现，但 19:42 / 21:37 已送达样本仍证明同根链路未关闭。
  - cloud PostgreSQL `cloud_cron_job_runs`
    - 22:30 CST Web heartbeat `持仓重大事件心跳提醒` `run_id=35494` 落成 `completed + sent + delivered=1`，response preview 写 `MU 最新可得价格约 $978.94（昨收 $991.64）`，继续把异常数量级 MU 价格作为持仓触发判断依据。
    - 23:01 CST Web heartbeat `闪迪关键事件心跳提醒` `run_id=35528` raw / response preview 继续围绕 `SNDK 当前价格约 $1,845（昨收 $1,858）`、市值约 `$2,733 亿` 等异常数量级信息进行判断，虽然最终 duplicate suppressed 为 `noop`，仍说明异常行情进入 heartbeat 判断上下文。
  - 判断：
    - 最新样本仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入 Feishu scheduler final 与 heartbeat 判断链路。
    - 直聊、调度和 Web push 主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响直聊 / 调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-10 07:01-11:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 08:32 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34` 的 `闪迪(SNDK)每日行情与行业简报` assistant final 正常收口，但继续输出 `SNDK 7月9日收盘：1,858.27 美元`、前收 `1,727.18 美元`、盘中区间 `1,801.00 到 1,952.59 美元`、盘后 `1,887.99 美元`、市值 `2,751.9 亿美元`，并据此给出 1,900 / 1,700 美元交易观察位。
    - 08:32 CST Feishu session `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 正常收口，但在宏观 / AI 硬件早报中继续输出 `SNDK 1,858.27`、`MU 991.64`、`CIEN 462.34`、`AVGO 401.11`、`GEV 1,075.26` 等异常或高风险数量级价格，并据此判断存储链、光通信链和 AI 电力链强弱。
    - 09:02 CST Feishu scheduler session `Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a` 的 `美股与A股重点标的跟踪晨报` assistant final 正常收口，但继续写出 `MU 991.64 美元`、`SNDK 1858.27 美元`、BofA 目标价 `1550美元`、Wedbush 目标价 `2000美元`、Bernstein 目标价 `3000美元` 等异常或高风险数量级价格，并把这些数值作为重点结论和操作建议依据；同条 final 还出现 `<absolute-path>/` 占位符和标题拼接破损，本轮先作为单次格式观察，不新建独立缺陷。
    - 09:31 CST Discord scheduler final 也输出 `S&P 500 7,543.64`、`Nasdaq 26,206.89`、`Dow 52,487.41` 等异常指数数量级，并据此解释降息预期与科技股反弹。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 07:30-11:00 CST `Monitor_Watchlist_11` heartbeat raw preview 多次继续使用 `MU 991.64` 对比 `MU <= 252.00` 判断未触发；部分 raw preview 已自我提示 `MU at $991.64 seems very...`，但异常价格仍进入 watchlist 判断上下文。
  - 判断：
    - 最新样本仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入用户可见投研、组合估值、宏观判断和 heartbeat 阈值判断链路。
    - 直聊和调度主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响直聊 / 调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-10 03:01-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 04:32 CST Feishu final session `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 正常收口，但输出 `SNDK 1,896.53`、`MU 999.56`、`CIEN 463.93`、`AVGO 404.82` 等异常或高风险数量级价格，并据此判断存储链和 AI 硬件主线。
    - 05:02 CST Web direct session `Actor_web__direct__web-user-afc1cabadbf8` 的美股盘后复盘 final 正常收口，但继续输出 `S&P 500 7,543.64`、`Dow 52,487.41`、`Nasdaq Composite 26,206.89`、`SNDK 1,858.27`、`MU 991.64` 等异常或高风险数量级价格，并据此判断存储链和市场状态。
    - 05:31 / 06:01 / 07:01 CST Feishu scheduler final 继续输出 `标普500 7,543.64`、`纳指 26,206.89`、`道指 52,487.41`、`SNDK 1,858.27`、`MU 991.64`、`AMD 546.72`、`DELL 450.22` 等价格，并用于组合市值、单日贡献和市场归因。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 `Monitor_Watchlist_11` heartbeat raw preview 继续使用 `MU 990.34`、`991.64` 等异常数量级价格对比 `MU <= 252.00` 判断未触发；部分 raw preview 已自我提示“这个价格看起来不对”，但最终仍进入 watchlist 判断上下文。
  - 判断：
    - 最新样本仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入用户可见投研、组合估值和 heartbeat 阈值判断链路。
    - 直聊和调度主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响直聊 / 调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-09 23:02-2026-07-10 03:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 01:50 CST Web direct session `Actor_web__direct__web-user-400794904801` 的美股盘中行情分析 assistant final 正常收口，但继续输出明显异常或高风险数量级存储链价格：`MU 1023.72 美元`、`SNDK 1878.92 美元`，并基于这些数值判断存储链、DRAM ETF、KMEM 和板块轮动。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 03:00 CST `Monitor_Watchlist_11` heartbeat raw preview 继续使用 `MU 1008.33` 对比 `MU <= 252.00` 判断未触发；该样本未进入正式用户可见 final，但证明异常价格仍进入 heartbeat 判断上下文。
  - 判断：
    - 最新样本仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入用户可见投研和 heartbeat 阈值判断链路。
    - 直聊和调度主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响直聊 / 调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-09 19:02-23:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 20:00-21:35 CST 多条 Feishu / Web scheduler assistant final 正常收口，但继续输出异常或高风险数量级市场价格，并将其作为盘前风险、观察池纪律或板块判断锚点。
    - 代表样本包括 20:00 CST 多条美股大盘类 scheduler final 输出 `S&P 500 7,482.71`、`Dow 52,348.39`、`Nasdaq Composite 25,870.65` 等指数数量级；20:02 CST `美股与A股重点标的跟踪晚报` 写 `MU盘前涨约3.5%至982.05美元`，并把 `BofA Global Research` 的 Micron 目标价写成 `1550美元`；21:00 CST `OWALERT_PreMarket` 输出 `MU 1,011.04`、`SNDK 1,823.82`、`BE` 等高风险数量级价格并给出持仓 / 观察池动作；21:30 CST `彩票组合风险监控与买卖点提醒` 输出 `MU 1,011.04`、`LITE 745.98`、`BE 274.58` 等；21:35 CST `科技核心股池 · 晚间击球区快报` 输出 `MU $1,009.79`、`SNDK $1,828.82`、`STX $917.41`、`WDC $590.13` 等。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat 仍有 86 条 `noop + skipped_noop` 与 26 条 `execution_failed + skipped_error`，但本轮主证据已经进入正式 assistant final，不只是 heartbeat raw preview。
  - 判断：
    - 最新样本仍是同一行情源 / 数值 sanity check 缺口：异常数量级价格进入用户可见投研、组合纪律和市场判断链路。
    - 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-09 11:01-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续使用异常或高风险行情数值，并进入 `PlainTextSuppressed`、`PlainTextNoop` 或 `JsonNoop` 链路。
    - 代表样本包括 14:00 CST `Monitor_Watchlist_11` raw preview 使用 `MU 948.8` 对比 `MU <= 252.00` 判断未触发；14:30 CST 同 job raw preview 写 `MU: $948.8, trigger <= $252.00 -> NOT triggered`；15:00 CST 同 job raw preview 继续写 `MU: Current $948.80, Trigger <= $252.00 -> NOT triggered`。这些数量级仍明显偏离常识区间，属于同一 StockAnalysis / 行情源异常价格问题。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗按真实 `timestamp` 有 4 条 assistant final，均正常收口；未确认新的正式用户可见 final 直接输出 `MU 948.80` 或同类异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-09 07:00-11:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 07:02 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3` 的 `美股持仓收盘后早报` assistant final 正常收口，但继续输出 `SNDK 1727.18`、`MU 948.80`、`DELL 432.01`、`GOOGL 361.92` 等异常或高风险数量级价格，并据此计算组合市值、浮盈和单日贡献。
    - 08:33 CST session `Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34` 的 `闪迪(SNDK)每日行情与行业简报` final 正常收口，但把 `SNDK 7月8日收盘：1,727.18 美元`、`盘后参考约 1,746.00 美元` 作为核心行情锚，并基于这些数值给出评级与交易区间判断。
    - 09:01 CST `核心观察池早间简报` final 继续输出 `MU $948.80`、`SNDK $1,727.18`、`STX $860.02`、`WDC $550.30`、`GEV $1,070.99` 等异常或高风险数量级价格，并用于击球区 / 财报日期简表。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw / 判断上下文仍有结构化失败与错误日期样本；本单主要记录异常价格进入正式 scheduler final 的证据。
- 本窗已有多条 scheduler final 正式落库样本，不只是 heartbeat raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-09 03:02-07:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 05:02 CST Web direct session `Actor_web__direct__web-user-afc1cabadbf8` 的美股盘后复盘 assistant final 正常收口，但继续输出明显异常或高风险数量级市场价格：`S&P 500 7,482.71`、`Dow 52,348.39`、`Nasdaq Composite 25,870.65`，并在 MU 段写出 `MU 小幅上涨约 0.3% 至 941.44 美元`。
    - 同条 final 基于这些数值判断存储链、指数和宏观压力，说明异常行情仍进入用户可见市场判断链路；该样本没有命中 `data_fetch` / `quote_short` / `StockAnalysis` 文案外露，属于行情数值质量链路而不是来源名净化链路。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 03:30 CST `DRAM 心跳监控` 成功送达并使用 `DRAM $61.565` 与错误数据日期作为触发依据；该样本主要归入时间口径缺陷，异常行情链路仍需继续共同关注。
- 本窗已有 Web direct final 正式落库样本，不只是 heartbeat raw preview；价格 sanity check 仍未覆盖 direct 投研 / scheduler / heartbeat 运行路径。
- 会话主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响直聊 / 调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-08 23:00-2026-07-09 03:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 23:01 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `核心观察股池晚间快报` assistant final 正常收口，但继续输出多只明显异常或高风险数量级价格：`MU 936.38`、`SNDK 1,657.92`、`STX 834.43`、`WDC 541.46`、`GEV 1,079.18`、`AMD 520.41`、`BE 271.18`。
    - 同条 final 把这些数值作为 25 支观察池当前价格锚点，继续用于击球区 / 财报日期简表，说明异常行情仍进入用户可见市场判断链路。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续出现时间戳、错误时间或行情口径混用信号；本单主要记录异常价格进入正式 scheduler final 的证据。
- 本窗已有 scheduler final 正式落库样本，不只是 heartbeat raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-08 19:01-23:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 21:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5fe09f5f16b20c06ee5962d1b6ca7a4cda` 的 `晚9点盘前推演(XME及加密ETF)` assistant final 正常收口，但继续把多只明显异常或高风险数量级 ETF 价格作为盘前焦点锚点：`SPY 743.41`、`QQQ 703.38`、`DIA 524.25`、`IWM 292.61`、`XME 101.60`。
    - 同条 final 基于这些价格判断 `今晚盘前定调：风险偏好继续降级`，说明异常行情仍进入用户可见市场判断链路。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续出现异常行情、时间戳或错误时间口径混用信号，并进入 `JsonNoop`、`PlainTextSuppressed`、`JsonMalformed`、`JsonUnknownStatus` 等链路；本单主要记录异常价格进入正式 scheduler final 的证据。
- 本窗已有 scheduler final 正式落库样本，不只是 heartbeat raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-08 15:03-19:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续使用异常或高风险行情数值，并进入 `PlainTextSuppressed`、`PlainTextNoop` 或结构化失败链路。
    - 代表样本为 16:00 CST `Monitor_Watchlist_11` raw preview 使用 `MU 938.38` 对比 `MU <= 252.00` 触发阈值并判断未触发；该数值仍明显高于常识区间，属于同一 StockAnalysis / 行情源异常数量级问题。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗 10 条 assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 `MU 938.38` 或同类异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-08 11:03-15:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续使用异常或高风险行情数值，并进入 `JsonNoop`、`PlainTextSuppressed` 或结构化失败链路。
    - 代表样本包括 13:00 CST `Monitor_Watchlist_11` raw preview 使用 `MU 938.38` 对比 `MU <= 252.00` 触发阈值并判断未触发；13:30 CST 同 job 继续使用 `HIMS 36.17`、`MU 938.38`、`BE 269.57` 等行情数值做 watchlist 阈值判断。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗 8 条 assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 `MU 938.38` 或同类异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-08 07:00-11:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续使用异常或高风险行情数值，并进入 `JsonNoop`、`PlainTextNoop` 或结构化失败链路。
    - 代表样本为 07:30 CST `Monitor_Watchlist_11` raw preview 使用 `MU 938.38` 对比 `MU <= 252.00` 触发阈值并判断未触发；该数值仍明显高于常识区间，属于同一 StockAnalysis / 行情源异常数量级问题。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗 23 条 assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 `MU 938.38` 或同类异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-07 19:02-23:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 21:35 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `科技核心股池 · 晚间击球区快报` assistant final 正常收口，但继续输出多只明显异常或高风险数量级价格：`MU 916.32`、`SNDK 1,590.42`、`STX 816.59`、`WDC 537.01`、`GEV 1,046.01`、`AMD 510.32`、`BE 277.05`、`CRDO 246.02`、`INTC 111.45`。
    - 同条 final 基于这些价格判断 `MU、SNDK、AMD、INTC、CRDO、STX、WDC、BE 仍明显偏离纪律区间，追高风险回报不佳`，说明异常行情仍进入用户可见纪律区间判断。
    - 23:00 CST 同 actor 的 `核心观察股池晚间快报` 已因最新行情与财报日期未完成稳定校验而全部标注 `当前价格待确认`，说明局部 sanity guard 有止血表现，但 21:35 已送达样本仍证明同根链路未关闭。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续使用异常或高风险行情数值；代表样本包括 19:30 / 20:00 / 20:30 / 21:00 CST `Monitor_Watchlist_11` raw preview 持续使用 `MU 984.75` 判断未触发阈值。
- 本窗已有 scheduler final 正式落库样本，不只是 heartbeat raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-07 15:00-19:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续使用异常或高风险行情数值，并进入 `PlainTextSuppressed`、`PlainTextNoop`、`JsonMalformed` 等链路。
    - 代表样本包括 15:30 / 16:00 CST `Monitor_Watchlist_11` raw preview 使用 `MU 984.75` 判断未触发阈值；16:00 CST 同 job 同时列出 `BE 294.93`、`AMAT 593.19`、`CAMT 141.84`、`MP 132.88` 等明显偏离常识或高风险数量级的价格；18:30 CST `TSLA 正负触发条件心跳监控` raw preview 使用 `TSLA $419.77`、`Market Cap $1.576 trillion`、`PE 231.92` 等上下文做触发判断。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗 6 条 assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 MU `$984.75` 或同类异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-07 11:02-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 11:25 CST Web direct session `Actor_web__direct__web-user-3162d236bd51` 用户在新增 AMAT / CAMT 持仓后询问“请给出目前持仓总盈亏”，assistant final 正常收口。
    - 同条 final 继续使用明显异常数量级价格作为持仓估值锚，包括 `MU 984.75`、`AMAT 592.79`、`CAMT 141.84`，并据此计算 `总成本 16,476.40`、`当前市值 18,195.72`、`总盈亏 +1,719.32`、`总收益率 +10.44%`。
    - 新增持仓 AMAT / CAMT 本轮已被用户明确录入成本 `700` / `170`，final 虽单项列出 AMAT `-321.63`、CAMT `-844.80`，但总组合结论仍被 MU 等异常价格强行拉成正收益，说明异常行情已进入 Web direct 持仓盈亏核算链路。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文仍继续使用异常或高风险价格，并进入 `PlainTextSuppressed`、`PlainTextNoop`、`JsonMalformed`、`JsonUnknownStatus` 等链路；本单主要记录异常价格进入正式 Web direct final 的证据。
- 本窗已有 Web direct final 正式落库样本，不只是 heartbeat raw preview；价格 sanity check 仍未覆盖 scheduler / heartbeat / direct 投研与持仓估值运行路径。
- 会话主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响直聊 / 调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-07 03:02-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 07:00 CST Feishu scheduler `美股持仓收盘后早报` assistant final 正常收口，但正式输出 `SNDK 1744.43`、`AMD 552.05`、`DELL 411.51`、`ARM 322.24`、`GOOGL 366.46` 等明显异常数量级价格。
    - 同条 final 继续基于这些价格计算组合市值约 `83,165` 美元、浮盈约 `23,623` 美元、单日变化约 `+1,983` 美元，并给出 DRAM / AMD / DELL / RKLB 等个股贡献归因。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗普通 scheduler 6 条均为 `completed + sent + delivered=1`，Feishu direct / scheduler 6 个 user turn 均以 assistant final 收口。
    - heartbeat 判断上下文仍有 27 条 `execution_failed + skipped_error`，覆盖 `PlainTextSuppressed`、`JsonMalformed`、`JsonUnknownStatus` 等结构化失败样本；本单仅记录异常行情进入正式 final 与判断上下文的持续证据。
- 本窗已有 scheduler final 正式落库样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-06 23:04-2026-07-07 03:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续出现异常或高风险行情数值，并进入 `PlainTextNoop`、`PlainTextSuppressed`、`JsonMalformed` 等链路。
    - 代表样本包括 01:30 CST `Monitor_Watchlist_11` raw preview 使用 `MU $987.08` 判断未触发阈值，03:00 CST 同 job 使用 `MU $999.21` 判断未触发阈值；`DRAM 心跳监控` 02:30 CST 使用 `DRAM current price: $64.6999`、`Previous close: $60.63` 等链路内价格；`TSLA 正负触发条件心跳监控` 多轮 raw preview 使用 `$414-$417` 与异常高 PE / market cap 上下文做触发判断。
  - `data/sessions.sqlite3`
    - 本窗普通 scheduler / direct assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 MU `$999.21` 或同类异常价。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-06 15:02-19:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续检出异常数量级价格或价格 / 时间戳混用信号，代表样本包括 15:30 / 16:00 CST `Monitor_Watchlist_11` 使用 `MU 975.56`、`RKLB 100.46` 等价格判断未触发阈值；15:30-19:00 CST `DRAM 心跳监控` 多次使用 `DRAM 60.63` 和 `1783022401` 判断触发状态；16:30 / 18:00 / 18:30 CST `持仓重大事件心跳检测` 使用 `ASTS 85.13`、`RKLB 100.46`、`TEM 60.27` 等批量行情；17:30-19:00 CST `AAOI 1.6T 光模块心跳检测` 围绕 `AAOI 120.95` 或历史 watchlist 价格做判断。
  - `data/sessions.sqlite3` / `session_messages`
    - 同窗 3 条 assistant final 均正常收口；未确认新的正式用户可见 final 直接输出 `MU 975.56`、`SNDK 1745.00`、`SPY 744.78`、`QQQ 712.60` 等异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-06 07:02-11:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 08:30 CST Feishu scheduler `闪迪(SNDK)每日行情与行业简报` assistant final 正常收口，但正式输出 `SNDK 最新完整交易日为 7月2日美股，收跌 14.13% 至 1,745.00 美元`，并继续列出 `前收 2,032.22`、盘后 `1,762.07`、市值约 `2,584 亿美元` 等异常数量级价格。
    - 09:00 CST Feishu scheduler `早9点市场复盘(XME及加密ETF)` assistant final 正常收口，但正式输出 `SPY：744.78`，继续把明显异常 ETF 数值作为市场锚。
    - 07:02 / 08:32 / 08:45 / 09:00 多条 Feishu scheduler final 继续命中 `SNDK 1745.00`、`MU 975.56`、`DRAM 60.63`、`AMD 517.82`、`RKLB 100.46` 等异常数量级价格或其派生组合判断。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat 另有 104 条运行记录；raw preview / delivered preview 继续围绕 `RKLB $100.46`、`DRAM $60.63`、`MU $975.56`、`1783022400/1783022401` 等价格 / 时间戳混用信号判断触发状态。
- 本窗仍有 scheduler final 正式落库样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-06 03:01-07:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `session_messages`
    - 04:30 CST Feishu scheduler `OWALERT_PostMarket` assistant final 正常收口，但正式输出 `QQQ 712.60`、`SPY 744.78`、`SNDK 1,745.00`、`COHR 333.36`、`CIEN 422.46`、`BE 270.89`、`MU 975.56` 等明显异常数量级价格，并据此判断 AI 硬件与动量股去拥挤。
    - 05:01 CST Feishu scheduler `科技成长赛道大盘极值与情绪监控` assistant final 正常收口，但正式输出 `QQQ 712.60`、`ARKK 81.25`、`SMH 592.29`、`IGV 93.57` 等异常 ETF / 指数价格口径，并继续据此判断未触发极值信号。
    - 07:02 CST Feishu scheduler `美股持仓收盘后早报` assistant final 正常收口，但正式输出 `DRAM 60.63`、`SNDK 1745.00`、`DELL 394.29`、`MU 975.56`、`AMD 517.82`、`AAOI 120.95`、`COHR 333.36`、`RKLB 100.46` 等异常数量级价格，并据此计算组合市值、单日变化和主要拖累。
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat 另有 107 条运行记录，其中 raw preview / 判断上下文继续围绕 `1783022401`、`RKLB $100.46`、`DRAM $60.63`、`MU $975.56` 等价格 / 时间戳混用信号判断触发状态。
- 本窗已有多条 scheduler final 正式落库样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-05 23:01-2026-07-06 03:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续检出 9 条异常行情或价格 / 时间戳混用信号，代表样本包括 `RKLB异动监控` 使用 `RKLB $100.46` 与 `1783022400`，`DRAM 心跳监控` 使用 `DRAM $60.63` 与 `1783022401`，`Monitor_Watchlist_11` 继续使用 `MU $975.56` 判断未触发阈值。
    - 23:01 CST Feishu scheduler `核心观察股池晚间快报` assistant final 正常收口，但正式输出 `MU $977.00`、`SNDK $1,762.07`、`STX $826.00`、`WDC $545.00` 等明显异常数量级价格，并据此给出击球区纪律说明。
  - 本窗已有 scheduler final 正式落库样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

- 本轮 2026-07-05 19:02-23:06 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续检出 26 条异常行情或价格 / 时间戳混用信号，代表样本包括 `RKLB异动监控` 使用 `RKLB $100.46` 与 `1783022400`，`DRAM 心跳监控` 使用 `DRAM $60.63`，`Monitor_Watchlist_11` 继续使用 `MU $975.56` 判断未触发阈值，`持仓重大事件心跳检测` 使用 `RKLB $100.46` 等批量行情。
    - 21:35 CST 与 23:00 CST Feishu scheduler `科技核心股池 / 核心观察股池` final 正常收口，继续写出 `专用行情链路本轮未取得可用返回，已用公开行情页补充校验`，但未检出上一轮 `公开行情页.com` 占位链接。
  - `data/sessions.sqlite3`
    - 本窗普通 scheduler / direct assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 MU `$975.56`、SPY `$744.78`、QQQ `$712.60` 或 `$1,745`。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

- 本轮 2026-07-05 11:01-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续检出 MU `$975.56`、SPY `$744.78`、QQQ `$712.60`、`$1,745` 或 `178302240` 相关异常行情 / 时间戳信号 20 条，并进入 `JsonNoop`、`PlainTextNoop`、`PlainTextSuppressed`、`JsonUnknownStatus` 或 `JsonMalformed` 链路。
    - 代表样本包括 11:01 / 11:31 / 12:00 / 12:30 / 15:00 CST `DRAM 心跳监控` 继续使用 MU `$975.56` 或 `1783022401` 做判断，13:00 / 13:30 / 14:00 / 14:30 / 15:00 CST `Monitor_Watchlist_11` 继续围绕同类异常价格判断未触发阈值。
  - `data/sessions.sqlite3`
    - 本窗普通 scheduler / direct assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 MU `$975.56`、SPY `$744.78`、QQQ `$712.60` 或 `$1,745`。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-05 11:01 CST）

- 本轮 2026-07-05 07:02-11:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续检出 7 条 MU `$975.56` 异常数量级价格，并进入 `JsonNoop`、`PlainTextNoop` 或 `JsonUnknownStatus` 链路。
    - 代表样本包括 07:30、08:00、08:30、09:00、09:30、10:00、10:30 CST `Monitor_Watchlist_11` raw preview 继续使用 MU `$975.56` 判断未触发阈值。
  - `data/sessions.sqlite3`
    - 本窗普通 scheduler / direct assistant final 均正常收口；未确认新的正式用户可见 final 直接使用 MU `$975.56` 或同类异常价。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-05 07:04 CST）

- 本轮 2026-07-05 03:02-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续检出 7 条 MU `$975.56` 异常数量级价格，并进入 `JsonNoop`、`PlainTextNoop` 或结构化失败链路。
    - 代表样本包括 `Monitor_Watchlist_11`、`持仓重大事件心跳检测` 等 heartbeat 判断上下文继续围绕同一异常价格做阈值 / 事件判断。
  - `data/sessions.sqlite3`
    - 本窗普通 Feishu scheduler 2 条 assistant final 均正常收口；未确认新的正式用户可见投资建议直接使用 MU `$975.56`。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-05 03:02 CST）

- 本轮 2026-07-04 23:02-2026-07-05 03:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3` / `cron_job_runs`
    - 同窗 heartbeat raw preview / 判断上下文继续出现 MU `$975.56`、SPY `$744.78`、QQQ `$712.60`、SNDK `$1,745` 等明显异常数量级价格，并进入 `JsonNoop`、`PlainTextNoop`、`JsonTriggered` 或结构化失败链路。
    - 代表样本包括 23:30 CST `Monitor_Watchlist_11` raw preview 使用 MU `$975.56`，23:30 CST `美股黄金坑信号心跳检测` raw preview 使用 SPY `$744.78` / QQQ `$712.60`，23:30 CST `闪迪关键事件心跳提醒` raw preview 使用 SNDK `$1,745`。
  - `data/sessions.sqlite3`
    - 本窗普通 scheduler / direct final 多数正常收口；未确认新的正式用户可见投资建议直接使用上述异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-04 19:04 CST）

- 本轮 2026-07-04 15:02-19:04 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-04` / `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview / 判断上下文继续出现 MU `$975.56`、SPY `$744.78`、QQQ `$712.60` 等明显异常数量级价格，并进入 `JsonNoop`、`JsonEmptyStatus`、`PlainTextNoop` 或结构化失败链路。
    - 结构化计数中同窗可检出 MU `$975.56` 5 条、SPY `$744.78` 5 条、QQQ `$712.60` 5 条异常价格信号。
  - `data/sessions.sqlite3`
    - 本窗按真实 `timestamp` 没有新的 assistant final；未确认新的正式用户可见投资建议直接使用上述异常价格。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-03 23:02 CST）

- 本轮 2026-07-03 19:02-23:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 21:35 CST Feishu scheduler `科技核心股池 · 晚间击球区快报` assistant final 正常收口，但正式输出 MU `$975.56`、SNDK `$1,745.00`、STX `$820.16`、WDC `$539.00`、AMD `$517.82`、INTC `$120.35`、BE `$270.89` 等明显异常数量级价格，并据此给出击球区纪律说明。
    - 23:00 CST Feishu scheduler `核心观察股池晚间快报` assistant final 再次输出同一批异常数量级价格。
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview / 判断上下文继续出现 SNDK `$1,745`、SNDK previous close `$2,032.22`、MU `$975.56` 等异常价格，并进入 `JsonNoop`、`PlainTextSuppressed`、`JsonTriggered`、`JsonUnknownStatus` 或结构化失败链路。
- 本窗已有 scheduler final 正式落库样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

## 最新进展（2026-07-03 19:02 CST）

- 本轮 2026-07-03 15:10-19:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 15:10-19:01 CST heartbeat raw preview / 判断上下文继续出现 SNDK `$1,745`、SNDK previous close `$2,032.22`、MU `$975.56` 等明显异常数量级价格，并进入 `JsonNoop`、`PlainTextSuppressed`、`JsonTriggered` 或结构化失败链路。
    - 结构化计数中同窗可检出 SNDK 66 条、MU 61 条、`$1,745` 11 条、`$2,032.22` 8 条、`$975.56` 6 条异常价格信号。
  - `data/sessions.sqlite3`
    - 本窗只有 18:00 Web scheduler `美股盘前 X 英文帖` 1 条正式 assistant final，正常收口；该 final 未直接把上述异常价格作为用户可见投资建议。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-03 15:10 CST）

- 本轮 2026-07-03 11:00-15:10 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-03`
    - 11:00-15:10 CST heartbeat raw preview / 判断上下文继续出现 SNDK `$1,745`、SNDK previous close `$2,032.22` 等明显异常数量级价格，并进入 `JsonNoop`、`PlainTextSuppressed`、`JsonTriggered` 或结构化失败链路。
    - 结构化计数中同窗可检出 SNDK 47 条、`$1,745` 22 条、`$2,032.22` 15 条异常价格信号。
  - `data/sessions.sqlite3`
    - 本窗 3 个 Feishu direct user turn 与 3 条 assistant final 均正常收口。
    - 其中 11:07 CST A 股存储链 reply 与 14:59 CST 韩股 reply 已出现用户可见强时效异常价格样本，归入 `feishu_direct_storage_price_unverified_before_tool_complete.md`；本单仅记录 heartbeat / scheduler 异常价格继续进入判定上下文。
- 本窗 heartbeat 异常价格主要停留在 raw preview、未命中或结构化失败路径；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-03 11:05 CST）

- 本轮 2026-07-03 07:00-11:05 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-03`
    - 07:00-11:05 CST heartbeat raw preview / 判断上下文继续出现 SNDK `$1,745`、SNDK previous close `$2,032.22`、MU `$975.56`、WDC `$539` 等明显异常数量级价格，并进入 `JsonNoop`、`PlainTextSuppressed`、`PlainTextNoop` 或结构化失败链路。
    - 结构化计数中同窗可检出 SNDK 7 条、MU 10 条、WDC 1 条异常价格信号。
  - `data/sessions.sqlite3`
    - 本窗 7 个 user turn 与 7 条 assistant final 均正常收口；未确认新的正式 assistant final 直接把上述异常价格作为用户可见投资建议。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径，未确认新的正式送达成功样本；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-03 03:02 CST）

- 本轮 2026-07-02 23:02-2026-07-03 03:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 23:30 CST Web `闪迪关键事件心跳提醒` raw preview 继续使用 SNDK `$1,813.20` 等明显异常数量级行情。
    - 00:30-03:00 CST `闪迪关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`Monitor_Watchlist_11` 等 heartbeat 判断上下文继续出现 SNDK `$2,032.22`、SNDK `$1,710.70`、MU `$956.08` 等异常价格，并进入 `JsonNoop`、`JsonTriggered`、`PlainTextSuppressed` 或执行失败链路。
  - `data/sessions.sqlite3`
    - 本窗只有 02:45 CST Feishu direct `cohr估值分析` 1 条正式 assistant final，正常收口；该 final 给出 COHR 精确行情 / 估值和占位式来源域名，另归入工具 / 来源口径外露文档；本单仅记录同根异常行情数值继续进入 heartbeat 判定上下文。
- 本窗异常价格主要停留在 heartbeat raw preview、未命中或结构化失败路径，未确认新的正式送达成功样本；调度 / 投递主链路没有因该问题被阻断。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-02 23:03 CST）

- 本轮 2026-07-02 19:02-23:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 20:00 CST Web scheduler `盘前美股要闻与持仓研报评级日报` assistant final 正常收口，但继续写出 MU `1,032.28`、盘前 `1,007.88` 等明显异常数量级价格。
    - 21:35 CST Feishu scheduler `科技核心股池 · 晚间击球区快报` assistant final 正式输出 MU `$1,054.23`、SNDK `$2,014.80`、STX `$905.80`、WDC `$599.87` 等异常数量级价格。
    - 22:55 CST Web direct 用户询问 SNDK 建仓点，assistant final 正常收口，但把 SNDK 最新口径写成约 `1887` 美元、前收 `2032.22` 美元，并据此给出 `1880-1900` 小仓观察、`1500-1750` 主建仓区等操作型区间。
    - 23:00 CST Feishu scheduler `核心观察股池晚间快报` assistant final 又输出 MU `$993.50`、SNDK `$1,841.00`、STX `$857.33`、WDC `$562.01` 等异常数量级价格。
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview / 判断上下文继续出现 MU `1032.28`、SNDK `$2,032.22` 等异常价格，并进入 `Monitor_Watchlist_11`、`闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒` 等判定上下文。
- 本窗已有 scheduler final 与 Web direct 投资问答正式落库样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat / direct 投研运行路径。
- 报告和直聊主链路均正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

## 最新进展（2026-07-02 19:03 CST）

- 本轮 2026-07-02 15:01-19:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 15:30 CST Web `闪迪关键事件心跳提醒` raw preview 继续使用 SNDK `$2,032.22`、日内跌幅 `-10.62%` 等明显异常数量级行情。
    - 同窗 `持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒` 等 heartbeat 判断上下文继续出现 SNDK `$2,032.22`、MU `1032.28` 等异常价格，并进入 `JsonNoop`、`JsonUnknownStatus` 或跳过发送链路。
  - `data/sessions.sqlite3`
    - 本窗只有 18:00 Web scheduler `美股盘前 X 英文帖` 正常 final；没有新的正式 assistant final 直接把上述异常价格作为用户可见投资建议。
- 本窗异常价格继续进入 heartbeat 判定上下文，但会话/投递主链路没有因该问题被阻断，未见错对象、空回复或原始工具 JSON。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-02 15:01 CST）

- 本轮 2026-07-02 11:01-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 12:45 CST Feishu direct 用户询问 DRAM 长 call 选择，assistant final 正常收口，但把 MU 7 月 1 日收盘价写成 `1032.28` 美元，并基于该价格、跌幅和“未拿到完整实时期权链”的口径继续给出长 call 结构建议。
    - 14:33-14:44 CST Feishu direct 围绕 Meta 出租算力、存储与海力士上市影响连续问答，均正常收口；未见空回复、错投或 raw tool JSON，但仍处在强时效金融判断场景。
  - `data/runtime/logs/acp-events.log`
    - 12:46 CST 同轮 search query 直接携带 `MU stock price July 1 2026 close Micron 1032.28 10.6%`，说明异常价格已被带入检索 / 校验链路，而不是只在最终文案中偶发写错。
  - `data/runtime/logs/hone_cli_screen.log`
    - 11:30-15:01 CST heartbeat raw preview / 判断上下文继续出现 MU `1032.28`、SNDK `2032.22`、`2273.73` 等异常数量级价格，并进入 `Monitor_Watchlist_11`、`持仓财报与重大新闻心跳提醒` 等判定上下文。
- 本窗异常价格已经进入 Feishu direct 用户可见投资建议和 heartbeat 判定上下文，但会话/投递主链路正常收口，未见错对象、空回复或系统失败。
- 因该问题仍主要影响行情质量和投资建议可信度，不阻断功能链路，因此继续按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-07-02 07:02 CST）

- 本轮 2026-07-02 03:02-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 07:00 CST Feishu `美股持仓收盘后早报` assistant final 正常收口，但正式输出 DRAM `65.86`、AMD `540.88`、MU `1032.28`、SNDK `2032.22`、DELL `425.25`、ARM `337.47` 等明显异常数量级行情。
    - 同条 final 继续基于这些价格计算组合约 `-6.48%`、单日亏损约 `6004` 美元、MU/DRAM/SNDK 合计约 `40.3%`、AI 硬件相关合计约 `79.1%`，并给出多个次日观察位。
  - `data/runtime/logs/web.log.2026-07-01`
    - 同窗 heartbeat raw preview / 判断上下文继续出现 SNDK `$2,058.34`、`$2,032.22` 等异常数量级价格信号。
- 本窗已有 Feishu scheduler assistant final 正式落库样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径。
- 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3 / New`。该问题不影响调度 / 投递主功能链路，因此不升级为 P2/P1，不创建 GitHub Issue。

## 最新进展（2026-07-02 03:03 CST）

- 本轮 2026-07-01 23:01-2026-07-02 03:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-01`
    - 03:00 CST `Monitor_Watchlist_11` raw preview 继续使用 MU `$1045.31`、BE `$290.62` 等异常数量级价格参与阈值判断。
    - 03:01 CST Web `存储板块关键事件心跳提醒` raw preview 继续写出 SNDK `$2,018.69`、前收 `$2,273.73`、52 周高点 `$2,354.39` 等异常价格口径。
    - 03:01 CST Web `闪迪关键事件心跳提醒` 生成 `JsonTriggered + deliver_preview`，送达预览继续写出 SNDK 收 `$2,017.67`、前收 `$2,273.73`。
  - `data/sessions.sqlite3` 本窗没有新的真实 assistant final timestamp；本条证据以 runtime heartbeat 日志和 deliver preview 为准。
- 本窗已有 Web heartbeat deliver preview 样本，但未确认最终移动端/频道送达；调度 / 投递主链路没有因该问题被阻断。异常价格仍进入 function-calling 结果、heartbeat 判定上下文与送达预览，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪，非 P1。

## 最新进展（2026-07-01 23:02 CST）

- 本轮 2026-07-01 19:06-23:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 21:35 CST `科技核心股池 · 晚间击球区快报` assistant final 正式输出 MU `$1,068.82`、SNDK `$2,062.00`、STX `$889.01`、WDC `$594.77`、AMD `$552.95` 等异常数量级价格。
    - 23:00 CST `核心观察股池晚间快报` assistant final 又输出 MU `$1,056.50`、SNDK `$2,034.95`、STX `$900.83`、WDC `$591.83`、AMD `$551.95` 等异常数量级价格。
  - `data/runtime/logs/web.log.2026-07-01`
    - 同窗 heartbeat raw preview / 判断上下文继续出现 SNDK、MU 等异常价格，并进入 `PlainTextSuppressed`、`PlainTextNoop` 或正式 final 上下文。
- 本窗已有 Feishu scheduler assistant final 正式落库样本，但任务主体仍正常收口，未见错投、空回复或投递主链路阻断；问题仍表现为价格 sanity check 未覆盖当前 scheduler / heartbeat 运行路径，按质量性 `P3 / New` 继续跟踪，非 P1。

## 最新进展（2026-07-01 15:03 CST）

- 本轮 2026-07-01 11:02-15:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-01`
    - 多条 heartbeat raw preview / 判断上下文继续围绕 SNDK / MU 等异常数量级价格生成判断。
    - 11:30 CST `闪迪关键事件心跳提醒` raw preview 与 deliver preview 继续写出 SNDK `$2,273.73`、日内高点 `$2,280.52`、52 周高点 `$2,354.39`；14:31 CST `存储板块关键事件心跳提醒` 也生成含 SNDK `$2,273.73` 的 deliver preview。
    - `Monitor_Watchlist_11` raw preview 在 11:30-14:30 CST 多次继续使用 MU `$1,154.29` 作为当前价参与阈值判断。
  - `data/sessions.sqlite3` 本窗唯一真实新会话是 Web direct 持仓录入，正常收口，未涉及观察池 scheduler 输出；本条证据以 runtime 日志为准。
- 本窗已有 Web heartbeat deliver preview 样本，但未确认最终移动/频道送达；调度 / 投递主链路没有因该问题被阻断。异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-07-01 11:03 CST）

- 本轮 2026-07-01 07:02-11:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 多条 heartbeat raw preview / 判断上下文继续围绕 SNDK / MU / WDC 等异常数量级价格生成判断。
    - 07:00-09:00 CST `持仓财报与重大新闻心跳提醒`、`Monitor_Watchlist_11`、`存储板块关键事件心跳提醒` 等仍可见 SNDK `$2,273.73`、MU `$1,154.29`、WDC `$638.72` 等异常价格口径进入 raw preview。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗唯一 Feishu direct assistant final 正常收口，未涉及观察池 scheduler 输出；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-07-01 07:01 CST）

- 本轮 2026-07-01 03:01-07:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-30`
    - 多条 heartbeat raw preview / 判断上下文继续围绕 SNDK / MU / WDC 等异常数量级价格生成判断。
    - 07:00 CST Web `持仓财报与重大新闻心跳提醒` raw preview 仍写出 SNDK `$2,273.73`、日内高点 `$2,280.52`、前收 `$2,050.39` 等异常价格口径。
    - 同窗 `Monitor_Watchlist_11` raw preview 继续使用 MU `$1,154.29`，`存储板块关键事件心跳提醒` raw preview 继续出现 WDC `$638.72` 等异常数量级。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗没有新的真实 direct message timestamp；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-07-01 03:01 CST）

- 本轮 2026-06-30 23:00-2026-07-01 03:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 23:00-03:01 CST heartbeat raw preview / 判断上下文继续围绕 SNDK / MU 异常数量级价格生成判断。
    - 23:00 CST 后多条 `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒` 等仍可见 SNDK 约 `$2,162.54`、MU `$1,142+`、SNDK 52 周高点 `$2,354.39` 等异常价格口径进入 raw preview。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗 direct 会话表继续实时增量，但 `cron_job_runs` 仍停在 `2026-06-30T09:30:52.069168+08:00`；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 23:01 CST）

- 本轮 2026-06-30 19:02-23:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/sessions.sqlite3`
    - 23:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `核心观察股池晚间快报` 正常收口，assistant final 正式输出多只明显异常数量级价格：`MU $1,142.40`、`SNDK $2,171.23`、`STX $969.64`、`WDC $649.37`、`GEV $1,135.22`、`AMD $559.37` 等。
    - 同条 final 继续围绕这些价格给出击球区、财报日期和观察池纪律说明；主链路正常收口，没有空回复、错投、投递失败或原始工具 JSON。
  - `data/runtime/logs/hone_cli_screen.log`
    - 同窗 heartbeat raw preview 继续出现 SNDK 异常价：`$2,050.39`、`previousClose $2,090.71`、`day range $1,895-$2,090.71`、`$2,162.54` 等，并进入触发判断上下文。
- 用户影响：
  - 本窗已有正式用户可见 scheduler final 样本，不只是内部 raw preview；价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径。
  - 报告主链路正常收口，未见功能阻断、错发对象或数据安全影响，因此仍按质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-30 19:02 CST）

- 本轮 2026-06-30 15:02-19:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 多条 heartbeat raw preview / 判断上下文继续围绕 SNDK / MU 异常数量级价格生成判断。
    - 15:30-19:00 CST `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒` 等仍可见 SNDK 约 `$2,050.39`、日内高低区间 `$1,895-$2,090.71`、52 周高点 `$2,354.39` 等异常数量级价格进入 raw preview。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗 direct 会话表继续实时增量，但 `cron_job_runs` 仍停在 09:30；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 15:02 CST）

- 本轮 2026-06-30 11:02-15:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-30`
    - 13:30 CST `Monitor_Watchlist_11` raw preview 继续把 `MU $1145.28` 作为行情锚，并与 `HIMS $33.39`、`RKLB ~$86-98` 等 watchlist 条件一起进入判断上下文。
    - 15:00 前后 `闪迪关键事件心跳提醒` / `持仓财报与重大新闻心跳提醒` 等 heartbeat raw preview 仍可见 SNDK / MU 异常数量级价格信号。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 本窗会话表已恢复实时增量，但 `cron_job_runs` 仍停在 09:30；本条证据以 runtime 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 07:03 CST）

- 本轮 2026-06-30 03:00-07:03 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 05:00 CST `闪迪关键事件心跳提醒` raw preview 继续把 `SNDK $2,050.39`、`previous close $2,090.71`、`yearHigh $2,354.39` 等异常数量级价格作为行情锚。
    - 本窗未确认新的正式送达成功样本；异常价格主要停留在 heartbeat raw preview、结构化失败、未命中或判断上下文。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。但异常价格仍进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-30 03:07 CST）

- 本轮 2026-06-29 23:00-2026-06-30 03:07 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - `闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`存储板块关键事件心跳提醒`、`持仓关键事件心跳检测` 等 heartbeat raw preview 继续把 `SNDK $2,090.71`、`MU $1,132.33` 或同级异常数量级价格作为行情锚。
    - 23:00-03:07 CST 未确认新的正式送达成功样本；异常价格主要停留在 raw preview、结构化失败、未命中或 heartbeat 判断上下文。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 19:01 CST）

- 本轮 2026-06-29 15:00-19:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - `持仓财报与重大新闻心跳提醒`、`闪迪关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`持仓关键事件心跳检测`、`Monitor_Watchlist_11` 等 heartbeat raw preview 继续把 `SNDK $2,090.71`、`MU $1,132.33` 作为行情锚。
    - 17:31 CST `闪迪关键事件心跳提醒` 返回 `JsonTriggered`，raw preview 继续围绕 `SNDK $2,090.71`、`-10.46%`、`52-week high $2,354.39` 等异常数量级价格生成触发判断；本窗未确认新的正式送达成功样本。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗主要停留在 heartbeat raw preview、未命中或结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告，因此不提升严重等级。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 15:07 CST）

- 本轮 2026-06-29 11:00-15:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-29`
    - 14:30 CST `闪迪关键事件心跳提醒`（`job_id=j_19dd9a1e`，`target=web-user-c2776780c59d`）以 `JsonTriggered` 生成正式 `deliver_preview`。
    - 同条预览写出 `最新价 $2,090.71，日内跌幅 -10.46%`、`日内区间 $2,063-$2,256`，继续把 SNDK 异常数量级价格作为用户可见行情锚。
    - 15:00 CST `持仓关键事件心跳检测` raw preview 又引用 `SNDK -10.46% ($2,090.71)`、`MU -6.69% ($1,132.33)` 等异常价格，但该样本落在 `JsonUnknownStatus + execution_failed` 路径。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 14:30 CST 样本已进入正式送达预览，不只是内部 raw preview，因此继续说明价格 sanity check 未覆盖当前 scheduler / heartbeat 运行路径。
  - 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 11:01 CST）

- 本轮 2026-06-29 07:00-11:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 10:30 CST `Monitor_Watchlist_11` raw preview 继续把 `MU $1,132.33` 作为价格锚，模型自己也写出 “MU at 1132? That seems wrong for Micron”，但该异常数值仍进入 heartbeat 判定上下文。
    - 本窗该样本落在 heartbeat raw preview / 结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告。
  - `data/runtime/logs/acp-events.log`
    - 09:19 CST Web direct 用户可见 chunk 出现 `StockAnalysis SND...` / `StockAnalysis` 来源名片段；该样本只证明来源名净化仍漏网，异常价格主证据仍来自 runtime heartbeat raw preview。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 07:02 CST）

- 本轮 2026-06-29 03:04-07:02 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 07:00 CST Web heartbeat `持仓关键事件心跳检测` raw preview 继续把 `SNDK -10.46% ($2,090.71)` 作为行情锚，并围绕该异常数量级价格总结板块抛压。
    - 本窗该样本落在 heartbeat raw preview / 结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-29 03:01 CST）

- 本轮 2026-06-28 23:02-2026-06-29 03:01 CST 真实运行态继续出现同根异常价格信号，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-28`
    - 多条 heartbeat raw preview 继续把异常数量级价格作为行情锚：`Monitor_Watchlist_11` 多次写出 `MU $1,132.33` 或 `$1132.33`，`闪迪关键事件心跳提醒` / `存储板块关键事件心跳提醒` / `持仓关键事件心跳检测` 多次写出 `SNDK $2,090.71`，`AI与科技持仓观察关键事件心跳提醒` 继续写出 `STX $899.90`，`闪迪关键事件心跳提醒` 继续写出 `WDC $586.45`。
    - 本窗这些样本主要停留在 heartbeat raw preview、未命中或结构化失败路径，未看到新的 Feishu scheduler / Web heartbeat 正式送达异常价格报告。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 本窗没有新增正式送达样本，因此不提升严重等级；调度 / 投递主链路没有因该问题被阻断。
  - 但异常价格仍持续进入 function-calling 结果和 heartbeat 判定上下文，说明价格 sanity check 仍未覆盖当前 scheduler / heartbeat 运行路径；作为既有质量性 `P3 / New` 继续跟踪。

## 最新进展（2026-06-28 11:01 CST）

- 本轮 2026-06-28 07:01-11:01 CST 真实运行态确认同根复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-28` 与 `data/runtime/logs/hone_cli_screen.log`
    - 11:00 CST Web heartbeat `持仓关键事件心跳检测`（`job_id=j_7a2adc11`，`target=web-user-cb1b46a2add4`）以 `JsonTriggered` 成功投递并记录 `定时任务完成`。
    - `deliver_preview` 写出 `SNDK -10.46%（$2,090.71）`、`VRT -6.64%（$303.95）`、`MU -6.69%（$1,132.33）` 等明显异常数量级价格，并把这些价格作为“AI 基础设施板块抛压延续”的用户可见摘要锚点。
    - 同窗其它 heartbeat raw preview 也继续围绕 `SNDK $2,090.71`、`MU $1,132.33` 解释或触发，但多数落为结构化失败 / noop；本条样本已经完成投递，因此不只是内部 raw preview 质量问题。
  - 查重结论：
    - 该样本与 2026-06-20/21 的同一缺陷根因一致：scheduler / heartbeat 对行情数值缺少稳定的数量级 sanity check，异常价仍可进入正式用户可见报告。
    - 这次发生在 Web heartbeat 投递链路，而不是原始 Feishu scheduler 早报，但受影响面仍是 scheduler / heartbeat 观察池行情锚，因此回退本单，不新建重复文档。
  - 用户影响：
    - 调度、收口和投递主链路均可用，没有错投、空回复或系统失败证据。
    - 但用户收到的行情摘要以异常价格作为判断依据，影响投资观察质量和可信度。
    - 因为不阻断功能链路，仍按质量性 `P3`；非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-28 23:02 CST）

- 本轮 2026-06-28 19:02-23:02 CST 真实运行态继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-28`
    - 21:35 CST Feishu scheduler 会话触发 `科技核心股池 · 晚间击球区快报`，21:36 CST `session.persist_assistant` 与 `done ... success=true` 收口。
    - 同条 final 继续把多只观察池标的输出为明显异常数量级价格：`MU $1,132.33`、`SNDK $2,090.71`、`STX $899.90`、`WDC $586.45` 等，并继续围绕这些价格给出击球区、财报日期和距离表。
    - 同窗 heartbeat raw preview 也继续围绕 `SNDK $2,090.71`、`MU $1,132.33` 解释，但多数未正式送达；本条 Feishu scheduler 样本已经正常收口，因此可作为用户可见质量证据。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 runtime / ACP 日志为准。
- 用户影响：
  - 报告主链路正常收口，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
  - 但异常价格继续进入正式观察池快报，说明价格 sanity check 仍未覆盖当前 scheduler / function-calling 路径。

## 最新进展（2026-06-21 23:03 CST）

- 本轮 19:02-23:01 CST 真实运行态继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/acp-events.log`
    - 本窗 ACP 可重构 21 次 `session/prompt`、21 次 `stopReason=end_turn`、0 个 ACP response error。
    - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 21:35 CST 的观察池快报以 `end_turn` 收口，正文再次把多只观察池标的输出为明显异常数量级价格：`MU $1,151.95`、`SNDK $2,209.28`、`STX $1,075.77`、`WDC $754.10` 等，并继续给出击球区和财报日期。
    - 同条 final 来源段继续出现 `StockAnalysis 各标的行情页`，说明问题不只是内部来源标签外露，而是异常行情数值仍被当作正式观察池价格锚。
  - `data/sessions.sqlite3` 仍停在 2026-06-17，最近会话证据继续以 ACP 日志为准。
- 用户影响：
  - 回复正常收口，观察池主链路仍可用，未见空回复、错投、投递失败或原始工具 JSON；因此仍按质量性 `P3`，非 P1，不创建 GitHub Issue。
  - 但异常价格已经在 6 月 20 日早间简报和 6 月 21 日晚间观察池快报复现，说明需要优先修复价格 sanity check，而不仅是改写 `StockAnalysis` 这个用户可见标签。

## 修复记录

- 2026-06-28 11:01 CST 状态回退为 `New`：
  - 10:00-11:01 CST 当前 live 运行态再次将异常价格写入 Web heartbeat `deliver_preview` 并完成投递，说明 2026-06-22 的 prompt 级 sanity 约束没有稳定覆盖当前 heartbeat / function_calling 运行路径。
  - 本轮是缺陷台账维护任务，未修改业务代码、测试代码或配置代码。
- 2026-06-22 03:08 CST 状态更新为 `Fixed`：
  - 观察池 scheduler prompt 增加价格 sanity 约束：如果某个标的最新价相对固定击球区或近期有效价明显偏离一个数量级，或疑似把市值、复权 / 拆股口径、页面其它数字误当股价，必须把该标的价格写为“最新行情未完成稳定校验”。
  - 同类异常价不得继续输出为精确价格，也不得基于该异常价计算距离击球区或给出交易判断。
  - 验证：`cargo test -p hone-channels scheduled_watchlist_hit_zone_prompt_keeps_stable_local_fields --lib -- --nocapture` 通过。
  - 无关联 GitHub Issue；本轮按本地代码与回归验证关闭，不依赖生产日志、线上渠道状态或 live 重启。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-20 07:02-11:02 CST。
  - 本窗 ACP 可重构 13 个 session、20 次 `session/prompt`、20 次 `stopReason=end_turn`，没有 ACP response error、空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径或思维痕迹进入用户可见 final。
  - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 的 `核心观察池早间简报` 以 `end_turn` 收口。
  - final 开头明确写出行情口径为 `StockAnalysis 对 25 支标的的最新可得统一口径：2026-06-18 美股盘后 19:59 EDT`，随后将多只观察池标的输出为明显异常价格：`MU $1,151.95`、`SNDK $2,209.28`、`STX $1,075.77`、`WDC $754.10` 等，并继续给出击球区、财报日期和观察池结论。
  - 同条 final 也正确说明 `6月19日为美股休市日`、`不覆盖 6月20日盘前实时价`，说明问题不是时间口径缺失，而是 scheduler 消费/展示的行情数值本身异常。
- `docs/bugs/feishu_direct_storage_price_unverified_before_tool_complete.md`
  - 旧缺陷覆盖 Feishu direct 中 MU / SNDK 异常价格与未充分核验链路，状态已在 2026-06-09 标为 `Fixed`。
  - 本轮样本发生在 Feishu scheduler 的核心观察池早间简报，影响多只观察池标的和定时报告行情锚，属于新的受影响链路，因此单独登记，不复用直聊文档。
- `data/sessions.sqlite3`
  - 只读快照仍显示 `sessions.max(updated_at)=2026-06-17T10:37:37.207669+08:00`、`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`cron_job_runs.max(executed_at)=2026-06-17T11:01:42.353141+08:00`，最近真实会话证据需以 ACP 日志为准。

## 端到端链路

1. Feishu scheduler 触发 `核心观察池早间简报`。
2. runner 为 25 支核心 / 拓展观察池标的获取或整理最新可得行情。
3. final 采用 `StockAnalysis` 作为统一行情口径，并把异常放大的价格数值写入用户可见报告。
4. 报告仍正常完成、收口并展示击球区 / 财报日期 / 观察结论。
5. 用户看到的是一条结构完整但价格锚明显不可信的观察池早报。

## 期望效果

- scheduler 对观察池价格应逐项完成稳定核验，且价格数量级应通过基本 sanity check。
- 当某个行情源返回异常数量级、拆股/复权口径不明或与常识区间明显冲突时，应标注该标的行情未完成稳定校验，而不是继续输出精确价格。
- 定时报告可以说明休市和盘后口径，但不能把异常放大的价格当作最新行情锚。

## 当前实现效果

- 报告链路没有中断，用户可见 final 结构完整并正常 `end_turn`。
- final 同时输出了明显异常的多标的精确价格，并继续围绕这些价格展示击球区和观察池简表。
- 该问题不同于单纯 `StockAnalysis` / `data_fetch` 名称外露：本轮实际影响了用户可见行情数值质量。
- 该问题也不同于旧的 direct MU / SNDK 文档：当前样本发生在 scheduler 观察池批量早报链路，影响范围更偏定时报告质量。

## 用户影响

- 用户仍收到核心观察池早间简报，调度、收口和投递主链路没有证据显示失败。
- 但观察池报告里的多只价格锚明显异常，会降低击球区、价格距离和风险判断的参考价值。
- 本轮没有看到错误交易指令、持久化写坏、投递失败或错发对象证据，因此不按 P2/P1 处理。
- 因为主功能链路可用，问题主要影响行情质量和用户决策参考可信度，所以定级为质量性 `P3`。

## 根因判断

- 初步判断 scheduler 对 `StockAnalysis` 或中间行情摘要的数值缺少跨源 / 数量级 sanity check。
- 现有金融 prompt 的“多标的最新行情约束”更偏要求独立核验来源、时间戳和交易时段口径；本轮说明即使给出统一口径，仍需要在批量 scheduler 层校验价格数量级是否异常。
- 现有 `feishu_scheduler_data_fetch_tool_name_exposed.md` 跟踪的是内部工具名 / 数据源名外露；本单跟踪的是异常价格被当作正式行情锚。

## 下一步建议

- 在 scheduler 观察池行情整理层增加价格 sanity check：同一标的最新价若相对固定击球区、历史画像价格或前次有效价偏离异常倍数，应降级为“未完成稳定校验”。
- 对批量行情报告增加回归样本：当 MU / SNDK / WDC / STX 等价格出现异常数量级时，final 不应输出精确价格或基于该价格判断距离击球区。
- 若继续使用 `StockAnalysis` 页面作为补充校验源，需明确解析字段来源，避免把市值、拆股/复权口径或其它页面数字误当股价。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`docs/bugs/README.md` / 既有 bug 文档查重、`data/sessions.sqlite3` 上界、`data/runtime/logs/acp-events.log` 07:02-11:02 CST 结构化解析、用户可见 final 关键词扫描、最近四小时非文档代码提交检查。

## 最新运行态复核（2026-07-01 19:06 CST）

- `data/runtime/logs/web.log.2026-07-01`
  - 巡检窗口：2026-07-01 15:00-19:05 CST。
  - 15:00 CST `Monitor_Watchlist_11` raw preview 继续把 `MU` 现价写为 `$1,154.29`，并明确模型也感知到“price seems very high / data is off”，但仍继续比较触发条件。
  - 15:00 / 18:30 / 19:00 CST 多条 SNDK / 存储板块 heartbeat raw preview 继续使用 `SNDK $2,273.73`、`MU $1,154.29` 等异常数量级价格、目标价或市值上下文。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 本窗未确认新的正式 assistant final 或最终送达正文使用异常价格做交易建议；因此维持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

## 最新运行态复核（2026-07-02 11:01 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-02 07:01-11:01 CST。
  - `session_id=Actor_web__direct__web-user-8988066ef1ac` 在 10:50 CST 的 Web direct 投研 final 中，继续把 `LITE` 写为 801.16 美元、`MU` 写为 1032.28 美元，并基于这些异常数量级价格讨论估值、回撤和周期风险。
  - 10:55 CST 同一会话的扩展投研 final 继续使用 `LITE` 801.16 美元、`MU` 相关异常财务 / 价格口径，并把它们纳入长期关注组合分析。
- `data/runtime/logs/web.log.2026-07-02`
  - 11:00 CST `闪迪关键事件心跳提醒` deliver preview 正式输出 `SNDK` 收于约 2032 美元、日内低点约 2002 美元、较 52 周高点约 2354 美元等明显异常数量级价格。
  - 11:00 CST `Monitor_Watchlist_11` raw preview 继续把 `MU` 写为 1032.28 美元，并用该数值判断未触发阈值。
- 本轮判断
  - 最新证据显示异常行情不只影响 Feishu scheduler 早报，也继续进入 Web direct 投研 final 与 Web heartbeat deliver preview；仍属于同一 StockAnalysis / 批量行情 sanity check 缺失根因，不新建重复缺陷。
  - 用户能收到完整分析，未见投递失败或错对象；但行情锚不可信会显著削弱投研质量，因此维持质量性 `P3 / New`。

## 最新运行态复核（2026-07-03 07:00 CST）

- `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-07-03 03:00-07:00 CST。
  - 本窗至少 21 条 heartbeat raw preview 继续出现明显异常数量级价格或基于异常价格的判断上下文。
  - 代表样本包括 `闪迪关键事件心跳提醒` 使用 `SNDK` 当前价约 `$1,717.68` / `$1,745` 并与 previous close `$2,032.22` 比较；`Monitor_Watchlist_11` 使用 `MU $975.56` 判断未触发阈值。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 本窗未确认新的最终送达正文或 direct final 基于异常价格给出交易建议；调度和投递主链路本身仍可运行，因此维持质量性 `P3 / New`，非 P1。

## 最新运行态复核（2026-07-04 03:05 CST）

- `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-07-03 23:02-2026-07-04 03:05 CST。
  - 本窗至少 28 条 heartbeat raw / deliver preview 命中异常行情或错误时间口径关键词，其中异常行情样本包括 `MU $975.56`、`SNDK $1,745`、`WDC $539`、`SPY $744.78`、`QQQ $712.6` 等明显偏离常识数量级的价格。
  - 代表样本包括 `Monitor_Watchlist_11` 用 `MU $975.56` 判断未触发阈值、`闪迪关键事件心跳提醒` 用 `SNDK $1,745` 与 previous close `$2,032.22` 比较、`存储板块关键事件心跳提醒` 使用 `WDC $539`。
- `data/sessions.sqlite3`
  - 同窗 00:00-00:05 Feishu scheduler final 成对收口；final 关键词扫描未命中 `data_fetch` / `quote_short` 或上述异常行情关键词。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 本窗未确认新的最终 direct final 基于异常价格给出交易建议；调度和投递主链路本身仍可运行，因此维持质量性 `P3 / New`，非 P1。

## 最新运行态复核（2026-07-04 11:01 CST）

- `data/runtime/logs/web.log.2026-07-04` / `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-07-04 07:01-11:01 CST。
  - 本窗 heartbeat raw / deliver preview 继续命中异常行情关键词；代表样本包括 `存储板块关键事件心跳提醒` 继续使用 `SNDK $1,745`、previous close `$2,032.22` 等明显异常数量级价格。
  - 同窗小米 30 港元破位预警可见送达预览，行情数值本身不属于美股 StockAnalysis 异常价格，但仍显示 heartbeat 会把旧行情时间与当前触发判断混用，已另归入时间口径缺陷。
- `data/sessions.sqlite3`
  - 同窗 3 条 assistant final 未命中上述异常行情关键词；未确认新的最终 direct final 基于异常价格给出交易建议。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 调度和投递主链路本身仍可运行，因此维持质量性 `P3 / New`，非 P1。

## 最新运行态复核（2026-07-04 15:02 CST）

- `data/runtime/logs/web.log.2026-07-04`
  - 巡检窗口：2026-07-04 11:02-15:02 CST。
  - 本窗 heartbeat raw / deliver preview 继续命中异常行情关键词；代表样本包括 `Monitor_Watchlist_11` 使用 `MU $975.56` 判断未触发阈值，`存储板块关键事件心跳提醒` 使用 `WDC $539`，以及 `美股黄金坑信号心跳检测` 使用 `SPY $744.78` / `QQQ $712.60`。
  - 这些样本主要停留在 raw preview、未命中或结构化失败路径；未确认新的最终 direct final 基于异常价格给出交易建议。
- `data/sessions.sqlite3`
  - 同窗 1 条 Feishu direct assistant final 未命中上述异常行情关键词。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 调度和投递主链路本身仍可运行，本窗未确认新的用户可见交易建议使用异常价格，因此维持质量性 `P3 / New`，非 P1。

## 最新运行态复核（2026-07-04 23:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-04 19:01-23:02 CST。
  - 21:35 CST 与 23:00 CST Feishu scheduler final 成对收口，但继续输出用户可见异常数量级价格：`MU $975.56`、`SNDK $1,745.00`、`STX $820.16`、`WDC $539.00`、`AMD $517.82` 等，并据此判断“明显高于既定击球区”。
- `data/runtime/logs/hone_cli_screen.log`
  - 同窗 heartbeat raw / deliver preview 继续命中异常行情关键词，代表信号包括 `SNDK $1,745`、`SPY $744.78`、`QQQ $712.60`、`MU $975.56`。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 本窗已确认异常价格进入 Feishu scheduler 用户可见 final；但会话正常收口、未见投递失败或错对象，问题主要削弱投研质量和价格判断可信度，因此维持质量性 `P3 / New`，非 P1。

## 最新运行态复核（2026-07-05 19:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-05 15:00-19:02 CST。
  - 同窗 heartbeat raw preview 继续命中异常行情或价格 / 时间戳混用关键词，代表样本包括 `Monitor_Watchlist_11` 使用 `MU $975.56` 判断未触发阈值，`Cerebras IPO与业务进展心跳监控` 使用 `CBRS $204.86` 与 `1783022401` 口径，`RKLB异动监控` 使用 `RKLB $100.46`，以及 `DRAM 心跳监控` 使用 `DRAM $60.63` 与 `1783022401`。
  - 17:31 CST Feishu scheduler `A股港股收盘后跨市场复盘` final 继续输出 `QQQ 712.60`、`MU 975.56`、`AMD 517.82` 等高波动行情锚，并把这些价格纳入美股预判和估值分层；该样本同样正常收口和送达。
  - 15:05 CST Feishu direct `LRCX` 分析 final 来源段还出现 `公开行情页.com` 占位链接，该用户态来源边界问题已归入 `feishu_scheduler_data_fetch_tool_name_exposed.md`，本单只记录其中对行情口径可信度的关联影响。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 本窗再次确认部分异常数量级行情进入用户可见 scheduler final；但会话正常收口、未见投递失败或错对象，问题主要削弱投研质量和价格判断可信度，因此维持质量性 `P3 / New`，非 P1。

## 最新运行态复核（2026-07-07 11:02 CST）

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-07 07:01-11:02 CST。
  - 07:02 CST `美股持仓收盘后早报` final 继续把 `SNDK` 写为 1744.43、`AMD` 写为 552.05、`DELL` 写为 411.51、`ARM` 写为 322.24，并据此计算组合市值、浮盈、单日贡献和仓位结构。
  - 08:32 CST `闪迪(SNDK)每日行情与行业简报` final 继续把 `SNDK` 收盘价写为 1744.43、盘后 1686.90，并基于该价格讨论目标价空间、估值和承接强弱。
  - 09:02 CST `核心股与拓展股分组简表` final 继续输出 `MU 960.00`、`SNDK 1689.75`、`STX 849.00`、`WDC 563.00`、`GEV 1150.98`、`GLW 192.37`、`CRDO 262.83` 等明显异常数量级价格，并将它们纳入击球区距离判断。
- 本轮判断
  - 最新证据仍落在 scheduler / direct 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 本窗异常价格已进入用户可见 final 并影响组合市值、估值和买点判断；但会话正常收口、未见投递失败、错对象或数据破坏，问题主要削弱投研质量和价格判断可信度，因此维持质量性 `P3 / New`，非 P1。

## 最新运行态复核（2026-07-12 15:01 CST）

- `data/runtime/logs/web.log.2026-07-12`
  - 巡检窗口：2026-07-12 11:00-15:01 CST。
  - 本窗 32 条 heartbeat raw / deliver preview 命中异常或高风险数量级行情锚。
  - 代表样本包括 `闪迪关键事件心跳提醒` 继续使用 `SNDK $1,915.92`、`AAOI 1.6T 光模块心跳检测` 使用 `AAOI $119.92`、`Monitor_Watchlist_11` 使用 `MU $979.30` 判断远高于触发价、`美股黄金坑信号心跳检测` 使用 `SPY $754.95` / `QQQ $725.51`，以及 `中际旭创关键事件心跳提醒` 使用 `中际旭创 ¥1,093.98`。
  - 多数样本处于 heartbeat preview、duplicate suppression、noop 或失败路径；本窗没有新的普通 scheduler final 异常价格样本。
- `data/sessions.sqlite3`
  - 11:00-15:01 CST 按真实 `timestamp` 没有新增 assistant final；12:33 CST `imported_at` 推进的是旧会话重导入，不作为本窗新的用户可见行情样本。
- 本轮判断
  - 最新证据仍落在 scheduler / heartbeat 批量行情数值 sanity check 缺失范围内，没有新的独立根因。
  - 本窗未确认新的普通 final 基于异常价格给出交易建议；调度和投递主链路本身仍可运行，因此维持质量性 `P3 / New`，非 P1。
