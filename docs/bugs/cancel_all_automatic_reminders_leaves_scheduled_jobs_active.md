# Bug: 取消所有自动提醒后定时与心跳任务仍可能继续触发

- 发现时间：2026-07-27 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：Fixed（代码级，待部署复核）
- GitHub Issue：无，非 P1

## 最新进展

- `2026-07-30 22:01-2026-07-31 02:02 CST` 真实 Feishu direct 会话确认该缺陷在取消全部自动发送链路复发，状态从代码级 `Fixed` 回退为 `New/P2`：
  - `data/sessions.sqlite3` -> `session_messages`
    - session_id: `Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a`
    - 23:32 CST 用户明确要求“请关掉以上的所有定时发送任务”，assistant 于 23:32 回复“已全部清理完毕。当前没有任何定时任务在运行，无需取消”。
    - 23:35 CST 用户追问“但为什么每天还要给我发几条消息呢”，assistant 才承认仍有两个每日摘要推送：08:30 盘前摘要、09:00 晨间摘要。
    - 23:40 CST 用户继续要求“这两个也都关掉”，assistant 于 23:41 暴露“推送总开关仍为开启状态（`enabled: true`）”，并要求用户再次确认 `disable_all`。
    - 23:41 CST 用户确认“好的。执行吧”，assistant 又回复“已执行。当前会话下没有任何定时任务或心跳任务，无需删除”，没有确认 digest slots 或总开关已关闭。
  - 判断：
    - 这与既有根因同链路：用户态“关闭所有自动发送”没有覆盖摘要 / notification prefs / cron / heartbeat 的统一语义，且最终回复伪成功。
    - 该问题会导致用户明确取消后仍可能继续收到自动推送，影响自动通知控制主功能链路；严重等级维持 `P2`。
    - 同窗未见跨用户错投、敏感凭据泄露、全渠道不可用或数据破坏，因此不是 P1，不创建 GitHub Issue。

- `2026-08-03 CST` 用户再次报告同一现象（Feishu 渠道）：“一直在关定时任务，怎么都关不掉，还是在继续推送提醒，消息回复说是关掉了，但是完全关不掉”。本轮定位到此前未修的结构性根因并补齐：
  - `cron_job` 只拥有 cron/heartbeat 一个存储；事件即时推送与每日摘要（08:30 / 09:00 unified digest slot）由 `notification_prefs` 拥有，删光全部 cron 也不会停。
  - `cron_job(action="remove_all")` 返回 `remaining_count: 0`（只指 cron），`cron_job(action="list")` 返回 `jobs: []`；两者都不携带其它推送来源。
  - `response_finalizer::recover_cron_job_confirmation` 由服务端直接生成用户可见句「你当前没有定时任务。」/「已删除全部 N 个定时或心跳任务。」——在 digest 仍按计划触发时，这两句就是用户看到的“说是关掉了”。
  - 用户 prefs 未显式设置 `digest_slots` 时 `effective_digest_slots()` 返回 `None`，落到系统默认槽位继续推送；用户从未“开启”过它，因此也不会想到要去关它。
  - 结论：这不是持久化失败或调度器不生效，而是一个工具回答了比它实际覆盖范围更宽的问题，且服务端把这个窄结果渲染成了全局结论。

## 用户反馈

- 用户明确要求取消所有自动提醒，但之后仍有自动任务触发。
- 这是重复出现的功能性问题；仅关闭一类推送或仅删除单个任务，会让用户误以为所有自动通知都已经停止。

## 根因

1. `notification_prefs(action="disable")` 只关闭 event-engine 的事件推送，不会删除 cron / heartbeat 任务。
2. `cron_job` 原来只有单任务 `remove`，没有 actor 级原子 `remove_all`；模型需要循环删除，容易遗漏。
3. `CronJobStorage::remove_job` 吞掉 `save_jobs` 失败，持久化未成功时仍可能返回成功。
4. scheduler 事件入队后，渠道执行和发送前没有重新确认任务是否仍存在；用户在任务排队或模型运行期间取消，旧事件仍可能完成投递。

