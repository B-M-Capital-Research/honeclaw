# Bug: Heartbeat 使用 `mimo-v2.5-pro` 时批量触发 `HTTP 429 quota exhausted` 并漏发

- **发现时间**: 2026-05-20 19:04 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - 最近四小时窗口 `2026-05-20T15:02:00+08:00` 到 `2026-05-20T19:04:00+08:00` 内，heartbeat 任务新增 `100` 条 `execution_failed + skipped_error + delivered=0`。
  - 错误统一为 `mimo-v2.5-pro` 上游 `HTTP 429` / `quota exhausted`，其中 `21` 条已有 `detail_json.failure_kind=provider_http_error`，另有 `79` 条旧形态未写入 failure_kind。
  - 受影响 job 覆盖 `15` 条 heartbeat：`光模块板块关键事件心跳提醒`、`存储板块关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`RKLB异动监控`、`TEM大事件心跳监控`、`TEM破位预警`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`全天原油价格3小时播报`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`heartbeat_绿田机械基本面跟踪`。
  - 同窗还有 `93` 条 heartbeat `running + pending` started 残留，另有 `5` 条 heartbeat 正常 `noop + skipped_noop`。
- `data/runtime/logs/web.log.2026-05-20`
  - `19:00:33-19:03:57 CST` 连续出现 `Rate limited: Too many requests` 与 `Rate limited: quota exhausted`。
  - 同窗有 `mimo-v2.5-pro` transport retry 记录，随后 heartbeat 台账继续落成 `skipped_error`。
- `data/runtime/logs/hone-feishu.runtime-recovery.log`
  - `2026-05-20T11:00:16Z` 起密集记录上游 rate limit / quota exhausted。
- 会话质量对照：
  - 最近四小时按消息时间统计 `49` 个 user turn 与 `49` 个 assistant final，未发现孤立 user turn。
  - assistant final 污染扫描未命中空回复、通用失败、绝对路径、工具轨迹、原始 ACP `session/update`、飞书标签、compact marker、`reasoning_content`、`Param Incorrect` 或 `Resource temporarily unavailable`。
  - 说明本轮新故障集中在 heartbeat provider quota 链路，而不是直聊回复结构污染或全局会话收口失败。
- 去重检查：
  - `scheduler_heartbeat_openrouter_402_credit_exhaustion_skips_alerts.md` 覆盖的是 OpenRouter `HTTP 402` / token budget / credits 不足，当前状态为 `Fixed`。
  - `scheduler_heartbeat_mimo_param_incorrect_batch_failures.md` 覆盖的是同一 `mimo-v2.5-pro` 的 `HTTP 400 Param Incorrect` / `reasoning_content` transcript 兼容问题，当前状态为 `Fixed`。
  - 本单是 `mimo-v2.5-pro` 在当前真实窗口里触发 `HTTP 429 quota exhausted`，状态码、直接原因和最新证据均不同，因此新建独立缺陷。

## 端到端链路

1. Heartbeat scheduler 在半点 / 整点窗口批量触发多条监控 job。
2. 公共 heartbeat runner 调用 `mimo-v2.5-pro`。
3. 上游返回 `HTTP 429` / `quota exhausted`。
4. 本地将多条 job 落成 `execution_failed + skipped_error + delivered=0`；部分记录缺少 `failure_kind`。
5. 用户侧收不到本轮 heartbeat 检查结果或触发提醒。

## 期望效果

- 单个 provider quota / rate limit 不应在连续窗口压垮大量 heartbeat 任务。
- Heartbeat 公共链路应对 `HTTP 429 quota exhausted` 做专门分类、熔断或短期 failover，避免同窗继续把所有 job 打向同一不可用 provider。
- 即使无法恢复，也应保留稳定可观测状态，避免 `failure_kind` 缺失和 started-row 残留放大巡检噪音。

## 当前实现效果

- 最近四小时 `100` 条 heartbeat 因 `HTTP 429 quota exhausted` 失败并未送达。
- 失败覆盖 `15` 个 heartbeat job，不是单个任务配置问题。
- 同窗直聊会话仍能正常收口，说明不是全局消息系统宕机；故障集中在 heartbeat + `mimo-v2.5-pro` provider quota 路径。

## 用户影响

- 这是功能性 bug，不是质量性 bug。
- 用户可能错过价格破位、重大事件、持仓财报、板块关键事件和观察池等自动监控提醒。
- 定级为 `P1`：最近四小时内多达 `100` 条 heartbeat 执行失败，覆盖多个真实监控 job，直接影响自动告警送达链路；虽然直聊未受影响，但自动监控主功能在该 provider 路径上出现批量失效。

## 根因判断

- 直接触发点是 `mimo-v2.5-pro` 上游 rate limit / quota exhausted。
- 当前 heartbeat 执行链路缺少面向 `HTTP 429 quota exhausted` 的专门熔断、降级或 provider failover；部分记录还缺少 `failure_kind`，说明错误分类不够稳定。
- 既有 `Param Incorrect` 修复只覆盖 reasoning transcript 回传，不覆盖本轮 provider quota / rate limit。

## 下一步建议

- 优先确认当前 `mimo-v2.5-pro` quota / rate limit 是否已耗尽，以及 heartbeat 是否应临时切到可用 provider。
- 在 heartbeat runner 中把 `HTTP 429` / quota exhausted 归入稳定 failure_kind，并考虑同窗 provider-level 熔断，避免批量 job 重复打爆同一 provider。
- 若短期无法恢复 provider 配额，先降低 heartbeat 并发或切换备用模型，避免后续半点 / 整点继续批量漏发。
