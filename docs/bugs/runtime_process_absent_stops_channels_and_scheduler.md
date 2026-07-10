# Bug: Runtime 进程缺席导致渠道直聊与 scheduler 台账停止推进

## 发现时间

- 2026-07-10 19:24 CST

## Bug Type

- System Error

## 严重等级

- P1

## 状态

- Fixed

## GitHub Issue

- [#53](https://github.com/B-M-Capital-Research/honeclaw/issues/53)

## 证据来源

- `data/sessions.sqlite3`
  - 本轮巡检时间：2026-07-10 19:21 CST。
  - 最近四小时窗口 `2026-07-10T15:21:00+08:00` 之后，`session_messages` 新增 user turn 为 0、assistant final 为 0，`cron_job_runs` 新增记录为 0。
  - 上次巡检后到本轮之间，`session_messages.max(timestamp)=2026-07-10T12:57:28.964094+08:00`，`session_messages.max(imported_at)=2026-07-10T12:57:28.971307+08:00`。
  - `cron_job_runs.max(executed_at)=2026-07-10T14:01:27.621121+08:00`；最新 run 为 `关注股重大事件心跳检测：SNDK LITE COHR MU 000660.KS RKLB TEM`，落成 `execution_failed + skipped_error + delivered=0`。
  - 11:02-14:01 CST 仍有 87 条 heartbeat run 和 1 条普通 scheduler run，说明同日早些时候调度台账仍在推进；14:01 之后完全停止。
- `data/runtime/logs/*.log`
  - `data/runtime/logs/feishu_screen.log` 最新修改时间为 2026-07-10 14:01:27 CST。
  - `data/runtime/logs/backend_screen.log` 最新修改时间为 2026-07-10 14:14:52 CST。
  - `data/runtime/logs/discord_screen.log` 最新修改时间为 2026-07-10 13:27:34 CST。
  - `data/runtime/logs/acp-events.log` 最新修改时间为 2026-07-10 12:57:28 CST。
- 进程表
  - 2026-07-10 19:23 CST 执行 `ps -axo pid,ppid,stat,lstart,command | rg -i 'hone|feishu|discord|telegram|scheduler|target/debug|target/release'`，没有发现 `hone-feishu`、`hone-discord`、`hone-web-api`、`hone-desktop`、`hone-cli` 或 scheduler 运行进程；命中项仅为本轮巡检命令本身和无关系统扩展。
- 最近提交
  - 2026-07-10 11:02 CST 之后没有非文档代码提交可解释运行态变化。

## 端到端链路

1. Feishu / Discord / Web runtime 进程负责接收直聊、执行 scheduler due jobs、写入 `session_messages` 和 `cron_job_runs`。
2. 2026-07-10 12:57 CST 后会话消息不再落库，14:01 CST 后调度运行台账不再新增。
3. 到 19:21 CST 巡检时，最近四小时没有任何真实会话消息或调度 run。
4. 同时进程表未见 Hone 渠道或后端运行进程，运行日志也停止刷新。
5. 因此新直聊、普通 scheduler、heartbeat scheduler 都可能无法被接收、触发、落账或投递。

## 期望效果

- 渠道 runtime 和 scheduler 应持续运行；若进程退出，应由 supervisor / launchctl / desktop runtime 自动拉起。
- 即使单个任务失败，也应继续产生后续 `cron_job_runs`，并记录失败原因。
- 若整体 runtime 不在，应有健康检查或告警记录，不能静默停止数小时。

## 当前实现效果

- 最近四小时没有会话消息、没有调度 run、没有新的 ACP 事件日志。
- 14:01 CST 之后，原本每 30 分钟应持续推进的 heartbeat 台账也停止新增。
- 进程表没有可见 Hone runtime 进程，说明问题不只是某个任务输出结构化失败，而是运行承载进程缺席或未被监督拉起。

## 用户影响

- 这是功能性缺陷，不是质量性 bug。
- 用户直聊可能无人接收或无回复；普通 scheduler 和 heartbeat scheduler 可能整轮漏执行，并且不会写入失败台账。
- 影响范围跨 Feishu / Discord / Web 后端与 scheduler，而不是单个 actor、单个任务或单次回答质量。
- 定级为 P1：核心消息接入和定时交付链路停止推进数小时，但当前证据未显示错投、数据破坏或跨用户泄漏，因此不是 P0。

## 根因判断

- 直接证据指向 runtime 进程缺席或进程已退出后未被 supervisor 拉起。
- 该问题不同于 `feishu_scheduler_no_runs_after_midnight.md`：旧缺陷是 Feishu direct 仍可运行但 scheduler loop 不再产生 run；本轮则是会话消息、ACP 事件、Feishu/Discord/backend 日志和调度台账整体停止推进，并且进程表没有 Hone runtime 进程。
- 该问题也不同于 `feishu_scheduler_running_rows_never_finalized.md`：本轮不是 started row 长期悬挂，而是 14:01 后没有新 run 被创建。
- 仍需后续修复任务继续确认：进程是正常退出、panic、被外部 supervisor 停止、资源耗尽后无法重启，还是当前环境的 launch/supervision 配置未覆盖这些 sidecar。

## 下一步建议

- 先核对当前运行方式和 supervisor 状态：launchctl / desktop managed children / sidecar supervisor 是否仍认为 Feishu、Discord、backend 应该运行。
- 检查 14:01-14:14 CST 之间的 backend / Feishu 日志尾部是否存在退出、panic、资源耗尽、SIGTERM 或 supervisor stop 信号。
- 为渠道 sidecar 和 scheduler 增加进程级健康检查：发现日志和 `cron_job_runs` 超过一个 heartbeat 周期不推进时，应告警或自动重启。
- 若确认是当前机器手工停止或外部维护导致，应在运行态台账中记录维护窗口，避免自动化把计划停机误判为产品缺陷。

## 运行态恢复复核（2026-07-10 23:03 CST）

- **结论**：本缺陷从 `New` 更新为 `Fixed`。19:24 CST 记录的“运行承载进程缺席、会话与调度台账完全不推进”在 23:03 CST 已不再成立。
- **证据来源**：
  - 进程表在 22:10 CST 后可见 `target/debug/hone-cli start --build`、`target/debug/hone-console-page`、`target/debug/hone-feishu`、`target/debug/hone-discord` 与 Web UI dev server 进程。
  - `data/sessions.sqlite3` 的 shadow 会话镜像已恢复推进：`sessions.max(updated_at)=2026-07-10T23:01:31.638783+08:00`，`session_messages.max(timestamp)=2026-07-10T23:01:31.624477+08:00`，最近四小时窗口新增 8 个 user turn 与 7 条 assistant final；Feishu direct、Feishu scheduler 与 Web scheduler 均有 assistant 收口。
  - 当前 runtime 日志显示 `cloud runtime config detected cloud_postgres=true`，SQLite `cron_job_runs` 是旧 shadow/本地表，不再作为本轮 cloud scheduler 主台账判断依据。
  - cloud PostgreSQL `cloud_cron_job_runs.max(executed_at)=2026-07-10T23:01:33.443243+08:00`，19:02-23:03 CST 新增普通 scheduler `completed + sent + delivered=1` 71 条、普通 scheduler `execution_failed + send_failed + delivered=0` 9 条、heartbeat `noop + skipped_noop` 143 条、heartbeat `completed + sent + delivered=1` 31 条、heartbeat 失败 19 条。
  - `cloud_web_push_messages.max(created_at)=2026-07-10T23:00:54.663752+08:00`，说明 Web push inbox 也在推进。
- **剩余观察**：
  - 本轮只证明运行态恢复，不证明已补齐进程级 supervisor / 健康检查根因修复。
  - 若后续再次出现 `cloud_cron_job_runs`、`session_messages`、runtime 日志与进程表同时停滞，应重新打开本单，而不是新建重复缺陷。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码。
- 已验证范围：`data/sessions.sqlite3` 最近四小时与上次巡检后的会话 / cron 台账、`data/runtime/logs/*.log` 最新修改时间、进程表、最近非文档代码提交。
- 未验证范围：未重启服务，未运行代码测试，未进入 supervisor / launchctl 状态修复。
- 2026-07-10 23:03 CST 复核新增验证：cloud PostgreSQL scheduler / Web push 表、SQLite shadow 会话镜像、当前 runtime 进程表与本轮 runtime 日志。
