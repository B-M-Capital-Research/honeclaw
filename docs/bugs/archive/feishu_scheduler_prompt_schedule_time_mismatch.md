# Bug: Feishu 定时任务持久化 `schedule` 与 prompt 触发时间错配，`20:45` 任务在 `08:30` 被错时执行

- **发现时间**: 2026-04-27 09:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: Fixed
- **GitHub Issue**: 无
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

## 状态更新（2026-05-05 15:01 CST）

- 本轮巡检确认：坏 job `j_acce16a6` 在下午窗口仍未退出活跃调度扫描。
- `data/runtime/logs/web.log.2026-05-05` 在最近一小时继续持续记录：
  - `2026-05-05 14:01:48.497` `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`
  - `2026-05-05 14:02:48.501` 到 `15:00:48.518` 基本每分钟都重复同一 warning，没有任何恢复、停用或迁移迹象。
- 这说明历史坏配置仍以“分钟级重复进入 due 集合”的方式滞留在线上；当前状态继续维持 `Approved`，因为它虽然不再错时投递，但用户声明的 `20:45` 任务依旧不可用。

## 状态更新（2026-05-05 18:13 CST）

- 本轮巡检确认：坏 job `j_acce16a6` 在最近一小时仍持续进入活跃调度扫描窗口，没有任何迁移或停用迹象。
- `data/runtime/logs/web.log.2026-05-05` 在本轮观察窗口继续新增：
  - `2026-05-05 17:12:12.882` `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`
  - `2026-05-05 17:28:29.997`、`17:45:50.028`、`18:12:25.894` 同一 warning 持续重复
- 最近一小时并没有新的 direct 会话落库，但 `cron_job_runs` 同窗已新增 `217` 条记录，说明不是调度器整体停摆；坏 job 只是继续被 due 扫描后跳过。
- 结论不变：
  - 2026-04-28 的止血仍只覆盖“阻断继续错时投递”，没有让历史坏任务恢复正确 schedule，也没有把它迁出生产扫描路径。
  - 当前状态继续维持 `Approved`，因为用户声明的 `20:45` 任务依旧不可用，且坏配置仍在最新窗口里反复制造 warning。

## 状态更新（2026-05-05 20:01 CST）

- 本轮巡检确认：坏 job `j_acce16a6` 在最近一小时仍未退出活跃调度扫描。
- `data/runtime/logs/web.log.2026-05-05` 在 `19:43:00.525`、`19:44:00.523`、`19:45:00.525`、`19:46:00.524`、`19:47:00.527`、`19:48:00.526`、`19:49:00.528`、`19:50:00.527`、`19:51:00.529`、`19:52:00.528`、`19:53:00.529`、`19:54:00.529`、`19:55:00.531`、`19:56:00.529`、`19:57:00.531`、`19:58:00.530`、`19:59:00.531`、`20:00:00.529`、`20:01:00.534` 持续记录同一 warning：`skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`。
- 这说明当前线上坏态已经不是“偶尔扫到一下”，而是历史坏任务在分钟级持续进入 due 扫描并被跳过；止血只拦住了错时投递，没有恢复用户声明的 `20:45` 任务可用性。

## 状态更新（2026-05-05 21:02 CST）

- 本轮巡检确认：坏 job `j_acce16a6` 在最近一小时仍未退出活跃调度扫描。
- `data/runtime/logs/web.log.2026-05-05` 在 `20:43:00.545`、`20:44:00.543`、`20:45:00.544`、`20:46:00.543`、`20:47:00.545`、`20:48:00.544`、`20:49:00.545`、`20:50:00.545`、`20:51:00.547`、`20:52:00.543`、`20:53:00.545`、`20:54:00.544`、`20:55:00.546`、`20:56:00.546`、`20:57:00.546`、`20:58:00.546`、`20:59:00.547`、`21:00:00.546`、`21:01:00.549`、`21:02:00.548` 持续记录同一 warning：`skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`。
- 这说明当前线上坏态已经跨到 `21:00` 盘前窗口后仍以分钟级重复进入 due 扫描并被跳过；止血只拦住了错时投递，没有恢复用户声明的 `20:45` 任务可用性，也没有把历史坏配置迁出生产扫描路径。

