# Bug: Heartbeat 监控任务触发 `context window exceeds limit` 后缺少恢复，故障会在不同任务间漂移复现

- **发现时间**: 2026-04-16 14:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `2026-05-23 11:01 CST` 本轮继续确认同一缺陷活跃：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - 07:30-11:01 CST 新增 `8` 条 heartbeat `ContextOverflowNoop + noop + skipped_noop + delivered=0`，均在 Web heartbeat。
      - `持仓财报与重大新闻心跳提醒` 在 07:30、10:30、11:00 CST 继续重复命中 `ContextOverflowNoop`。
      - `AI与科技持仓观察关键事件心跳提醒` 在 07:30、10:30 CST 继续命中 `ContextOverflowNoop`；11:00 CST 虽恢复为 `JsonNoop`，但同一类新建多标的 heartbeat 已连续多窗被超窗吞掉。
    - 结论：这仍是同一根因 / 同一影响范围的运行态复现，不新建重复文档；严重等级与状态维持 `P2 / New`，不是 P1，本轮不创建 GitHub Issue。
  - `2026-05-23 07:08 CST` 本轮从 `Fixed` 重新打开：旧修复结论是 heartbeat context overflow 改为 `ContextOverflowNoop + skipped_noop` 后“本轮跳过、下轮正常重试”，但最近四小时真实窗口显示同一类超窗已经重复静默吞掉多条 heartbeat：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - 03:00-07:00 CST 共 `9` 条 heartbeat 落成 `noop + skipped_noop + delivered=0`，`detail_json.parse_kind=ContextOverflowNoop`。
      - `持仓财报与重大新闻心跳提醒` 在 03:00、03:30、04:30、06:00、06:30、07:00 CST 共 `6` 次重复命中 `ContextOverflowNoop`，没有体现“下一轮正常重试”。
      - 06:42 CST 用户新建 Web heartbeat `AI与科技持仓观察关键事件心跳提醒` 后，07:00 CST 首轮执行即命中 `ContextOverflowNoop`，终态仍被记为正常 `noop + skipped_noop`。
      - 同窗还影响 `存储板块关键事件心跳提醒` 与 `heartbeat_绿田机械基本面跟踪`，说明问题不是单一旧任务配置损坏。
    - `data/runtime/logs/web.log.2026-05-22`
      - 07:00 CST 多条 `[HeartbeatDiag] run_finish ... context window exceeds limit (2013)` 随后被记录为 `transient_noop parse_kind=ContextOverflowNoop`，没有写入用户可见失败，也没有保留为 `execution_failed`。
    - 会话对照：
      - 最近四小时 Web / Feishu 直聊和 06:00 普通 scheduler 均有 assistant final 收口；assistant final 污染扫描未命中空回复、内部路径、工具轨迹、`<think>` 或 provider 原始错误。故障集中在 heartbeat 超窗恢复/台账语义。
    - 结论：这是功能性 bug。它不会直接暴露原始 provider 错误，但会把超窗执行失败伪装成合法未触发，使用户刚创建或依赖中的 heartbeat 任务在关键窗口静默失效；严重等级维持 `P2 / New`。不是 P1，本轮不创建 GitHub Issue。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=1887`，`job_id=j_78d08da1`，`job_name=TEM_动态监控`，`executed_at=2026-04-16T14:00:19.471571+08:00`，`execution_status=execution_failed`，`message_send_status=skipped_error`，`delivered=0`，`error_message=LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)`
    - `run_id=1890`，`job_id=j_977ac60c`，`job_name=AAOI_动态监控`，`executed_at=2026-04-16T14:00:32.420976+08:00`，`execution_status=execution_failed`，`message_send_status=skipped_error`，`delivered=0`，`error_message=LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)`
    - 最近一小时新增漂移样本：
      - `run_id=1896`，`job_id=j_78d08da1`，`job_name=TEM_动态监控`，`executed_at=2026-04-16T14:32:28.254643+08:00`，仍为 `execution_failed + skipped_error`
      - `run_id=1897`，`job_id=j_977ac60c`，`job_name=AAOI_动态监控`，`executed_at=2026-04-16T14:32:34.632337+08:00`，已恢复为 `completed + sent`
      - `run_id=1900`，`job_id=j_cee5b540`，`job_name=RKLB_动态监控`，`executed_at=2026-04-16T15:00:19.157340+08:00`，首次出现同样的 `execution_failed + skipped_error` 与 `context window exceeds limit (2013)`
      - `run_id=1902`，`job_id=j_977ac60c`，`job_name=AAOI_动态监控`，`executed_at=2026-04-16T15:00:32.472180+08:00`，进一步恢复为 `noop + skipped_noop`
      - `run_id=1903`，`job_id=j_78d08da1`，`job_name=TEM_动态监控`，`executed_at=2026-04-16T15:00:36.640304+08:00`，也恢复为 `completed + sent`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 14:00:19.467` `job_id=j_78d08da1` `job=TEM_动态监控` `success=false` `error="LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"`
    - `2026-04-16 14:00:32.413` `job_id=j_977ac60c` `job=AAOI_动态监控` `success=false` `error="LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"`
    - 同一批 `14:00:00.989-14:00:00.990` 启动的其他 heartbeat 任务中，`RKLB_动态监控`、`小米30港元破位预警`、`全天原油价格3小时播报` 仍可正常返回 `JsonNoop`，说明并非整批 scheduler 全面宕掉，而是特定任务首轮 prompt/上下文预算失控
    - `2026-04-16 14:32:28.254` `job_id=j_78d08da1` 继续记录同样的 `context window exceeds limit (2013)`
    - `2026-04-16 15:00:19.156` 新增 `job_id=j_cee5b540 job=RKLB_动态监控` 的同类失败；而 `15:00:35.342` `TEM_动态监控` 已恢复为 `JsonTriggered`，`15:00:32.471` `AAOI_动态监控` 已恢复为 `JsonNoop`
    - 说明故障不是“某两个新建任务永久超窗”，而是在相近 heartbeat 配置之间漂移复现
    - 最近半小时新增样本：
      - `2026-04-16 16:31:21.524` `job_id=j_977ac60c job=AAOI_动态监控` 再次记录 `context window exceeds limit (2013)`，而同批次 `RKLB_动态监控` 已恢复为 `JsonNoop`
      - `2026-04-16 17:01:36.857` `job_id=j_cee5b540 job=RKLB_动态监控` 进一步恢复为 `completed + sent`
      - `2026-04-16 17:31:18.245` `job_id=j_cee5b540 job=RKLB_动态监控` 再次落回 `context window exceeds limit (2013)`
      - `2026-04-16 17:31:42.323` `job_id=j_977ac60c job=AAOI_动态监控` 也继续记录同样的 `context window exceeds limit (2013)`
      - `2026-04-16 18:01:38.761` `job_id=j_78d08da1 job=TEM_动态监控` 恢复为 `completed + sent`
      - `2026-04-16 18:01:43.997` `job_id=j_cee5b540 job=RKLB_动态监控` 恢复为 `completed + sent`
      - `2026-04-16 20:01:21.806` `job_id=j_cee5b540 job=RKLB_动态监控` 再次记录 `context window exceeds limit (2013)`
      - `2026-04-16 20:01:52.924` `job_id=j_78d08da1 job=TEM_动态监控` 同轮也再次落成 `context window exceeds limit (2013)`
      - `2026-04-16 20:31:23.362` `job_id=j_cee5b540 job=RKLB_动态监控` 进一步复现同样错误
      - `2026-04-16 20:31:37.468` `job_id=j_78d08da1 job=TEM_动态监控` 已恢复为 `completed + sent`
  - 关联会话：
    - `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 在 `2026-04-16T13:57:52.064021+08:00` 刚创建并激活三条心跳监控任务；其中 `TEM_动态监控` 与 `AAOI_动态监控` 在下一轮 14:00 首次执行即失败

## 端到端链路

1. 用户在 Feishu 直聊中要求“有新动态时及时告诉我”，系统于 `2026-04-16 13:57:52 CST` 创建心跳监控任务。
2. scheduler 在 `14:00` 首次触发这些新建 heartbeat 任务。
3. 受影响的 heartbeat 任务进入 `function_calling` runner 后，会直接返回 `context window exceeds limit (2013)`。
4. 当前 heartbeat 执行链路没有像普通会话那样做自动 compact/retry，也没有降级成可见的用户态提示。
5. 结果是某一轮 heartbeat 会直接落成 `execution_failed + skipped_error`，用户不会收到任何监控结果；即便下一轮短暂恢复，同根因也可能转移到另一条监控任务上。

## 期望效果

- 新建 heartbeat 任务首次运行应稳定完成，至少能返回 `triggered`、`noop` 或产品化失败状态。
- 当 heartbeat prompt 超出上下文预算时，执行链路应自动收缩上下文、压缩任务负载或重试一次，而不是直接失败。
- 若最终仍失败，系统至少应留下可定位的原始上下文线索，并让用户或运维知道“任务本轮执行失败”，而不是让监控 silently 失效。

## 当前实现效果

- `2026-05-23 07:08 CST` 最新真实窗口表明，现有 `ContextOverflowNoop` 止血不足：它确实避免了原始 `context window exceeds limit` 外泄，但把 runner 超窗错误写成 `noop + skipped_noop`，导致台账和用户侧都无法区分“条件未触发”和“本轮根本没跑起来”。
- `2026-05-23 11:01 CST` 最新窗口继续新增 8 条同类 `ContextOverflowNoop`，其中 `持仓财报与重大新闻心跳提醒` 和 `AI与科技持仓观察关键事件心跳提醒` 在前一轮巡检后仍重复命中，说明该问题不是 07:00 单窗偶发。
- 同一 `持仓财报与重大新闻心跳提醒` 在最近四小时连续 6 次命中 `ContextOverflowNoop`，已经不符合旧修复结论里的“跳过本轮、下轮正常重试”。
- 06:42 CST 新建的 `AI与科技持仓观察关键事件心跳提醒` 在 07:00 CST 首轮就超窗并被静默记为 noop，说明该问题仍会影响新建 heartbeat 的首轮可用性。
- `TEM_动态监控` 与 `AAOI_动态监控` 都在创建后的第一轮 heartbeat 执行中直接命中 `context window exceeds limit (2013)`，说明首轮确实存在预算失控。
- `cron_job_runs` 只记录了统一的 `heartbeat_model`，没有保留足够的原始 prompt/上下文摘要，当前无法仅靠台账直接判断是任务模板过长、上下文继承过多，还是新建任务初始化状态异常。
- 同一轮其他 heartbeat 任务可以继续返回 `JsonNoop` 或正常触发，说明当前故障不是 scheduler 全面不可用，而是与特定监控任务载荷相关。
- 由于执行结果落成 `execution_failed + skipped_error`，这次没有伪装成成功；但对用户而言，监控任务仍然在最关键的“首轮验证”阶段直接失效。
- 到 `15:00` 这一轮，`TEM_动态监控` 已恢复为 `completed + sent`，`AAOI_动态监控` 恢复为 `noop + skipped_noop`，但 `RKLB_动态监控` 又新出现同样的 `context window exceeds limit (2013)`。
- 到 `16:31` 这一轮，`AAOI_动态监控` 又再次落回 `context window exceeds limit (2013)`，而同批的 `RKLB_动态监控` 一度恢复为 `noop`；到 `17:01`，`RKLB_动态监控` 进一步恢复为 `completed + sent`。
- 但 `17:31` 的最新样本显示 `RKLB_动态监控` 与 `AAOI_动态监控` 已再次同时落回 `context window exceeds limit (2013)`，说明这条缺陷不仅没有收口，当前还会在同一批 heartbeat 任务上并发复现。
- 到 `18:01` 这一轮，`TEM_动态监控` 与 `RKLB_动态监控` 又同时恢复为 `completed + sent`，说明超窗故障仍然呈现“相邻轮次恢复、随后再回落”的抖动特征，而不是线性修复。
- 到 `20:01` 这一轮，`RKLB_动态监控` 与 `TEM_动态监控` 又同时重新命中 `context window exceeds limit (2013)`；而仅过 30 分钟，`20:31` 的 `TEM_动态监控` 已恢复为 `completed + sent`，但 `RKLB_动态监控` 仍继续失败。
- 同批次里 `AAOI_动态监控` 虽然没有再次出现 `context window exceeds limit (2013)`，但仍落成 `JsonUnknownStatus + execution_failed`，说明 heartbeat 整体稳定性问题仍未收口，只是从“超窗”漂移成了“结构化收口失败”。
- 这表明当前问题不是“单个任务配置写坏后永久失败”，而是 heartbeat 任务集合中存在不稳定的上下文预算失控，故障会在相似任务之间持续漂移，且会阶段性放大为多任务同时失败；`20:01 -> 20:31` 的最新窗口再次证明这条缺陷仍处于活跃态。

## 用户影响

- 这是功能性缺陷，不是单纯质量波动。用户刚创建好的监控任务在首次实际执行时就失败，意味着“已创建成功”的承诺无法兑现。
- 问题会直接影响心跳监控是否可用，导致用户无法收到本应持续送达的自动巡检结果，因此定级为 `P2`，而不是仅影响表达质量的 `P3`。
- 即便某些任务下一轮恢复，用户仍会面对“同一批监控里有的正常、有的突然超窗失败”的不稳定体验，难以信任 heartbeat 结果。

## 根因判断

- 最新样本显示，当前仓库仍会在 heartbeat runner 返回 context overflow 时执行 `ContextOverflowNoop` 分支：错误被吸收为 `ScheduledTaskExecution { should_deliver=false, error=None }`，并以 `parse_kind=ContextOverflowNoop` 写入台账。
- 这条分支只能降低原始错误外泄风险，不能恢复执行，也没有把“超窗失败”暴露给任务健康或用户可见状态；当同一任务反复超窗时，缺陷就从“错误文案不友好”变成“功能链路静默漏跑”。
- 高概率是 heartbeat/function-calling 链路缺少上下文预算控制与 `context window exceeds limit` 的自动恢复能力。
- 从证据看，普通会话链路已有上下文溢出恢复单独建档并标记 `Fixed`，但 heartbeat 任务在 `14:00` 仍然直接失败，说明这条执行路径没有复用同样的恢复策略，或其首轮构造的 prompt 规模已经超过当前模型容忍上限。
- 结合 `15:00` 到 `17:31` 的连续样本看，问题也不完全等同于“新建任务首轮继承了过长历史”；因为同根因已经从 `TEM/AAOI` 漂移到 `RKLB`，随后又回漂到 `AAOI`，并在最新一轮同时打到 `RKLB + AAOI`，更像是 heartbeat prompt 预算在不同任务之间缺少稳定上限控制。
- `18:01` 批次里 `TEM`/`RKLB` 能恢复，而 `AAOI` 继续退化成另一种失败形态，进一步说明当前更像是“同一批 heartbeat 任务共享的不稳定预算/协议环境”，而不是某个固定 job 配置永久损坏。
- 目前证据仍不足以断言具体是“上下文继承抖动”“工具结果拼接过长”还是“任务模板自身过长”，需要后续结合实际 heartbeat 输入拼装逻辑确认。

## 下一步建议

- 不要继续把重复 `context window exceeds limit` 视为合法 noop。建议把 `ContextOverflowNoop` 至少计入可审计的降级/失败状态，或在连续 N 次同任务命中时升级为 `execution_failed` 并提示用户任务过大。
- 对多标的 heartbeat 增加 prompt 预算上限、分批检查或压缩输入摘要，避免新建任务首轮就超过模型上下文。
- 优先排查 heartbeat/function-calling 路径是否具备与普通会话一致的 overflow 检测、compact 和 retry 逻辑。
- 为 `cron_job_runs.detail_json` 增补受控长度的请求摘要或 prompt 预算指标，否则后续很难快速判断是模板过长还是上下文继承异常。
- 在修复前，可对 heartbeat 的 `context window exceeds limit` 做聚合告警与任务级重试观察，避免问题在不同监控任务间漂移时被误判成单点偶发。
