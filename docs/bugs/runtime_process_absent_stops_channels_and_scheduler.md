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

## 最新进展

- 2026-08-02 22:04 CST 运行态复核，状态从 `New/P1` 调整为 `Fixed/P1`：
  - 证据来源：
    - 进程表：
      - 22:02 CST 可见 source runtime 进程 `data/releases/source/39ce9ce54f5cbfea26e664459cb70edf3fd97292/hone-console-page`，启动时间约 20:58 CST；不再是上一轮“未见 Hone runtime / Web / scheduler 进程”的整体缺席状态。
    - `data/logs/hone-console-page-source.log`
      - 21:00-22:02 CST 有 44128 行新增运行日志。
      - 21:00 / 21:30 / 22:00 CST 多批 `HeartbeatDiag run_finish`、`deliver`、`duplicate_suppressed` 持续出现，说明 scheduler runner 已恢复执行。
      - 20:59-22:02 CST event-engine poller 也持续记录 `poller ok`，包括 `fmp.news`、`fmp.earnings`、`fmp.macro`、`fmp.price`、`fmp.extended_hours`。
    - `data/sessions.sqlite3`
      - `sessions.max(imported_at)=2026-08-02T20:59:58.506373+08:00`，说明本地 session mirror 有重新导入动作。
      - 真实消息时间仍停在 `session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`，`cron_job_runs.max(executed_at)=2026-08-01T14:00:52.724451+08:00`，但这已不再证明 runtime 进程缺席；调度台账未随 live runtime 推进另回退到 `sessions_sqlite_mirror_stalled_after_successful_direct_replies.md`。
    - 最近提交：
      - 18:00-22:04 CST 有非文档提交 `39ce9ce5 feat: add admin usage analytics`，与本缺陷修复链路无直接关系；本轮状态变化来自 live runtime 观测恢复。
  - 判断：
    - 本单根因范围是运行承载进程缺席导致直聊 / scheduler 无法推进；本轮已有 source runtime、scheduler 和 event-engine live 证据，因此不能继续保持活跃 P1。
    - 由于 `cron_job_runs` / `web_push_messages` 本地 SQLite 台账仍未跟上 live 日志，本轮只将本单从活跃队列移出为 `Fixed`，不推进 `Closed`。
    - 已有关联 Issue #53；本轮没有新增活跃 P1，不创建重复 GitHub Issue。

