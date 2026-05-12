# Bug: Heartbeat 监控使用 `mimo-v2.5-pro` 时批量命中 `Param Incorrect` 并漏发

- **发现时间**: 2026-05-12 23:03 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-13 03:02 CST` 复核：该缺陷仍为活跃 `New`。从 `2026-05-12T23:30:12+08:00` 到 `2026-05-13T03:00:25+08:00`，最近四小时继续新增 `82` 条同类 heartbeat 失败，覆盖 `11` 个 job；其中 10 个核心 heartbeat job 在 23:30、00:00、00:30、01:00、01:30、02:00、02:30、03:00 窗口基本连续失败。
  - `2026-05-12T22:30 CST`：`7` 条 heartbeat 同窗失败，覆盖 `DRAM 心跳监控`、`TEM破位预警`、`Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控`。
  - `2026-05-12T23:00 CST`：`9` 条 heartbeat 同窗失败，覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`伦敦金跌破4500提醒`、`TEM大事件心跳监控`、`TEM破位预警`、`RKLB异动监控`、`小米30港元破位预警`、`TSLA 正负触发条件心跳监控`。
  - 上述失败均落成 `execution_failed + skipped_error + delivered=0`。
  - `error_message` 均为 `LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)`。
  - `detail_json.failure_kind=provider_http_error`，`detail_json.heartbeat_model=mimo-v2.5-pro`。
- 最近四小时同窗仍有非 heartbeat 定时任务成功送达，例如 `run_id=19503` 的 `核心观察股池晚间快报`、`run_id=19517` 的 `科技成长股持仓买卖点日内预警`、`run_id=19521/19526/19528` 的每日动态监控均为 `completed + sent + delivered=1`，说明不是 scheduler 或 Feishu 出站全局停摆。

## 端到端链路

1. Feishu heartbeat 调度在半点 / 整点窗口批量执行多个监控 job。
2. 公共 heartbeat runner 调用 `mimo-v2.5-pro`。
3. 上游返回 `HTTP 400 Param Incorrect`。
4. 本地已能把失败分类为 `provider_http_error`，但没有自动切换模型、重试兼容参数或降级到可用 provider。
5. 多个监控 job 同窗落成 `execution_failed + skipped_error + delivered=0`，用户侧收不到本轮 heartbeat 检查结果。

## 期望效果

- Heartbeat 公共链路遇到 provider 参数兼容类 `HTTP 400` 时，应有稳定止血方案，例如切换已知可用 heartbeat provider、移除不兼容参数后重试，或将整窗故障提升为可观测告警。
- 单个 provider / 模型参数错误不应在连续半点 / 整点窗口批量压垮多个 heartbeat 监控。
- 错误分类已记录后，应进一步避免同一参数组合在同窗重复打爆所有 job。

## 当前实现效果

- 最新四小时内已累计 `82` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败。
- 失败已被正确记为 `provider_http_error`，没有被伪装成 noop；但业务效果仍是本轮监控漏发。
- 同窗普通 scheduler 仍可送达，故障集中在 heartbeat provider 参数 / 模型兼容路径。

## 用户影响

- 这是功能性 bug：影响自动监控告警链路的执行与送达，不是单纯回答质量问题。
- 多条 heartbeat 在两个连续窗口漏发，用户可能错过价格、重大事件、破位和观察池监控。
- 定级为 `P2`：影响多个监控 job，但当前证据不是全渠道不可用，也没有达到 P1 的整批连续长时间全量失效；同窗仍有普通定时任务成功送达。

## 根因判断

- 直接触发点是 heartbeat 当前使用的 `mimo-v2.5-pro` 对请求参数或调用格式返回 `Param Incorrect`。
- 既有 `scheduler_heartbeat_openrouter_402_credit_exhaustion_skips_alerts.md` 关注 OpenRouter credits / `HTTP 402` 额度问题；本单是不同 provider / model 的 `HTTP 400` 参数兼容问题。
- 既有 `scheduler_heartbeat_unknown_status_silent_skip.md` 关注模型已产出 triggered 正文但 JSON 结构坏掉导致漏发；本单在模型输出前就被 provider 拒绝，根因不同。

## 下一步建议

- 检查 `mimo-v2.5-pro` heartbeat 请求参数，确认是否包含该 provider 不支持的字段、工具配置、response format 或 token 参数。
- 对 `provider_http_error + Param Incorrect` 增加短路保护：同窗第一次失败后暂停同一 provider 参数组合，避免继续批量失败。
- 若存在备用 heartbeat provider，优先补自动 failover 或手动配置切换路径，并用 `cron_job_runs` 复核下一轮半点 / 整点窗口。
