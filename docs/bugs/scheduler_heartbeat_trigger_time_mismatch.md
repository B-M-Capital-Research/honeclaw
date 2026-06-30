# Bug: Heartbeat 触发提醒把实际执行时间写成错误的北京时间

- **发现时间**: 2026-05-29 15:03 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 最新进展（2026-06-30 23:01 CST）

- 本轮 2026-06-30 19:02-23:01 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 21:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，当前日志窗口为 2026-06-30。
    - 同条预览正文写出 `行情（数据时间：2026年7月3日）`，与实际执行日 2026-06-30 不一致。
    - 22:30 CST `全天原油价格3小时播报` raw preview 写出 `2026年7月3日 09:16 北京时间，非检查时间节点，静默`；23:00 CST `闪迪关键事件心跳提醒` raw preview 又写出 `2026-04-04 08:01 北京时间`。这两条未确认正式送达，仅作为时间上下文漂移辅助信号。
  - 查重结论：
    - 小米样本仍属于 heartbeat 成功生成触发提醒后的用户可见数据日期 / 执行日期口径错误；与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-06-30 03:07 CST）

- `crates/hone-channels/src/scheduler.rs` 的 heartbeat 出站归一化新增 `今日（M月D日）` 口径修正：当触发提醒正文把“今日”括号日期写成与 scheduler 权威北京时间不一致的未来/错误日期时，会在送达前自动改写为当前北京时间日期。
- 该修复与既有 `北京时间 YYYY-MM-DD HH:MM` / `北京时间 HH:MM` 归一化并行生效，避免同一提醒同时把绝对检查时间和“今日（…）”相对日期写错。
- 新增回归 `heartbeat_normalizes_conflicting_relative_today_date`，覆盖 `今日（6月30日）` 在 `2026-06-29T13:00:21+08:00` 执行窗口内被归一为 `今日（6月29日）`。
- 验证通过：
  - `cargo check -p hone-channels --tests`
  - `cargo test -p hone-channels heartbeat_normalizes_conflicting_relative_today_date --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- 本轮未重启当前 live runtime；线上送达预览是否完全止血仍待后续巡检窗口复核，因此先更新为代码级 `Fixed`，不直接标 `Closed`。

## 最新进展（2026-06-30 15:02 CST）

- 本轮 2026-06-30 15:02-19:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 15:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，当前日志窗口为 2026-06-30。
    - 同条预览正文写出 `数据时间 2026年7月3日`，与实际执行日 2026-06-30 不一致。
    - 16:30 CST 同 job 再次生成 `deliver_preview`，正文继续写 `行情（数据时间：2026年7月3日，数据戳 1782806885）`。
    - 19:00 CST Feishu `ORCL 大事件监控` raw preview 写出 `Current time: 2026-04-04 22:17:16 CST (Saturday night)`；该样本最终为 `JsonNoop` 且未送达，本轮仅作为时间上下文漂移辅助信号。
  - 查重结论：
    - 小米样本仍属于 heartbeat 成功生成触发提醒后的用户可见数据日期 / 执行日期口径错误；与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

- 本轮 2026-06-30 11:02-15:02 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-30`
    - 13:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，当前日志窗口为 2026-06-30。
    - 同条预览正文写出 `今日（7月4日，北京时间 14:40）低开低走`，与实际执行日 2026-06-30 不一致。
    - 13:30 / 14:00 CST 同 job 后续预览改写为 `数据截至 2026 年 6 月 26 日 18:35 北京时间`，说明正文仍混用执行日期、数据日期与模型推断日期；15:00 CST 再次生成 `今日（6月30日）`，相对日期口径在同一 job 内漂移。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-30 07:03 CST）

- 本轮 2026-06-30 03:00-07:03 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 06:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成 `deliver_preview`，正文仍写出 `今日（6月30日）高开高走`。
    - 07:00 CST 同 job 的 `duplicate_suppressed` 继续匹配带 `今日（6月30日）` 的旧 preview。
  - 03:12 CST 非文档提交 `a00e5131 fix: harden heartbeat noop compatibility` 已包含 `今日（M月D日）` 归一化回归，但本窗 live 日志仍出现同类错误日期 preview；当前按运行态 `New` 处理，不能关闭。
  - 调度 / 解析 / 预览生成链路可用，但用户可见提醒新鲜度和交易日判断仍可能被误导；没有错投、数据安全或全渠道不可用证据，因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-30 03:07 CST）

