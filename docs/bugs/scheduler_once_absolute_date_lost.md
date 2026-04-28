# Bug: 一次性定时任务丢失绝对日期，提前执行并禁用原本未来提醒

- **发现时间**: 2026-04-23 09:00 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实会话与消息落库：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34`
    - `2026-04-23T08:30:59.889542+08:00` 用户态触发消息为 `ADTN财报后总结`，正文明确写着“在北京时间2026年5月5日早上执行”
    - `2026-04-23T08:31:39.777795+08:00` assistant 最终回复承认“这是一次提前触发”，并说明真正判断窗口仍是 `2026年5月5日`
  - 最近一小时定时任务运行台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4889`
    - `job_id=j_1745ff3c`
    - `job_name=ADTN财报后总结`
    - `executed_at=2026-04-23T08:31:42.531850+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
  - 任务配置证据：`data/cron_jobs/cron_jobs_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34.json`
    - `schedule={"hour":8,"minute":30,"repeat":"once"}`
    - `task_prompt` 仍包含“在北京时间2026年5月5日早上执行”
    - 巡检时该任务已 `enabled=false`，且 `last_run_at=2026-04-23T08:30:59.870829+08:00`

## 端到端链路

1. 用户创建或保留了一个一次性财报后复盘任务，业务意图是 `2026-05-05` 早上执行。
2. 持久化任务配置只保存了 `hour=8`、`minute=30`、`repeat=once`，没有保存目标年月日。
3. 调度器在 `2026-04-23 08:30 CST` 到达时把该任务视为到期任务触发。
4. Agent 执行后识别出财报尚未发布，给用户发送了一份“提前触发/财报前状态校验”的内容。
5. 因为任务是 `once`，执行后被禁用，原本 `2026-05-05` 的提醒窗口存在丢失风险。

## 期望效果

- 一次性任务如果来自“北京时间2026年5月5日早上执行”这类绝对日期，应把目标日期、时区和时分一起持久化。
- 调度器只能在目标日期当天到达目标时间后触发，不能仅凭时分在创建后的第一个同名时间点触发。
- 如果任务配置缺少绝对日期但 prompt 内含未来日期，系统应拒绝保存、要求澄清，或在执行前判定为未到期并跳过。
- 对财报、公告、电话会等事件型提醒，提前触发后不应自动消耗一次性任务。

## 当前实现效果

- 2026-04-28 修复后，`CronSchedule` 新增结构化 `date` 字段，仅允许 `repeat=once` 使用。
- `memory/src/cron_job/storage.rs` 在触发前校验 `repeat=once + date`，当前北京时间日期未到目标日时不会返回 due job，因此不会提前投递或禁用未来一次性提醒。
- `cron_job` 工具与 Web cron API 已同步支持保存和更新 `date`，scheduler event 也会把权威触发配置传给渠道执行层。

## 用户影响

- 用户会在错误日期收到提醒，且收到的是“尚未发布/提前触发”的降级内容，而不是目标日期的财报后复盘。
- 真正的未来提醒可能被提前消耗，用户在财报电话会后无法收到原计划的自动复盘。
- 这是功能性 bug，不是 P3 质量波动：它影响定时任务的核心正确性和持久化任务生命周期，且造成用户可见的错误投递，因此定级为 `P2`。
- 之所以不是 `P1`，是因为本轮证据只覆盖单个 once 任务，且 Agent 已识别时间不匹配，未把未发布财报伪装成已发布结论。

## 根因判断

- `once` 任务的数据模型疑似只表达“下一次到达某个时分时执行”，没有表达绝对执行日期。
- 创建任务时，绝对日期被保留在自然语言 `task_prompt`，但没有进入调度器可校验的结构化字段。
- 执行前缺少“自然语言未来日期 vs 当前日期”的到期一致性校验，导致 Agent 只能在任务已经触发后做内容层自救。

## 下一步建议

- 后续可单独做一次历史任务健康检查，把 prompt 中含明确未来日期但旧配置缺少 `schedule.date` 的存量任务迁移到结构化日期；本次代码已防止新建和已带日期任务再次提前触发。

## 修复与验证

- 2026-04-28: `CronSchedule` 新增 `date`，cron tool / Web API / schedule view / scheduler event 同步透传。
- 2026-04-28: `memory/src/cron_job/storage.rs` 在 due job 判断中跳过未到目标日期的一次性任务。
- 2026-04-28: `cargo test -p hone-memory once_jobs_with_future_date_do_not_run_today --lib`
