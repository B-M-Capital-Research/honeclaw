# Bug: Scheduler commodity guard falsely replaces non-commodity market reviews with oil guard notice

- **发现时间**: 2026-05-25 19:05 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 复发记录（2026-05-26 23:05 CST）

- 最近四小时真实窗口确认同一根因继续大面积复发：`2026-05-26 19:02-23:02 CST` 普通 scheduler 共有 37 条 `completed + sent + delivered=1`，其中 12 条命中 `detail_json.scheduler.commodity_causality_guarded=true`。
- 12 条 guard 命中里，`Oil_Price_Monitor_Premarket` 是原油价格播报，属于预期商品任务；其余至少 11 条为非商品或广义市场/盘前分析任务，原始完整市场分析被全量替换为“本轮原油/大宗商品播报包含未完成同窗来源核验...”安全提示并仍记录为已送达。
- `data/sessions.sqlite3` -> `cron_job_runs` 关键样本：
  - `run_id=34084`，`job_name=美股大盘晚间简报`，`executed_at=2026-05-26T20:01:06.477788+08:00`，`completed + sent + delivered=1`，`detail_json.scheduler.commodity_causality_guarded=true`。`raw_preview` 是美股盘前大盘风险偏好、利率、油价和 AI 预期分析；`response_preview` 被替换为原油 / 大宗商品安全提示。
  - `run_id=34102`，`job_name=每日20点美股大盘风控简报`，`executed_at=2026-05-26T20:01:10.539120+08:00`，同样被替换。原始正文是 Memorial Day 后美股恢复交易、指数高位和情绪区间风控。
  - `run_id=34087`，`job_name=每日美股大盘温度检查`，`executed_at=2026-05-26T20:01:23.762620+08:00`，同样被替换。原始正文是美股盘前温度检查，包含 Nasdaq、S&P 500、VIX 和风险偏好。
  - `run_id=34098`，`job_name=每日美股大盘晚间复盘`，`executed_at=2026-05-26T20:01:28.389959+08:00`，同样被替换。原始正文是休市后现货指数和盘前风险偏好复盘。
  - `run_id=34088`，`job_name=美股大盘晚间风控简报`，`executed_at=2026-05-26T20:01:33.478830+08:00`，同样被替换。原始正文是纳指期货、标普期货、伊朗和平谈判预期、AI 芯片情绪与追涨风险。
  - `run_id=34110`，`job_name=美股纳斯达克盘前简报`，`executed_at=2026-05-26T20:31:43.309816+08:00`，同样被替换。原始正文是纳斯达克盘前、AI 半导体、美债收益率和中东局势分析。
  - `run_id=34116`，`job_name=美股盘前宏观与财报日历梳理`，`executed_at=2026-05-26T20:32:14.881555+08:00`，同样被替换。原始正文是房价数据、消费者信心、AI 芯片/云/电力和财报日历。
  - `run_id=34140`，`job_name=OWALERT_PreMarket`，`executed_at=2026-05-26T21:02:06.668531+08:00`，同样被替换。原始正文是油价低于 100、美股期货/QQQ 修复和 AI 二阶链跟踪的盘前结论，不是纯原油价格播报。
  - `run_id=34148`，`job_name=晚9点盘前推演(XME及加密ETF)`，`executed_at=2026-05-26T21:02:29.992905+08:00`，同样被替换。原始正文是美股节后复盘、期指、加密底层和 XME / ETF 盘前推演。
  - `run_id=34135`，`job_name=美股盘前分析与个股推荐`，`executed_at=2026-05-26T21:03:00.655689+08:00`，同样被替换。原始正文是 risk-on 盘面、AI/半导体领涨和追高赔率分析。
  - `run_id=34172`，`job_name=每日美股大盘风控简报`，`executed_at=2026-05-26T21:46:47.203221+08:00`，同样被替换。原始正文是美股开盘后 09:45 早盘风险偏好和指数表现分析。
- 本轮未发现新的独立 P1；这是已打开缺陷的同一 scheduler 出站 guard false positive 链路，严重等级仍为 P2，状态保持 `New`。当前影响已经从单条 A/H 收盘复盘扩展到多条美股盘前/开盘/风控类定时任务，修复优先级应保持在活跃队列。

## 复发记录（2026-05-26 19:05 CST）

