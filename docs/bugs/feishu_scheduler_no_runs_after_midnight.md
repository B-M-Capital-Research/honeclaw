# Bug: Feishu scheduler 00:26 后不再产生新 run，导致 trading_day 任务漏执行

- **发现时间**: 2026-06-01 23:04 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#47](https://github.com/B-M-Capital-Research/honeclaw/issues/47)

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 本轮巡检窗口按 `datetime(timestamp)` 归一化为 `2026-06-01 18:58:34` 到 `2026-06-01 22:58:34` CST。
  - 窗口内共有 `34` 个 user turn 与 `35` 个 assistant turn；Feishu direct 最新会话均有 assistant 收口，多出的 assistant 是 `21:58` 的对话上限提示，不构成未回复缺陷。
  - `session_id=Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3`：
    - `2026-06-01T21:23:31+08:00` 用户询问定时任务是否仍在。
    - `2026-06-01T21:24:16+08:00` assistant 确认常规定时任务仍启用，并指出 `美股持仓开盘前晚报` 上次运行停在 `2026-05-29 20:00`。
    - `2026-06-01T21:25:41+08:00` 用户明确指出今天北京时间 `20:00` 的任务没有执行。
    - `2026-06-01T21:27:04+08:00` assistant 确认 `2026-06-01 20:00` 没有写入成功执行记录，且 2026-06-01 是美股交易日。
    - `2026-06-01T21:28:16+08:00` 用户要求补充执行漏掉的任务。
    - `2026-06-01T21:29:02+08:00` assistant 声称已补建一次性任务 `j_15913f67`，执行时间为 `2026-06-01 21:30`。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 全库 `max(executed_at)` 仍为 `2026-06-01T00:26:00.908925+08:00`。
  - 最新三条 run 是 `AAOI / RKLB / TEM 每日动态监控` 的 `running + pending + detail.phase=started`，均为 `2026-06-01T00:26:00+08:00`。
  - 对用户确认的常规任务 `job_id=j_b81788a6` 查询，最近真实成功运行仍是 `2026-05-29T20:03:23+08:00` 的 `completed + sent + delivered=1`；`2026-06-01T00:25:22+08:00` 只有启动恢复写入的 `execution_failed + send_failed` 历史收口记录。
  - 对补跑任务 `job_id=j_15913f67` 查询没有任何 `cron_job_runs` 记录，说明 `21:30` 补跑没有进入调度执行台账。
- 最近四小时运行日志
  - `data/runtime/logs/acp-events.log` 同窗仅见 Feishu direct 流式 chunk、tool update 与 `stopReason=end_turn`，未见 scheduler run 生成或终态写入。
- 最近四小时非文档代码提交
  - 无。

## 端到端链路

1. 用户已有 Feishu `trading_day` 常规定时任务 `美股持仓开盘前晚报`，北京时间 `20:00` 应在美股交易日触发。
2. `2026-06-01` 是周一且不是 NYSE 休市日，用户在 `21:25` 明确反馈 `20:00` 任务未执行。
3. assistant 查询任务状态后确认任务仍启用，但成功运行时间仍停在 `2026-05-29 20:00`。
4. 用户要求补跑后，assistant 创建了 `21:30` 一次性补跑任务。
5. `cron_job_runs` 全库在 `00:26` 后没有任何新 run，既没有 `20:00` 常规任务，也没有 `21:30` 补跑任务。

## 期望效果

- Feishu scheduler 在进程启动后应持续扫描 due jobs。
- 到点任务至少应写入一条 `cron_job_runs` started row；随后收口为成功、失败或 noop。
- 若 scheduler 全局停摆，应有健康检查、失败告警或重启恢复，不能只在用户追问时才暴露。

## 当前实现效果

- `session_messages` 说明 Feishu direct 直聊仍可正常收发，底层会话写入没有全局停摆。
- 但 `cron_job_runs` 在 `2026-06-01 00:26 CST` 后完全没有新记录。
- 用户确认的 `20:00` trading_day 任务漏执行；系统侧补建的 `21:30` 一次性任务也没有落入 run 台账。

## 用户影响

- 这是功能性缺陷，不是回答质量问题。
- 用户配置的定时投研任务不会按计划执行，且补跑任务也无法兑现。
- 影响范围可能覆盖所有 Feishu scheduler due jobs，而不是单个任务正文生成失败。
- 定级为 `P1`：核心 scheduler 交付链路停止产生新 run，用户已经感知到漏执行；但直聊主链路仍可用，未见跨用户错投或数据破坏，因此不是 `P0`。

## 根因判断

- 该问题不同于 `feishu_scheduler_run_stuck_without_cron_job_run.md` 中的“任务已注入会话 / started row 长期不收口”：本轮新增证据显示 `00:26` 之后 due job 根本没有进入 `cron_job_runs`。
- 该问题也不同于 `feishu_scheduler_running_rows_never_finalized.md` 的 started-row finalize 噪音：这里的主要损害是新任务不再触发 / 不再落账。
- 代码复核确认一个可本地加固的根因：cloud mode 下 `CronJobStorage::get_due_jobs(...)` 会通过 `run_cloud_cron(...)` 同步桥访问 PG authoritative cron backend；此前该桥对 `list_cron_job_records` / `try_claim_cron_due_job` 等 future 没有超时边界。
- `HoneScheduler::check_due_jobs()` 在单一 scheduler loop 内执行。只要一次 cloud cron future 无界等待，Feishu runtime heartbeat 和直聊仍可继续，但 scheduler loop 会停在该 tick，后续不再扫描 due jobs，也不会创建新的 `cron_job_runs`。

## 修复与验证（2026-06-02）

- `memory/src/cron_job/mod.rs`
  - `run_cloud_cron(...)` 新增默认 15 秒超时边界。
  - 可通过 `HONE_CLOUD_CRON_TIMEOUT_SECS` 调整超时时间。
  - cloud cron future 超时后返回 `HoneError::Storage("cloud cron operation timed out ...")`，由现有调用方按错误记录 warning / 跳过本次操作，避免永久卡死 scheduler loop。
- 新增回归 `cloud_cron_timeout_returns_storage_error_instead_of_blocking`，证明 stuck cloud cron future 会在有界时间内返回 storage error。
- 验证通过：
  - `cargo test -p hone-memory cloud_cron_timeout_returns_storage_error_instead_of_blocking -- --nocapture`
  - `cargo test -p hone-memory --lib -- --nocapture`
  - `cargo check -p hone-scheduler --tests`
  - `rustfmt --edition 2024 --config skip_children=true --check memory/src/cron_job/mod.rs`

## 剩余观察点

- 本轮不依赖当前机器生产运行态作为恢复证据；当前机器不是生产机器，不能用 live `cron_job_runs` 推断线上已恢复。
- 若后续继续出现 cloud cron timeout warning，应优先排查 PG 连接池、网络延迟或 cloud runtime schema/claim SQL 性能，而不是写 Feishu 渠道特判。