- 本轮 2026-06-29 23:00-2026-06-30 03:07 CST 真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 23:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口仍为 2026-06-29。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`，与实际执行日 2026-06-29 不一致；随后 `duplicate_suppressed` 也匹配到同一错误日期预览，说明错误时间口径会进入重复抑制判断基线。
    - 00:00 CST 同 job 再次生成 `deliver_preview`，正文写成 `今日（7月1日）高开高走`，继续与实际执行日不一致。
    - 03:00 CST Web `持仓关键事件心跳检测` raw preview 还把检查窗口写成 `北京时间 2026-05-30`，但该样本最终为 `PlainTextNoop` 且未送达，本轮仅作为时间上下文漂移的辅助信号，不单独升严重级别。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 23:01 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 21:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`，与实际执行日 2026-06-29 不一致；随后 `duplicate_suppressed` 也匹配到同一错误日期预览，说明错误时间口径会进入重复抑制判断基线。
    - 23:00 CST Web `中际旭创关键事件心跳提醒` raw preview 内部还把系统时间写成 `2026-04-04 15:00 CST`，但该样本最终为 `JsonNoop` 且未送达，本轮仅作为时间上下文漂移的辅助信号，不单独升严重级别。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 19:01 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 15:00 / 17:30 / 18:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29。
    - 15:00 CST `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`；17:30 CST 又写成 `今日（7月4日）高开高走`；18:00 CST 再写成 `今日（6月30日）高开高走`，均与实际执行日 2026-06-29 不一致。
    - 该样本已进入送达预览文本；同窗还可见同 job 的触发 / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 15:07 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-29` 与 `data/runtime/logs/hone_cli_screen.log`
    - 15:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29 15:00 CST。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（6月30日）高开高走`，与实际执行日 2026-06-29 不一致。
    - 该样本已进入送达预览文本；同窗还可见同 job 的触发 / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-29 03:01 CST）

- 本轮最近四小时真实运行态继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-28` 与 `data/runtime/logs/hone_cli_screen.log`
    - 03:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成送达预览，当前日志窗口为 2026-06-29 03:00 CST。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（7月3日）成交量约 1.92 亿股`，与实际执行日 2026-06-29 不一致。
    - 随后的 `duplicate_suppressed` 又匹配到旧预览中的 `今日（7月2日）`，说明错误日期口径会进入重复抑制判断基线。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功生成触发提醒后的用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
    - 本轮同时存在 `小米30港元破位预警` triggered / noop / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`；本单只跟踪已生成提醒的日期口径错误。
  - 用户影响：
    - 调度、解析和预览生成链路可用，用户可见正文仍可能误导提醒新鲜度和交易日判断。
    - 没有错投、数据安全或全渠道不可用证据；因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-28 19:02 CST）

- 本轮最近四小时真实运行态复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-28` 与 `data/runtime/logs/hone_cli_screen.log`
    - 17:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 以 `JsonTriggered` 生成成功送达预览，当前日志窗口为 2026-06-28 17:00 CST。
    - 同条 `deliver_preview` 正文把当前窗口写成 `今日（7月1日）低点 21.30 港元已刷新 52 周新低`，与实际执行日 2026-06-28 不一致。
    - 同一轮 raw preview 还包含 `timestamp":"2026-07-02T12:48:09+08:00"`；这说明时间 / 日期口径不只出现在标题型“北京时间”归一化范围内，数据日期与正文“今日”也可能以未来日期进入用户可见提醒。
  - 查重结论：
    - 该样本仍属于 heartbeat 成功投递后用户可见时间 / 日期口径错误；与本文档既有“触发提醒时间口径漂移”同一受影响链路，不新建重复缺陷。
    - 本轮同时存在 `小米30港元破位预警` triggered / noop / 未命中漂移，继续归入 `scheduler_heartbeat_near_threshold_false_trigger.md`；本单只跟踪已送达提醒的日期口径错误。
  - 用户影响：
    - 调度、解析、投递链路成功，用户能收到提醒。
    - 但用户可见正文把 6 月 28 日执行窗口写成 7 月 1 日，容易误判提醒新鲜度和交易日；没有错投、漏投、数据安全或全渠道不可用证据。
    - 因此维持质量性 `P3`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-06-21 23:07 CST）

