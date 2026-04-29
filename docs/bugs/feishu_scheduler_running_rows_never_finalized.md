# Bug: Feishu scheduler 预写的 `running/pending` 台账不会被终态覆盖，长期残留为悬挂运行中

- **发现时间**: 2026-04-28 17:02 CST
- **Bug Type**: System Error
- **严重等级**: P3
- **状态**: New

## 证据来源

- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 09:02 CST` 再次复核，started 残留继续在最新 `08:45`、`09:00` 两个窗口实时新增，而且普通 scheduler 与 heartbeat 在同一小时窗同时复现：
    - `08:45` 窗口普通 scheduler 先写入 `run_id=9764`（`A股盘前高景气产业链推演`）`running + pending`，随后另起 `run_id=9765` 并在 `08:46:11` 落成 `completed + sent + delivered=1`
    - `09:00` 窗口又先写入 `run_id=9766-9780` 共 `15` 条 started 行，其中既包含 heartbeat，也包含 `09:00 美股AI与航空科技晨报`、`早9点市场复盘(XME及加密ETF)`、`核心观察池早间简报`、`特斯拉与火箭实验室新闻日报`
    - 同窗终态随后另起为 `run_id=9781-9796`：`9794`、`9792`、`9793`、`9787` 都已落成 `completed + sent + delivered=1`，`9795` 已落成 `execution_failed + skipped_error`，`9781-9791` 大多已回写为 `noop + skipped_noop`，`9796` 还落成 `completed + send_failed`
    - 但对应 started 行 `9766-9780` 仍全部保留 `running + pending`，说明 started 行不会被任何一种终态覆盖，无论终态是 `sent`、`send_failed`、`skipped_noop` 还是 `skipped_error`
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态继续扩大为：
    - `running + pending = 35`
    - `noop + skipped_noop = 21`
    - `completed + sent = 16`
    - `completed + send_failed = 1`
    - `execution_failed + skipped_error = 1`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1514` 条，较 `08:02` 巡检时的 `1479` 再增 `35` 条，说明这条缺陷继续按“每新增一轮 started 行就永久堆积”的模式稳定恶化
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 08:02 CST` 再次复核，started 残留继续在最新 `07:30`、`08:00` 两个窗口实时新增：
    - `07:30` 窗口 started 行为 `run_id=9678-9681`，并与同窗其它 started 行 `run_id=9679-9693` 共同形成新一批 `running + pending`
    - 同窗终态另起为 `run_id=9682-9693`，其中 `9687`（`小米30港元破位预警`）与 `9692`（`Monitor_Watchlist_11`）都已落成 `completed + sent + delivered=1`，但对应 started 行 `9679` 与 `9678` 仍永久保留 `running + pending`
    - `08:00` 窗口 started 行为 `run_id=9694-9709`，同窗终态已开始另起为 `run_id=9710-9721`
    - 其中 `run_id=9711`（`HoneClaw每日使用Tips`）已落成 `completed + sent + delivered=1`，`run_id=9710/9712-9721` 大多已回写为 `noop + skipped_noop`，但对应 started 行 `9695`、`9694/9696-9709` 仍继续残留
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态仍是占比最高的状态：
    - `running + pending = 28`
    - `noop + skipped_noop = 22`
    - `completed + sent = 3`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1479` 条，较 `07:02` 巡检时的 `1451` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 07:02 CST` 再次复核，started 残留继续在最新 `22:30`、`23:00` 两个 heartbeat 窗口实时新增：
    - `22:30` 窗口 started 行为 `run_id=9622-9633`，同窗终态另起为 `run_id=9634-9645`
    - `23:00` 窗口 started 行为 `run_id=9646-9657`，同窗终态已开始另起为 `run_id=9658-9669`
    - 其中 `run_id=9639`（`小米破位预警`）、`9643`（`ORCL 大事件监控`）与 `9669`（`持仓重大事件心跳检测`）都已分别落成 `completed + sent + delivered=1`，但同窗 started 行 `9633`、`9627` 与 `9646` 仍永久保留 `running + pending`
    - `run_id=9638` 还已落成 `execution_failed + skipped_error`，但对应 started 行 `9626` 同样没有被覆盖；`9658-9668` 里大多已回写为 `noop + skipped_noop`，对应 `9647-9657` 这批 started 行也仍继续残留
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态仍是占比最高的状态：
    - `running + pending = 24`
    - `noop + skipped_noop = 20`
    - `completed + sent = 3`
    - `execution_failed + skipped_error = 1`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1451` 条，较 `06:02` 巡检时的 `1427` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 06:02 CST` 再次复核，started 残留继续在最新 `05:30`、`06:00` 两个 heartbeat 窗口实时新增：
    - `05:30` 窗口 started 行为 `run_id=9574-9585`，同窗终态另起为 `run_id=9586-9597`
    - `06:00` 窗口 started 行为 `run_id=9598-9609`，同窗终态另起为 `run_id=9610-9621`
    - 其中 `run_id=9591`（`ASTS 重大异动心跳监控`）与 `run_id=9611`（`全天原油价格3小时播报`）都已分别落成 `completed + sent + delivered=1`，但同窗 started 行 `9581` 与 `9606` 仍永久保留 `running + pending`
    - `run_id=9590/9593/9595/9597` 与 `9614/9616/9619/9620` 这类终态都已经收口为 `noop + skipped_noop`，但对应 started 行 `9578/9585/9576/9582` 与 `9608/9599/9602/9598` 也同样没有被覆盖
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态仍是占比最高的状态：
    - `running + pending = 24`
    - `noop + skipped_noop = 22`
    - `completed + sent = 2`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1427` 条，较 `05:01` 巡检时的 `1403` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 05:01 CST` 再次复核，started 残留继续在最新 `04:30`、`05:00` 两个 heartbeat 窗口实时新增：
    - `04:30` 窗口 started 行为 `run_id=9522-9532`，同窗终态另起为 `run_id=9533-9546`
    - `05:00` 窗口 started 行为 `run_id=9548-9560`，而同窗终态已开始另起为 `run_id=9561-9571`
    - 其中 `run_id=9541`（`小米30港元破位预警`）与 `run_id=9546`（`持仓重大事件心跳检测`）都已分别落成 `completed + sent + delivered=1`，但同窗 started 行 `9525` 与 `9526` 仍永久保留 `running + pending`
    - `05:00` 窗口里 `run_id=9568-9571` 已先回写为 `noop + skipped_noop`，但对应 started 行 `9552/9553/9556/9559` 依旧没有被终态覆盖
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态仍是占比最高的状态：
    - `running + pending = 26`
    - `noop + skipped_noop = 23`
    - `completed + sent = 4`
    - `execution_failed + skipped_error = 1`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1403` 条，较 `04:02` 巡检时的 `1377` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 04:02 CST` 再次复核，started 残留继续在最新 `03:30`、`04:00` 两个 heartbeat 窗口实时新增：
    - `03:30` 窗口 started 行为 `run_id=9472-9483`，同窗终态另起为 `run_id=9484-9495`
    - `04:00` 窗口 started 行为 `run_id=9496-9508`，同窗终态另起为 `run_id=9509-9521`
    - 其中 `run_id=9494`（`持仓重大事件心跳检测`）与 `run_id=9518`（`Oil_Price_Monitor_Closing`）都已分别落成 `completed + sent + delivered=1`，但同窗 started 行 `9482` 与 `9499` 仍永久保留 `running + pending`
    - `04:02` 同窗还出现 `run_id=9519`（`持仓重大事件心跳检测`）`execution_failed + skipped_error` 与 `run_id=9521`（`Cerebras IPO与业务进展心跳监控`）`execution_failed + skipped_error`，但对应 started 行 `9506` / `9496` 依旧没有被终态覆盖
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态仍是占比最高的状态：
    - `running + pending = 25`
    - `noop + skipped_noop = 18`
    - `completed + sent = 5`
    - `execution_failed + skipped_error = 2`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1377` 条，较 `03:03` 巡检时的 `1352` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 03:03 CST` 再次复核，started 残留继续在最新 `02:30`、`03:00` 两个 heartbeat 窗口实时新增：
    - `02:30` 窗口 started 行为 `run_id=9424-9435`，同窗终态另起为 `run_id=9436-9447`
    - `03:00` 窗口 started 行为 `run_id=9448-9459`，同窗终态另起为 `run_id=9460-9471`
    - 其中 `run_id=9443`（`小米30港元破位预警`）已落成 `completed + sent + delivered=1`，但同窗 started 行 `run_id=9432` 仍永久保留 `running + pending`
    - `03:00` 窗口的 12 个 job 则全部在一分钟内另起终态行，其中 `9460-9471` 全部已收口为 `noop + skipped_noop`，但 started 行 `9448-9459` 仍全部保留
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态仍是占比最高的状态：
    - `running + pending = 24`
    - `noop + skipped_noop = 23`
    - `completed + sent = 1`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1352` 条，较 `02:03` 巡检时的 `1328` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 02:03 CST` 再次复核，started 残留继续在最新 `01:30`、`02:00` 两个 heartbeat 窗口实时新增：
    - `01:30` 窗口 started 行为 `run_id=9376-9387`，同窗终态另起为 `run_id=9388-9399`
    - `02:00` 窗口 started 行为 `run_id=9400-9411`，同窗终态另起为 `run_id=9412-9423`
    - 其中 `run_id=9398`（`ORCL 大事件监控`）与 `run_id=9423`（`ASTS 重大异动心跳监控`）都已分别落成 `completed + sent + delivered=1`，但同窗 started 行 `9378` 与 `9411` 仍永久保留 `running + pending`
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 聚合，最近一小时坏态仍是占比最高的状态：
    - `running + pending = 24`
    - `noop + skipped_noop = 22`
    - `completed + sent = 2`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1328` 条，较 `01:03` 巡检时的 `1304` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 01:03 CST` 再次复核，started 残留继续在最新 `00:30`、`01:00` 两个 heartbeat 窗口实时新增，而且普通 scheduler 的 started 行也还在同窗并存：
    - `00:30` 窗口 started 行为 `run_id=9328-9339`，同窗终态另起为 `run_id=9340-9351`
    - `01:00` 窗口 heartbeat started 行为 `run_id=9352-9363`，同窗终态另起为 `run_id=9364-9375`
    - 其中 `run_id=9347`（`小米30港元破位预警`）与 `run_id=9370`（`小米破位预警`）都已分别落成 `completed + sent + delivered=1`，但同窗 started 行 `9336` 与 `9362` 仍永久保留 `running + pending`
  - 按 `datetime(executed_at) >= datetime('now','-2 hours')` 聚合，最近两小时坏态仍是占比最高的状态：
    - `running + pending + heartbeat=1 = 48`
    - `noop + skipped_noop + heartbeat=1 = 46`
    - `running + pending + heartbeat=0 = 4`
    - `noop + skipped_noop + heartbeat=0 = 3`
    - `completed + sent + heartbeat=1 = 2`
    - `completed + sent + heartbeat=0 = 1`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1304` 条，较 `00:03` 巡检时的 `1280` 继续上升，说明 started 行仍在随着半小时轮询稳定堆积

- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 00:03 CST` 再次复核，started 残留继续在最新 `23:30`、`00:00` 两个 heartbeat 窗口实时新增，而且普通 scheduler 仍共用同一种“started 行不 finalize”的坏态：
    - `23:30` 窗口 started 行为 `run_id=9272-9284`，同窗终态另起为 `run_id=9285-9297`
    - `00:00` 窗口 heartbeat started 行为 `run_id=9298-9310`，同窗终态另起为 `run_id=9313-9324`
    - 同窗普通 scheduler 也继续复现：`run_id=9300`（`RKLB 每日动态监控`）与 `9301`（`TEM 每日动态监控`）在 `00:00:00` 先写成 `running + pending`，随后另起 `run_id=9325/9326` 并在 `00:01:42`、`00:02:43` 落成 `noop + skipped_noop`
  - 按 `datetime(executed_at) >= datetime('now','-1 hour')` 的最近一小时全量 scheduler 聚合，当前坏态仍是窗口内占比最高的状态：
    - `running + pending = 28`
    - `noop + skipped_noop = 26`
    - `completed + sent = 1`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1280` 条，较 `22:02` 巡检时的 `1227` 继续上升，说明 started 行没有随着整点轮询推进被自动收口

- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 22:02 CST` 再次复核，started 残留继续在最新 `21:30`、`22:00` 两个窗口实时新增，而且普通 scheduler 与 heartbeat 仍共用同一种“started 行不 finalize”的坏态：
    - `21:30` 窗口 started 行为 `run_id=9166-9180`，同窗终态另起为 `run_id=9181-9195`
    - `22:00` 窗口 heartbeat started 行为 `run_id=9198-9209`，同窗终态又另起为 `run_id=9210-9221`
    - 同窗普通 scheduler 也继续复现：`run_id=9196`（`科技核心股池 · 晚间击球区快报`）在 `21:35:00` 先写成 `running + pending`，随后另起 `run_id=9197` 并在 `21:36:18` 落成 `completed + sent + delivered=1`
  - 按 `executed_at >= datetime('now','-1 hour')` 的最近一小时全量 scheduler 聚合，当前坏态仍是窗口内占比最高的状态：
    - `running + pending = 28`
    - `noop + skipped_noop = 23`
    - `completed + sent = 5`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1227` 条，较 `21:01` 巡检时的 `1199` 继续上升，说明 started 行没有随着整点轮询推进被自动收口

- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 21:01 CST` 再次复核，started 残留继续在最新 `20:00`、`20:30`、`21:00` 三个窗口滚动累积，而且已不只停留在 heartbeat：
    - `20:00` 窗口 started 行为 `run_id=9077-9090`，同窗终态另起为 `run_id=9091-9103`
    - `20:30` 窗口 started 行为 `run_id=9110-9119`，同窗终态另起为 `run_id=9120-9133`
    - `21:00` 窗口 started 行为 `run_id=9134-9149`，其中 heartbeat/普通 scheduler 的终态又另起为 `run_id=9150-9164`
    - 仅最近一小时窗口内，新增 `running + pending` 残留已扩大到 `523` 条；其中 `21:00` 同窗 `晚9点盘前推演(XME及加密ETF)`、`持仓与关注股交易日晚间合并研判`、`美股盘前分析与个股推荐` 虽已分别落成 `run_id=9162/9163/9164` 的 `completed + sent + delivered=1`，原 started 行 `run_id=9140/9147/9141` 仍保留 `running + pending`
  - 按 `executed_at >= datetime('now','-1 hour')` 的最近一小时全量 scheduler 聚合，当前坏态已进一步扩大为：
    - `running + pending = 523`
    - `noop + skipped_noop = 326`
    - `execution_failed + skipped_error = 163`
    - `completed + sent = 34`
    - `completed + send_failed = 2`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1199` 条，较 `20:02` 巡检时的 `1169` 继续上升，说明新窗 started 行仍在持续堆积

- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 20:02 CST` 再次复核，最新 `19:00`、`19:30`、`20:00` 三个窗口继续把 started 行与终态行拆成两批记录：
    - `19:00` 窗口 started 行为 `run_id=9029-9040`，同窗终态行另起为 `run_id=9041-9052`
    - `19:30` 窗口 started 行为 `run_id=9053-9064`，同窗终态行另起为 `run_id=9065-9076`
    - `20:00` 窗口 heartbeat started 行为 `run_id=9077-9090` 中的 12 条 heartbeat，终态行另起为 `run_id=9091-9103`
    - 这意味着仅最近一小时 3 个 heartbeat 窗口内，就又累计留下 `36` 条未被终态覆盖的 started 行；若把 `20:00` 同窗两条 Feishu 非 heartbeat scheduler（`run_id=9082/9083`）也计入 started 残留，最近一小时新增 `running/pending` 已达到 `38` 条
  - 按 `executed_at >= 2026-04-28T19:00:00+08:00` 的最近一小时全量 scheduler 聚合，坏态进一步扩大：
    - `running + pending = 38`
    - `noop + skipped_noop = 36`
    - `completed + sent = 2`
    - `completed + send_failed = 1`
  - 全库聚合时，当前 `execution_status=running` 且 `message_send_status=pending` 的残留总量已升到 `1169` 条，说明这已经不是单个小时窗的短暂噪声
- 最近一小时真实运行语义对照：
  - `run_id=9104`（`A股盘后高景气产业链推演`）在 `2026-04-28T20:02:12.491329+08:00` 已成功落成 `completed + sent + delivered=1`
  - `run_id=9105`（`美股盘前与持仓新闻综述`）在 `2026-04-28T20:02:29.449255+08:00` 也已成功落成 `completed + sent + delivered=1`
  - 但同窗对应的 started 行 `run_id=9083` / `9082` 仍永久保留 `running + pending`
  - 说明该缺陷已经不只污染 heartbeat 台账，也开始覆盖最近一小时的普通 Feishu scheduler 成功任务

- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 19:01 CST` 再次复核，最新 `19:00` heartbeat 窗口继续把 started 行与终态行拆成两批记录：
    - started 行为 `run_id=9029-9040`，`executed_at=2026-04-28T19:00:00.534475+08:00` 到 `19:00:00.543875+08:00`，12 个 job 全部仍是 `execution_status=running`、`message_send_status=pending`
    - 同窗终态行则另起为 `run_id=9041-9052`，在 `19:00:30.019454+08:00` 到 `19:01:48.874222+08:00` 之间全部落成 `noop + skipped_noop`
    - 这意味着仅 heartbeat 窗口内，`18:00`、`18:30`、`19:00` 三个批次已经累计留下 `36` 条未被终态覆盖的 started 行
  - 若按最近一小时全量 scheduler 聚合，坏态已经不只是 heartbeat 局部噪声：
    - `running + pending = 446`
    - `noop + skipped_noop = 278`
    - `execution_failed + skipped_error = 163`
    - `completed + sent = 5`
    - 说明 started 行残留已经成为最近一小时里占比最高的台账状态，而不是边缘偶发
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 18:03 CST` 再次复核，started 残留在最新整点窗口继续累积：
    - 最近一小时聚合结果已变成 `running + pending = 24`、`noop + skipped_noop = 24`、`completed + sent = 1`
    - 说明 `17:30` 与 `18:00` 两个 heartbeat 窗口都各自额外留下 12 条 started 行，而终态行继续另起新记录
  - `18:00` 窗口的 started 行为 `run_id=8981-8992`，对应 job 包含 `ASTS 重大异动心跳监控`、`全天原油价格3小时播报`、`小米30港元破位预警`、`RKLB异动监控`、`持仓重大事件心跳检测`、`CAI破位预警`、`小米破位预警`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TEM破位预警`、`Cerebras IPO与业务进展心跳监控`、`ORCL 大事件监控`
  - 同一批 job 的终态行继续单独存在：
    - `run_id=8993-9003` 在 `18:00:18-18:01:32` 已分别落成 `noop + skipped_noop`
    - `run_id=8985`（`持仓重大事件心跳检测`）的终态还继续延后到 `2026-04-28T18:02:14.065110+08:00` 才新增 `run_id=9004` 对应的 `noop + skipped_noop`
  - 这说明 started 行不是短暂延迟；即使终态已在同一窗口内补齐，原 started 行也不会被覆盖或关闭
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 17:02 CST` 复核最近两小时 Feishu heartbeat 类任务后，发现 12 个 job 都同时存在：
    - 预写的 `execution_status=running`、`message_send_status=pending`
    - 同一 `delivery_key` 对应的终态行（`noop/skipped_noop`、`completed/sent` 或 `execution_failed/skipped_error`）
  - 聚合结果显示，这 12 个 job 在 `15:30`、`16:00`、`16:30`、`17:00` 四个窗口里各自都保留了 4 条 `running/pending` 残留：
    - `持仓重大事件心跳检测`
    - `TEM破位预警`
    - `CAI破位预警`
    - `ORCL 大事件监控`
    - `ASTS 重大异动心跳监控`
    - `Monitor_Watchlist_11`
    - `RKLB异动监控`
    - `全天原油价格3小时播报`
    - `小米30港元破位预警`
    - `Cerebras IPO与业务进展心跳监控`
    - `TEM大事件心跳监控`
    - `小米破位预警`
  - 同一批 job 的终态行并未缺失：
    - 例如 `小米30港元破位预警` 在最近两小时同时存在
      - `run_id=8866` / `8885` / `8910` / `8936`: `running + pending`, `detail_json.phase="started"`
      - `run_id=8878`: `noop + skipped_noop`, `delivery_key=j_654aef9b:2026-04-28:15:30:heartbeat`
      - `run_id=8900`: `noop + skipped_noop`, `delivery_key=j_654aef9b:2026-04-28:16:00:heartbeat`
      - `run_id=8930`: `completed + sent + delivered=1`, `delivery_key=j_654aef9b:2026-04-28:16:30:heartbeat`
      - `run_id=8950`: `noop + skipped_noop`, `delivery_key=j_654aef9b:2026-04-28:17:00:heartbeat`
    - `持仓重大事件心跳检测` 同样在四个窗口都出现“started 行 + 终态行”并存：
      - `run_id=8863` / `8889` / `8919` / `8939`: `running + pending`
      - `run_id=8882`: `noop + skipped_noop`
      - `run_id=8907`: `execution_failed + skipped_error`
      - `run_id=8931`: `noop + skipped_noop`
      - `run_id=8954`: `noop + skipped_noop`
- 最近一小时真实运行语义对照：
  - `run_id=8930`（`小米30港元破位预警`）在 `2026-04-28T16:30:40.310894+08:00` 已成功落成 `completed + sent + delivered=1`
  - 但同一 delivery window 的 `run_id=8910` 仍永久保留 `running + pending`
  - 说明这不是“终态没写出来”，而是“started 行没有在收口时被更新或关闭”
- 已检索相关缺陷文档：
  - [`feishu_scheduler_run_stuck_without_cron_job_run.md`](./feishu_scheduler_run_stuck_without_cron_job_run.md)
  - [`scheduler_heartbeat_unknown_status_silent_skip.md`](./scheduler_heartbeat_unknown_status_silent_skip.md)
  - 当前坏态不同于旧的“整轮卡住且无台账”或“heartbeat 输出契约漂移”：
    - 现在台账有 started 行
    - 终态也能写出
    - 但 started 行没有被同一 run 的终态覆盖

## 端到端链路

1. Feishu scheduler 到点触发 heartbeat / 报警任务。
2. 触发入口先写一条 `cron_job_runs` started 行：`execution_status=running`、`message_send_status=pending`、`detail_json.phase=started`。
3. 任务继续执行，并在几十秒后写出真正终态：`noop`、`completed` 或 `execution_failed`。
4. 实际数据库里，终态不是覆盖 started 行，而是新增了第二条 run 记录。
5. 结果是同一个 delivery window 同时被记录成“仍在运行”和“已经收口”。

## 期望效果

- 同一个 `delivery_key` 在调度台账里应只有一个最终可判定状态。
- started 行应在任务收口时被更新为终态，或至少被明确标记为 superseded/closed。
- 巡检查询 `execution_status=running` 时，不应把已经 `noop`、`completed`、`execution_failed` 的历史窗口误判成仍在执行。

## 当前实现效果

- 到 `2026-04-29 08:02 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 已升到 `28` 条，继续高于同窗真正已收口的 `completed + sent = 3`。
- `07:30` 与 `08:00` 新窗再次证明坏态持续实时产生：`run_id=9679/9678` 这类 started 行写出后不到一分钟，同一 delivery window 的真实终态就已经另起为 `9687/9692` 的 `completed + sent`；`run_id=9695` 与 `9694/9696-9709` 这批 started 行即便对应终态在 `9711` 和 `9710/9712-9721` 已收口为 `completed + sent` 或 `noop + skipped_noop`，也仍不会被覆盖。
- 全库 `running + pending` 残留总量已升到 `1479` 条，较 `07:02` 的 `1451` 继续上涨，说明整点与半点窗口每推进一轮都会继续留下新 started 脏行。
- 到 `2026-04-29 06:02 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 仍为 `24` 条，继续高于同窗真正已收口的 `completed + sent = 2`。
- `05:30` 与 `06:00` 新窗再次证明坏态持续实时产生：`run_id=9581/9606` 这类 started 行写出后不到两分钟，同一 delivery window 的终态就已经另起为 `9591 completed + sent` 与 `9611 completed + sent`，而 `9578/9585/9576/9582` 与 `9608/9599/9602/9598` 这种 started 行即便对应终态在 `9590/9593/9595/9597` 与 `9614/9616/9619/9620` 已落成 `noop + skipped_noop` 也仍不会被覆盖。
- 全库 `running + pending` 残留总量已升到 `1427` 条，较 `05:01` 的 `1403` 继续上涨，说明半小时巡检窗口每推进一轮都会继续留下新 started 脏行。
- 到 `2026-04-29 05:01 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 仍为 `26` 条，继续高于同窗真正已收口的 `completed + sent = 4`。
- `04:30` 与 `05:00` 新窗再次证明坏态持续实时产生：`run_id=9525/9526` 这类 started 行写出后不到两分钟，同一 delivery window 的终态就已经另起为 `9541 completed + sent` 与 `9546 completed + sent`，而 `9552/9553/9556/9559` 这种 started 行即便对应终态在 `9568-9571` 已落成 `noop + skipped_noop` 也仍不会被覆盖。
- 全库 `running + pending` 残留总量已升到 `1403` 条，较 `04:02` 的 `1377` 继续上涨，说明半小时巡检窗口每推进一轮都会继续留下新 started 脏行。
- 到 `2026-04-29 04:02 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 仍为 `25` 条，继续高于同窗真正已收口的 `completed + sent = 5`。
- `03:30` 与 `04:00` 新窗再次证明坏态持续实时产生：`run_id=9482/9499` 这类 started 行写出后不到两分钟，同一 delivery window 的终态就已经另起为 `9494 completed + sent` 与 `9518 completed + sent`，而 `9496/9506` 这种 started 行即便对应终态在 `9521/9519` 已落成 `execution_failed + skipped_error` 也仍不会被覆盖。
- 全库 `running + pending` 残留总量已升到 `1377` 条，较 `03:03` 的 `1352` 继续上涨，说明半小时巡检窗口每推进一轮都会继续留下新 started 脏行。
- 到 `2026-04-29 03:03 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 仍为 `24` 条，继续高于同窗真正已收口的 `completed + sent = 1`。
- `02:30` 与 `03:00` 新窗再次证明坏态持续实时产生：`run_id=9432` 这类 started 行写出后不到半分钟，同一 delivery window 的终态就已另起为 `run_id=9443 completed + sent`，而 `03:00` 整窗 `9448-9459` 也都在一分钟内另起为 `9460-9471 noop + skipped_noop`；started 行仍不会被覆盖。
- 全库 `running + pending` 残留总量已升到 `1352` 条，较 `02:03` 的 `1328` 继续上涨，说明半小时巡检窗口每推进一轮都会继续留下新 started 脏行。
- 到 `2026-04-29 01:03 CST` 为止，最近两小时聚合里的 `running + pending` 已达到 `52` 条，其中 heartbeat 残留 `48` 条、普通 scheduler 残留 `4` 条，仍高于同窗真正已收口的 `completed + sent = 3`。
- `00:30` 与 `01:00` 新窗再次证明坏态持续实时产生：`run_id=9336/9362` 这类 started 行写出后不到一分钟，同一 delivery window 的终态就已经另起为 `9347/9370` 的 `completed + sent`，started 行仍不会被覆盖。
- 全库 `running + pending` 残留总量已升到 `1304` 条，较 `00:03` 的 `1280` 继续上涨，说明半小时巡检窗口每推进一轮都会继续留下新 started 脏行。
- 到 `2026-04-29 00:03 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 仍为 `28` 条，继续高于同窗真正已收口的 `completed + sent = 1`。
- `00:00` 同窗的普通 scheduler 样本再次证明坏态不只存在于 heartbeat：`run_id=9300/9301` 先落成 `running + pending`，同一 delivery window 的真实终态另起为 `run_id=9325/9326 noop + skipped_noop`。
- 全库 `running + pending` 残留总量已升到 `1280` 条，较 `22:02` 的 `1227` 继续上涨，说明新窗 started 行仍在持续堆积。
- 到 `2026-04-28 22:02 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 仍为 `28` 条，继续高于同窗真正已收口的 `completed + sent = 5`。
- `21:35` 的普通 scheduler 样本再次证明坏态不只存在于 heartbeat：`run_id=9196` 先落成 `running + pending`，同一 delivery window 的真实终态另起为 `run_id=9197 completed + sent + delivered=1`。
- 全库 `running + pending` 残留总量已升到 `1227` 条，较 `21:01` 的 `1199` 继续上涨，说明新窗 started 行仍在持续堆积。
- 到 `2026-04-28 21:01 CST` 为止，最近一小时全量 scheduler 聚合里的 `running + pending` 已抬升到 `523` 条，全库残留总量升到 `1199` 条。
- `21:00` 新窗再次证明坏态仍在实时产生：`run_id=9140/9141/9147` 等 started 行写出后不到三分钟，终态就已另起为 `9162/9164/9163` 的 `completed + sent`，started 行仍不会被覆盖。
- 最近一小时 `cron_job_runs` 汇总到 `2026-04-28 19:01 CST` 时，仅 heartbeat 三个窗口就已累计 `36` 条 `running + pending` 残留；按最近一小时全量 scheduler 聚合则进一步放大到 `446` 条 `running + pending`。
- `19:00` 新窗再次证明坏态不是历史遗留脏数据：`run_id=9029-9040` 的 started 行刚写出不到两分钟，同窗终态就已经另起为 `9041-9052`，started 行仍不会被覆盖。
- 最近一小时 `cron_job_runs` 汇总到 `2026-04-28 18:03 CST` 已进一步扩大成 `24` 条 `running + pending` 残留，对应两个窗口各 `12` 条 started 行全部没有收口。
- 最近一小时 `cron_job_runs` 汇总里，`running + pending` 反而是最多的组合，达到 24 条；同窗真正终态只有 23 条。
- 对同一个 job / 同一个 `delivery_key`，数据库会同时出现 `running/pending` 与终态行。
- 这会把活跃运行中的数量系统性抬高，并让人工巡检难以分辨“真卡住”与“已经完成但 started 行未清理”。

