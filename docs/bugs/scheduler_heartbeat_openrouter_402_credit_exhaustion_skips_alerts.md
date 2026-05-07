# Bug: Heartbeat 监控批量触发 OpenRouter `HTTP 402` 后整轮跳过并漏发告警

- **发现时间**: 2026-05-05 13:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
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

## 修复记录（2026-05-05 15:04 CST）

- 状态更新为 `Fixed`。
- 公共 heartbeat runner 已为上游 provider 失败补充 `failure_kind` 分类：
  - `provider_quota_exhausted`：覆盖 `HTTP 402`、credits / balance / quota 耗尽等资源不足错误。
  - `provider_http_error`：覆盖其它上游 `4xx/5xx` HTTP 失败。
  - `runner_error`：保留为非 HTTP / 非 quota 的执行失败兜底。
- Feishu 与 Web scheduler 外层日志已区分“条件未命中”和“执行失败”：只有无错误 noop 才继续记录“心跳任务未命中”；带 `error` 的 heartbeat 失败会记录为定时任务执行失败，并输出 `failure_kind`。
- 本修复不针对单次 OpenRouter 波动写特殊兼容，也不假设当前机器生产状态；它只把可预见的 provider quota / HTTP 故障纳入稳定错误边界和可观测字段，避免继续被 noop 文案掩盖。
- 验证：
  - 通过：`cargo test -p hone-channels heartbeat_provider_ --lib -- --nocapture`
  - 通过：`cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
  - 通过：`cargo test -p hone-web-api scheduler_failure_trace_required --lib -- --nocapture`
  - 通过：`cargo check -p hone-channels -p hone-web-api --tests`
- 通过：`cargo check -p hone-feishu --tests`
- 已执行：`cargo fmt --all`
- 通过：`rustfmt --edition 2024 --config skip_children=true --check bins/hone-feishu/src/scheduler.rs crates/hone-channels/src/scheduler.rs crates/hone-web-api/src/routes/events.rs`
- 未完成：`bash scripts/ci/check_fmt_changed.sh` 在当前 macOS 系统 Bash 3.2 下因缺少 `mapfile` 退出，且本机没有 `/opt/homebrew/bin/bash` 或 `/usr/local/bin/bash` 可重跑；格式以 `cargo fmt --all` 兜底。

## 状态更新（2026-05-05 17:12 CST）

- 本轮巡检确认：该缺陷在修复记录之后仍持续复发，`Fixed` 结论不成立，状态回调为 `New`。
- `data/sessions.sqlite3` 的 `cron_job_runs` 在 `2026-05-05T15:30`、`16:00`、`16:30` 三个窗口再次各出现 `11` 条 heartbeat 失败，全部落成 `execution_failed + skipped_error + delivered=0`。
- `2026-05-05T16:00` 对应 `run_id=15801-15811`，`2026-05-05T16:30` 对应 `run_id=15812-15822`；覆盖 `TEM破位预警`、`RKLB异动监控`、`全天原油价格3小时播报`、`CAI破位预警`、`持仓重大事件心跳检测`、`Monitor_Watchlist_11`、`ASTS 重大异动心跳监控`、`小米30港元破位预警`、`Cerebras IPO与业务进展心跳监控`、`ORCL 大事件监控`、`TEM大事件心跳监控`。
- `data/runtime/logs/web.log.2026-05-05` 在 `15:30:50-15:30:51` 与 `16:30:50-16:30:51` 继续记录多条 `failed deserialization ... "code":402`，随后每条 job 又落成对应 `HeartbeatDiag run_finish` / `runner_error`。
- 到本轮巡检时，`2026-05-05 12:30` 到 `16:30` 已连续 `9` 个整点/半点 heartbeat 窗口、共 `99` 条 job 落成同根因失败，说明这不是单次上游抖动，而是仍在扩大的活跃生产故障。
- 先前修复只改善了 provider 配额故障的可观测口径，并未消除 `HTTP 402` 本身，也没有阻止后续窗口继续整批漏发；因此本单应继续保留在活跃 `P1` 队列。

## 状态更新（2026-05-05 20:01 CST）

- 本轮巡检确认：故障在最近一小时继续活跃，且 `19:30` 与 `20:00` 两个窗口再次各有 `11` 条 heartbeat 全量失败。
- `data/sessions.sqlite3` -> `cron_job_runs` 最近一小时汇总：
  - `2026-05-05T19:30`：`11/11` 条 heartbeat 落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-05T20:00`：`11/11` 条 heartbeat 再次落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-05T20:01`：另有 `2` 条非 heartbeat 定时任务（Feishu `A股盘后高景气产业链推演`、Web `英伟达每日消息`）正常 `completed + sent`
- `data/runtime/logs/web.log.2026-05-05` 在 `20:00:02.320-20:00:03.134` 继续连续记录 `TEM破位预警`、`原油价格3小时播报`、`Monitor_Watchlist_11`、`RKLB`、`ORCL`、`ASTS`、`Cerebras IPO`、`持仓重大事件`、`CAI` 等 heartbeat job 的 `runner_error`，错误统一为 `upstream HTTP 402 ... can only afford 10349 ... (code: 402)`。
- 同窗存在正常送达的非 heartbeat 任务，说明当前不是 scheduler 全局停摆；故障仍集中在 heartbeat 调用 `moonshotai/kimi-k2.5` 的公共链路。
- 到本轮巡检时，`2026-05-05 12:30` 到 `20:00` 已连续 `11` 个整点/半点 heartbeat 窗口、累计 `121` 条 job 落成同根因失败；本单继续维持活跃 `P1`。

## 状态更新（2026-05-05 21:02 CST）

- 本轮巡检确认：故障在最近一小时继续活跃，且 `20:30` 与 `21:00` 两个窗口再次各有 `11` 条 heartbeat 全量失败。
- `data/sessions.sqlite3` -> `cron_job_runs` 最近一小时汇总：
  - `2026-05-05T20:30`：`11/11` 条 heartbeat 落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-05T21:00`：`11/11` 条 heartbeat 再次落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-05T21:01`：另有 `2` 条非 heartbeat 定时任务（Feishu `晚9点盘前推演(XME及加密ETF)`、`美股盘前分析与个股推荐`）正常 `completed + sent`
- `data/runtime/logs/web.log.2026-05-05` 在 `21:00:02.001-21:00:02.592` 继续连续记录多条 `failed deserialization ... "code":402`，随后 `RKLB异动监控`、`ORCL 大事件监控`、`Cerebras IPO与业务进展心跳监控` 等 heartbeat job 落成 `runner_error`，错误统一为 `upstream HTTP 402 ... can only afford 10032 ... (code: 402)`。
- 同窗存在正常送达的非 heartbeat 任务，说明当前不是 scheduler 全局停摆；故障仍集中在 heartbeat 调用 `moonshotai/kimi-k2.5` 的公共链路。
- 到本轮巡检时，`2026-05-05 12:30` 到 `21:00` 已连续 `12` 个整点/半点 heartbeat 窗口、累计 `132` 条 job 落成同根因失败；本单继续维持活跃 `P1`。

## 状态更新（2026-05-05 22:02 CST）

- 本轮巡检确认：故障在最近一小时继续活跃，且 `21:30` 与 `22:00` 两个窗口再次各有 `11` 条 heartbeat 全量失败。
- `data/runtime/logs/web.log.2026-05-05` 在 `21:30:01.888-21:30:02.854` 与 `22:00:01.843-22:00:03.447` 再次连续记录多条 `failed deserialization ... "code":402`，随后 `TEM破位预警`、`ASTS 重大异动心跳监控`、`ORCL 大事件监控`、`Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测`、`Monitor_Watchlist_11` 等 heartbeat job 全部落成 `runner_error`。
- 同窗 `web.log.2026-05-05` 仍记录 Feishu direct 会话 `Actor_feishu__direct__ou_5fb47bd113e7776b05e7a5c2c56e310652` 在 `21:54:59`、`22:02:10` 两轮正常 `session.persist_assistant -> reply.send`，说明当前不是调度器或 Feishu 出站全局不可用；故障继续集中在 heartbeat 调用 `moonshotai/kimi-k2.5` 的公共链路。
- 到本轮巡检时，`2026-05-05 12:30` 到 `22:00` 已连续 `14` 个整点/半点 heartbeat 窗口、累计 `154` 条 job 落成同根因失败；本单继续维持活跃 `P1`。

## 状态更新（2026-05-05 23:02 CST）

- 本轮巡检确认：故障在最近一小时继续活跃，且 `22:30` 与 `23:00` 两个窗口再次各有 `11` 条 heartbeat 全量失败。
- `data/sessions.sqlite3` -> `cron_job_runs` 最近一小时汇总：
  - `2026-05-05T22:30`：`11/11` 条 heartbeat 落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-05T23:00`：`11/11` 条 heartbeat 再次落成 `execution_failed + skipped_error + delivered=0`
