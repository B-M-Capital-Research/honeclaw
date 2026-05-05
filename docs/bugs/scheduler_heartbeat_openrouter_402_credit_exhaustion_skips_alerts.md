# Bug: Heartbeat 监控批量触发 OpenRouter `HTTP 402` 后整轮跳过并漏发告警

- **发现时间**: 2026-05-05 13:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#36](https://github.com/B-M-Capital-Research/honeclaw/issues/36)

## 证据来源

- 最近一小时真实台账：
  - `data/sessions.sqlite3`
  - `cron_job_runs` 最近一小时出现连续两轮整批 heartbeat 失败：
    - `2026-05-05T12:30:50-12:30:51+08:00`：`run_id=15724-15734` 共 `11` 条 heartbeat 全部落成 `execution_failed + skipped_error + delivered=0`
    - `2026-05-05T13:00:51-13:00:53+08:00`：`run_id=15735-15745` 共 `11` 条 heartbeat 再次全部落成 `execution_failed + skipped_error + delivered=0`
  - 两轮错误统一为 `LLM 错误: upstream HTTP 402: This request requires more credits, or fewer max_tokens... (code: 402)`，覆盖 `ORCL`、`ASTS`、`Cerebras`、`持仓重大事件`、`RKLB`、`TEM`、`CAI`、`Monitor_Watchlist_11`、`全天原油价格3小时播报`、`小米30港元破位预警`
- 最近一小时运行日志：
  - `data/runtime/logs/web.log.2026-05-05`
  - `2026-05-05 12:30:49.904-12:30:50.097` 与 `13:00:50.066-13:00:52.018` 连续记录多条 `failed deserialization of: {"error":{"message":"This request requires more credits, or fewer max_tokens...","code":402}}`
  - 同一窗口随后记录对应 `HeartbeatDiag run_finish ... success=false error="LLM 错误: upstream HTTP 402 ..."` 与 `runner_error`
  - 但每条失败后仍紧跟 `[Feishu] 心跳任务未命中，本轮不发送`，说明链路把真实上游额度故障压扁成“未触发”
- 去重检查：
  - `scheduler_heartbeat_unknown_status_silent_skip.md` 覆盖的是 heartbeat 输出结构化状态漂移、失败口径与 noop 口径不一致
  - `scheduler_heartbeat_minimax_http_transport_failure_no_retry.md` 覆盖的是 `MiniMax` 传输失败不重试
  - 本单是 `moonshotai/kimi-k2.5` heartbeat 公共链路在最近一小时因 OpenRouter `402 credits` 连续整批失败，根因与上游类别均不同，应独立建档

## 端到端链路

1. Feishu heartbeat 定时任务在 `12:30` 与 `13:00` 两个窗口同时触发，共执行 `11` 条监控 job。
2. 调度链路调用 `moonshotai/kimi-k2.5` 时，上游 OpenRouter 直接返回 `HTTP 402`，提示当前 credits 不足以支撑 `max_tokens=32768` 的请求。
3. 本地解析层先记录 `failed deserialization`，随后每条 job 落成 `execution_failed + skipped_error + delivered=0`。
4. 渠道侧日志仍把这些失败统一打印成“心跳任务未命中，本轮不发送”，没有显式暴露是上游额度故障。
5. 结果是：两轮 heartbeat 监控全部未送达，用户应收到的监控告警与定时播报被整批漏发。

## 期望效果

- Heartbeat 调度在上游 credits 不足或 `HTTP 402` 时，不应把任务伪装成“未命中”。
- 至少应把任务明确记为 provider / quota 故障，并保留可观测告警，避免静默漏发。
- 对这类可预见的额度失败，应有降级策略，例如降低 `max_tokens`、切换可用模型或及时中止整批调度并报警。

## 当前实现效果

- 最近一小时 `12:30` 与 `13:00` 两轮 heartbeat 任务全部失败，没有一条成功送达。
- 失败覆盖多个真实用户监控链路，不是单个 job、单个 actor 或单次抖动。
- 运行日志先暴露上游 `HTTP 402` 与 credits 不足，随后渠道侧又统一写成“未命中”，导致运维口径与真实故障原因脱节。

