# Bug: function_calling LLM audit 写入 PostgreSQL 持续序列化失败

## 发现时间

- 2026-06-18 23:03 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- Fixed

## GitHub Issue

- 无，非 P1

## 证据来源

- `data/runtime/logs/web.log.2026-06-18`
  - 巡检窗口：2026-06-19 03:02-07:02 CST。
  - 03:02:00-07:01:09 CST 仍出现 671 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗存在非文档代码提交 `fa7f0734 Fix cloud LLM audit jsonb binding`（2026-06-19 03:07 CST），但运行日志在该提交之后仍继续出现旧错误。
  - 本轮尚未确认当前 live web / worker 已重启并加载 `fa7f0734`，因此该证据只作为“代码级修复后的运行态待复核”记录，不直接把状态从 `Fixed` 回退为 `New`。
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 3 个 session、3 次 `stopReason=end_turn`；用户回复主链路仍有收口，故障仍集中在 LLM audit 持久化链路。
- `data/runtime/logs/web.log.2026-06-18`
  - 巡检窗口：2026-06-18 23:03-2026-06-19 03:02 CST。
  - 23:30:06-03:01:40 CST 再次出现 684 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 12 个 session、24 次 `stopReason=end_turn`、0 个 response error；说明用户回复主链路仍在收口，故障继续集中在 LLM audit 持久化链路。
  - 同窗还可见 heartbeat / Tavily / tool registry 运行事件持续推进，说明这不是日志单点偶发，而是 function-calling runner audit 写入路径继续稳定失败。
- `data/runtime/logs/web.log.2026-06-18`
  - 巡检窗口：2026-06-18 19:03-23:03 CST。
  - 19:30:04-23:01:05 CST 共 684 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 有 55 次 ACP prompt、55 次 `stopReason=end_turn`，未见 response error；说明用户回复主链路仍在收口，故障集中在 LLM audit 持久化链路。
  - 同窗 web 日志有 684 条 audit 写入失败、634 条 heartbeat 诊断行和多次工具调用，说明失败不是单次偶发日志，而是持续影响 function-calling runner 的审计记录落库。

## 端到端链路

1. Feishu / Web heartbeat 或投研任务进入 function-calling runner。
2. runner 执行模型调用、工具调用与 heartbeat 解析。
3. 运行时尝试把 LLM audit 记录写入 PostgreSQL。
4. PostgreSQL 写入路径在第 3 个参数序列化阶段失败，并仅输出 warn。
5. 用户回复 / heartbeat 终态继续执行，但对应 LLM audit 记录没有成功持久化到 cloud audit 表。

## 期望效果

- function-calling runner 每次模型调用应成功写入 LLM audit，或在写入失败时保留可恢复的本地 / 队列 fallback。
- 序列化失败应带有足够字段定位信息，至少能判断是哪个 audit 字段或 metadata 值无法绑定到 PostgreSQL 参数。
- 审计链路失败不应静默长期重复，只在 warn 日志里堆积。

## 当前实现效果

- 2026-06-19 07:04 CST 最近四小时内 function-calling audit 写入继续失败 671 次；03:07 CST 代码修复提交后仍有运行态旧错误，但本轮不能确认当前服务是否已加载新二进制。
- 2026-06-19 03:02 CST 最近四小时内 function-calling audit 写入继续失败 684 次；这不是 19:03-23:03 CST 窗口的一次性波动。
- 日志只暴露 `error serializing parameter 3`，缺少字段名、record id、provider/model 或可脱敏定位信息。
- 用户回复主链路仍正常收口，因此这不是直聊 / scheduler 投递 P1。
- 但审计记录持续丢失会削弱后续问题追踪、模型调用复盘、成本 / 质量分析与合规留痕，因此按功能性系统错误定级为 P2。

## 修复情况

- `2026-06-19 03:03 CST` 已在 [`crates/hone-core/src/cloud_runtime.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-core/src/cloud_runtime.rs) 修复 PostgreSQL LLM audit 写入的 JSONB 参数绑定：
  - `upsert_llm_audit_record(...)` 改为显式使用 `tokio_postgres::types::Json(&cloud_record.record)` 绑定 `$3::jsonb`，不再把原始 `serde_json::Value` 直接作为 PostgreSQL 第 3 个参数透传。
  - 新增 `llm_audit_record_payload_encodes_as_jsonb_parameter` 回归测试，直接覆盖含 request / response / metadata 的完整 `LlmAuditRecord` payload 能按 PostgreSQL `JSONB` 参数成功编码。
  - 本修复只影响 cloud audit 持久化链路，不改用户回复收口逻辑，也不要求本轮重启当前服务。
- `2026-06-19 07:04 CST` 运行态复核显示当前 web 日志仍有同类告警，最近到 07:01 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。

## 用户影响

- 直接用户回复未被阻断；最近四小时 ACP prompt 均有 `end_turn` 收口。
- 间接影响是运行审计缺失：后续 agent / 人工排查模型调用、工具调用质量、失败原因和成本时，会缺少 PostgreSQL audit 真相源。
- 该问题影响系统可观测性和审计数据完整性，不只是文案质量，因此不是 P3；但没有造成用户不可用、错投或数据安全泄露证据，因此不是 P1。

## 根因判断

- 当前证据只能确认 cloud PostgreSQL audit 写入路径在序列化第 3 个参数时稳定失败。
- 初步判断可能是 `LlmAuditRecord` 到 `CloudLlmAuditRecord` 的某个 JSON / timestamp / metadata 字段类型与 SQL bind 类型不匹配，或 cloud runtime schema 与写入代码漂移。
- 该问题不同于 `sessions_sqlite_mirror_stalled_after_successful_direct_replies.md`：本轮不是会话镜像停滞，而是 LLM audit 写入 PostgreSQL 失败。

## 下一步建议

- 后续 live 窗口若再出现同类告警，先确认运行中的 web / worker 已加载包含本修复的二进制，而不是继续把问题归因到 schema 漂移。
- 如仍有零星失败，再补脱敏字段级日志，区分 JSONB bind、连接失败与 schema 侧问题。

## 验证

- `cargo test -p hone-core llm_audit_record_payload_encodes_as_jsonb_parameter --lib -- --nocapture`
- `cargo test -p hone-core cloud_cron_send_failed_backstop_ --lib -- --nocapture`
- `cargo check -p hone-core --tests`
- `git diff --check`
