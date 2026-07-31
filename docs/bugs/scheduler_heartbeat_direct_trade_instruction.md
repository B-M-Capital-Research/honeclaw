# Bug: Heartbeat 破位预警直接输出无条件止损交易指令

- **发现时间**: 2026-05-10 07:04 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 最新进展

- `2026-07-31 10:00-14:02 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 10:01 `ASTS 全面心跳检测` `run_id=50875` 作为自动 heartbeat 出站，围绕 52 周低点反弹、催化、成本均价和盈亏校验展开，而不是只报告本轮触发事实。
    - 10:01 `AAOI 全面心跳检测` `run_id=50876` 写“当前不是基本面型买入窗口”，继续把自动监控扩展成买入窗口判断。
    - 10:30 `ASTS 全面心跳检测` `run_id=50884`、11:01 `AAOI 全面心跳检测` `run_id=50899` 继续围绕持仓数字、成本均价、盈亏和决策前提展开。
    - 11:31 `ASTS 全面心跳检测` `run_id=50910` 却以 RKLB 行情口径展开 `RKLB` 深度分析并写“当前不是补仓好时机”；11:31 `AAOI 全面心跳检测` `run_id=50906` 又串入 AAPL 深度分析和持仓盈利口径，任务主体错配后仍形成投研 / 交易语义输出。
  - 判断：这些样本不是原始“无条件止损”字面话术，但仍来自自动 heartbeat，而非用户主动直聊投研；它们把事实核验扩展成成本、盈亏、买入 / 补仓窗口与持仓动作边界判断。严重等级维持 `P2`；当前主发送链路未整体阻断，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-31 06:00-10:02 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 06:00 `AAOI 全面心跳检测` `run_id=50777` 作为自动 heartbeat 出站，围绕用户数字冲突、现价、昨收、52 周高低和财报前事件博弈展开，而不是只报告本轮触发事实。
    - 06:00 `RKLB 全面心跳检测` `run_id=50785` 使用 ASTS 行情口径并继续围绕成本、盈亏百分比和持仓数字冲突展开，任务主体错配后仍形成投研 / 交易语义输出。
    - 07:30 `TSLA 正负触发条件心跳监控` `run_id=50812` 把系统触发输入误当成用户只发 `1` 并要求用户澄清，还列出持仓和止损口径；这是 heartbeat 执行意图污染，也会让监控链路丢失原始任务边界。
    - 09:30 / 10:00 `ASTS 全面心跳检测` `run_id=50864/50875` 继续把自动 heartbeat 扩展成超卖修复、催化时间、成本数字和“博弈价值”判断；10:00 `AAOI 全面心跳检测` `run_id=50876` 直接写“当前不是基本面型买入窗口”。
  - 判断：这些样本不是原始“无条件止损”字面话术，但仍来自自动 heartbeat，而非用户主动直聊投研；它们把事实核验扩展成成本、盈亏、买入窗口、催化与交易动作边界判断。严重等级维持 `P2`；当前主发送链路未整体阻断，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-31 02:02-06:01 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 04:00 `TSLA 正负触发条件心跳监控` `run_id=50740` 作为自动 heartbeat 出站，却把系统触发输入误当成用户只发 `1` 并要求用户澄清“你想做什么”；这是 heartbeat 执行意图污染，也会让监控链路丢失原始任务边界。
    - 05:30 `RKLB 全面心跳检测` `run_id=50774` 写明“本轮无新增触发事实，noop”，但仍继续展开重复触发、价格阈值和后续是否新增触发判断。
    - 06:00 `关注股重大事件心跳检测` `run_id=50786` 以 fenced JSON 送达，正文继续围绕 `MU` 盘后价格、50 日均线、机构空翻多观察和财报验证节点扩展成自动交易触发判断。
  - 判断：这些样本不是原始“无条件止损”字面话术，但仍来自自动 heartbeat，而非用户主动直聊投研；它们把事实核验扩展成交易触发、技术节点或动作边界判断。严重等级维持 `P2`；当前主发送链路未整体阻断，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-30 22:01-2026-07-31 02:02 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 22:01 `AAOI 全面心跳检测` 作为自动 heartbeat 出站，用户可见 preview 却使用 `TEM NASDAQ` 行情口径并展开 TEM 财报框架，任务主体错配后仍形成投研 / 交易语义输出。
    - 00:00 `AAOI 全面心跳检测` 作为自动 heartbeat 出站，直接写“单日涨幅 +17.08% 已超过 ±8% 阈值”，并继续展开评级上调、驱动事件和跟踪判断。
  - `data/runtime/logs/web.log.2026-07-30`
    - 01:00 `AAOI 全面心跳检测` 写“触发：单日涨幅 +16.30%”，继续展开驱动事件溯源。
    - 01:00 / 02:01 `RKLB 全面心跳检测` 围绕 ±8% 阈值、当前价格、成交量和新增发射合同生成触发提醒；02:01 preview 直接以 fenced JSON `status=triggered` / `condition=单日涨跌幅超过 ±8%` 开头。
    - 01:30 / 02:00 多条 `AAOI / RKLB / ASTS / 德业股份 / 珠海冠宇` heartbeat 明写 `NOOP/noop` 后仍保留技术节点、成交量、买入信号或触发条件判断并送达。
  - 判断：这些样本不是原始“无条件止损”字面话术，但仍来自自动 heartbeat，而非用户主动直聊投研；它们把事实核验扩展成交易触发、技术节点和动作边界判断。严重等级维持 `P2`；当前主发送链路未整体阻断，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-30 18:00-22:03 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 18:00 `RKLB 全面心跳检测` 作为自动 heartbeat 出站，用户可见 preview 围绕“成本约 $53、浮亏约 20%”和“判断要不要卖”展开，而不是只报告本轮触发事实。
    - 19:01 `RKLB 全面心跳检测` 串到 TEM 财报前持仓框架，写出用户成本、浮亏和是否加仓前需要区分估值修正 / 逻辑受损。
    - 21:31 `ASTS 全面心跳检测` 再次围绕“成本约 $53、亏了 20% 左右”推断实际成本，扩展成持仓成本和风险判断。
    - 21:32 `彩票组合风险监控与买卖点提醒` 作为普通 scheduler 成功送达，写出 `YINN` “明日无法放量突破 $32 -> 执行清仓”等强动作话术；该样本与自动 heartbeat 边界不同，先作为同一金融出站动作边界风险的近窗参照，不拆新缺陷。
  - 判断：这些样本不是原始“无条件止损”字面话术，但仍来自自动 / 定时金融出站，而非用户主动要求实时下单；它们把触发事实核验扩展成持仓成本、浮亏和执行动作判断。严重等级维持 `P2`；当前主发送链路未整体阻断，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-30 14:01-18:02 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 17:01 `ASTS 全面心跳检测` `run_id=50488` 作为自动 heartbeat 出站，用户可见 preview 写“你说成本约 $53，现在亏了 20% 左右”，并继续讨论实际成本、持仓信心和是否应处理仓位。
    - 18:00 `RKLB 全面心跳检测` `run_id=50502` 作为自动 heartbeat 出站，继续围绕“成本约 $53、浮亏约 20%”与“判断要不要卖”展开，而不是只报告本轮触发事实。
    - 18:01 `AAOI 全面心跳检测` `run_id=50505` 使用 RKLB / ASTS 行情口径，输出两只航天股技术位对比与下跌解释，仍越过自动 heartbeat 应只做触发事实提醒的边界。
  - 判断：这些样本不是原始“无条件止损”字面话术，但仍来自自动 heartbeat，而非用户主动直聊投研；它们把触发事实核验扩展成持仓成本、浮亏和是否卖出的交易判断边界。严重等级维持 `P2`；当前主发送链路未整体阻断，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-30 10:01-14:02 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 13:31 `AAOI 全面心跳检测` `run_id=50419` 作为自动 heartbeat 出站，用户可见 preview 写 `AAOI $76.70 — 本轮有以下增量，triggered`，并展开盘后价格核验、收盘 / 盘后变化与后续触发判断。
    - 14:01 同 job `run_id=50425` 又写 `AAOI $76.52 — 本轮无新增触发事实，noop`，仍继续保留盘后价格、触发条件和技术节点说明并送达。
  - 判断：这些样本不是原始“无条件止损”字面话术，但仍来自自动 heartbeat，而非用户主动直聊投研；它们把触发事实核验扩展成盘后价格、技术节点和是否新增触发的交易判断边界。严重等级维持 `P2`；当前主发送链路未整体阻断，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-30 06:02-10:01 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 06:31 `AAOI 全面心跳检测` `run_id=50254` 作为自动 heartbeat 出站，用户可见 preview 写 `$76.52 是一个值得认真考虑的平均成本区间`，并分析“现在的价格便宜吗”。
    - 07:00 `AAOI 全面心跳检测` `run_id=50269` 继续写 `$76.52 不是简单的便宜价格，而是高不确定性下的高赔率机会。是否买，取决于...`。
    - 08:31 `AAOI 全面心跳检测` `run_id=50311` 写 `AAOI 当前 $76.52 是一个值得认真评估的价位`，并继续扩展为公司商业模式和风险分析。
    - 09:01 / 09:30 `AAOI 全面心跳检测` `run_id=50315/50335` 继续写 `$76.52 是技术性超卖区间 / 强技术超卖区间` 和财报验证节点判断。
    - 10:00 `TSLA 正负触发条件心跳监控` `run_id=50346` 已命中 `direct_trade_instruction_guarded=true`，出站前加了风险提示，但仍把 heartbeat 正文组织成 `是否是好买点需结合仓位和持有周期判断`。
  - 判断：这些样本继续来自自动 heartbeat，而非用户主动直聊投研；它们把触发事实核验扩展成买点、平均成本、是否值得买或持仓动作边界，仍会影响交易决策语义。严重等级维持 `P2`，同窗未见错投、敏感泄露或 P1 级故障，不创建 GitHub Issue。

- `2026-07-30 02:01-06:04 CST` 真实运行态继续确认同一自动 heartbeat 出站交易动作边界活跃，状态维持 `New/P2`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - 05:01 CST `AAOI 全面心跳检测` `run_id=50224` 作为自动 heartbeat job 送达，正文写 `AAOI $76.52 — 触发 ±8% 阈值，推送`，随后进入行情事实和跌破均线判断。
    - 06:01 CST 同 job `run_id=50247` 继续 `completed + sent + delivered=1`，用户可见 preview 写出 `当前 AAOI 处于强技术超卖区间...是否值得买，取决于你的时间轴和催化等待能力`，并展开公司分析和财报验证节点。
  - 判断：
    - 最新样本不是原始“无条件止损”字面话术，但仍是自动 heartbeat 出站直接进入买点 / 是否值得买判断，而不是只报告触发事实、风险边界和需要用户自行决策的条件化框架。
    - 该问题影响自动金融预警的动作边界和用户投资决策安全，严重等级维持 `P2`；当前是 heartbeat 出站样本，主发送链路未整体阻断，非 P1，不创建 GitHub Issue。

- 本轮最近四小时巡检确认同一自动 heartbeat 出站交易动作边界复发，状态从 `Fixed` 回退为 `New/P2`：
  - `data/runtime/logs/web.log.2026-07-25`
    - 09:30 CST `NVDA 关键事件心跳提醒` 作为已创建 heartbeat job 周期触发，deliver preview 写出 `结论：$206.84 可考虑小幅加仓，但需严控仓位上限和触发条件`，并继续围绕 50 日均线、200 日均线等行情条件给加仓动作建议。
    - 该样本不是原文档旧样本里的“无条件止损”，但仍是自动监控提醒直接给出买入 / 加仓动作，而不是只报告触发事实、风险边界和需要用户自行决策的条件化框架。
  - 会话质量对照：
    - 同窗 `data/sessions.sqlite3` 新增 38 条 user / 23 条 assistant / 10 条 system compact，覆盖 13 个更新 session；其它 direct / scheduler 多条正常收口，未见错投、敏感信息泄露、数据破坏或全渠道不可用。
  - 判断：
    - 与既有“heartbeat 直接交易指令 guard 覆盖不足”同一链路，不新建重复缺陷。
    - 该问题影响自动金融预警的动作边界和用户投资决策安全，严重等级维持 `P2`；当前是单个 heartbeat 出站样本，非 P1，不创建 GitHub Issue。

## 修复结论复核（2026-05-11 23:02 CST）

- 本轮最近四小时巡检继续看到当前机器旧运行态坏样本，但仍不足以推翻仓库代码层面的 `Fixed` 结论：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18842`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-11T19:30:41.153099+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 继续包含 `建议动作：无条件止损`，并写出 `不宜持有等待反弹`。
  - `2026-05-11 23:02 CST` 结论：该样本晚于上一轮 15:30 CST 旧运行态样本，但当前仓库代码已在 `1d405f2` 扩展 guard 并刷新 `deliver_preview`，本轮没有证明 live 进程已经部署 / 重启到该修复后仍复现；因此仅补充证据，不把状态从 `Fixed` 回退为 `New`。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18752`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-11T15:30:23.885121+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `detail_json.scheduler.heartbeat_model=mimo-v2.5-pro`
    - `response_preview` 与 `detail_json.scheduler.deliver_preview` 均包含 `建议动作：无条件止损`。
  - `data/runtime/logs/web.log.2026-05-11` 在 `2026-05-11 15:30 CST` 同窗记录 `parse_kind=JsonTriggered` 后进入 `deliver`，`deliver_preview` 仍保留直接交易指令，说明问题发生在用户可见最终出站内容，不是中间草稿。
  - 该样本晚于上轮 `2026-05-11 15:02 CST` 巡检，但当前仓库代码已在 `1d405f2` 扩展 guard 并刷新 `deliver_preview`，本轮没有证明 live 进程已经部署 / 重启到该修复后仍复现。
- 结论：保留为旧运行态补充证据，不新建重复文档，也不把状态从 `Fixed` 回退为 `New`。后续若确认部署当前代码后仍出现同样 `deliver_preview`，再重新打开。

## 修复记录（2026-05-10 23:11 CST）

- `crates/hone-channels/src/scheduler.rs` 扩展 heartbeat 直接交易指令 guard：
  - 除了 `无条件止损` / `必须卖出` / `立即清仓` 等硬词，也覆盖 `建议动作` / `操作建议` 标题下的止损、清仓、买卖、抄底、持有等待反弹等动作措辞。
  - guard 改写后同步刷新 `metadata.deliver_preview`，避免台账里的最终可见 preview 仍保留被替换前的直接交易指令。
- 新增回归：`heartbeat_direct_trade_instruction_detects_action_heading`。
- 验证：
  - `cargo test -p hone-channels heartbeat_direct_trade_instruction --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs memory/src/session.rs`
- 关联 GitHub Issue：无。

## 旧运行态复核（2026-05-11 03:02 CST）

- `2026-05-11 11:02 CST` 本轮最近四小时巡检继续看到一条旧运行态尾部坏样本，但后续窗口已连续恢复为 `noop`，仍不足以推翻仓库代码层面的 `Fixed` 结论：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18544`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-11T07:30:34.678118+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 与 `detail_json.scheduler.deliver_preview` 继续包含 `建议动作：无条件止损...周一盘前或开盘后第一时间执行卖出`
  - 同任务在 `2026-05-11 08:00 / 08:30 / 10:00 / 10:30 / 11:00 CST` 均落成 `noop / skipped_noop`，没有继续送达直接交易指令。
  - 结论：该样本仍按当前本机旧运行态 / 未确认重启后的 live 尾部窗口处理；仓库 HEAD 已包含 `1d405f2` 的 guard 修复，本轮不把状态从 `Fixed` 回退为 `New`。后续若确认部署新代码后仍出现同样 `deliver_preview`，再重新打开。