- 最近四小时真实窗口再次确认同一根因复发，且有用户侧反馈：`A股港股收盘后跨市场复盘` 在 2026-05-26 17:30 CST 生成了完整 A/H 市场复盘，但 `cron_job_runs.run_id=34001` 出站前被 `commodity_causality_guarded=true` 替换成原油 / 大宗商品安全提示，仍记录为 `completed + sent + delivered=1`。
- `data/sessions.sqlite3` -> `cron_job_runs`：
  - `run_id=34001`
  - `job_id=j_fddd1589`
  - `job_name=A股港股收盘后跨市场复盘`
  - `actor_channel=feishu`
  - `executed_at=2026-05-26T17:32:32.987564+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.commodity_causality_guarded=true`
  - `detail_json.scheduler.raw_preview` 开头为“北京时间 2026年5月26日 17:30。A股、港股今天均正常开市；今天不是普涨，而是指数稳住、个股分化...”，属于非商品主任务市场复盘。
  - `response_preview` / `deliver_preview` 被替换为“本轮原油/大宗商品播报包含未完成同窗来源核验...本轮未保留原正文中的价格或归因句...”，与任务主题不符。
- 同一任务的会话落库保留原始完整复盘：
  - `session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`
  - `ordinal=319`
  - `timestamp=2026-05-26T17:32:30.206326+08:00`
  - assistant final 是完整 A 股 / 港股收盘复盘，包含事实、主线、美股预判、映射代码池、估值分层、风险与证伪条件。
- 用户侧随后在同一 Feishu 会话反馈没有看到 17:30 复盘：
  - `ordinal=320`
  - `timestamp=2026-05-26T18:07:14.409410+08:00`
  - 用户摘要：“今天17点30的复盘我没看到，重新发一遍。已经好几次这样了，你找下原因”
  - `ordinal=321` assistant 重发了复盘，但误判为飞书展示 / 投递链路边界，没有识别到台账里 `commodity_causality_guarded=true` 已把最终送达内容替换。
- 这是既有缺陷的同一 scheduler 出站 guard 链路复发，不新建重复缺陷；但当前 live 仍影响用户可见内容，导航页应回到活跃待修复。若确认只是旧二进制未重启，应以部署 / 重启 / 运行态复核作为关闭条件；若已是含 `63442662` 的二进制，则说明代码修复不足，需要继续收窄 guard 条件。

## 修复记录（2026-05-26 03:05 CST）