- `data/runtime/logs/web.log.2026-05-05` 在 `23:00:01.938-23:00:04.187` 再次连续记录多条 `failed deserialization ... "code":402`，随后 `TEM大事件心跳监控`、`CAI破位预警`、`持仓重大事件心跳检测`、`ORCL 大事件监控`、`Cerebras IPO与业务进展心跳监控`、`Monitor_Watchlist_11`、`小米30港元破位预警`、`ASTS 重大异动心跳监控`、`全天原油价格3小时播报`、`RKLB异动监控` 等 heartbeat job 全部落成 `runner_error`，错误统一为 `can only afford 9103 ... (code: 402)`。
- 同窗 `cron_job_runs` 里的非 heartbeat 任务 `run_id=15923`（`核心观察股池晚间快报`）已在 `23:01:27` 正常 `completed + sent + delivered=1`，说明当前不是 scheduler 全局停摆；故障仍集中在 heartbeat 调用 `moonshotai/kimi-k2.5` 的公共链路。
- 到本轮巡检时，`2026-05-05 12:30` 到 `23:00` 已连续 `16` 个整点/半点 heartbeat 窗口、累计 `176` 条 job 落成同根因失败；本单继续维持活跃 `P1`。

## 修复记录（2026-05-05 23:09 CST）