- `2026-05-11 07:03 CST` 本轮最近四小时巡检继续看到旧运行态坏样本，但仍不足以推翻仓库代码层面的 `Fixed` 结论：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18444`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-11T03:30:27.706375+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 与 `detail_json.scheduler.deliver_preview` 继续包含 `建议动作：无条件止损。当前非美股交易时段，开盘后请立即执行。`
  - `data/runtime/logs/sidecar.log`
    - `2026-05-11 03:30:25 CST` 同一任务记录 `parse_kind=JsonTriggered` 后直接 `deliver`，`deliver_preview` 仍保留直接交易指令。
  - `2026-05-11 06:30` 与 `07:00` 同任务已回到 `noop / skipped_noop`，没有新增直接交易指令送达。
  - 结论：这仍按当前本机旧运行态 / 未确认重启后的 live 窗口处理；仓库 HEAD 已包含 `1d405f2` 的 guard 修复，本轮不把状态从 `Fixed` 回退为 `New`。后续若确认部署新代码后仍出现同样 `deliver_preview`，再重新打开。

- `2026-05-11 03:02 CST` 本轮在本机 live 数据中仍看到修复前坏态延续：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18335`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-10T23:30:22.811640+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 与 `detail_json.scheduler.deliver_preview` 继续包含 `建议动作：无条件止损`。
  - 结论：该样本来自当前本机旧运行态 / 未确认重启后的 live 窗口；由于仓库代码已在 `2026-05-10 23:11 CST` 修复直接交易指令 guard，本轮不把状态从 `Fixed` 回退为 `New`。后续若部署新代码后仍复现，再重新打开。