## 用户影响

- 当前证据里，受影响窗口的大部分 heartbeat 仍能正常收口为 `noop`，个别窗口还能成功 `completed + sent`，因此暂未看到用户因这条缺陷直接收不到消息。
- 影响主要集中在调度台账正确性、巡检判断和后续故障排查：真实已经结束的窗口会长期伪装成“仍在运行”。
- 因此这次定级为 `P3`：它没有直接打断主功能链路，但会显著污染缺陷台账与运维判断，且会掩盖真正的悬挂 run。

## 根因判断

- 高概率是 2026-04-26 为止血“无台账”问题加入的 started 行写入逻辑，只负责 insert，不负责在终态阶段按 `delivery_key` 回写同一条记录。
- 终态链路当前更像是“另起一条 run”，而不是“完成 started 行”。
- 这与 [`feishu_scheduler_run_stuck_without_cron_job_run.md`](./feishu_scheduler_run_stuck_without_cron_job_run.md) 是相关但独立的后续问题：旧问题是没有 started 行；当前问题是 started 行写出来后没有被 finalize。

## 下一步建议

- 按 `delivery_key` 或等价唯一键，把 started 行与终态行收敛成同一条 `cron_job_runs` 记录。
- 若历史兼容必须保留多行，也应在 started 行补一个明确的 closed/superseded 标记，并让默认巡检查询过滤掉已终结的 started 行。
- 增加一个只读巡检规则：同一 `delivery_key` 若同时存在 `phase=started` 和终态行，则记录为台账异常，而不是普通运行中。
