# Bug: 新建 Heartbeat 监控任务首次运行即触发 `context window exceeds limit`，整轮直接失败

- **发现时间**: 2026-04-16 14:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=1887`，`job_id=j_78d08da1`，`job_name=TEM_动态监控`，`executed_at=2026-04-16T14:00:19.471571+08:00`，`execution_status=execution_failed`，`message_send_status=skipped_error`，`delivered=0`，`error_message=LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)`
    - `run_id=1890`，`job_id=j_977ac60c`，`job_name=AAOI_动态监控`，`executed_at=2026-04-16T14:00:32.420976+08:00`，`execution_status=execution_failed`，`message_send_status=skipped_error`，`delivered=0`，`error_message=LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 14:00:19.467` `job_id=j_78d08da1` `job=TEM_动态监控` `success=false` `error="LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"`
    - `2026-04-16 14:00:32.413` `job_id=j_977ac60c` `job=AAOI_动态监控` `success=false` `error="LLM 错误: bad_request_error: invalid params, context window exceeds limit (2013)"`
    - 同一批 `14:00:00.989-14:00:00.990` 启动的其他 heartbeat 任务中，`RKLB_动态监控`、`小米30港元破位预警`、`全天原油价格3小时播报` 仍可正常返回 `JsonNoop`，说明并非整批 scheduler 全面宕掉，而是特定任务首轮 prompt/上下文预算失控
  - 关联会话：
    - `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 在 `2026-04-16T13:57:52.064021+08:00` 刚创建并激活三条心跳监控任务；其中 `TEM_动态监控` 与 `AAOI_动态监控` 在下一轮 14:00 首次执行即失败

## 端到端链路

1. 用户在 Feishu 直聊中要求“有新动态时及时告诉我”，系统于 `2026-04-16 13:57:52 CST` 创建心跳监控任务。
2. scheduler 在 `14:00` 首次触发这些新建 heartbeat 任务。
3. `TEM_动态监控` 与 `AAOI_动态监控` 进入 `function_calling` heartbeat runner 后，模型直接返回 `context window exceeds limit (2013)`。
4. 当前 heartbeat 执行链路没有像普通会话那样做自动 compact/retry，也没有降级成可见的用户态提示。
5. 结果是任务刚创建成功，首次实际巡检就 `execution_failed + skipped_error`，用户不会收到任何监控结果。

## 期望效果

- 新建 heartbeat 任务首次运行应稳定完成，至少能返回 `triggered`、`noop` 或产品化失败状态。
- 当 heartbeat prompt 超出上下文预算时，执行链路应自动收缩上下文、压缩任务负载或重试一次，而不是直接失败。
- 若最终仍失败，系统至少应留下可定位的原始上下文线索，并让用户或运维知道“任务本轮执行失败”，而不是让监控 silently 失效。

## 当前实现效果

- `TEM_动态监控` 与 `AAOI_动态监控` 都在创建后的第一轮 heartbeat 执行中直接命中 `context window exceeds limit (2013)`。
- `cron_job_runs` 只记录了统一的 `heartbeat_model`，没有保留足够的原始 prompt/上下文摘要，当前无法仅靠台账直接判断是任务模板过长、上下文继承过多，还是新建任务初始化状态异常。
- 同一轮其他 heartbeat 任务可以继续返回 `JsonNoop` 或正常触发，说明当前故障不是 scheduler 全面不可用，而是与特定监控任务载荷相关。
- 由于执行结果落成 `execution_failed + skipped_error`，这次没有伪装成成功；但对用户而言，监控任务仍然在最关键的“首轮验证”阶段直接失效。

## 用户影响

- 这是功能性缺陷，不是单纯质量波动。用户刚创建好的监控任务在首次实际执行时就失败，意味着“已创建成功”的承诺无法兑现。
- 问题会直接影响心跳监控是否可用，导致用户无法收到本应持续送达的自动巡检结果，因此定级为 `P2`，而不是仅影响表达质量的 `P3`。
- 如果这一现象集中出现在新建任务上，用户会误以为监控已经开始工作，实际却在第一轮就断掉。

## 根因判断

- 高概率是 heartbeat/function-calling 链路缺少上下文预算控制与 `context window exceeds limit` 的自动恢复能力。
- 从证据看，普通会话链路已有上下文溢出恢复单独建档并标记 `Fixed`，但 heartbeat 任务在 `14:00` 仍然直接失败，说明这条执行路径没有复用同样的恢复策略，或其首轮构造的 prompt 规模已经超过当前模型容忍上限。
- 目前证据还不足以断言问题一定来自“新建任务继承了整段直聊历史”还是“任务模板自身过长”，需要后续结合实际 heartbeat 输入拼装逻辑确认。

## 下一步建议

- 优先排查 heartbeat/function-calling 路径是否具备与普通会话一致的 overflow 检测、compact 和 retry 逻辑。
- 为 `cron_job_runs.detail_json` 增补受控长度的请求摘要或 prompt 预算指标，否则后续很难快速判断是模板过长还是上下文继承异常。
- 在修复前，可对新建 heartbeat 任务增加“首轮自检失败”告警，避免任务刚创建就静默失效。