## 修复

- 新增 `cron_job(action="remove_all")`：actor-scoped、幂等地删除当前用户全部 cron 与 heartbeat 任务，并清理对应待确认更新。
- 新增 `notification_prefs(action="disable_all")`：关闭事件即时/摘要推送，清空 digest slots，并删除当前用户全部 cron / heartbeat 任务。
- 增加 fallible `try_load_jobs` 与 `remove_all_jobs`；删除读写失败会向上返回错误，不再伪报成功。
- scheduler 在模型执行前、执行完成后和各渠道发送前重新检查 durable job；已取消任务按 `job_cancelled / skipped_cancelled` 静默收口。
- 保留 one-shot 调度语义：一次性任务在成功 claim 后虽已标记停用，仍允许完成当前唯一一次运行。
- prompt 只补充动作路由，不改变既有回答格式；finalizer 在模型未产出可见正文时回收明确的取消结果。

## 修复（2026-08-03 补充）

- `CronJobTool` 新增 `with_push_context(notif_prefs_dir, default_digest_slot_times)`；`list` 与 `remove_all` 结果附带 `automatic_push`：`all_automatic_push_stopped`、`remaining_sources`、`event_push_enabled`、`digest_source`（`user` / `system_default` / `disabled`）、`digest_times`、`stop_all_action`。
- `remaining_count: 0` 保留原义（仅 cron），不再是“没有推送了”的依据；判断依据改为 `all_automatic_push_stopped`。
- `response_finalizer` 的 cron 确认文案在 `all_automatic_push_stopped=false` 时追加仍会触发的来源，并提示一句话关闭全部；为 true 时保持原文案不变。
- `DEFAULT_CRON_TASK_POLICY` 增加两条：只有 `all_automatic_push_stopped=true` 才可以说“没有任何自动提醒”；用户反复表示“关不掉”时先按 `automatic_push` 核对来源，再直接 `notification_prefs(action="disable_all")` 收口，不重复 `remove_all` 并重复同一句确认。

## 验证

- `cargo test -p hone-memory remove_all_jobs -- --nocapture`
- `cargo test -p hone-tools --lib -- --nocapture`
- `cargo test -p hone-scheduler -- --nocapture`
- `cargo test -p hone-channels execute_scheduler_event_ --lib -- --nocapture`
- `cargo test -p hone-channels response_finalizer::tests --lib -- --nocapture`
- `cargo check -p hone-feishu -p hone-telegram -p hone-discord -p hone-web-api --all-targets`

覆盖 actor 隔离、幂等取消、损坏持久化失败、不再执行已取消队列事件、one-shot 当前 claim 正常完成及统一用户确认文案。

## 风险与后续复核

- 本修复不批量改动现有生产用户数据；只有用户再次明确发出“取消所有自动提醒”时，才执行 actor 级删除。
- 部署后需在下一轮真实用户取消请求和 scheduler 窗口复核：同 actor 不再出现新 `completed/sent`，其他 actor 的任务继续正常运行。

## 2026-08-03 验证

- `cargo test -p hone-tools --lib`（168 passed）：新增 `cron_removal_discloses_digest_and_event_pushes_that_still_fire`、`cron_removal_reports_a_full_stop_once_every_push_source_is_off`。
- `cargo test -p hone-channels --lib`（730 passed）：新增 `remove_all_confirmation_discloses_push_sources_that_survive_it`、`empty_cron_list_does_not_read_as_no_automatic_pushes`、`a_full_stop_confirmation_stays_unqualified`。
- `cargo test --workspace --lib` 全绿；44/44 finance acceptance contracts 通过。
- 待部署后复核：同一 Feishu actor 再次要求关闭定时任务时，回复必须列出仍会触发的摘要时段，或在用户确认后由 `disable_all` 收口且不再出现新的 digest 投递。
