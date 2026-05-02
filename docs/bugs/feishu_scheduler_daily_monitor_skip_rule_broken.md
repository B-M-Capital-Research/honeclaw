# Bug: Feishu 每日动态监控在“无新增催化应跳过”时仍照常推送长文

- **发现时间**: 2026-04-20 01:01 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **证据来源**:
  - `2026-05-03 00:02 CST` 最近一小时真实窗口确认本单重新活跃，且 2026-04-26 标记 `Fixed` 的 skip-signal 止血没有覆盖最新措辞：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=14020` / `job_id=j_5f0b686a` / `job_name=RKLB 每日动态监控` / `executed_at=2026-05-03T00:01:00+08:00`
      - `execution_status=completed`、`message_send_status=sent`、`should_deliver=1`、`delivered=1`
      - `response_preview` 明确写入：`今日不触发重大催化或风险证伪推送`
      - `run_id=14023` / `job_id=j_379acc40` / `job_name=TEM 每日动态监控` / `executed_at=2026-05-03T00:01:41+08:00`
      - 同样落成 `completed + sent + delivered=1`
      - `response_preview` 明确写入：`今日不触发新增重大催化或风险证伪推送`
      - `run_id=14024` / `job_id=j_101f5e64` / `job_name=AAOI 每日动态监控` / `executed_at=2026-05-03T00:02:17+08:00`
      - 同样落成 `completed + sent + delivered=1`
      - `response_preview` 明确写入：`今日不触发新增重大推送`
    - `data/sessions/Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b.json`
      - `2026-05-03T00:00:02.499262+08:00` 注入 `RKLB 每日动态监控`
      - `2026-05-03T00:00:55.475332+08:00` assistant final 正文已写明：`今日不触发重大催化或风险证伪推送`
      - `2026-05-03T00:00:55.478725+08:00` 紧接着注入 `TEM 每日动态监控`
      - `2026-05-03T00:01:40.017341+08:00` assistant final 正文已写明：`今日不触发新增重大催化或风险证伪推送`
      - `2026-05-03T00:01:40.034444+08:00` 再注入 `AAOI 每日动态监控`
      - `2026-05-03T00:02:15.149227+08:00` assistant final 正文已写明：`今日不触发新增重大推送`
    - `data/runtime/logs/sidecar.log`
      - `2026-05-03 00:00:55.477`、`00:01:40.028`、`00:02:15.150` 三轮都记录 `step=session.persist_assistant ... detail=done`
      - 同窗没有出现 `SchedulerDiag skip_signal`、`rolled back skipped assistant turn` 或 `本轮不发送`
    - 结论：到 `2026-05-03 00:02` 为止，系统再次把“正文已明确声明不触发推送”的每日动态监控任务记成 `completed + sent` 并写入 direct session，本单状态从 `Fixed` 回退为 `New`。
  - 2026-04-26 线上复核：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=6361`，`job_id=j_379acc40`，`job_name=TEM 每日动态监控`，`executed_at=2026-04-26T00:01:30.415080+08:00`
    - 本轮已正确落成 `execution_status=noop`、`message_send_status=skipped_noop`、`should_deliver=0`、`delivered=0`
    - `run_id=6362`，`job_id=j_5f0b686a`，`job_name=RKLB 每日动态监控`，`executed_at=2026-04-26T00:02:11.656192+08:00`
    - 本轮同样已正确落成 `noop + skipped_noop`
    - `data/runtime/logs/web.log.2026-04-25` 同秒记录 `SchedulerDiag skip_signal ...` 与 `[Feishu] 心跳任务未命中，本轮不发送`
    - 说明“正文已判断应跳过却仍继续外发”的旧缺陷在当前线上样本里已不再复现；残留的 session 写库污染另拆到 [`feishu_scheduler_noop_reply_persisted_to_direct_session.md`](./feishu_scheduler_noop_reply_persisted_to_direct_session.md)
  - 最新复发证据：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5257`，`job_id=j_101f5e64`，`job_name=AAOI 每日动态监控`，`executed_at=2026-04-24T00:01:07.356294+08:00`
    - 本轮落成 `execution_status=completed`、`message_send_status=sent`、`should_deliver=1`、`delivered=1`
    - `response_preview` 明确写入：`AAOI 今日未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送`
    - 最近一小时真实会话 `session_id=Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 也显示：
      - `2026-04-24T00:00:00.368631+08:00` 调度触发 `AAOI 每日动态监控`
      - `2026-04-24T00:01:05.641306+08:00` assistant 正文先写明 `按规则可跳过正式推送`
      - 但同一条 assistant 仍被完整落库为正式长文，并在调度台账里记成 `sent + delivered=1`
    - 这说明当前生产链路不仅没有处理旧措辞 `按规则应跳过正式推送`，连最新常见措辞 `按规则可跳过正式推送` 也仍然直接出站
  - 最新复发证据：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4695`，`job_id=j_5f0b686a`，`job_name=RKLB 每日动态监控`，`executed_at=2026-04-23T00:03:26.575982+08:00`
    - 本轮落成 `execution_status=completed`、`message_send_status=sent`、`should_deliver=1`、`delivered=1`
    - `response_preview` 明确写入：`RKLB 今日未发现新的实质性催化或风险证伪信号，按规则可跳过正式推送`，并在“动作”段写 `不触发正式推送`
    - `run_id=4696`，`job_id=j_379acc40`，`job_name=TEM 每日动态监控`，`executed_at=2026-04-23T00:04:15.585007+08:00`
    - 本轮同样落成 `execution_status=completed`、`message_send_status=sent`、`should_deliver=1`、`delivered=1`
    - `response_preview` 明确写入：`TEM 今日未发现新的实质性催化或风险证伪信号，按规则可跳过正式推送`，并在“动作”段写 `不触发正式推送`
    - 这说明 2026-04-20 标记为 `Fixed` 的发送前 skip-signal 止血没有覆盖当前生产链路，或者规则只覆盖了旧措辞 `按规则应跳过`，未覆盖当前常见措辞 `按规则可跳过` / `不触发正式推送`
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b`
    - `2026-04-20T00:00:59.970078+08:00` 调度触发 `AAOI 每日动态监控`
    - `2026-04-20T00:01:48.936198+08:00` assistant 正文先写明：`AAOI 今日没有出现新的实质性催化或风险证伪信号，按规则应跳过正式推送`
    - 但同一条 assistant 仍被完整落库为 600+ 字正文，说明会话侧确实生成了一条可见播报，而不是静默跳过
    - 同一 `session_id`
    - `2026-04-20T00:01:48.940220+08:00` 调度继续触发 `TEM 每日动态监控`
    - `2026-04-20T00:02:39.875867+08:00` assistant 正文再次写明：`TEM 今日没有出现新的实质性催化或风险证伪信号，按规则应跳过正式推送`
    - 但本轮同样落库为 700+ 字正式长文，而不是 `noop` 或静默结束
  - 最近一小时调度台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=3166`，`job_name=AAOI 每日动态监控`，`executed_at=2026-04-20T00:01:50.059774+08:00`
    - `execution_status=completed`，`message_send_status=sent`，`delivered=1`
    - `response_preview` 明确包含：`AAOI 今日没有出现新的实质性催化或风险证伪信号，按规则应跳过正式推送`
    - `run_id=3167`，`job_name=TEM 每日动态监控`，`executed_at=2026-04-20T00:02:41.173911+08:00`
    - `execution_status=completed`，`message_send_status=sent`，`delivered=1`
    - `response_preview` 同样明确包含：`TEM 今日没有出现新的实质性催化或风险证伪信号，按规则应跳过正式推送`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-20 00:01:48.939` 记录 `session.persist_assistant ... detail=done`
    - 同轮 `done ... success=true ... reply.chars=665`
    - `2026-04-20 00:02:39.877` 再次记录 `session.persist_assistant ... detail=done`
    - 同轮 `done ... success=true ... reply.chars=771`
    - 说明系统并不是只在内部得出“应跳过”的判断，而是实际完成了正式答复写入与发送链路

