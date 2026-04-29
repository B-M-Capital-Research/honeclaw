# Bug: Feishu 定时任务持久化 `schedule` 与 prompt 触发时间错配，`20:45` 任务在 `08:30` 被错时执行

- **发现时间**: 2026-04-27 09:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实会话与消息落库：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
    - `2026-04-27T08:33:17.201578+08:00` user 消息为 `[定时任务触发] 任务名称：美股盘后AI及高景气产业链推演`
    - 同条 user 正文明确写着 `【触发时间】每个交易日 20:45（交易日）`
    - `2026-04-27T08:35:07.159373+08:00` assistant 最终回复首句直接承认：`当前时间是北京时间2026年4月27日08:33，不是你设定的20:45触发时点`
  - 最近一小时定时任务运行台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=7398`
    - `job_id=j_acce16a6`
    - `job_name=美股盘后AI及高景气产业链推演`
    - `executed_at=2026-04-27T08:30:00.810435+08:00`
    - `execution_status=running`
    - `message_send_status=pending`
    - `detail_json={"delivery_key":"j_acce16a6:2026-04-27:08:30","phase":"started"}`
    - `run_id=7415`
    - `job_id=j_acce16a6`
    - `executed_at=2026-04-27T08:35:13.789339+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 仍以 `不是你设定的20:45触发时点` 开头
  - 任务配置真相源：`data/cron_jobs/cron_jobs_feishu__direct__ou_5f995a704ab20334787947a366d62192f7.json`
    - `job.id=j_acce16a6`
    - `job.name=美股盘后AI及高景气产业链推演`
    - 持久化 `schedule={"hour":8,"minute":30,"repeat":"trading_day"}`
    - 同一 job 的 `task_prompt` 却写着 `【触发时间】每个交易日 20:45（交易日）`
    - `last_run_at=2026-04-27T08:30:00.789742+08:00`
  - 同文件对照样本：
    - `job.id=j_f4cca5ab`
    - `job.name=A股盘前高景气产业链推演`
    - `schedule={"hour":8,"minute":45,"repeat":"trading_day"}`
    - `task_prompt` 同样写 `08:45`
    - `2026-04-27 08:45` 实际按预期触发，说明本轮不是整个调度器的北京时间偏移，而是单条 job 配置错配
  - 全量配置扫描：
    - 对 `data/cron_jobs/*.json` 扫描 `schedule.hour/minute` 与 prompt 中 `【触发时间】每个交易日 HH:MM` 后，当前只发现 `j_acce16a6` 这一条错配样本
  - 现有已知缺陷对照：
    - `docs/bugs/scheduler_once_absolute_date_lost.md` 关注的是 `repeat=once` 任务丢失绝对日期
    - 本轮样本是 `repeat=trading_day` 的 recurring 任务，且错配发生在 `schedule` 与 `task_prompt` 之间，不是同一坏态

## 端到端链路

1. 用户侧保留了一条名为 `美股盘后AI及高景气产业链推演` 的 recurring Feishu 定时任务。
2. 持久化配置里的结构化 `schedule` 被保存成 `08:30 trading_day`。
3. 同一任务的自然语言 prompt 仍声明该任务应在 `每个交易日 20:45` 触发。
4. 调度器按结构化 `schedule` 在 `2026-04-27 08:30 CST` 实际触发并投递。
5. agent 在正文里识别出当前时间与 prompt 冲突，只能输出一份“不是你设定的20:45触发时点”的降级分析。
6. 用户在错误时段收到了不该在该窗口触发的任务结果，而真正的 `20:45` 盘前推演窗口没有被这条 job 覆盖。

## 期望效果

- recurring 定时任务的结构化 `schedule` 必须与对外 prompt 中声明的触发时间一致，不能一条写 `08:30`、另一条写 `20:45`。
- 调度器应只在与任务定义一致的时间点触发；像“美股盘前推演”这类任务不应在 A 股盘前的 `08:30` 窗口被送达。
- 如果用户更新了定时任务时间，持久化 `schedule`、展示文案和最终 prompt 必须原子一致更新。
- 执行前若检测到结构化 schedule 与 prompt 触发时间冲突，应阻断投递并记录为配置错误，而不是直接给用户发送错时内容。

## 当前实现效果

- 当前 `j_acce16a6` 的结构化时间与 prompt 时间处于长期分裂状态：前者是 `08:30 trading_day`，后者是 `20:45 trading_day`。
- 调度器优先相信结构化 `schedule`，因此在 `2026-04-27 08:30 CST` 实际触发，并在同一用户会话中送达。
- agent 只能在内容层承认“现在不是 20:45”，但这并没有阻止错时执行本身，用户仍然收到了错误时间窗的报告。
- 同一文件中的 `j_f4cca5ab`（`08:45`）运行正常，说明问题不是整个 scheduler 的时区统一错配，而是单个 job 持久化状态损坏或任务治理更新未完全落盘。

## 用户影响

- 用户会在错误时间收到“本应 20:45 才执行”的盘前推演，打乱自己的信息消费节奏。
- 因为输出正文明确承认当前并非设定时点，用户会感知到系统的定时任务不可信，影响任务治理的基本信任。
- 这是功能性 bug，不是 P3 质量波动：问题直接落在 scheduler 核心正确性，属于“在错误时段执行并投递错误任务”。
- 之所以定级为 `P2` 而不是 `P1`，是因为当前证据只覆盖单条 job、单一用户和错时投递，没有出现跨用户扩散、数据安全风险或整批任务失控。

## 根因判断

- 高概率是定时任务创建/更新链路在某次任务治理后只更新了 prompt 文案或任务名称，没有同步更新结构化 `schedule`。
- 另一种可能是任务复制/重命名时复用了旧 `08:30` schedule，但把 prompt 改成了 `20:45` 的新语义，导致“结构化真相源”和“自然语言真相源”分裂。
- 当前运行前缺少配置一致性校验，因此坏数据会直接进入调度执行，而不是被拒绝或标记为损坏。

## 下一步建议

- 排查 `cron_job` 创建/更新路径，确认哪些入口会修改 `task_prompt`、`name`、`schedule`，并补原子一致性校验。
- 为持久化任务增加健康检查：读取 `task_prompt` 中的 `【触发时间】` 与结构化 `schedule` 比对，发现冲突则阻断执行并告警。
- 巡检现有 `data/cron_jobs/*.json`，把所有同类 `schedule/prompt` 错配任务列出来并修正；本轮扫描暂只发现 `j_acce16a6` 一条。
- 补回归测试覆盖“更新 recurring 任务触发时间后，`schedule` 与 `task_prompt` 同步变更，且只在新时点触发”。

## 修复情况（2026-04-28）

- `memory/src/cron_job/schedule.rs` 新增 `prompt_declared_schedule_time` / `prompt_schedule_conflict`，只解析 `【触发时间】` 所在行里的 `HH:MM`。
- `CronJobStorage::add_job` / `update_job` 会拒绝 prompt 声明时间与结构化 schedule 不一致的新写入。
- `CronJobStorage::get_due_jobs` 会跳过历史错配任务并记录 warning，避免坏数据继续按错误结构化时间投递。
- 验证：`cargo test -p hone-memory prompt_schedule_time_mismatch --lib`。
