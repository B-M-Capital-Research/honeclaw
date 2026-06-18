# Bug: function_calling LLM audit 写入 PostgreSQL 持续序列化失败

## 发现时间

- 2026-06-18 23:03 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- New

## GitHub Issue

- 无，非 P1

## 证据来源

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

- 最近四小时内 function-calling audit 写入持续失败 684 次。
- 日志只暴露 `error serializing parameter 3`，缺少字段名、record id、provider/model 或可脱敏定位信息。
- 用户回复主链路仍正常收口，因此这不是直聊 / scheduler 投递 P1。
- 但审计记录持续丢失会削弱后续问题追踪、模型调用复盘、成本 / 质量分析与合规留痕，因此按功能性系统错误定级为 P2。

## 用户影响

- 直接用户回复未被阻断；最近四小时 ACP prompt 均有 `end_turn` 收口。
- 间接影响是运行审计缺失：后续 agent / 人工排查模型调用、工具调用质量、失败原因和成本时，会缺少 PostgreSQL audit 真相源。
- 该问题影响系统可观测性和审计数据完整性，不只是文案质量，因此不是 P3；但没有造成用户不可用、错投或数据安全泄露证据，因此不是 P1。

## 根因判断

- 当前证据只能确认 cloud PostgreSQL audit 写入路径在序列化第 3 个参数时稳定失败。
- 初步判断可能是 `LlmAuditRecord` 到 `CloudLlmAuditRecord` 的某个 JSON / timestamp / metadata 字段类型与 SQL bind 类型不匹配，或 cloud runtime schema 与写入代码漂移。
- 该问题不同于 `sessions_sqlite_mirror_stalled_after_successful_direct_replies.md`：本轮不是会话镜像停滞，而是 LLM audit 写入 PostgreSQL 失败。

## 下一步建议

- 在 `CloudPgRuntime::upsert_llm_audit_record(...)` 写入失败时补充脱敏字段级定位：record id、source、provider/model、失败参数对应字段名。
- 增加覆盖 function-calling audit metadata 的 PostgreSQL 写入回归，复现含工具调用 / heartbeat metadata 的记录。
- 若 PostgreSQL schema 已漂移，补 migration 或兼容序列化；若是单字段无法写入，应先降级清洗该字段，避免整条 audit 丢失。