- 本轮补齐 2026-06-21 19:03 回退样本的日期型标题归一化：
  - `normalize_heartbeat_beijing_trigger_time(...)` 现在不仅处理 `北京时间 HH:MM ...触发`，也处理 `【...监控 · 北京时间 YYYY-MM-DD HH:MM】` 这类 heartbeat 标题时间。
  - 归一化仍限制在 `监控 / 检查 / 心跳 / 任务 / 触发` 上下文，避免把普通数据时间误写成执行时间。
  - 命中日期型触发时间后会把标题改写为 scheduler 权威北京时间日期和分钟，并在 metadata 保留 `beijing_trigger_time_normalized=true` 与原始 `YYYY-MM-DD HH:MM`。
  - 新增回归 `heartbeat_normalizes_conflicting_beijing_trigger_datetime_title`，覆盖 `NBIS 高权重事件监控 · 北京时间 2026-06-19 17:30` 在 2026-06-21 19:01 CST 执行窗口内被归一到 `北京时间 2026-06-21 19:01`。
- 验证：
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- 无关联 GitHub Issue；当前按本地代码和回归验证更新为 `Fixed`，未依赖当前机器生产日志、线上渠道状态或 live 服务重启复核。

## 最新进展（2026-06-21 19:03 CST）

- 本轮最近四小时真实运行态复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-21`
    - 19:00:02 CST Web heartbeat `NBIS关键事件心跳提醒` 启动，target=`web-user-c2776780c59d`。
    - 19:01:02 CST 同 job 收口为 `success=true`、`parse_kind=JsonTriggered`。
    - 19:01:02 CST deliver preview 开头写成 `【NBIS 高权重事件监控 · 北京时间 2026-06-19 17:30】`。
  - 该送达标题时间与实际调度执行窗口 `2026-06-21 19:01 CST` 明显不一致，且 raw preview 中模型自行推断 `It's approximately 17:30 Beijing time on June 19, 2026`。
  - 代码对照显示当前调度路径仍调用 `heartbeat_execution_from_content(&content, &heartbeat_model)`，没有把 scheduler 当前北京时间传入 `heartbeat_execution_from_content_at_beijing(...)`，因此 2026-05-29 的触发时间归一化修复没有覆盖这条 live 出站路径。
- 用户影响：
  - 调度、解析、投递链路成功，用户能收到提醒；但用户可见标题把提醒时间写早两天，容易误判提醒新鲜度和交易时段。
  - 该问题不涉及错投、漏投、数据安全或系统级失败，因此保持 `P3 / New`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-05-29 16:35 CST）

- 已修复 heartbeat 用户可见触发时间口径漂移：heartbeat prompt 现在显式注入“本轮权威检查时间（北京时间）”，并要求 `message` 中的检查/触发时间必须使用该权威时间；市场时段、数据时间或美东盘前/盘后不得写成另一个“北京时间触发”。
- 出站前新增轻量归一化：若 `JsonTriggered` 正文出现类似 `北京时间 HH:MM ...监控/检查/心跳/任务触发`，且该时间与 scheduler 当前北京时间不一致，会把该触发时间归一到 scheduler 权威检查时间，并在 metadata 中记录 `beijing_trigger_time_normalized=true` 与原始时间。
- 回归验证：`cargo test -p hone-channels heartbeat_normalizes_conflicting_beijing_trigger_time --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_ --lib -- --nocapture` 通过。
- 状态更新为 `Fixed`；后续如当前 HEAD 运行态仍出现 heartbeat 把美东/UTC/数据时间错误标成“北京时间触发”，再用新样本重新打开。

## 证据来源

- `data/runtime/logs/web.log.2026-06-21`
  - 巡检窗口：2026-06-21 15:03-19:03 CST。
  - 19:00:02 CST `NBIS关键事件心跳提醒` 触发，target=`web-user-c2776780c59d`。
  - 19:01:02 CST `run_finish job_id=j_eab1a3b2 job=NBIS关键事件心跳提醒 ... success=true content_chars=4179`，随后 `parse_kind=JsonTriggered`。
  - 同一秒 `deliver_preview` 开头为 `【NBIS 高权重事件监控 · 北京时间 2026-06-19 17:30】`，但本轮运行日志时间为 2026-06-21 19:01 CST。