## 状态更新（2026-05-05 22:02 CST）

- 本轮巡检确认：坏 job `j_acce16a6` 在最近一小时仍未退出活跃调度扫描。
- `data/runtime/logs/web.log.2026-05-05` 在 `21:42:03.444` 之后到 `22:02:00.569` 之间继续几乎逐分钟记录同一 warning：`skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 ... schedule=08:30 prompt=20:45`。
- 这说明历史坏配置在盘后窗口继续以分钟级重复进入 due 集合并被跳过；止血仍只拦住了错时投递，没有恢复用户声明的 `20:45` 任务可用性，也没有把坏配置迁出生产扫描路径。

## 修复情况（2026-05-06）

- `CronJobStorage::get_due_jobs` 在扫描历史 cron JSON 时不再对 schedule/prompt 时间错配任务无限 warning + skip。
- 对非 heartbeat 且 `task_prompt` 的 `【触发时间】` 行可解析出 `HH:MM` 的历史 job，扫描阶段会一次性把结构化 `schedule.hour/minute` 对齐到 prompt 声明时间并写回 cron JSON。
- 修复后的任务随后按新 schedule 继续参与正常 due 判定：旧 `08:30` 槽不再反复进入活跃扫描，用户声明的 `20:45` 任务可在对应窗口恢复可用。
- 新写入和更新路径既有一致性校验保持不变，仍会拒绝新产生的 schedule/prompt 错配。
- 回归测试：`due_jobs_repair_existing_prompt_schedule_time_mismatch` 覆盖历史坏配置会被修复、持久化并按修复后的时间触发。
- 验证：
  - `cargo test -p hone-memory prompt_schedule_time_mismatch --lib -- --nocapture`
  - `cargo check -p hone-memory --tests`
  - `rustfmt --edition 2024 --check memory/src/cron_job/storage.rs memory/src/cron_job/mod.rs`

## 状态更新（2026-05-06 23:10 CST）

- 本轮巡检确认：上述修复结论在 live 日志里仍未成立，状态从 `Fixed` 回调为 `New`。
- `data/runtime/logs/web.log.2026-05-06` 在最近四小时继续记录同一历史坏 job 的 `schedule/prompt mismatch`：
  - `2026-05-06 21:42:00-22:00:00 CST` 几乎逐分钟出现 `skipping cron job with schedule/prompt mismatch: job_id=j_acce16a6 job=美股盘后AI及高景气产业链推演 schedule=08:30 prompt=20:45`
  - `2026-05-06 23:01:00`、`23:02:00`、`23:03:00`、`23:04:00` 继续出现同一 warning
- `data/runtime/logs/launch_web.latest` 同窗也记录 `2026-05-06T15:03:00Z` 的同类 skip warning，说明不是单个日志文件残留。
- 当前坏态仍是历史 `08:30` 槽没有被迁移出活跃扫描；止血避免错时发送，但用户声明的 `20:45` 任务仍未恢复可用，且调度器继续分钟级扫描并跳过。
- 这仍是功能性 bug：它影响 scheduler 任务正确触发与历史坏配置收敛，不属于 P3 质量波动；影响范围仍限单条历史 job，因此严重等级维持 `P2`。

## 复核结论（2026-05-07 00:35 CST）

- 本轮按当前自动化约束，不再用当前机器旧生产进程日志维持活跃判定。
- 代码复核确认当前仓库 `CronJobStorage::get_due_jobs` 会在扫描时修复历史非 heartbeat job 的 prompt/schedule 时间错配，并把结构化 `schedule.hour/minute` 持久化对齐到 `【触发时间】` 行声明的 `HH:MM`。
- 新写入 / 更新路径仍保留一致性校验，避免继续产生 schedule/prompt 分裂。
- 状态更新为 `Fixed`；若部署当前代码后仍出现 `j_acce16a6 schedule=08:30 prompt=20:45` 的新 warning，再按新证据重开。
- 验证：
  - `cargo test -p hone-memory prompt_schedule_time_mismatch --lib -- --nocapture`