- 2026-08-02 18:02 CST 运行态持续活跃复核，状态保持 `New/P1`：
  - 证据来源：
    - `data/sessions.sqlite3`
      - `session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`
      - `session_messages.max(imported_at)=2026-08-01T14:13:46.194864+08:00`
      - `sessions.max(updated_at)=2026-08-01T14:13:46.184727+08:00`
      - `sessions.max(last_message_at)=2026-08-01T14:13:46.183054+08:00`
      - `cron_job_runs.max(executed_at)=2026-08-01T14:00:52.724451+08:00`
      - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`
      - 2026-08-02 14:02-18:02 CST 窗口内新增 session / message / cron run / web push 均为 0。
    - `data/runtime/logs/*.log` 与 `data/logs/*.log`
      - 2026-08-02 14:02-18:02 CST 窗口内没有新的 `.log` mtime 推进。
      - 最近 runtime 相关日志仍停在 `sunny_ngrok_screen_20260731.log` 的 2026-08-01 14:21 CST，以及 `web.log.2026-08-01` / `web_deploy_49ef8dd4_20260731.log` / `feishu_deploy_49ef8dd4_20260731.log` 的 14:14 CST。
    - 进程表：
      - 18:02 CST 仍未见 `hone-cli`、`hone-feishu`、`hone-discord`、`hone-web-api`、`hone-desktop` 或 scheduler 运行进程；命中项仅为无关系统任务、本地 PostgreSQL 和本轮查询进程。
    - 最近提交：
      - 14:02-18:02 CST 没有非文档代码提交。
  - 判断：
    - 本轮没有新的真实 assistant final 可检查答非所问、格式污染、错投或内部错误外泄；缺陷信号仍集中在运行承载链路缺席。
    - 该问题仍是功能性 P1：会话接入、scheduler 触发、Web push 与运行台账在真实窗口内继续整体停滞。
    - 该证据与既有 Issue #53 和本单同根因、同链路、同影响范围，不新建重复缺陷，也不重复创建 GitHub Issue。

- 2026-08-02 14:02 CST 运行态持续活跃复核，状态保持 `New/P1`：
  - 证据来源：
    - `data/sessions.sqlite3`
      - `session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`
      - `session_messages.max(imported_at)=2026-08-01T14:13:46.194864+08:00`
      - `sessions.max(updated_at)=2026-08-01T14:13:46.184727+08:00`
      - `sessions.max(last_message_at)=2026-08-01T14:13:46.183054+08:00`
      - `cron_job_runs.max(executed_at)=2026-08-01T14:00:52.724451+08:00`
      - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`
      - 2026-08-02 10:02-14:02 CST 窗口内新增 session / message / cron run / web push 均为 0。
    - `data/runtime/logs/*.log` 与 `data/logs/*.log`
      - 最近 260 分钟没有新的 `.log` mtime 推进。
      - 最近 runtime 相关日志仍停在 `sunny_ngrok_screen_20260731.log` 的 2026-08-01 14:21 CST，以及 `web.log.2026-08-01` / `web_deploy_49ef8dd4_20260731.log` / `feishu_deploy_49ef8dd4_20260731.log` 的 14:14 CST。
    - 进程表：
      - 14:02 CST 仍未见 `hone-cli`、`hone-feishu`、`hone-discord`、`hone-web-api`、`hone-desktop` 或 scheduler 运行进程；命中项仅为无关系统任务和本轮查询进程。
    - 最近提交：
      - 10:02-14:02 CST 没有非文档代码提交；期间只有缺陷台账文档提交。
  - 判断：
    - 本轮没有新的真实 assistant final 可检查答非所问、格式污染、错投或内部错误外泄；缺陷信号仍集中在运行承载链路缺席。
    - 该问题仍是功能性 P1：会话接入、scheduler 触发、Web push 与运行台账在真实窗口内继续整体停滞。
    - 该证据与既有 Issue #53 和本单同根因、同链路、同影响范围，不新建重复缺陷，也不重复创建 GitHub Issue。

- 2026-08-02 10:02 CST 运行态持续活跃复核，状态保持 `New/P1`：
  - 证据来源：
    - `data/sessions.sqlite3`
      - `session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`
      - `session_messages.max(imported_at)=2026-08-01T14:13:46.194864+08:00`
      - `sessions.max(updated_at)=2026-08-01T14:13:46.184727+08:00`
      - `sessions.max(last_message_at)=2026-08-01T14:13:46.183054+08:00`
      - `cron_job_runs.max(executed_at)=2026-08-01T14:00:52.724451+08:00`
      - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`
      - 2026-08-02 06:01-10:02 CST 窗口内新增 session / message / cron run / web push 均为 0。
    - `data/runtime/logs/*.log` 与 `data/logs/*.log`
      - 最近 260 分钟没有新的 `.log` mtime 推进。
      - 最近 runtime 相关日志仍停在 `web.log.2026-08-01` / `web_deploy_49ef8dd4_20260731.log` / `feishu_deploy_49ef8dd4_20260731.log` 的 14:14 CST，以及 `sunny_ngrok_screen_20260731.log` 的 14:21 CST。
    - 进程表：
      - 10:02 CST 仍未见 `hone-cli`、`hone-feishu`、`hone-discord`、`hone-web-api`、`hone-desktop` 或 scheduler 运行进程；命中项仅为无关系统进程、本地 PostgreSQL 和本轮 Codex 进程。
    - 最近提交：
      - 06:01-10:02 CST 唯一提交为上一轮缺陷台账文档提交 `99bfc069 docs: update bug patrol ledger`；无非文档代码提交。
  - 判断：
    - 本轮没有新的真实 assistant final 可检查答非所问、格式污染、错投或内部错误外泄；缺陷信号仍集中在运行承载链路缺席。
    - 该问题仍是功能性 P1：会话接入、scheduler 触发、Web push 与运行台账在真实窗口内继续整体停滞。
    - 该证据与既有 Issue #53 和本单同根因、同链路、同影响范围，不新建重复缺陷，也不重复创建 GitHub Issue。

- 2026-08-02 06:01 CST 运行态复核，状态从代码级 `Fixed` 回退为 `New/P1`：
  - 证据来源：
    - `data/sessions.sqlite3`
      - `session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`
      - `session_messages.max(imported_at)=2026-08-01T14:13:46.194864+08:00`
      - `sessions.max(updated_at)=2026-08-01T14:13:46.184727+08:00`
      - `cron_job_runs.max(executed_at)=2026-08-01T14:00:52.724451+08:00`
      - 2026-08-02 02:01-06:01 CST 窗口内新增 session / message / cron run 均为 0。
    - `data/runtime/logs/*.log` 与 `data/logs/*.log`
      - 最近 5 小时没有新的 `.log` mtime 推进。
      - 最近 runtime 相关日志仍停在 `web.log.2026-08-01` / `web_deploy_49ef8dd4_20260731.log` / `feishu_deploy_49ef8dd4_20260731.log` 的 14:14 CST，以及 `sunny_ngrok_screen_20260731.log` 的 14:21 CST。
    - 进程表：
      - 06:01 CST 仍未见 `hone-cli`、`hone-feishu`、`hone-discord`、`hone-web-api`、`hone-desktop` 或 scheduler 运行进程；命中项仅为无关系统进程、本地 PostgreSQL 和本轮 Codex 进程。
    - 最近提交：
      - 02:01-06:01 CST 唯一非文档代码提交为 `d896fb46 fix: detach source runtime supervisor startup`，属于本缺陷的 source runtime 代码级修复；但本轮没有看到 live runtime 进程、日志或台账恢复证据。
  - 判断：
    - 该问题仍是功能性 P1：会话接入、scheduler 触发和运行台账在真实窗口内继续整体停滞。
    - 03:08 CST 的代码修复降低了下次按新入口启动后的复发风险，但当前 live 运行态仍未恢复，不能保持 `Fixed`。
    - 该证据与既有 Issue #53 和本单同根因、同链路、同影响范围，不新建重复缺陷，也不重复创建 GitHub Issue。

- 2026-08-01 11:45 CST 代码级修复，状态更新为 `Fixed`：
  - 修复内容：
    - `bins/hone-cli/src/start.rs` 为 `hone-cli start` 新增 `--detach`。当前台源码入口使用 `cargo run -p hone-cli -- start --build --detach` 时，前台 CLI 先完成本地 build，再由同一 CLI 自行拉起 detached supervisor，统一把启动日志写到 `data/logs/hone-cli-start.log`，并继续沿用 `data/runtime/current.pid` 作为权威 supervisor pid。
    - `scripts/restart_hone.sh` 同步改为调用 `hone-cli start --build --detach`，避免后台重启链路和人工 source-start 链路再使用两套生命周期语义。
    - `docs/runbooks/desktop-dev-runtime.md` 与 `docs/repo-map.md` 同步切到新的 source runtime 契约，避免继续建议把长期运行态绑在临时 shell 前台。
  - 验证：
    - `cargo test -p hone-cli start::tests -- --nocapture`
    - `cargo check -p hone-cli --tests`
    - `rustfmt --edition 2024 --check bins/hone-cli/src/start.rs`
    - `bash -n scripts/restart_hone.sh`
    - `git diff --check`
  - 当前边界：
    - 本轮没有重启现有 live runtime，也没有做 launchctl / 外部 supervisor 运行态复核；因此先记代码级 `Fixed`，不推进 `Closed`。
    - 2026-08-01 18:03 / 22:03 以及先前台账里的“运行承载进程缺席”证据，仍代表旧启动契约或旧运行实例上的真实坏态；需要后续在新入口下继续观察是否复发。

- 2026-08-01 22:03 CST 运行态持续活跃复核，状态保持 `New/P1`：
  - 证据来源：
    - `data/sessions.sqlite3`
      - `session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`
      - `session_messages.max(imported_at)=2026-08-01T14:13:46.194864+08:00`
      - `sessions.max(updated_at)=2026-08-01T14:13:46.184727+08:00`
      - `cron_job_runs.max(executed_at)=2026-08-01T14:00:52.724451+08:00`
      - 18:03-22:03 CST 窗口内新增 session / message / cron run 均为 0。
    - `data/runtime/logs/*.log` 与 `data/logs/*.log`
      - 18:03 CST 后没有新的 `.log` mtime 推进。
      - 最近 runtime 相关日志仍停在 `web.log.2026-08-01` / `web_deploy_49ef8dd4_20260731.log` / `feishu_deploy_49ef8dd4_20260731.log` 的 14:14 CST，以及 `sunny_ngrok_screen_20260731.log` 的 14:21 CST。
    - 进程表：
      - 22:03 CST 仍未见 `hone-cli`、`hone-feishu`、`hone-discord`、`hone-web-api`、`hone-desktop` 或 scheduler 运行进程；命中项仅为无关系统进程、本地 PostgreSQL 和本轮 Codex 进程。
    - 最近提交：
      - 18:03-22:03 CST 无非文档代码提交。
  - 判断：
    - 本轮没有新的真实 assistant final 可检查答非所问、格式污染或错投；缺陷信号仍集中在运行承载链路缺席。
    - 该证据与 18:03 CST 回退记录属于同一根因、同一影响范围，不新建重复缺陷。
    - 已有关联 GitHub Issue #53，本轮确认活跃 P1 但不重复创建 Issue。

- 2026-08-01 18:03 CST 运行态重新确认并从 `Fixed/P1` 回退为 `New/P1`：
  - 证据来源：
    - `data/sessions.sqlite3`
      - `session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`
      - `session_messages.max(imported_at)=2026-08-01T14:13:46.194864+08:00`
      - `sessions.max(updated_at)=2026-08-01T14:13:46.184727+08:00`
      - `cron_job_runs.max(executed_at)=2026-08-01T14:00:52.724451+08:00`
      - 14:00-18:03 CST 窗口内只有 1 个 Feishu direct user turn 与 1 条 assistant final，14:12 用户要求更新 NVDA 画像，14:13 assistant 正常收口并发送成功。
    - `data/runtime/logs/web.log.2026-08-01`
      - 14:14:36 CST 记录 `Feishu 渠道已停止`，之后该日志不再刷新。
    - `data/runtime/logs/web_deploy_49ef8dd4_20260731.log`
      - 同窗尾部记录 `[INFO] shutdown requested` 与 `当前没有活动聊天任务，继续关闭服务`。
    - 18:03 CST 进程表未见 `hone-cli`、`hone-feishu`、`hone-discord`、`hone-web-api`、`hone-desktop` 或 scheduler 运行进程；命中项仅为无关系统进程、本轮 Codex 进程和本地 PostgreSQL。
  - 判断：
    - 14:14 后会话消息、runtime 日志和 scheduler 台账均停止推进，且本地没有可见 Hone runtime 进程；当前证据再次落在本单“运行承载进程缺席导致直聊 / scheduler 无法推进”的同一链路。
    - 日志表现为主动 shutdown，可能是维护或迁移动作；但本地台账没有维护窗口标记，且当前巡检目标是记录真实运行链路状态，因此先按活跃 P1 回退。
    - 已有关联 GitHub Issue #53，本轮不重复创建。若后续确认这是受控迁移且新生产 runtime 有独立健康证据，应将本单转回 `Fixed` 或 `Closed` 并补充维护窗口证据。

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