- 当前 HEAD 已把普通 scheduler commodity guard 的触发范围收窄到“商品任务”或“正文主体本身就是商品播报”的场景；对 `A股港股收盘后跨市场复盘`、`每日美股大盘风险简报` 这类广义市场复盘，只因局部出现“油价回落”“油气板块承压”等从句，不再整篇替换成原油 / 大宗商品安全提示。
- 具体实现位于 [`/Users/fengming2/Desktop/honeclaw/crates/hone-channels/src/scheduler.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-channels/src/scheduler.rs)：新增 broad-market-review 识别，要求普通非商品任务同时满足“任务/提示明显是市场复盘”与“正文存在多处股指/市场上下文锚点”时跳过 commodity rewrite；原油任务与正文主体明显为商品播报的普通 scheduler 仍继续命中原 guard。
- 新增回归：
  - `commodity_guard_skips_broad_market_review_with_secondary_oil_clause`
  - `commodity_guard_skips_cross_market_review_with_oil_sector_mention`
- 验证：
  - `cargo test -p hone-channels commodity_guard_covers_non_heartbeat_market_scheduler_output --lib -- --nocapture`
  - `cargo test -p hone-channels commodity_guard_skips_broad_market_review_with_secondary_oil_clause --lib -- --nocapture`
  - `cargo test -p hone-channels commodity_guard_skips_cross_market_review_with_oil_sector_mention --lib -- --nocapture`
  - `cargo test -p hone-channels commodity_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`
- 当前 live 在 2026-05-26 17:30 CST 再次复现用户可见错误，因此本缺陷状态已从 `Fixed` 调回 `New`。后续需要区分“旧二进制未重启”与“修复条件仍不足”。

## 证据来源

- 2026-05-26 11:08 CST 复核补充：旧 live 进程在最近四小时窗口继续复现同一 false positive，且样本扩展到 Feishu AI 早报、Feishu XME / 加密 ETF 早盘复盘和 Discord 降息概率推送。`hone-console-page` / `hone-feishu` 当前进程仍启动于 2026-05-22 22:52 CST，早于 2026-05-26 03:10 CST 修复提交 `63442662`，因此本轮只记录为“代码已修、旧运行态未部署/未重启”的证据，不把状态从 `Fixed` 回退为 `New`。
  - `run_id=33693`，`job_name=Hone_AI_Morning_Briefing`，`executed_at=2026-05-26T08:32:10.854666+08:00`，`completed + sent + delivered=1`，`detail_json.scheduler.commodity_causality_guarded=true`。`raw_preview` 是 AI 基建 / 宏观 / 持仓观察早报，最终 `response_preview` / `deliver_preview` 被替换成“本轮原油/大宗商品播报包含未完成同窗来源核验”的安全提示。
  - `run_id=33720`，`job_name=早9点市场复盘(XME及加密ETF)`，`executed_at=2026-05-26T09:02:18.175944+08:00`，同样命中 `commodity_causality_guarded=true`。原始 assistant final 是 XME、港股加密 ETF 与宏观大盘早盘复盘，非原油或大宗商品主任务。
  - `run_id=33745`，`job_name=每日美股降息概率推送`，`actor_channel=discord`，`executed_at=2026-05-26T09:30:53.636006+08:00`，同样命中 `commodity_causality_guarded=true`。同一 session assistant final 是降息概率 / FedWatch / PCE 风险分析，最终送达预览被替换为原油 / 大宗商品归因提示。
  - `data/runtime/logs/hone-feishu.runtime-recovery.log` 同窗记录 `[SchedulerDiag] commodity_causality_guarded`，覆盖 `Hone_AI_Morning_Briefing` 与 `早9点市场复盘(XME及加密ETF)`；`data/runtime/logs/hone-discord.runtime-recovery.log` 同窗记录 Discord `每日美股降息概率推送` 命中 commodity guard。
  - 这组样本继续说明生产运行态需要重启 / 部署后复核；如果重启到包含 `63442662` 的二进制后仍在上述非商品主任务上复现，应重新打开为 `New`。
- 2026-05-26 07:03 CST 复核补充：代码修复后，旧 live 进程仍在最近四小时窗口复现同一 false positive；但 `hone-console-page` / `hone-feishu` 当前进程启动于 2026-05-22 22:52 CST，早于 2026-05-26 03:10 CST 修复提交 `63442662`，因此本轮只记录为“代码已修、旧运行态未部署/未重启”的证据，不把状态从 `Fixed` 回退为 `New`。
  - `run_id=33558`，`job_name=OWALERT_PostMarket`，`executed_at=2026-05-26T04:32:06.154009+08:00`，`completed + sent + delivered=1`，`detail_json.scheduler.commodity_causality_guarded=true`。同一 session assistant final 是 Memorial Day 休市下的持仓股 / 观察池盘后扫描与宏观新闻复盘，但最终 `response_preview` / `deliver_preview` 被替换成“本轮原油/大宗商品播报包含未完成同窗来源核验”的安全提示。
  - `run_id=33596`，`job_name=美股收盘后跨市场复盘`，`executed_at=2026-05-26T05:31:29.535191+08:00`，同样命中 `commodity_causality_guarded=true`。原始 assistant final 明确说明美股因 Memorial Day 休市，只做 5 月 22 日简短复盘；最终送达预览仍被替换成原油 / 大宗商品提示。
  - 同窗 `run_id=33547`，`job_name=Oil_Price_Monitor_Closing`，`executed_at=2026-05-26T04:01:46.977762+08:00`，也命中 `commodity_causality_guarded=true`，但该任务本身是原油价格播报，属于修复后仍应保留的 commodity guard 适用范围，不计入 false positive。
  - 这组样本说明生产运行态仍需要重启 / 部署后复核；如果重启到包含 `63442662` 的二进制后仍在 `OWALERT_PostMarket` 或跨市场复盘上复现，应重新打开为 `New`。
- 2026-05-25 23:03 CST 复核补充：本缺陷在最近四小时继续复发，且不再局限于 A/H 复盘；普通美股大盘风控 / 温度任务也被整篇替换成原油 / 大宗商品安全提示。
  - `run_id=33277`，`job_name=每日美股大盘温度检查`，`executed_at=2026-05-25T20:01:06.183612+08:00`，`completed + sent + delivered=1`，`detail_json.scheduler.commodity_causality_guarded=true`。`raw_preview` 是 Memorial Day 休市下的美股大盘温度检查，包含 Nasdaq、S&P 500、VIX、Fear & Greed 等口径；`response_preview` 被替换为原油 / 大宗商品归因提示。
  - `run_id=33259`，`job_name=每日美股大盘风险简报`，`executed_at=2026-05-25T20:01:26.102944+08:00`，同样命中 `commodity_causality_guarded=true`。`raw_preview` 是美股大盘风险简报，非原油或大宗商品播报。
  - `run_id=33279`，`job_name=每日20点美股大盘风控简报`，`executed_at=2026-05-25T20:01:41.092619+08:00`，同样命中 `commodity_causality_guarded=true`。`raw_preview` 是休市日美股风控简报，最终送达预览变成大宗商品归因提示。
  - `run_id=33336`，`job_name=每日美股大盘风控简报`，`executed_at=2026-05-25T21:46:42.453305+08:00`，同样命中 `commodity_causality_guarded=true`。`raw_preview` 是 Nasdaq / QQQ / S&P 500 / VIX 等美股风险口径，最终送达预览被替换。
  - `session_messages` 同窗仍保留这些任务的原始 assistant final，例如 `2026-05-25T20:01:02.121877+08:00` 的 `每日美股大盘温度检查` 和 `2026-05-25T21:46:36.765517+08:00` 的 `每日美股大盘风控简报` 都有完整市场简报；错误发生在 scheduler 出站 guard 后。
  - `data/runtime/logs/hone-feishu.runtime-recovery.log` 同窗记录 `[SchedulerDiag] commodity_causality_guarded`，覆盖 `每日美股大盘温度检查`、`每日美股大盘风险简报`、`每日20点美股大盘风控简报`、`每日美股大盘风控简报`。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=33185`
  - `job_name=A股港股收盘后跨市场复盘`
  - `actor_channel=feishu`
  - `executed_at=2026-05-25T17:32:09.927021+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.commodity_causality_guarded=true`
  - `detail_json.scheduler.raw_preview` 开头是正常的 A 股 / 港股 / 美股休市复盘，包含 A 股正常开市、港股佛诞翌日休市、美股 Memorial Day 休市、A 股硬科技行情等内容。
  - `response_preview` / `detail_json.scheduler.deliver_preview` 被替换为“本轮原油/大宗商品播报包含未完成同窗来源核验的原因归因，已移除原正文中的宏观、地缘、供需、库存等主因叙述；不能视为已确认油价主因。本轮未保留原正文中的价格或归因句；请等待下一轮核验或手动查询交易所/官方数据。”
- 同一任务的会话落库仍保留原始完整复盘：
  - `session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`
  - `ordinal=291`
  - `timestamp=2026-05-25T17:32:05.982852+08:00`
  - assistant final 长度约 `4714`，正文为完整 A 股 / 港股收盘复盘。
- 用户侧随后在同一 Feishu 会话反馈没有看到 17:30 复盘，并要求重发：
  - `2026-05-25T18:37:42.488493+08:00` 用户问“今天5点半为什么没有复盘？”
  - `2026-05-25T18:38:54.665866+08:00` 用户进一步明确“17:30 的“A股港股收盘后跨市场复盘”没看到，重新发一下”
  - `2026-05-25T18:39:35.815520+08:00` assistant 手动重发完整 17:30 复盘。
- `data/runtime/logs/hone-feishu.runtime-recovery.log`
  - `2026-05-25T09:32:05.999185Z` 记录 `[SchedulerDiag] commodity_causality_guarded job_id=j_fddd1589 job=A股港股收盘后跨市场复盘 target=...`

## 端到端链路

1. Feishu scheduler 在北京时间 17:30 触发 `A股港股收盘后跨市场复盘`。
2. LLM 生成了完整市场复盘，并写入 direct session assistant final。
3. 普通 scheduler 出站前的 commodity causality guard 命中该非原油 / 非商品任务。
4. guard 把最终投递内容替换为原油 / 大宗商品安全提示。
5. `cron_job_runs` 仍记录 `completed + sent + delivered=1`，调度侧认为任务成功送达。
6. 用户在 18:37-18:38 反馈没有看到 17:30 复盘，需直聊手动重发。

## 期望效果

- 商品 / 原油安全 guard 只应拦截原油、大宗商品或明确包含未核验商品价格 / 归因的任务。
- 对跨市场复盘这类普通市场任务，即使正文中出现“油气板块承压”等市场板块描述，也不应整篇替换为大宗商品播报安全提示。
- 如果 guard 确实需要删改部分高风险归因，也应保留与任务主题相关的主体复盘内容，并在台账中可审计地标明删改范围。

## 当前实现效果

- `A股港股收盘后跨市场复盘` 以及多条美股大盘风控 / 温度简报的原始完整答案被 guard 全量替换成无关的原油 / 大宗商品提示。
- 台账仍显示发送成功和已送达，导致调度健康状态与用户实际收到的有用内容不一致。
- 会话落库保留完整 assistant final，但最终 `response_preview` 与送达内容是 guard 后的无关短提示，形成“落库看似正确、用户侧内容错误”的分叉。

## 用户影响

- 用户没有收到应有的 17:30 A 股 / 港股收盘复盘，需要主动追问和手动重发。
- 定时任务表面成功，后续巡检如果只看 `completed + sent + delivered=1` 会漏掉内容被错误替换的问题。
- 该问题影响 scheduler 的用户可见内容正确性和台账可信度，因此属于功能性 bug，定级为 P2。

## 根因判断

- 初步判断是 `guard_commodity_causality_for_event(...)` 或相关触发条件过宽：普通市场复盘正文中的“油气”、`VIX`、风险、宏观等词可能命中了商品归因 guard，但任务本身不是原油或大宗商品播报。
- 既有 `oil_price_scheduler_geopolitical_hallucination.md` 跟踪的是商品 guard 覆盖不足导致未核验油价 / 地缘归因外发；本缺陷是相反方向的 false positive，受影响链路和用户结果不同，因此单独建档。
- 需要同时复核任务名、job 类型、原文商品相关片段占比和 guard 后正文保留策略，避免用“整篇替换”处理非商品主任务。

## 下一步建议

- 在 scheduler 出站 guard 前增加任务域判断：仅任务名 / prompt / 主体输出明确为原油、大宗商品、能源价格播报时启用整篇 commodity rewrite。
- 对跨市场复盘、行业复盘、持仓复盘中的局部商品 / 油气提及，改为局部删改或追加风险提示，不要替换整篇正文。
- 新增回归：A/H 收盘复盘正文包含“油气承压”时，以及美股大盘风控正文包含 Nasdaq / S&P 500 / VIX / Fear & Greed / 长端利率 / 油价观察项时，不应触发整篇 `commodity_causality_guarded=true`；原油播报包含未核验 WTI / Brent 与地缘归因时仍应触发 guard。
- 修复后复核 `cron_job_runs.response_preview`、`detail_json.scheduler.deliver_preview` 和实际 Feishu 送达正文三者一致。

## 修复记录

- 2026-05-26 00:29 CST：已修复。`guard_commodity_causality_for_event(...)` 对商品 / 原油任务继续按原逻辑拦截；对非商品任务不再因为正文里局部出现油价 / 能源归因词就整篇改写，只有正文主体明显以商品内容为主时才启用整篇 commodity rewrite。
- 新增回归 `commodity_guard_does_not_rewrite_broad_ah_market_review` 与 `commodity_guard_does_not_rewrite_broad_us_market_risk_brief`，覆盖 A/H 跨市场复盘和美股大盘风控正文里局部提到油气 / 油价 / 能源需求时不触发 `commodity_causality_guarded`。
- 既有 `Oil_Price_Monitor_Closing`、contract-month 油价样本与 `OWALERT_PostMarket` 的 commodity guard 回归仍通过，确保未核验 WTI / Brent 价格和归因仍会被拦截。

## 验证结果

- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`：通过。
- `cargo test -p hone-channels commodity_guard_ --lib -- --nocapture`：通过，5 passed。
- `cargo test -p hone-channels commodity_ --lib -- --nocapture`：通过，13 passed。
- `cargo check -p hone-channels --tests`：通过。