## 最新进展（2026-05-10 19:02 CST）

- `2026-05-10 23:10 CST` 本轮继续确认同一缺陷活跃：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18222`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-10T19:30:40.801091+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 与 `detail_json.scheduler.deliver_preview` 继续包含 `建议动作：无条件止损`。
  - 结论：直接交易指令 guard 在 19:30 真实窗口仍未覆盖 live 出站路径，维持 `P2 / New`。

- 本轮缺陷巡检确认该缺陷在最近四小时真实 heartbeat 窗口复发，状态从 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18117`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-10T15:30:30.441923+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 继续向用户送达 `建议动作：无条件止损`。
  - 同条 `detail_json.scheduler.deliver_preview` 与 `raw_preview` 都保留了相同直接交易指令，说明这是模型结构化触发后进入最终出站的用户可见内容，不是中间草稿。
- 结论：这是同一根因/同一链路复发，不新建重复文档。主发送链路成功，但自动化金融预警仍越过“只报告事实和条件化风险边界”的要求，影响投研输出安全性，因此继续按功能性 `P2 / New` 跟踪。

## 修复进展（2026-05-10 07:05 CST）

- `crates/hone-channels/src/scheduler.rs` 为 heartbeat prompt 增加“交易动作边界”：自动预警只能报告触发事实、价格 / 成交量 / 时间口径和条件化风险管理框架，不得输出 `无条件止损`、`必须卖出`、`立即清仓`、`马上买入` 等直接交易指令。
- 同时在 scheduler 出站前增加通用 guard：命中直接交易指令时，会把正文改写为风险提示，保留价格与触发事实片段，移除无条件买卖 / 止损 / 清仓动作句。
- 新增回归：
  - `heartbeat_prompt_rejects_direct_trade_instructions`
  - `heartbeat_direct_trade_instruction_gets_risk_guard`