## 用户影响

- 这是功能性 bug，不是回答质量问题。
- 直接影响监控告警与定时播报送达：最近一小时至少 `22` 次 heartbeat 执行全部漏发。
- 之所以定级为 `P1`，是因为它让一整批生产监控任务在连续两个窗口里完全失效，影响多个用户与多条告警链路，而不是单个任务偶发降质。

## 根因判断

- 最近一小时的直接触发原因是上游 OpenRouter 额度不足，当前 `max_tokens=32768` 与剩余 credits 不匹配。
- 本地心跳调度对 `HTTP 402` 没有专门降级或限流策略，导致同一窗口内多个 job 连续打到同类失败。
- 渠道侧沿用了“未命中，本轮不发送”的收口文案，使真实 provider 故障被观测口径掩盖。

## 下一步建议

- 先按 P1 处理这条生产故障：确认 heartbeat 使用的 OpenRouter credits / 配额是否已耗尽，以及 `max_tokens` 是否明显高于当前预算。
- 在 heartbeat runner 对 `HTTP 402` 增加专门错误分类与告警，不再复用 `noop` 文案。
- 若短期无法恢复额度，先提供临时止血方案，例如收紧 `max_tokens` 或切换到可用 provider，避免后续整点窗口继续整批漏发。

## 状态更新（2026-05-05 14:01 CST）

- 本轮巡检确认：该缺陷在最近一小时内仍持续活跃，并且影响窗口继续扩大。
- `data/runtime/logs/web.log.2026-05-05` 在当前窗口又新增两轮整批失败：
  - `2026-05-05 13:30:50-13:30:53 CST` 再次出现多条 `failed deserialization of: {"error":{"message":"This request requires more credits...","code":402}}`，随后至少 `全天原油价格3小时播报`、`TEM大事件心跳监控` 落成 `run_finish + runner_error`。
  - `2026-05-05 14:00:50-14:00:52 CST` 同类 `HTTP 402` 再次覆盖 `CAI`、`Monitor_Watchlist_11`、`Cerebras IPO`、`原油价格播报`、`TEM`、`持仓重大事件`、`ASTS`、`小米30港元破位`、`ORCL`、`TEM破位`、`RKLB` 等监控 job。
- 这说明该故障并非 `12:30 / 13:00` 两个窗口的单次波动，而是在 `13:30` 与 `14:00` 窗口继续复发；到 `2026-05-05 14:01 CST` 为止，最近连续四个整点/半点 heartbeat 窗口都已出现 `HTTP 402 -> delivered=0` 的批量漏发形态。

## 状态更新（2026-05-05 15:01 CST）

- 本轮巡检确认：该缺陷在最近一小时仍持续活跃，且故障窗口继续向后滚动。
- `data/runtime/logs/web.log.2026-05-05` 在当前窗口又新增两轮整批失败：
  - `2026-05-05 14:30:49-14:30:52 CST` 连续记录多条 `failed deserialization of: {"error":{"message":"This request requires more credits...","code":402}}`，随后 `Monitor_Watchlist_11`、`全天原油价格3小时播报` 等 job 落成 `run_finish + runner_error`。
  - `2026-05-05 15:00:49-15:00:52 CST` 同类 `HTTP 402` 再次覆盖 `小米30港元破位预警`、`Cerebras IPO`、`Monitor_Watchlist_11`、`全天原油价格3小时播报`、`ORCL`、`ASTS`、`CAI`、`RKLB`、`持仓重大事件`、`TEM`、`TEM破位` 等监控 job。
- 到 `2026-05-05 15:01 CST` 为止，最近连续六个整点/半点 heartbeat 窗口都已出现 `HTTP 402 -> delivered=0` 的批量漏发形态；这已不是单窗抖动，而是生产 heartbeat 公共链路持续失效。
