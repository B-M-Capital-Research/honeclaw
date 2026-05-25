# Bug: Scheduler commodity guard falsely replaces non-commodity market reviews with oil guard notice

- **发现时间**: 2026-05-25 19:05 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 证据来源

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
