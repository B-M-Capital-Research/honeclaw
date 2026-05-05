# Bug: Feishu 定时任务持久化 `schedule` 与 prompt 触发时间错配，`20:45` 任务在 `08:30` 被错时执行

- **发现时间**: 2026-04-27 09:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: Approved
- **证据来源**:
  - 首次命中时的真实会话与消息落库：`data/sessions.sqlite3` -> `session_messages`
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
  - 最近一小时运行日志：`data/runtime/logs/web.log.2026-05-04`
    - `2026-05-05 00:36:00.368` WARN `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 job=美股盘后AI及高景气产业链推演 schedule=08:30 prompt=20:45`
    - `2026-05-05 00:37:00.369` 同一 warning 再次出现
    - `2026-05-05 00:38:00.367` 同一 warning 再次出现
    - `2026-05-05 01:00:00.379` 同一 warning 再次出现
  - 当前配置再次核对：`data/cron_jobs/cron_jobs_feishu__direct__ou_5f995a704ab20334787947a366d62192f7.json`
    - `job.id=j_acce16a6`
    - `schedule={"hour":8,"minute":30,"repeat":"trading_day"}`
    - `task_prompt` 仍写 `【触发时间】每个交易日 20:45（交易日）`
    - `last_run_at=2026-05-04T08:30:03.113964+08:00`
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
4. 旧逻辑曾按结构化 `schedule` 在 `2026-04-27 08:30 CST` 实际触发并投递。
5. `2026-04-28` 加入一致性校验后，新写入坏任务会被拒绝，历史坏任务到点会被直接跳过并记录 warning。
6. 但历史坏 job `j_acce16a6` 并未自动修复；最近一小时仍在 `00:36`、`00:37`、`00:38`、`01:00` 被重复扫描并跳过。
7. 用户虽然不再收到错时内容，但这条任务仍不会在其声明的 `20:45` 时点正常执行。

## 期望效果

- recurring 定时任务的结构化 `schedule` 必须与对外 prompt 中声明的触发时间一致，不能一条写 `08:30`、另一条写 `20:45`。
- 调度器应只在与任务定义一致的时间点触发；像“美股盘前推演”这类任务不应在 A 股盘前的 `08:30` 窗口被送达，也不应长期处于“每分钟扫描一次然后跳过”的坏态。
- 如果用户更新了定时任务时间，持久化 `schedule`、展示文案和最终 prompt 必须原子一致更新。
- 执行前若检测到结构化 schedule 与 prompt 触发时间冲突，应阻断投递并记录为配置错误，而不是直接给用户发送错时内容。

## 当前实现效果

- 当前 `j_acce16a6` 的结构化时间与 prompt 时间仍处于长期分裂状态：前者是 `08:30 trading_day`，后者是 `20:45 trading_day`。
- `2026-04-28` 的止血只覆盖了“阻断新坏写入 + 跳过历史坏任务”，没有把现存坏 job 修正为一致配置。
- 因此旧坏数据虽然不再对用户错时投递，但仍会在调度窗口里持续被扫描、告警、跳过，任务本身保持不可用。
- 同一文件中的 `j_f4cca5ab`（`08:45`）与 `j_0be83582`（`20:00`）运行正常，说明问题不是整个 scheduler 的时区统一错配，而是单个 job 持久化状态损坏且缺少自动修复/停用治理。

## 用户影响

- 用户历史上已经收到过错误时间的盘前推演；当前虽然不再错时投递，但任务仍然不会在声明的 `20:45` 正常执行，等价于这条定时任务长期失效。
- 调度器持续输出 mismatch warning，说明系统仍保留损坏配置且缺少自动收敛，任务治理可信度继续受损。
- 这是功能性 bug，不是 P3 质量波动：问题直接落在 scheduler 核心正确性，当前坏态从“错时执行”转成了“持续跳过导致任务不可用”。
- 之所以定级为 `P2` 而不是 `P1`，是因为当前证据只覆盖单条 job、单一用户和错时投递，没有出现跨用户扩散、数据安全风险或整批任务失控。

## 根因判断

- 高概率是定时任务创建/更新链路在某次任务治理后只更新了 prompt 文案或任务名称，没有同步更新结构化 `schedule`。
- 另一种可能是任务复制/重命名时复用了旧 `08:30` schedule，但把 prompt 改成了 `20:45` 的新语义，导致“结构化真相源”和“自然语言真相源”分裂。
- 当前已补运行前一致性校验，但缺少“历史坏数据修复/停用/迁移”步骤，因此坏 job 会长期残留在生产配置里反复命中 warning。

## 下一步建议