- 验证：
  - `cargo test -p hone-channels heartbeat_direct_trade_instruction --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_prompt_rejects_direct_trade_instructions --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=17796`
  - `job_name=CAI破位预警`
  - `executed_at=2026-05-10T03:30:24.716901+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `response_preview` 在列出 CAI 跌破 52 周低点、当前价、盘中低点和成交量后，直接写出 `建议动作：无条件止损...不建议抄底或持有等待反弹`。

## 端到端链路

1. Feishu heartbeat scheduler 触发 `CAI破位预警`。
2. function-calling heartbeat runner 返回合法 `JsonTriggered`。
3. scheduler 将模型正文作为用户可见预警发送，台账落成 `completed + sent + delivered=1`。
4. 最终送达文本从价格/阈值提醒升级为直接交易指令，缺少“仅供分析参考”“需结合仓位与风险评估”“触发/失效条件”等边界。

## 期望效果

- 破位预警应报告已核验的价格、阈值、成交量、时间口径与风险事实。
- 如给动作建议，只能表达为条件化的风险管理框架，例如“若用户原本以该阈值作为止损线，应复核仓位和风险承受能力”，不能替用户下达“无条件止损”这类确定性交易指令。
- 对涉及买卖、止损、加仓、减仓的输出，应明确保持分析参考口径，并提供触发条件、证伪条件和风险边界。

## 当前实现效果

- 真实窗口已成功送达的 `CAI破位预警` 曾直接输出 `无条件止损`。
- 这不是发送失败、重复投递或 JSON 解析失败；链路本身成功，但用户可见内容越过投研助手的动作边界。
- 同窗其它 heartbeat 能正常 `noop` 或送达，说明问题集中在 heartbeat 预警文案约束与最终出站安全边界。

## 用户影响

- 用户可能把系统预警理解为直接交易指令，而不是风险提示或分析参考。
- 该问题发生在自动化 heartbeat 推送里，用户没有即时追问澄清上下文；错误口径会以主动通知形式影响风险管理决策。
- 定为 `P2`：主投递链路没有阻断，但它影响金融投研输出正确性和风险管理安全边界，不属于只影响表达观感的 `P3`。

## 根因判断

- heartbeat prompt / 输出约束允许模型在破位场景中生成过强动作词。
- scheduler 当前只校验结构化状态和基础出站净化，没有对 `无条件止损`、`必须卖出`、`立即买入` 等直接交易指令做渠道级降级或改写。
- 该根因不同于 `scheduler_heartbeat_retrigger_duplicate_alerts.md` 的重复提醒，也不同于 `scheduler_heartbeat_unknown_status_silent_skip.md` 的结构化解析漂移。

## 下一步建议

- 在 heartbeat 系统提示中增加“预警只报告事实和条件，不输出无条件买卖/止损指令”的硬约束。
- 在 scheduler 出站前增加轻量 guard：命中直接交易指令词时，将动作改写为条件化风险提示，或加上明确的分析参考边界。
- 增加回归样本，覆盖 `无条件止损`、`立即清仓`、`马上买入` 等不应原样外发的自动预警文案。