- 状态更新为 `Fixed`。
- 本轮修复继续沿通用错误边界处理，不针对单次 OpenRouter 波动写特判：
  - 保留上一轮已落地的 `provider_quota_exhausted` / `provider_http_error` 分类与非 noop 日志收口；
  - 为 heartbeat 这类后台短检查单独把 auxiliary function-calling 的 completion token 上限固定为 `8192`，不再沿用全局 `llm.openrouter.max_tokens=32768` 或 `llm.auxiliary.max_tokens=32768`；
  - 其它普通对话、长报告和非 heartbeat scheduler 不受该 token cap 影响。
- 这直接覆盖最新证据里的 `can only afford 9103 ... max_tokens ... (code: 402)`：heartbeat 后续请求不会再以 `32768` 的 completion budget 打到同一配额边界。
- 验证：
  - 通过：`cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`
  - 通过：`cargo test -p hone-channels heartbeat_provider_ --lib -- --nocapture`
  - 通过：`cargo test -p hone-channels execution::tests::prepare_ --lib -- --nocapture`
  - 通过：`cargo check -p hone-channels --tests`

## 状态更新（2026-05-06 08:02 CST）

- 本轮巡检确认：故障跨日后仍持续活跃，`08:00` 窗口再次出现 `11/11` 条 heartbeat 全量失败。
- `data/sessions.sqlite3` -> `cron_job_runs` 最近一小时汇总：
  - `2026-05-06T08:00:02-08:00:03+08:00`：`run_id=15955-15965` 共 `11` 条 heartbeat 全部落成 `execution_failed + skipped_error + delivered=0`
  - 覆盖 `TEM破位预警`、`TEM大事件心跳监控`、`RKLB异动监控`、`Monitor_Watchlist_11`、`ORCL 大事件监控`、`持仓重大事件心跳检测`、`全天原油价格3小时播报`、`Cerebras IPO与业务进展心跳监控`、`ASTS 重大异动心跳监控`、`小米30港元破位预警`、`CAI破位预警`
- 同批 `error_message` 与 `detail_json.failure_kind` 已统一收敛为 `upstream HTTP 402 ... can only afford 6268` + `provider_quota_exhausted`，说明前一轮修复仅改善可观测字段，未消除 live provider 配额故障本身。
- 同窗 `cron_job_runs` 里 `run_id=15951-15954` 四条非 heartbeat 定时任务仍在 `07:56-07:58` 正常 `completed + sent + delivered=1`，说明当前不是 scheduler 全局停摆；故障继续集中在 heartbeat 公共链路。
- 到本轮巡检时，`2026-05-05 12:30` 到 `2026-05-06 08:00` 已累计 `17` 个整点/半点 heartbeat 故障窗口、至少 `187` 条 job 落成同根因失败；本单继续维持活跃 `P1`。

## 状态更新（2026-05-06 09:04 CST）