## 端到端链路

1. Feishu 定时任务按“每日动态监控”模板触发 AAOI、TEM 巡检。
2. 模型正文已经明确判断“没有新增实质性催化/风险证伪信号，应跳过正式推送”。
3. 上层链路没有把这个结论映射成 `noop` 或不发送，而是继续把整段分析性长文持久化并投递给用户。
4. 用户最终收到的是一条自称“应跳过”的正式推送。

## 期望效果

- 当任务配置明确写有“若当日无重要更新，可跳过不推送”时，系统应在无新增催化的场景进入静默跳过或至少 `noop + skipped_noop`。
- 若产品决定仍发送“无更新日报”，则不应在正文里同时声称“按规则应跳过正式推送”，避免自相矛盾。
- 调度台账、会话落库与用户可见语义应保持一致，不能出现“宣称跳过”但实际已送达的双重口径。

## 当前实现效果

- `2026-05-03 00:02` 的最新样本显示，`RKLB`、`TEM`、`AAOI 每日动态监控` 都已经把“今日不触发重大推送”写进正文，但台账仍统一记成 `completed + sent + delivered=1`，且 direct session 尾部连续新增三条 assistant final。
- 相比 `2026-04-26` 已修复窗口，最新坏态不再使用 `按规则可跳过正式推送` / `不触发正式推送` 旧措辞，而是换成 `今日不触发重大催化或风险证伪推送`、`今日不触发新增重大推送` 等新变体后再次穿透发送过滤。
- 2026-04-24 00:01 最新样本显示，`AAOI 每日动态监控` 已明确判断“未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送”，但仍被记成 `completed + sent + delivered=1` 并向用户落库正式正文。
- `AAOI 每日动态监控` 与 `TEM 每日动态监控` 在最近一小时都明确判定“无新增实质性催化”。
- 但两轮都仍被记成 `completed + sent + delivered=1`，并在会话中落为正式 assistant 长文。
- 2026-04-23 复发样本显示，`RKLB 每日动态监控` 与 `TEM 每日动态监控` 已经使用“按规则可跳过正式推送 / 不触发正式推送”这类变体措辞，但发送前过滤仍未中止投递。
- 当前坏态不是单次措辞偏差，而是“业务规则写着跳过，出站行为却仍发送”的执行偏差。

