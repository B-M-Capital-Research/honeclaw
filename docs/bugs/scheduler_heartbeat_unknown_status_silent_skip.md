# Bug: Heartbeat 定时任务遇到 `JsonUnknownStatus` 时静默跳过，监控提醒可能长期失效

- **发现时间**: 2026-04-15 14:05 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
  - 最近一小时同一任务持续异常：
    - `run_id=1889`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T14:00:27.633510+08:00`，`execution_status=execution_failed`，`message_send_status=skipped_error`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1865`，`job_id=j_c1c1be63`，`job_name=存储板块加仓信号监控`，`executed_at=2026-04-16T11:00:31.512294+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `2026-04-16T11:30:28.836+08:00`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，运行日志再次记录 `parse_kind=JsonUnknownStatus`，且同轮仍被记为“心跳任务未命中，本轮不发送”
    - `run_id=1860`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T10:30:32.268455+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1855`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T10:00:32.184986+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - `run_id=1849`，`job_id=j_ab7e8fb1`，`job_name=Monitor_Watchlist_11`，`executed_at=2026-04-16T09:30:22.379738+08:00`，`execution_status=noop`，`message_send_status=skipped_noop`，`delivered=0`，`detail_json.parse_kind=JsonUnknownStatus`
    - 上一轮最近一小时巡检中的同一任务也持续异常：`run_id=1830 (01:31, JsonUnknownStatus)`、`1826 (01:01, JsonNoop)`、`1822 (00:30, JsonNoop)`
    - 往前回溯同一任务仍可见连续多次 `JsonUnknownStatus`：`run_id=1813 (23:00)`、`1806 (22:00)`、`1791 (21:00)`、`1787 (20:31)`、`1781 (20:01)`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 14:00:27.632` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 14:00:27.632` 同轮 `raw_preview` 仍直接输出 11 只股票的“当前价格 vs 触发价”分析，但末尾没有合法状态 JSON
    - `2026-04-16 14:00:27.632` 同轮已不再打印“心跳任务未命中，本轮不发送”，而是升级为 `parse failure escalated`
    - `2026-04-16 10:30:32.267` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 11:00:31.512` `job_id=j_c1c1be63` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 11:30:28.836` `job_id=j_ab7e8fb1` 再次出现 `parse_kind=JsonUnknownStatus`，随后仍打印 `心跳任务未命中，本轮不发送`
    - `2026-04-16 10:00:32.184` `job_id=j_ab7e8fb1` `parse_kind=JsonUnknownStatus`
    - `2026-04-16 10:00:32.184` 同轮 `raw_preview` 直接列出 11 只股票的“当前价格 vs 触发价”，但仍未返回合法状态 JSON
    - `2026-04-16 09:30:22.379` 同一任务上一轮仍为 `parse_kind=JsonUnknownStatus`
    - `2026-04-16 01:31:19.950` 与 `01:01:16.495` 说明上一轮巡检窗口里也一直在 `JsonUnknownStatus / JsonNoop` 之间漂移
  - 最近半小时新增样本：
      - `2026-04-16 16:31:18.288` `job_id=j_ab7e8fb1` 再次记录 `parse_kind=JsonUnknownStatus`，并升级为 `execution_failed + skipped_error`
      - `2026-04-16 17:01:30.375` 同一任务又恢复为 `JsonNoop + skipped_noop`
      - `2026-04-16 17:01:47.317` `job_id=j_977ac60c`（`AAOI_动态监控`）也新增 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 17:31:16.099` `job_id=j_654aef9b`（`小米30港元破位预警`）新增 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 17:31:28.801` `job_id=j_ab7e8fb1` 再次记录 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 17:31:42.919` `job_id=j_c1c1be63`（`存储板块加仓信号监控`）同样新增 `parse_kind=JsonUnknownStatus + execution_failed`
      - `2026-04-16 18:01:42.443` `job_id=j_ab7e8fb1`（`Monitor_Watchlist_11`）恢复为 `JsonNoop + skipped_noop`
      - `2026-04-16 18:01:48.288` `job_id=j_977ac60c`（`AAOI_动态监控`）仍落成 `JsonUnknownStatus + execution_failed`
      - `2026-04-16 19:01:18.801` `job_id=j_ab7e8fb1`（`Monitor_Watchlist_11`）再次回落为 `JsonUnknownStatus + parse failure escalated`
      - 同一轮日志仍同时打印 `心跳任务未命中，本轮不发送`，说明失败升级与用户侧/渠道侧文案仍未完全对齐
      - `2026-04-16 19:31:08.188` `job_id=j_654aef9b`（`小米30港元破位预警`）最新再次落成 `JsonUnknownStatus + execution_failed`
      - 同一时间 `raw_preview` 已明确给出“当前小米价格 32.06 港元，高于 30 港元，应该返回 noop”，但最终仍只输出 `<think> ... {}`，没有稳定收口到合法状态
      - `2026-04-16 19:31:08.188` 同轮仍紧接着打印 `心跳任务未命中，本轮不发送`，说明渠道日志口径仍把失败描述成 noop
      - `2026-04-16 20:01:10.971` 同一任务又恢复为 `noop + skipped_noop`，说明该任务也进入了“上一轮未知状态、下一轮又恢复”的抖动形态
      - `2026-04-16 20:31:20.588` `job_id=j_ab7e8fb1`（`Monitor_Watchlist_11`）再次落成 `JsonUnknownStatus + execution_failed`
      - `2026-04-16 20:32:14.171` `job_id=j_c1c1be63`（`存储板块加仓信号监控`）同轮也新增 `JsonUnknownStatus + execution_failed`
  - 最近一小时新增样本：
      - `run_id=2009`，`job_id=j_ab7e8fb1`（`Monitor_Watchlist_11`），`executed_at=2026-04-16T22:01:21.215496+08:00`，再次落成 `execution_failed + skipped_error`，`error_message=heartbeat 输出包含未知状态，任务已标记失败`
      - `run_id=2013`，同一任务在 `2026-04-16T22:31:18.346190+08:00` 再次复现 `JsonUnknownStatus + execution_failed`
      - `2026-04-16 22:01:21.213` 与 `22:31:18.344` 的 `web.log` 都继续记录 `parse_kind=JsonUnknownStatus`
      - `run_id=2017`，同一任务在 `2026-04-16T23:01:19.570052+08:00` 又短暂回到 `noop + skipped_noop`
      - 这说明“未知状态已升级为失败”仍在生效，但输出协议本身依旧在相邻轮次间抖动，没有稳定收口
  - 2026-04-16 23:31 至 2026-04-17 00:01 最近一小时新增样本：
      - `run_id=2022`，`job_id=j_c1c1be63`（`存储板块加仓信号监控`），`executed_at=2026-04-16T23:31:58.742906+08:00`，落成 `execution_failed + skipped_error`，`error_message=heartbeat 输出包含未知状态，任务已标记失败`
      - 同轮 `2026-04-16 23:31:58.741` `web.log` 记录 `parse_kind=JsonUnknownStatus`，`raw_preview` 已明确给出 Rubin / HBM4 信号分析，但末尾仍未收口到合法状态 JSON
      - `run_id=2024`，`job_id=j_654aef9b`（`小米30港元破位预警`），`executed_at=2026-04-17T00:01:12.430772+08:00`，再次落成 `execution_failed + skipped_error`
      - `2026-04-17 00:01:12.429` 的 `raw_preview` 明确写出“当前价格高于30港元，所以应返回 `{\\\"status\\\":\\\"noop\\\"}` 或 `{}`”，但最终仍以前置 `<think>` 文本破坏协议，继续进入 `JsonUnknownStatus`
      - `run_id=2025`，`job_id=j_ab7e8fb1`（`Monitor_Watchlist_11`），`executed_at=2026-04-17T00:01:19.320420+08:00`，紧接着再次落成 `execution_failed + skipped_error`
      - `2026-04-17 00:01:19.319` 的 `raw_preview` 已完成 11 只股票价格与触发线的逐项判断，但仍未输出可解析状态，说明该任务并没有随 `23:01` 的短暂恢复而真正收口
      - 这三个样本串起来说明：`JsonUnknownStatus` 仍在不同 heartbeat 模板间轮流冒出，当前不是单任务 prompt 偶发，而是公共输出契约在最近一小时继续抖动
  - 2026-04-17 00:31 最近一小时新增样本：
      - `run_id=2032`，`job_id=j_654aef9b`（`小米30港元破位预警`），`executed_at=2026-04-17T00:31:06.520202+08:00`，已恢复为 `noop + skipped_noop`
      - `2026-04-17 00:31:06.519` `web.log` 记录 `parse_kind=JsonNoop`，`raw_preview` 末尾已能稳定收口到 `{\"status\":\"noop\"}`
      - `run_id=2033`，`job_id=j_c1c1be63`（`存储板块加仓信号监控`），`executed_at=2026-04-17T00:31:11.351156+08:00`，同样恢复为 `noop + skipped_noop`
      - `2026-04-17 00:31:11.350` 的 `raw_preview` 虽仍包含 `<think>`，但解析器已成功识别为 `JsonNoop`
      - `run_id=2034`，`job_id=j_ab7e8fb1`（`Monitor_Watchlist_11`），`executed_at=2026-04-17T00:31:18.580235+08:00`，仍落成 `execution_failed + skipped_error`
      - `2026-04-17 00:31:18.578` 的 `raw_preview` 已改成英文逐项对比 11 只股票价格，但依旧没有稳定收口到合法状态 JSON，说明当前活跃问题已收敛到 `Monitor_Watchlist_11` 这条 watchlist 模板，而不是整批 heartbeat 全面失效
      - 同一时间日志仍打印 `心跳任务未命中，本轮不发送`，说明即使数据库已按失败落账，渠道/日志口径仍把 `execution_failed` 描述成 noop
  - 对比同一小时其他 heartbeat 任务：
    - `j_38745baf`（`全天原油价格3小时播报`）在 `run_id=1847`（`09:30:04`）也短暂出现 `JsonUnknownStatus`，`run_id=1853`（`10:00:10`）又恢复为 `JsonNoop`
    - `j_654aef9b`（`小米30港元破位预警`）在 `10:00:10` 仍为 `JsonNoop -> noop / skipped_noop`
    - `j_ab7e8fb1` 在 `run_id=1864`（`11:00:24`）短暂恢复为 `JsonNoop`，但同一窗口另一条 heartbeat `j_c1c1be63` 又落回 `JsonUnknownStatus`，说明缺陷并未整体收口，只是不同任务间漂移
  - 24 小时聚合：
    - `j_ab7e8fb1` 共运行 59 次，其中 29 次为 `JsonUnknownStatus`
  - 生命周期聚合：
    - `j_ab7e8fb1` 自 `2026-04-04T21:30:31.191391+08:00` 起累计运行 454 次，仅 3 次 `completed` / `delivered`

## 端到端链路

1. 用户创建 heartbeat / watchlist 监控任务，预期在满足条件时收到提醒，在不满足条件时收到可判定的 `noop`。
2. 调度器按计划执行 heartbeat 任务，模型需要返回符合约定的结构化状态。
3. 当前 `Monitor_Watchlist_11` 在最近一小时连续返回无法被解析器识别的结果，数据库记录为 `parse_kind=JsonUnknownStatus`。
4. 当前线上已经有一部分实例把这类解析失败升级为 `execution_failed + skipped_error`，但模型输出本身仍然频繁落入 `JsonUnknownStatus`。
5. 结果是用户侧仍然收不到提醒；区别只是从“静默伪装成 noop”变成了“后台记失败但仍未恢复功能”。

## 期望效果

- heartbeat 任务应稳定返回可解析的结构化状态，至少能明确区分 `triggered`、`noop`、`error`。
- 当模型输出不符合约定时，调度器不应静默吞掉，而应记录可追踪错误并进入可观测状态。
- 对“监控提醒”类任务，解析失败至少应进入 `execution_failed` 或运维可见告警，而不是伪装成正常 `noop`。

## 当前实现效果

- `Monitor_Watchlist_11` 在 `2026-04-16 14:00:27` 再次落回 `JsonUnknownStatus`；与 `10:00`、`10:30`、`11:30` 一样，模型已经完成逐项价格判断，但末尾仍未收口成合法状态 JSON，说明问题仍在当前活跃时段持续出现。
- `web.log` 在 `2026-04-16 10:00:32.184` 明确记录 `parse_kind=JsonUnknownStatus`；同一轮 `raw_preview` 已经枚举 11 只股票的实时价格和触发价，但因为没有落成合法状态 JSON，最终仍被当成 `noop / skipped_noop` 静默吞掉。
- 到 `14:00` 这一轮，线上行为已从 `noop / skipped_noop` 变成 `execution_failed / skipped_error`，说明“不要静默吞掉未知状态”的修复开始在当前实例生效；但结构化收口本身仍未修好，所以该缺陷不能关闭。
- 到 `16:31` 这一轮，`Monitor_Watchlist_11` 又一次落成 `JsonUnknownStatus + execution_failed`；到 `17:01` 它短暂恢复为 `JsonNoop + skipped_noop`，说明问题不是线性修复，而是在相邻轮次之间抖动。
- `17:01:47` 的样本显示 `AAOI_动态监控` 也开始产出 `JsonUnknownStatus + execution_failed`；到 `17:31`，同类问题又继续扩散到 `小米30港元破位预警` 与 `存储板块加仓信号监控`。
- 到 `18:01` 这一轮，同批 heartbeat 一度出现明显分化：`Monitor_Watchlist_11` 与 `小米30港元破位预警` 都恢复为 `JsonNoop + skipped_noop`，但 `AAOI_动态监控` 仍继续落成 `JsonUnknownStatus + execution_failed`。
- 但 `19:01:18.801` 的最新样本显示 `Monitor_Watchlist_11` 又再次落回 `JsonUnknownStatus + parse failure escalated`，说明此前“已恢复”为短暂波动，而不是稳定收口。
- 同一轮 `19:01` 里，`小米30港元破位预警`、`存储板块加仓信号监控`、`RKLB_动态监控` 已恢复为 `JsonNoop`，`TEM_动态监控` 与 `AAOI_动态监控` 甚至成功投递 `JsonTriggered`；这进一步说明缺陷不是整批任务同时失效，而是在不同任务上反复漂移。
- 最新日志还显示 `AAOI_动态监控` 在 `parse failure escalated` 之后仍打印 `心跳任务未命中，本轮不发送`，说明“未知状态已升级为失败”和“渠道日志仍按 noop 口径描述”之间还存在观测口径不一致。
- 到 `19:31` 这一轮，`小米30港元破位预警` 也新增 `JsonUnknownStatus + execution_failed`，且 `raw_preview` 已经明确写出“当前价格高于触发线，应返回 noop”，最后却只落成 `<think> ... {}`；说明问题并非监控任务自身判断错误，而是最终状态封装仍不稳定。
- `20:01` 同一任务又立刻恢复为 `noop + skipped_noop`，进一步证明 `JsonUnknownStatus` 已在多个 heartbeat 任务上呈现“相邻轮次抖动”，而不是某一条任务永久损坏。
- 到 `20:31` 这一轮，`Monitor_Watchlist_11` 再次落回 `JsonUnknownStatus + execution_failed`，`20:32` 的 `存储板块加仓信号监控` 同轮也命中同样症状，说明这条缺陷在最新窗口仍然活跃，且受影响任务并未收敛。
- 到 `22:01` 与 `22:31` 的最新两个窗口，`Monitor_Watchlist_11` 继续落成 `JsonUnknownStatus + execution_failed`；到 `23:01` 又恢复为 `noop + skipped_noop`，说明缺陷已从“是否静默吞掉”阶段转成“失败与恢复在相邻轮次间来回摆动”。
- 到 `23:31` 与 `00:01` 的最新窗口，抖动并没有收敛，反而继续跨任务漂移：`存储板块加仓信号监控`、`小米30港元破位预警`、`Monitor_Watchlist_11` 在 30 分钟内依次落成 `execution_failed + skipped_error`。
- `00:01` 的 `小米30港元破位预警` 样本尤其明确：模型在自由文本里已经知道“应返回 noop”，却仍输出 `<think>` 和解释性文字，导致解析器拿不到合法状态；说明当前问题不在业务判断本身，而在最终协议收口仍不稳定。
- `00:01` 的 `Monitor_Watchlist_11` 也证明 `23:01` 的 `noop` 只是短暂波动，而非稳定修复，因为同一任务在一个小时内又再次回到 `JsonUnknownStatus + execution_failed`。
- `00:31` 的窗口则说明故障形态又发生了收缩：`小米30港元破位预警` 与 `存储板块加仓信号监控` 已恢复成 `JsonNoop + skipped_noop`，但 `Monitor_Watchlist_11` 仍继续产出 `JsonUnknownStatus + execution_failed`。
- 这意味着当前活跃风险已经从“多模板同时漂移”收敛为“个别 watchlist 模板持续不稳定”，但由于仍会让用户失去监控结果，缺陷状态依然不能关闭。
- 同轮日志仍把 `execution_failed` 说成“未命中，本轮不发送”，说明除了协议收口抖动之外，运维/渠道可观测口径不一致的问题也还在。
- 这说明当前并不是单个 watchlist prompt 失配，而是 heartbeat 输出协议在多条不同模板的监控任务上都可能失去稳定收口。
- 对照 `11:30` 的最新日志、`run_id=1860`（`10:30`）、`1855`（`10:00`）、`1849`（`09:30`）和更早的 `01:31` 记录可以看到，这类 heartbeat 已经连续多轮维持同一症状，并没有随着时间窗切换自然恢复。
- 这也说明问题不只是“偶发返回乱码”，而是模型已经完成了业务判断，却在最后结构化封装一步失配，监控链路因此丢失了本该可追踪的判定结果。
- 数据库没有保存可供人工直接复核的最终文本预览，导致一旦进入 `JsonUnknownStatus`，排障信息同时丢失。
- 由于最近样本已经同时出现 `parse_kind=JsonUnknownStatus + execution_failed` 与下一轮自动恢复为 `JsonNoop`，当前缺陷的状态应理解为“部分止血但强烈抖动”：错误不再总是伪装成 noop，但 heartbeat 仍会在不同任务、不同轮次上失去稳定的结构化收口。
- 最近一小时的 `22:01 -> 22:31 -> 23:01` 连续三轮再次证明：同一个 `Monitor_Watchlist_11` 不需要任何配置变更，就会在失败与 noop 之间自行摆动，当前仍不具备可依赖的稳定性。

## 用户影响

- 这是功能性缺陷，不是单纯质量波动。用户依赖 heartbeat 监控来发现触发条件，一旦解析失败被静默当成 `noop`，就可能漏掉本应触发的提醒。
- 即便 14:00 这一轮已升级为 `execution_failed + skipped_error`，用户侧仍旧拿不到监控结果，所以功能损失没有消失，只是可观测性有所改善。
- 问题影响的是“自动监控是否按约工作”，会直接破坏任务可信度，因此定级为 `P2`，而不是只影响表达质量的 `P3`。
- 由于系统对用户和运维都没有显式失败信号，这类问题更容易长期潜伏。

## 根因判断

- heartbeat 输出协议对模型返回格式过于脆弱，出现非标准 JSON 或状态枚举漂移时，会落入 `JsonUnknownStatus`。
- 最近一小时同一任务在相邻轮次间会在 `JsonNoop` 与 `JsonUnknownStatus` 之间来回抖动，说明除了单个任务 prompt 外，解析器对“先给分析过程、最后未严格收口到状态 JSON”的输出也缺少足够稳健的兼容或强制约束。
- 最近一小时新增的 `AAOI_动态监控`、`小米30港元破位预警` 与 `存储板块加仓信号监控` 样本说明这不是 `Monitor_Watchlist_11` 单任务 prompt 特例，而是 heartbeat 输出协议对多条不同模板的监控任务都不够稳健。
- `23:31` 到 `00:01` 的最新样本进一步证明，问题并不是“模型不会判断触发条件”，而是模型即便在自由文本里已经写出“应该返回 noop / {}”，也仍无法稳定给出解析器认可的最终状态对象。
- `18:01` 与 `19:01` 两个相邻批次一起看，同组任务已经出现“上一轮恢复、下一轮再掉回未知状态”的抖动形态，进一步说明问题不只是某个固定任务模板写坏，而是 heartbeat 输出协议缺少稳定的最终收口约束。
- `00:31` 的恢复样本说明解析器与公共协议并非完全不可用；更像是 `Monitor_Watchlist_11` 这类多标的 watchlist 模板在自由文本收束成最终 JSON 时仍更容易失配，因此当前根因判断需要从“所有 heartbeat 都不稳”收缩为“公共协议脆弱 + 某些复杂模板更易触发”。
- 调度器曾把“无法识别状态”错误地归并进 `noop` 路径，造成功能性失败被静默吞掉；而 14:00 的运行结果表明，这一收口正在部分修正，但尚未彻底消除所有旧路径或旧实例。
- 渠道侧日志仍沿用“心跳任务未命中，本轮不发送”的 noop 文案，说明即使数据库台账已按 `execution_failed` 落账，部分运行日志和可观测口径还没有完全跟上新的失败语义。
- 现有落库字段只保留 `parse_kind` 与字符数，没有把原始响应片段保留下来，进一步放大了排障盲区。

## 修复情况（2026-04-16，待重新验证）

- `crates/hone-channels/src/scheduler.rs` 已把 heartbeat 的解析失败从静默 `noop` 分支中拆出：
  - `JsonUnknownStatus` 现在会返回 `error`，由各渠道 scheduler 落库为 `execution_failed + skipped_error`
  - `JsonMalformed` 也同步升级为失败，不再继续伪装成正常 `noop`
- 同一修复里补上了受控长度的 `raw_preview` 留存：
  - heartbeat detail 现在会把原始响应摘要写进 `detail_json.raw_preview`
  - 后续可以直接区分是“未知 status 枚举”“非法 JSON”还是“正常 noop”
- 这样一来，监控类 heartbeat 任务在模型已经跑完但结构化收口失败时，不会再被后台静默吞掉。
- 到 `2026-04-16 14:00` 的 `run_id=1889`，未知状态已经落成 `execution_failed + skipped_error`，说明这部分止血开始在线上生效。
- 但更早轮次的 `run_id=1860` 与 `1865` 仍落成 `noop + skipped_noop`，而且 `14:00` 这一轮依然持续产出 `JsonUnknownStatus`；因此本单只能更新为“状态有变化但问题仍活跃”，不能关闭或降级。

## 回归验证

- `cargo test -p hone-channels heartbeat_unknown_json_status_marks_execution_failed -- --nocapture`
- `cargo test -p hone-channels heartbeat_malformed_json_marks_execution_failed -- --nocapture`
- `cargo test -p hone-channels heartbeat_ -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/scheduler.rs`
- `git diff --check`

## 后续建议

- 先核对当前运行实例是否都已部署包含上述修复的 scheduler 版本；14:00 这轮已经看到 `execution_failed + skipped_error`，但更早轮次仍是 `noop + skipped_noop`，说明版本或路径可能处于混合态。
- 如果后面还观察到 heartbeat 在 `JsonNoop` 和 `JsonUnknownStatus` 之间抖动，可以继续收紧 prompt / parser 契约，让模型在末尾 JSON 收口更稳定。
- 如需更强的运维可观测性，可以再把 `parse_kind` 聚合到状态页或告警面板，而不只停留在 `cron_job_runs.detail_json`。