- 本轮巡检确认：故障在最近一小时继续活跃，`08:30` 与 `09:00` 两个窗口再次各有 `11/11` 条 heartbeat 全量失败。
- `data/sessions.sqlite3` -> `cron_job_runs` 最近一小时汇总：
  - `2026-05-06T08:30:02-08:30:04+08:00`：`run_id=15967-15979` 中 heartbeat `11/11` 条全部落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-06T09:00:03-09:00:04+08:00`：`run_id=15986-15998` 中 heartbeat `11/11` 条再次全部落成 `execution_failed + skipped_error + delivered=0`
  - 同一小时另有 `run_id=15983-15985` 与 `15990/15997` 等 `11` 条非 heartbeat 定时任务正常 `completed + sent + delivered=1`
- `data/runtime/logs/web.log.2026-05-06` 在 `08:00:02-08:00:03` 与 `09:00:03-09:00:04` 连续记录对应 `HeartbeatDiag run_finish` / `runner_error`，错误统一为 `upstream HTTP 402 ... provider_quota_exhausted`；可负担 token 预算还从 `6268` 进一步下滑到 `4596`。
- 同窗 `09:00:16-09:00:47` 还新增 `event_dedupe LLM call failed`、`pass2 baseline failed`、`pass2 personalize failed`，说明 OpenRouter credits 枯竭已开始外溢到 event-engine 摘要链路；但当前最直接的用户面故障仍是 heartbeat 整批漏发。
- 到本轮巡检时，`2026-05-05 12:30` 到 `2026-05-06 09:00` 已累计 `19` 个整点/半点 heartbeat 故障窗口、至少 `209` 条 job 落成同根因失败；本单继续维持活跃 `P1`。

## 修复记录（2026-05-06 19:04 CST）

- 状态更新为 `Fixed`。
- 本轮修复继续沿通用后台短任务预算加固处理，不依赖生产日志或当前机器线上状态：
  - heartbeat 专用 auxiliary function-calling completion token 上限从 `8192` 收紧到 `4096`。
  - 普通对话、长报告、非 heartbeat scheduler 与全局 `llm.openrouter.max_tokens` / `llm.auxiliary.max_tokens` 不受影响。
  - 继续保留既有 `provider_quota_exhausted` / `provider_http_error` 失败分类，若 provider credits 已低于 `4096` 或账号余额完全耗尽，后续仍会明确记录为 provider quota 故障，而不是伪装成 noop。
- 这次修复覆盖最新证据中的 `can only afford 4596 ... max_tokens ... (code: 402)`：heartbeat 后续请求不会再以高于该预算的 completion cap 打到同一 provider 边界。
- 验证：
  - 通过：`cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`
  - 通过：`cargo test -p hone-channels heartbeat_provider_ --lib -- --nocapture`
  - 通过：`cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
  - 通过：`cargo check -p hone-channels --tests`
  - 通过：`rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`

## 状态更新（2026-05-06 23:10 CST）