- 排查 `cron_job` 创建/更新路径，确认哪些入口会修改 `task_prompt`、`name`、`schedule`，并补原子一致性校验。
- 为历史坏任务补迁移或自愈：至少在读取配置时自动停用/纠正 `j_acce16a6` 这类 mismatch job，而不是无限期 warning + skip。
- 巡检现有 `data/cron_jobs/*.json`，把所有同类 `schedule/prompt` 错配任务列出来并修正；本轮再次扫描仍只发现 `j_acce16a6` 一条。
- 补回归测试覆盖“更新 recurring 任务触发时间后，`schedule` 与 `task_prompt` 同步变更，且历史坏数据不会长期留在活跃调度集合中”。

## 修复情况（2026-04-28）

- `memory/src/cron_job/schedule.rs` 新增 `prompt_declared_schedule_time` / `prompt_schedule_conflict`，只解析 `【触发时间】` 所在行里的 `HH:MM`。
- `CronJobStorage::add_job` / `update_job` 会拒绝 prompt 声明时间与结构化 schedule 不一致的新写入。
- `CronJobStorage::get_due_jobs` 会跳过历史错配任务并记录 warning，避免坏数据继续按错误结构化时间投递。
- 验证：`cargo test -p hone-memory prompt_schedule_time_mismatch --lib`。

## 状态更新（2026-05-05 01:04 CST）

- 本轮巡检确认：上述修复只完成“止血”，没有让历史坏任务恢复正确可用状态。
- `j_acce16a6` 仍保留在生产 `cron_jobs` 配置里，且最近一小时继续出现 `schedule=08:30 prompt=20:45` mismatch warning。
- 因此该缺陷不应继续标记为 `Fixed`；当前更准确的状态是 `Approved`，表示根因已知、止血存在，但用户任务仍未恢复到期望行为。

## 状态更新（2026-05-05 07:40 CST）

- 本轮继续确认历史坏 job 仍未退出活跃生产扫描窗口：
  - `data/runtime/logs/web.log.2026-05-04`
    - `2026-05-05 05:23:40.815` WARN `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 job=美股盘后AI及高景气产业链推演 schedule=08:30 prompt=20:45`
    - `2026-05-05 05:59:43.559` 同一 warning 再次出现
    - `2026-05-05 06:16:30.531` 同一 warning 再次出现
    - `2026-05-05 06:39:13.211` 同一 warning 再次出现
    - `2026-05-05 06:55:27.564` 同一 warning 再次出现
    - `2026-05-05 07:12:31.265` 同一 warning 再次出现
    - `2026-05-05 07:23:41.313` 同一 warning 再次出现
    - `2026-05-05 07:40:10.285` 同一 warning 再次出现
- 结论不变：
  - 2026-04-28 的修复只阻断了继续错时投递，没有修复生产中的历史坏任务。
  - `j_acce16a6` 仍以“每分钟被扫描一次、每次都 warning + skip”的方式长期存在，用户声明的 `20:45` 任务实际仍不可用。

## 状态更新（2026-05-05 02:09 CST）

- 本轮巡检确认：这条历史坏 job 仍未被迁移或停用，warning 还在持续追加。
- `data/runtime/logs/web.log.2026-05-04` 又新增：
  - `2026-05-05 01:52:49.368` `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`
  - `2026-05-05 02:09:38.027` 同一 warning 再次出现
- 这说明 2026-04-28 的一致性校验仍只覆盖“阻断未来坏写入”，没有让现存坏配置退出活跃调度扫描；状态继续维持 `Approved`。

## 状态更新（2026-05-05 08:00 CST）

- 本轮巡检确认：坏 job `j_acce16a6` 依然没有退出活跃调度扫描，最近一小时仍在按分钟追加 warning。
- `data/runtime/logs/web.log.2026-05-04` 在当前窗口又新增：
  - `2026-05-05 06:39:13.211` `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`
  - `2026-05-05 06:55:27.564` 同一 warning 再次出现
  - `2026-05-05 07:12:31.265`、`07:23:41.313`、`07:40:10.285`、`07:41:10.285`、`07:42:10.286`、`07:58:26.990`、`07:59:27.000` 同一 warning 持续追加
- 这说明当前线上坏态已经不是“偶发扫到一次再跳过”，而是历史坏任务在分钟级反复进入 due 集合；2026-04-28 的止血仍只做到了阻断错时投递，没有恢复任务可用性，也没有把坏配置迁出生产扫描路径。

## 状态更新（2026-05-05 14:01 CST）

- 本轮巡检确认：坏 job `j_acce16a6` 仍在最新日志文件中持续反复进入调度扫描。
- `data/runtime/logs/web.log.2026-05-05` 在最近一小时持续记录：
  - `2026-05-05 13:00:48.479` `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`
  - `2026-05-05 13:01:48.476` 到 `14:00:48.498` 基本每分钟都重复同一 warning，没有任何恢复或迁移迹象。
- 这说明线上坏态已经跨越多轮巡检持续到下午时段，且坏配置仍留在活跃生产扫描路径里；当前状态继续维持 `Approved`，因为止血只阻断了错时投递，没有让用户声明的 `20:45` 任务恢复可用。
