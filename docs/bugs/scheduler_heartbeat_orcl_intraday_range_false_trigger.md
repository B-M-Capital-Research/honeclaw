# Bug: ORCL heartbeat 将盘中振幅误判为单日涨跌幅并发送错误触发提醒

- **发现时间**: 2026-04-22 03:06 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4260`
    - `job_id=j_39a96b7a`
    - `job_name=ORCL 大事件监控`
    - `executed_at=2026-04-22T03:00:37.681551+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 写出：`触发条件已满足。满足条件：单日盘中涨跌幅超过5%...当日最高$185.34，当日最低$176.01，盘中振幅约5.3%，当日收涨+3.55%至$183.88`
    - `detail_json.scheduler.raw_preview` 同一轮内部分析先写出：`今日变化：+$6.3（+3.55%）`、`当前是 +3.55%，没有超过5%`
  - `data/runtime/logs/web.log`
    - `2026-04-22 03:00:34.556` 记录同一任务 `parse_kind=JsonTriggered`
    - 同一日志的 `raw_preview` 先判断 `当前是 +3.55%，没有超过5%`
    - 紧接着 `deliver_preview` 却写成 `触发条件已满足。满足条件：单日盘中涨跌幅超过5%` 并实际投递
  - 对照最近同一任务前序窗口：
    - `run_id=4239`，`2026-04-22T02:00:24.297529+08:00`，同任务落成 `noop + skipped_noop`
    - `run_id=4252`，`2026-04-22T02:30:37.407803+08:00`，同任务仍落成 `noop + skipped_noop`
    - 两轮 `detail_json.raw_preview` 都将 ORCL 当前涨幅约 `3.44%-3.59%` 判为未超过 `5%`；到 `03:00` 才将同一日高低点区间改解释为触发条件。

## 端到端链路

Feishu heartbeat scheduler 触发 `ORCL 大事件监控` -> function-calling runner 查询 ORCL 行情与新闻 -> 模型在自由文本中判断当前涨幅未超过 5% -> 输出被调度器解析为 `JsonTriggered` -> Feishu scheduler 按 `completed + sent` 发送给用户。

## 期望效果

`单日盘中涨跌幅超过5%` 这类价格阈值应有稳定、明确的计算口径。若任务语义要求相对昨收的单日涨跌幅，则 ORCL 当前 `+3.55%` 不应触发；若允许按日内高低点振幅触发，也应在任务配置或提示中明确区分“涨跌幅”和“振幅”，并避免与前序内部判断冲突。

## 当前实现效果

同一轮运行里，模型先判断 ORCL 当前涨幅 `+3.55%` 未超过 `5%`，但最终发送的用户可见消息改用 `当日最高 $185.34` 与 `当日最低 $176.01` 计算约 `5.3%` 的盘中振幅，并宣称“触发条件已满足”。系统把这条结果落为 `completed + sent + delivered=1`。

## 用户影响

用户会收到一次错误或至少口径不一致的 ORCL 重大事件提醒，误以为 ORCL 已满足“单日涨跌幅超过 5%”的监控条件。该问题会直接影响定时监控可信度和用户后续交易/关注决策，因此是功能性告警错误，定级为 P2。

## 根因判断

初步判断不是 Feishu 发送失败或通用 JSON 解析失败，而是 heartbeat 任务的条件判定缺少结构化、可验证的阈值计算层：模型自由文本同时存在“当前涨幅未超过 5%”与“盘中振幅超过 5% 应触发”两套口径，调度器只消费最终 `triggered` 结果，没有校验触发原因与任务阈值语义是否一致。

## 下一步建议

- 为 heartbeat 价格阈值增加机器可计算的结构化判定字段，例如 `metric=close_to_prev_close_change_pct` / `intraday_range_pct`，避免模型临场混用口径。
- 对 `JsonTriggered` 增加最低限度的触发依据校验：当 message 声称命中涨跌幅阈值时，应携带基准价、当前价、计算结果和阈值。
- 短期可先收紧 ORCL heartbeat prompt，将“单日盘中涨跌幅”明确改写为用户真实需要的口径，并禁止把 high-low 振幅当作涨跌幅。