- 本轮巡检确认：`2026-05-06 19:04 CST` 的修复结论在 live 窗口里再次失效，状态从 `Fixed` 回调为 `New`。
- `data/sessions.sqlite3` -> `cron_job_runs` 在最近四小时继续出现多轮 heartbeat 整批失败：
  - `2026-05-06T22:00:02-22:00:04+08:00`：`11/11` 条 heartbeat 落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-06T22:30:02-22:30:05+08:00`：`11/11` 条 heartbeat 再次落成同类失败
  - `2026-05-06T23:00:02-23:00:10+08:00`：`run_id=16146-16157` 中 heartbeat `11/11` 条继续全部失败
- `data/runtime/logs/web.log.2026-05-06` 与 `data/runtime/logs/launch_web.latest` 在 `22:00` 与 `23:00` 窗口仍记录 `HeartbeatDiag run_start ... max_tokens=8192`，随后 OpenRouter 返回 `HTTP 402`：
  - `22:00` 窗口提示 `can only afford 4774`
  - `23:00` 窗口提示 `can only afford 4455`
- 这说明“heartbeat 专用 completion token 上限从 `8192` 收紧到 `4096`”尚未在当前 live 进程生效，或仍有路径继续用 `8192` 发起 heartbeat function-calling 请求。
- 同窗非 heartbeat 任务 `run_id=16148`（`核心观察股池晚间快报`）仍 `completed + sent + delivered=1`，说明不是 scheduler 全局停摆；故障继续集中在 heartbeat provider quota / token budget 链路。
- 该缺陷已有 GitHub Issue [#36](https://github.com/B-M-Capital-Research/honeclaw/issues/36)，本轮不重复创建 issue。

## 复核结论（2026-05-07 00:35 CST）

- 本轮按当前自动化约束，不再把当前机器旧生产进程日志作为活跃判定依据。
- 代码复核确认当前仓库 `crates/hone-channels/src/scheduler.rs` 已将 heartbeat 专用 completion token 上限固定为 `4096`，旧日志里的 `HeartbeatDiag run_start ... max_tokens=8192` 更符合未部署/旧运行态证据。
- `provider_quota_exhausted` / `provider_http_error` 分类仍保留；若外部账号余额低于 `4096` 或完全耗尽，后续会显式记录 provider quota 故障，而不是伪装成 noop。
- 状态更新为 `Fixed`；关联 GitHub Issue [#36](https://github.com/B-M-Capital-Research/honeclaw/issues/36) 建议部署当前代码后复测真实 heartbeat 窗口。
- 验证：
  - `cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`

## 状态更新（2026-05-07 07:43 CST）

- 本轮巡检确认：最近四小时该缺陷继续活跃，`02:30`、`03:03`、`05:32` 三个 heartbeat 窗口再次全量失败。
- `data/sessions.sqlite3` -> `cron_job_runs` 最近四小时汇总：
  - `2026-05-07T02:30:02-02:30:06+08:00`：`11/11` 条 heartbeat 落成 `execution_failed + skipped_error + delivered=0`
  - `2026-05-07T03:03:57-03:04:01+08:00`：`11/11` 条 heartbeat 再次落成同类失败
  - `2026-05-07T05:32:26-05:32:27+08:00`：`11/11` 条 heartbeat 继续全部失败
- 三个窗口覆盖 `ORCL 大事件监控`、`ASTS 重大异动心跳监控`、`Monitor_Watchlist_11`、`持仓重大事件心跳检测`、`TEM大事件心跳监控`、`TEM破位预警`、`RKLB异动监控`、`CAI破位预警`、`Cerebras IPO与业务进展心跳监控`、`全天原油价格3小时播报`、`小米30港元破位预警`。
- `data/runtime/logs/web.log.2026-05-06` 在 `05:32` 窗口继续记录 `failure_kind=provider_quota_exhausted`，错误统一为 OpenRouter `HTTP 402`，且 live 请求仍显示 `max_tokens=8192`、`can only afford 4434`。
- 最近四小时另有 3 条非 heartbeat Feishu scheduler run 落成 `execution_failed + sent + delivered=1`，说明 scheduler 进程仍在运行；本单仍集中在 heartbeat provider quota / token budget 链路，不是全局调度停摆。
- 该缺陷已有 GitHub Issue [#36](https://github.com/B-M-Capital-Research/honeclaw/issues/36)，本轮不重复创建 issue。

## 状态更新（2026-05-07 07:56 CST）

- 本轮补充确认：`07:43` 之后最新台账又出现两个 heartbeat 全量失败窗口，本单继续维持 `New`。
- `data/sessions.sqlite3` -> `cron_job_runs` 自上次自动化运行后的增量汇总：
  - `2026-05-07T03:03:57-03:04:01+08:00`：`11/11` 条 heartbeat 落成 `execution_failed + skipped_error + delivered=0`，错误为 OpenRouter `HTTP 402`。
  - `2026-05-07T05:32:26-05:32:27+08:00`：`11/11` 条 heartbeat 再次落成同类 `HTTP 402`。
  - `2026-05-07T06:41:35+08:00`：`11/11` 条 heartbeat 全量失败，错误统一为 `http error: error sending request for url (https://openrouter.ai/api/v1/chat/completions)`。
  - `2026-05-07T07:32:22-07:32:23+08:00`：`11/11` 条 heartbeat 又回到 OpenRouter `HTTP 402`，可负担 token 预算为 `4434`。

- `06:41` 的 transport failure 与 `07:32` 的 quota failure 都发生在同一批 heartbeat job 集合，仍覆盖 `ORCL`、`ASTS`、`Monitor_Watchlist_11`、`持仓重大事件`、`TEM`、`RKLB`、`CAI`、`Cerebras IPO`、`全天原油价格3小时播报`、`小米30港元破位预警`。
- 这不是新的独立缺陷：影响链路、用户面结果和受影响 job 集合均与本单一致，区别只是 provider 失败形态在 `HTTP 402` 与 transport send failure 之间漂移。
- 该缺陷已有 GitHub Issue [#36](https://github.com/B-M-Capital-Research/honeclaw/issues/36)，本轮不重复创建 issue。

## 复核结论（2026-05-07 15:07 CST）

- 本轮按当前自动化约束，不再把当前机器旧生产进程日志、线上 heartbeat 窗口或 OpenRouter 账号实时额度作为活跃判定依据。
- 代码复核确认当前仓库仍将 heartbeat 专用 completion token 上限固定为 `4096`，并通过 `ExecutionOptions.max_tokens_override` 进入 auxiliary LLM provider；这覆盖此前 `max_tokens=8192/32768` 触发的配额边界问题。
- `provider_quota_exhausted` / `provider_http_error` 分类仍保留；如果外部账号余额低于 `4096` 或完全耗尽，后续会显式记录 provider quota 故障，而不是伪装成 noop。
- 本轮未新增代码，因为可本地修复的通用加固已存在，后续旧运行态样本不再作为当前机器上的活跃缺陷证据。
- 状态更新为 `Fixed`；关联 GitHub Issue [#36](https://github.com/B-M-Capital-Research/honeclaw/issues/36) 建议基于当前代码部署后复测。
- 验证：
  - `cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`

## 状态更新（2026-05-08 03:05 CST）

- 本轮巡检确认：该缺陷在最近四小时再次活跃，且先前 `Fixed` 归档状态不成立，状态回调为 `New` 并移回活跃台账。
- `data/sessions.sqlite3` -> `cron_job_runs` 在 `2026-05-07T20:00:40-20:00:43+08:00` 出现 `11/11` 条 heartbeat 全量失败，全部落成 `execution_failed + skipped_error + should_deliver=0 + delivered=0`。
- 失败覆盖 `ORCL 大事件监控`、`TEM破位预警`、`RKLB异动监控`、`小米30港元破位预警`、`持仓重大事件心跳检测`、`全天原油价格3小时播报`、`CAI破位预警`、`Monitor_Watchlist_11`、`TEM大事件心跳监控`、`Cerebras IPO与业务进展心跳监控`、`ASTS 重大异动心跳监控`。
- 同批 `error_message` 统一为 OpenRouter `HTTP 402`，并明确显示 `You requested up to 8192 tokens, but can only afford 217`；`detail_json.failure_kind=provider_quota_exhausted`，`heartbeat_model=moonshotai/kimi-k2.5`。
- 同窗非 heartbeat 任务 `英伟达每日消息` 与 `A股盘后高景气产业链推演` 在 `20:02` 正常完成送达，说明不是 scheduler 全局停摆；故障仍集中在 heartbeat provider quota / token budget 链路。
- `data/runtime/logs/web.log.2026-05-07` 还在 `2026-05-08 02:42:19-02:42:25 CST` 记录多条 mainline distill / style distill 的 OpenRouter `HTTP 402`，请求 `max_tokens=30000`、可负担 `539`，说明 OpenRouter credits 枯竭仍在外溢到后台摘要链路；本单继续以 heartbeat 漏发作为 P1 用户面主故障跟踪。
- 该缺陷已有 GitHub Issue [#36](https://github.com/B-M-Capital-Research/honeclaw/issues/36)，且当前状态为 `OPEN`；本轮不重复创建 issue。

## 复核结论（2026-05-08 07:04 CST）

- 状态更新为 `Fixed`。
- 本轮按当前自动化约束，不再把当前机器旧生产进程日志、旧 sqlite 台账或 OpenRouter 账号实时额度作为活跃判定依据。
- 代码复核确认当前仓库仍将 heartbeat 专用 completion token 上限固定为 `4096`，并通过 `ExecutionRunnerSelection::AuxiliaryFunctionCalling.max_tokens_override` 进入 `create_auxiliary_llm_provider_with_max_tokens`；`run_start` 日志也会记录 `max_tokens=4096`。
- `provider_quota_exhausted` / `provider_http_error` 分类仍保留；如果外部 OpenRouter credits 低于 `4096` 或完全耗尽，后续会显式记录 provider quota 故障，而不是伪装成 noop。
- 2026-05-08 03:05 的复活证据仍来自当前机器旧运行态 / 外部额度状态，且 `can only afford 217` 已低于任何可维护的 heartbeat completion budget；继续降低到单次异常额度会让提醒质量不可解释，不符合本任务“不针对外部额度单次状态写脆弱特判”的约束。
- 本轮未新增代码给 #36，因为可本地验证的通用加固已经存在；建议部署当前代码后复测 heartbeat 窗口，若仍出现当前代码 `run_start max_tokens=4096` 下的系统性失败，再以新证据重开。
- 验证：
  - `cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`