## 用户影响

- 这是质量与业务语义缺陷，不是主链路不可用。
- 用户仍能收到消息，但会收到本应被抑制的“无更新长文”，增加噪音并削弱用户对“真正触发才推送”规则的信任。
- 之所以定级为 `P3`，是因为它没有造成漏报、错投、数据错误或系统失败；当前主要问题是推送策略违背了任务约定。

## 根因判断

- `2026-05-03` 的复发说明，发送前过滤依然依赖有限的自然语言词表，而不是消费一个稳定的结构化 `should_deliver=false` 信号；一旦 answer 改写成 `今日不触发...` 这类新措辞，就会再次漏检。
- 当前“每日动态监控”链路缺少从自然语言结论回收到调度状态机的稳定收口步骤；既有止血可能只覆盖了有限 skip 关键词，未覆盖 `按规则可跳过`、`不触发正式推送` 等新变体。
- 模型已经在正文里完成“应跳过”的判断，但上层仍按普通成功答复处理，未转成 `noop`。
- 问题更像是 direct scheduler 模板与 heartbeat/noop 模板没有共享统一的“无需发送”协议，而不是单次数据误判。

## 修复情况（2026-04-24）

- 已在 `crates/hone-channels/src/scheduler.rs` 扩展 `has_skip_delivery_signal(...)` 的收口词表，新增覆盖：
  - `按规则可跳过正式推送`
  - `按规则可跳过`
  - `可跳过正式推送`
  - `不触发正式推送`
  - `不触发本次正式推送`
  - `无需正式推送`
- 非 heartbeat 定时任务在成功返回后，会先经过这组更完整的 skip-signal 检查；命中后统一收口为 `should_deliver=false`，后续 Feishu 调度记录会落成 `noop + skipped_noop`，不再把“声明跳过”的正文继续外发。
- 新增回归测试：
  - `cargo test -p hone-channels skip_delivery_signal_detected`
  - `cargo test -p hone-channels scheduler::tests`

## 2026-04-26 巡检结论

- 最新 `TEM` / `RKLB 每日动态监控` 已稳定收口为 `noop + skipped_noop`，运行日志同步明确写出 `skip_signal` 与 `本轮不发送`。
- 因此这条缺陷维持 `Fixed`：当前线上不再是“应跳过却仍外发长文”。
- 但同一时间窗发现新的状态边界问题：未发送长文仍被写入 direct session，已拆分为独立缺陷 [`feishu_scheduler_noop_reply_persisted_to_direct_session.md`](./feishu_scheduler_noop_reply_persisted_to_direct_session.md) 跟踪。

## 2026-05-03 巡检结论

- 最新 `RKLB` / `TEM` / `AAOI 每日动态监控` 已不再落成 `noop + skipped_noop`，而是全部回退到 `completed + sent + delivered=1`。
- 最新 answer 不再使用 `按规则可跳过正式推送` 旧措辞，而是改写为 `今日不触发重大催化或风险证伪推送`、`今日不触发新增重大催化或风险证伪推送`、`今日不触发新增重大推送`，说明旧止血只是词表级匹配，当前又被新措辞绕过。
- 因此本单状态从 `Fixed` 回退为 `New`，重新进入活跃缺陷队列；`feishu_scheduler_noop_reply_persisted_to_direct_session.md` 仍是独立边界问题，但不覆盖本轮“明知应跳过却仍实际发送”的主症状。

## 下一步建议

- 为“每日动态监控”类任务补充显式的“无需发送”结构化出口，不要只靠正文自然语言表达“应跳过”。
- 在发送前增加一致性检查：若正文包含“按规则应跳过正式推送”“无实质新增催化，跳过推送”等结论，则应中止正式发送或改写为内部 `noop`。
- 回归样本至少覆盖 `AAOI/TEM` 这类“无新增公司级事件 + 未触发阈值”的日常巡检场景。