- `crates/hone-channels/src/scheduler.rs`
  - 当前调度路径在 heartbeat 内容收口后调用 `heartbeat_execution_from_content(&content, &heartbeat_model)`。
  - 带权威北京时间的 `heartbeat_execution_from_content_at_beijing(...)` 只在测试 / helper 路径出现，未接入本条 live 调度路径。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=36255`
  - `job_id=j_bb4bbb99`
  - `job_name=AI与科技持仓观察关键事件心跳提醒`
  - `actor_channel=web`
  - `executed_at=2026-05-29T11:31:32.698046+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.heartbeat_model=MiniMax-M2.7-highspeed`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `response_preview` / `detail_json.scheduler.deliver_preview` 开头写成 `2026年5月29日 北京时间 04:00 盘后监控触发。已核验事实...`
- 最近四小时巡检窗口 `2026-05-29 11:02-15:03 CST`
  - 按消息时间共有 47 个 user turn 与 47 个 assistant final，最新活跃会话均已 assistant final 收口。
  - 普通 scheduler 2 条 `completed + sent + delivered=1`，未见 `commodity_causality_guarded=true`。
  - Heartbeat 新增 1 条 `completed + sent + delivered=1`、81 条 `execution_failed + skipped_error + delivered=0`、40 条 `noop + skipped_noop + delivered=0`。
  - Assistant final 污染扫描未命中空回复、本机绝对路径、`rawOutput`、`tool_call`、`session/update`、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400 Bad Request` 或 `open_id cross app`。

## 端到端链路

1. Web heartbeat scheduler 在 `2026-05-29 11:31 CST` 执行 `AI与科技持仓观察关键事件心跳提醒`。
2. Heartbeat runner 返回 `JsonTriggered`，scheduler 将结果落成 `completed + sent + delivered=1`。
3. 送达正文开头却把触发时间写为 `北京时间 04:00`。
4. 该时间与 `cron_job_runs.executed_at=2026-05-29T11:31:32+08:00` 不一致，用户可见提醒的时间口径错误。

## 期望效果

- Heartbeat 触发提醒应使用调度器权威执行时间或明确的数据时间字段，不能把 UTC 时间、市场时段说明或模型推断时间写成“北京时间”。
- 如果正文需要区分数据时间、交易时段与触发时间，应分别标注，例如“执行时间”“数据口径时间”“美东盘后”。

## 当前实现效果

- 本轮 heartbeat 内容已成功触发并送达，但用户可见首句把实际 `11:31 CST` 执行写成 `北京时间 04:00`。
- 当前证据只覆盖一条 Web heartbeat 成功送达样本；同窗直聊与普通 scheduler 没有同类时间口径污染。

## 用户影响

- 用户看到的 heartbeat 触发时间与系统实际执行时间不一致，可能误判提醒的新鲜度和所处交易时段。
- 该问题不影响主功能链路：任务有正常执行、解析、落库和送达；没有错误投递对象、没有漏发、没有把工具原始输出暴露给用户，也没有直接给出错误交易指令。
- 因此本轮定级为 P3：它是用户可见输出质量 / 时间口径问题，而不是调度、投递、数据安全或交易正确性链路失效。

## 根因判断

- 初步判断是 heartbeat 模型在生成 `JsonTriggered` 正文时把 UTC 时间、市场时段或内部数据时间错误表述为“北京时间”。
- Scheduler 送达前目前没有校验触发正文里的显式北京时间是否与 `executed_at` 一致，也没有强制区分执行时间和数据时间。

## 下一步建议

- 在 heartbeat prompt 或输出 schema 中显式传入并要求使用 `executed_at_beijing`，同时禁止模型自行换算“北京时间”。
- 在 scheduler 出站前增加轻量校验：若 `JsonTriggered` 正文出现“北京时间 HH:MM”且与 `executed_at` 偏差明显，降级为待复核或重写时间口径。
- 后续巡检优先观察其它 `JsonTriggered + delivered=1` heartbeat 是否继续出现类似 UTC/CST 混淆，再决定是否提升严重等级。

## 最新运行态复核（2026-06-28 23:02 CST）

- `data/runtime/logs/web.log.2026-06-28` / `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-06-28 19:02-23:02 CST。
  - 20:00 CST `小米30港元破位预警` 生成 `JsonTriggered + deliver_preview`，但 preview 把当前 2026-06-28 执行窗口写成 `今日（7月3日）`。
  - 21:00 CST 同一 job 再次生成 `JsonTriggered + deliver_preview`，preview 把当前执行窗口写成 `今日（7月2日）`，随后因重复抑制未正式发送。
  - 该样本与 17:00 CST 的 `今日（7月1日）` 同根，均为 heartbeat 触发提醒把数据日期 / 模型推断日期写成用户可见“今日”口径。
- 本轮判断
  - 最新证据仍属于 heartbeat 成功生成触发提醒后的日期 / 时间口径错误，不新建重复缺陷。
  - 调度和解析链路仍可运行，问题主要影响用户对提醒新鲜度和交易日的判断，因此维持质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
