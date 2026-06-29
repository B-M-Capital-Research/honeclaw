# Bug: function_calling LLM audit 写入 PostgreSQL 持续序列化失败

## 发现时间

- 2026-06-18 23:03 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- New

## 修复记录

- 2026-06-29 19:01 CST
  - 15:00-19:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 764 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 5 次 `session/prompt`、5 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-29 15:07 CST
  - 11:00-15:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 789 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 4 次 `session/prompt`、5 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-29 11:01 CST
  - 07:00-11:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 786 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 38 次 `session/prompt`、36 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-29 07:02 CST
  - 03:04-07:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 696 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 7 次 `session/prompt`、6 个 session、3 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-29 03:01 CST
  - 23:02-03:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 719 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 3 次 `session/prompt`、1 个 session、3 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-28 19:02 CST
  - 15:02-19:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 641 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 9 次 `session/prompt`、6 个 session、8 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-28 15:02 CST
  - 11:01-15:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 633 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 8 次 `session/prompt`、6 个 session、8 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-28 11:01 CST
  - 07:01-11:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 692 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 19 次 `session/prompt`、12 个 session、19 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-27 23:01 CST
  - 19:01-23:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 692 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 23 次 `session/prompt`、17 个 session、22 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-27 19:01 CST
  - 15:01-19:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 662 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 11 次 `session/prompt`、5 个 prompt session、11 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-27 15:02 CST
  - 11:02-15:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 612 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 2 次 `session/prompt`、2 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-27 11:01 CST
  - 07:01-11:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 499 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 16 次 `session/prompt`、11 个 prompt session、16 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-27 07:02 CST
  - 03:00-07:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 771 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 7 次 `session/prompt`、4 个 session、7 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-27 03:01 CST
  - 23:03-03:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 674 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 9 个 prompt session、16 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-26 19:02 CST
  - 15:01-19:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 717 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可见 18 次 `session/prompt`、7 个 session、74 个 response、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-26 15:01 CST
  - 11:05-15:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 682 条。
  - 同窗 `data/runtime/logs/acp-events.log` 可用窗口可见 3 次 `session/prompt`、3 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
  - 状态维持 `New`，不创建 GitHub Issue。
- 2026-06-26 11:05 CST
  - 07:01-11:05 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 549 条。
  - 11:00 CST `data/runtime/logs/web.log.2026-06-26` 出现 cloud table / `conversation_quota` 初始化日志，晚于 03:07 CST 非文档代码提交 `952af973 fix: harden llm audit postgres writes`；同一 11:00 CST 窗口仍继续输出 `error serializing parameter 3`。
  - 因此本轮不再按“代码级 Fixed、待重启复核”处理，将状态从 `Fixed` 回退为 `New`。
  - 同窗 ACP 直聊 / scheduler 侧可见 33 次 `session/prompt`、34 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-26 07:01 CST
  - 03:00-07:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 706 条。
  - 同窗存在 03:07 CST 非文档代码提交 `952af973 fix: harden llm audit postgres writes`，但本轮未见 web / worker 进程重启并加载该提交的明确证据；因此这批 live 错误先按旧运行态 / 待重启复核处理。
  - 状态仍保持代码级 `Fixed`。同窗 ACP 直聊 / scheduler 侧 7 次 `session/prompt`、7 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-26 03:04 CST
  - `crates/hone-core/src/cloud_runtime.rs` 的 `upsert_llm_audit_record(...)` 现改为把 audit payload 先序列化为 JSON 文本，再以 `$3::text::jsonb` 写入 PostgreSQL；`created_at` 也先做 RFC3339 校验，再以 `$4::text::timestamptz` 写入。
  - 这次修复不再依赖 `tokio-postgres` 对 `Json<T>` 和 `chrono::DateTime` 的参数编码分支，直接收敛到 PostgreSQL 自身的 `jsonb/timestamptz` 文本 cast，覆盖此前两轮“单测通过但 live 仍报 `error serializing parameter 3`”的剩余缺口。
  - 新增回归 `llm_audit_record_uses_text_cast_inputs_for_postgres_insert`，直接覆盖当前真实写入路径使用的文本参数编码；既有 `llm_audit_record_payload_encodes_as_jsonb_parameter` 与 `llm_audit_record_created_at_encodes_as_timestamptz_parameter` 继续保留，分别覆盖 JSONB / `timestamptz` 的底层类型编码。
  - 验证通过：
    - `cargo test -p hone-core llm_audit_record_ --lib -- --nocapture`
    - `cargo check -p hone-core --tests`
    - `git diff --check`
  - 本轮未重启当前 web / worker 进程；运行态日志是否止血仍待后续巡检窗口复核，因此本单为代码级 `Fixed`，未直接标 `Closed`。
- 2026-06-26 03:01 CST
  - 23:01-03:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 632 条。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 13 次 `session/prompt`、14 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-25 23:02 CST
  - 19:01-23:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 659 条。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 41 次 `session/prompt`、41 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-25 19:01 CST
  - 15:01-19:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 692 条，最近到 19:00 CST。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 2 次 `session/prompt`、2 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-25 15:01 CST
  - 11:01-15:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 670 条，最近到 15:01 CST。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 6 次 `session/prompt`、6 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-25 11:01 CST
  - 07:04-11:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 561 条，最近到 11:01 CST。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 36 次 `session/prompt`、36 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-25 07:04 CST
  - 03:01-07:04 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 1360 条，最近到 07:01 CST。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 9 次 `session/prompt`、8 个 session、9 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-25 03:04 CST
  - 23:02-03:04 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 685 条，最近到 03:01:20 CST。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 15 次 `session/prompt`、8 个 session、15 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-24 23:02 CST
  - 19:00-23:02 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 628 条，最近到 23:01:05 CST。
  - 状态维持 `New`。同窗 ACP 直聊 / scheduler 侧 43 次 `session/prompt`、42 次 `stopReason=end_turn`、0 个 response error；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-24 19:01 CST
  - 15:01-19:01 CST 当前 live 运行态仍持续输出同类 PostgreSQL 参数序列化失败，共 653 条，最近到 19:00:54 CST。
  - 状态维持 `New`。同窗 ACP 直聊侧 11 次 `session/prompt` 均 `stopReason=end_turn`，0 个 response error，用户可见 chunk 污染扫描为 0；问题仍集中在 function-calling audit 持久化与后续排障审计，不直接阻断用户回复或投递，严重等级维持 P2，非 P1。
- 2026-06-24 15:03 CST
  - 10:05 CST 的 `created_at` 参数绑定修复后，当前 live 运行态在 11:30-15:02 CST 仍持续输出同类 PostgreSQL 参数序列化失败，最近到 15:02:17 CST。
  - 本轮将状态从 `Fixed` 回退为 `New`。该问题仍只影响 function-calling audit 持久化与后续排障审计，没有证据显示用户回复、投递或渠道主链路被直接阻断，因此严重等级保持 P2，非 P1。
- 2026-06-24 10:05 CST
  - `crates/hone-core/src/cloud_runtime.rs` 的 `upsert_llm_audit_record(...)` 不再把 `created_at: String` 直接绑定到 `$4::timestamptz`；现在先按 RFC3339 解析为 `chrono::DateTime<Utc>` 再写入 PostgreSQL。
  - 这次修复针对运行态长期出现的 `error serializing parameter 3`。结合 `tokio-postgres` 的参数序列化行为，根因更符合“第 4 个参数 `created_at` 以 `String` 绑定到 `timestamptz` 失败”，而不是此前只覆盖到的 JSONB `$3` 参数。
  - 新增回归 `llm_audit_record_created_at_encodes_as_timestamptz_parameter`，补上“完整 SQL 参数绑定里时间戳参数可序列化”的缺口；既有 `llm_audit_record_payload_encodes_as_jsonb_parameter` 继续保留，用于覆盖 `$3::jsonb`。
  - 验证通过：
    - `cargo test -p hone-core llm_audit_record_ --lib -- --nocapture`
    - `cargo check -p hone-core --tests`
    - `git diff --check`
  - 本轮未重启当前 web / worker 进程；运行态日志是否止血仍待后续巡检窗口复核。

## GitHub Issue

- 无，非 P1

## 证据来源

- `data/runtime/logs/web.log.2026-06-25`
  - 巡检窗口：2026-06-25 11:01-15:01 CST。
  - 同窗继续出现 670 条同类告警，最近到 15:01 CST：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 6 次 `session/prompt`、6 次 `stopReason=end_turn`、0 个 response error；用户可见文本聚合未确认新的内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或资源耗尽外泄。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-25`
  - 巡检窗口：2026-06-25 07:04-11:01 CST。
  - 同窗继续出现 561 条同类告警，最近到 11:01 CST：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 36 次 `session/prompt`、36 次 `stopReason=end_turn`、0 个 response error；用户可见 chunk 污染扫描未确认新的内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或资源耗尽外泄。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-24`
  - 巡检窗口：2026-06-25 03:01-07:04 CST。
  - 同窗继续出现 1360 条同类告警，最近到 07:01 CST：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 9 次 `session/prompt`、8 个 session、9 次 `stopReason=end_turn`、0 个 response error；用户可见 chunk 污染扫描未确认新的内部路径、raw tool 字段、思维痕迹、provider 原始错误、panic 或资源耗尽外泄。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-24`
  - 巡检窗口：2026-06-24 23:02-2026-06-25 03:04 CST。
  - 同窗继续出现 685 条同类告警，最近到 03:01:20 CST：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 15 次 `session/prompt`、8 个 session、15 次 `stopReason=end_turn`、0 个 response error；用户可见 chunk 污染扫描未命中内部路径、raw tool 字段、思维痕迹、provider 原始错误或 panic。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-24`
  - 巡检窗口：2026-06-24 19:00-23:02 CST。
  - 同窗继续出现 628 条同类告警，最近到 23:01:05 CST：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 43 次 `session/prompt`、30 个 session、42 次 `stopReason=end_turn`、0 个 response error；23:00 CST 新启动的 Feishu 任务在巡检截止时只运行约 2 分钟，不按失败或超时处理。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-24`
  - 巡检窗口：2026-06-24 15:01-19:01 CST。
  - 同窗继续出现 653 条同类告警，最近到 19:00:54 CST：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 11 次 `session/prompt`、6 个 session、11 次 `stopReason=end_turn`、0 个 response error；用户可见 chunk 污染扫描未命中内部路径、raw tool 字段、思维痕迹、provider 原始错误或 panic。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-24`
  - 巡检窗口：2026-06-24 11:02-15:02 CST。
  - 10:05 CST 代码级修复后，11:30-15:02 CST 仍出现 696 条同类告警，最近到 15:02:17 CST：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 4 个 session、4 次 `session/prompt`、4 次 `stopReason=end_turn`、0 个 response error、1141 个用户可见 chunk 污染命中 0；故障继续集中在 function-calling audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态从 `Fixed` 回退为 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-23`
  - 巡检窗口：2026-06-23 19:02-23:02 CST。
  - 19:02-23:02 CST 仍出现 677 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 29 个 session、49 次 `stopReason=end_turn`、0 个 response error；故障继续集中在 function-calling audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-23`
  - 巡检窗口：2026-06-23 15:02-19:02 CST。
  - 15:02-19:02 CST 仍出现 662 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 5 次 `session/prompt`、4 个 session、5 次 `stopReason=end_turn`，只有 15:02 CST 边界上的既有 Web direct 旧会话 `context_window_exceeded` response error；故障继续集中在 function-calling audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-23`
  - 巡检窗口：2026-06-23 11:02-15:02 CST。
  - 11:02-15:02 CST 仍出现 618 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 9 次 `session/prompt`、7 个 session；除同一 Web direct 旧会话的 2 个 `context_window_exceeded` response error 外，故障继续集中在 function-calling audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-23`
  - 巡检窗口：2026-06-23 07:02-11:02 CST。
  - 07:02-11:02 CST 仍出现 562 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` JSON 事件统计可见 33 次 `session/prompt`、21 个 session、33 次 `stopReason=end_turn`、0 个 response error；用户可见 `agent_message_chunk` 流未见原始工具 JSON、本机绝对路径、provider 原始错误或思维痕迹，故障继续集中在 LLM audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-22`
  - 巡检窗口：2026-06-23 03:04-07:02 CST。
  - 03:04-07:02 CST 仍出现 640 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 文本扫描可见 8 次 `session/prompt`、7 个 session、8 次 `stopReason=end_turn`、0 个 response error；Feishu / Web direct 用户回复主链路仍正常收口，故障继续集中在 LLM audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-22`
  - 巡检窗口：2026-06-22 23:03-2026-06-23 03:02 CST。
  - 23:03-03:02 CST 仍出现 570 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 7 次 `session/prompt`、3 个 session、0 个 response error；用户回复主链路仍正常收口，故障继续集中在 LLM audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-22`
  - 巡检窗口：2026-06-22 19:00-23:03 CST。
  - 19:00-23:03 CST 仍出现 726 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 48 次 `session/prompt`、29 个 session、47 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍正常收口，故障继续集中在 LLM audit 持久化链路。
  - 结论：当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-22`
  - 巡检窗口：2026-06-22 15:04-19:00 CST。
  - 15:04-19:00 CST 仍出现 652 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 4 个 session、5 次 `session/prompt`、5 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍正常收口，故障继续集中在 LLM audit 持久化链路。
  - 结论：15:04 后当前 runtime 窗口继续丢失 function-calling audit 记录，状态维持 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-22`
  - 巡检窗口：2026-06-22 11:02-15:04 CST。
  - 10:30 CST 后日志出现 schema migration / cloud table 初始化信息，说明 web runtime 已重新加载当前服务进程；后续坏态不再按“未确认 live web / worker 已加载修复”处理。
  - 11:02-15:04 CST 仍出现 614 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 3 个 session、3 次 `session/prompt`、3 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍正常收口，故障继续集中在 LLM audit 持久化链路。
  - 结论：2026-06-19 的代码级修复未能在当前 runtime 窗口消除 PostgreSQL 参数序列化失败，审计记录继续丢失；状态从 `Fixed` 回退为 `New`。该问题影响排障 / 回归审计，不直接阻断用户答复或投递，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。
- `data/runtime/logs/web.log.2026-06-20`
  - 巡检窗口：2026-06-20 15:02-19:02 CST。
  - 15:02-19:02 CST 仍出现 628 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 6 次 `session/prompt`、6 次 `stopReason=end_turn`、0 个 ACP response error；用户回复主链路仍正常收口，故障继续集中在 LLM audit 持久化链路。
  - 本轮仍未确认当前 live web / worker 已重启并加载 `fa7f0734`，因此该证据只作为“代码级修复后的运行态待复核”记录，不直接把状态从 `Fixed` 回退为 `New`。
- `data/runtime/logs/web.log.2026-06-19`
  - 巡检窗口：2026-06-20 03:05-07:02 CST。
  - 03:05:07-07:00:57 CST 仍出现 656 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 4 个真实 prompt、4 次 `stopReason=end_turn`；用户回复主链路仍有收口，故障继续集中在 LLM audit 持久化链路。
  - 本轮尚未确认当前 live web / worker 已重启并加载 `fa7f0734`，因此该证据只作为“代码级修复后的运行态待复核”记录，不直接把状态从 `Fixed` 回退为 `New`。
- `data/runtime/logs/web.log.2026-06-19`
  - 巡检窗口：2026-06-19 23:01-2026-06-20 03:01 CST。
  - 23:01:07-03:01:03 CST 仍出现 687 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可见 2 个 session、4 次 `stopReason=end_turn`；用户回复主链路仍有收口，故障继续集中在 LLM audit 持久化链路。
  - 本轮尚未确认当前 live web / worker 已重启并加载 `fa7f0734`，因此该证据只作为“代码级修复后的运行态待复核”记录，不直接把状态从 `Fixed` 回退为 `New`。
- `data/runtime/logs/web.log.2026-06-19`
  - 巡检窗口：2026-06-19 19:01-23:01 CST。
  - 19:01:04-23:01:19 CST 仍出现 785 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 21 个 session、29 次 prompt、29 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍有收口，故障继续集中在 LLM audit 持久化链路。
  - 本轮尚未确认当前 live web / worker 已重启并加载 `fa7f0734`，因此该证据只作为“代码级修复后的运行态待复核”记录，不直接把状态从 `Fixed` 回退为 `New`。
- `data/runtime/logs/web.log.2026-06-19`
  - 巡检窗口：2026-06-19 15:00-19:01 CST。
  - 15:00:06-19:00:57 CST 仍出现 798 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 4 个 session、5 次 `session/prompt`、5 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍有收口，故障继续集中在 LLM audit 持久化链路。
  - 本轮尚未确认当前 live web / worker 已重启并加载 `fa7f0734`，因此该证据只作为“代码级修复后的运行态待复核”记录，不直接把状态从 `Fixed` 回退为 `New`。
- `data/runtime/logs/web.log.2026-06-19`
  - 巡检窗口：2026-06-19 07:02-11:02 CST。
  - 08:00:10-11:01:09 CST 仍出现 625 条同类告警：
    - `[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 15 个 session、26 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍有收口，故障仍集中在 LLM audit 持久化链路。
  - 本轮尚未确认当前 live web / worker 已重启并加载 `fa7f0734`，因此该证据只作为“代码级修复后的运行态待复核”记录，不直接把状态从 `Fixed` 回退为 `New`。
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

- 2026-06-24 15:03 CST 最近四小时内 function-calling audit 写入继续失败 696 次；这些日志晚于 10:05 CST `created_at` 绑定修复，且当前 `web.log.2026-06-24` 仍在持续写入同一错误。
- 2026-06-19 07:04 CST 最近四小时内 function-calling audit 写入继续失败 671 次；03:07 CST 代码修复提交后仍有运行态旧错误，但本轮不能确认当前服务是否已加载新二进制。
- 2026-06-19 03:02 CST 最近四小时内 function-calling audit 写入继续失败 684 次；这不是 19:03-23:03 CST 窗口的一次性波动。
- 日志只暴露 `error serializing parameter 3`，缺少字段名、record id、provider/model 或可脱敏定位信息。
- 用户回复主链路仍正常收口，因此这不是直聊 / scheduler 投递 P1。
- 但审计记录持续丢失会削弱后续问题追踪、模型调用复盘、成本 / 质量分析与合规留痕，因此按功能性系统错误定级为 P2。

## 历史修复情况

- `2026-06-19 03:03 CST` 已在 [`crates/hone-core/src/cloud_runtime.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-core/src/cloud_runtime.rs) 修复 PostgreSQL LLM audit 写入的 JSONB 参数绑定：
  - `upsert_llm_audit_record(...)` 改为显式使用 `tokio_postgres::types::Json(&cloud_record.record)` 绑定 `$3::jsonb`，不再把原始 `serde_json::Value` 直接作为 PostgreSQL 第 3 个参数透传。
  - 新增 `llm_audit_record_payload_encodes_as_jsonb_parameter` 回归测试，直接覆盖含 request / response / metadata 的完整 `LlmAuditRecord` payload 能按 PostgreSQL `JSONB` 参数成功编码。
  - 本修复只影响 cloud audit 持久化链路，不改用户回复收口逻辑，也不要求本轮重启当前服务。
- `2026-06-19 07:04 CST` 运行态复核显示当前 web 日志仍有同类告警，最近到 07:01 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。
- `2026-06-19 11:02 CST` 运行态复核显示当前 web 日志仍有同类告警 625 条，最近到 11:01 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。
- `2026-06-19 19:01 CST` 运行态复核显示当前 web 日志仍有同类告警 798 条，最近到 19:00 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。
- `2026-06-19 23:01 CST` 运行态复核显示当前 web 日志仍有同类告警 785 条，最近到 23:01 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。
- `2026-06-20 03:01 CST` 运行态复核显示当前 web 日志仍有同类告警 687 条，最近到 03:01 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。
- `2026-06-20 07:02 CST` 运行态复核显示当前 web 日志仍有同类告警 656 条，最近到 07:00 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。
- `2026-06-20 19:02 CST` 运行态复核显示当前 web 日志仍有同类告警 628 条，最近到 19:00 CST；由于尚未确认 live web / worker 已部署或重启到 `fa7f0734`，本单状态仍保持代码级 `Fixed`，待新运行态再复核是否真正止住。
- `2026-06-24 10:05 CST` 已在 [`crates/hone-core/src/cloud_runtime.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-core/src/cloud_runtime.rs) 修复遗漏的 `created_at` 参数绑定：先把 RFC3339 字符串解析为 `chrono::DateTime<Utc>`，再绑定到 `$4::timestamptz`；验证 `cargo test -p hone-core llm_audit_record_ --lib -- --nocapture`、`cargo check -p hone-core --tests` 与 `git diff --check` 通过。
- `2026-06-24 15:03 CST` 运行态复核显示 10:05 CST 修复后仍有同类告警 696 条，最近到 15:02:17 CST；本轮按当前 live 证据将状态从代码级 `Fixed` 回退为 `New`。
- `2026-06-26 03:04 CST` 已在 [`crates/hone-core/src/cloud_runtime.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-core/src/cloud_runtime.rs) 把 LLM audit 写入参数统一收敛为文本输入再由 PostgreSQL 显式 cast：`record` 用 `$3::text::jsonb`，`created_at` 用 `$4::text::timestamptz`。这样不再依赖 Rust 侧 `Json<T>` / `chrono` 参数编码分支，可直接覆盖 live 一直报的 `error serializing parameter 3`。新增回归 `llm_audit_record_uses_text_cast_inputs_for_postgres_insert`，并通过 `cargo test -p hone-core llm_audit_record_ --lib -- --nocapture`、`cargo check -p hone-core --tests`、`git diff --check`。当前运行态是否止血仍待后续巡检窗口复核，因此状态回到代码级 `Fixed`。

## 用户影响

- 直接用户回复未被阻断；最近四小时 ACP prompt 均有 `end_turn` 收口。
- 间接影响是运行审计缺失：后续 agent / 人工排查模型调用、工具调用质量、失败原因和成本时，会缺少 PostgreSQL audit 真相源。
- 该问题影响系统可观测性和审计数据完整性，不只是文案质量，因此不是 P3；但没有造成用户不可用、错投或数据安全泄露证据，因此不是 P1。

## 根因判断

- 当前证据只能确认 cloud PostgreSQL audit 写入路径在序列化第 3 个参数时稳定失败；两次已知代码级修复分别覆盖 JSONB 与 `created_at` timestamptz 绑定，但 live 运行态仍复现同一错误。
- 初步判断仍可能是 `LlmAuditRecord` 到 `CloudLlmAuditRecord` 的某个 JSON / timestamp / metadata 字段类型与 SQL bind 类型不匹配、参数序号与日志口径存在偏移，或 live cloud runtime schema / SQL 绑定路径并非已修复测试覆盖的那一支。
- 该问题不同于 `sessions_sqlite_mirror_stalled_after_successful_direct_replies.md`：本轮不是会话镜像停滞，而是 LLM audit 写入 PostgreSQL 失败。

## 下一步建议

- 先在 cloud audit 写入路径补一条脱敏字段级诊断，至少记录 SQL 参数序号、逻辑字段名、Rust 类型类别和 audit record id 哈希，避免继续只能看到 `parameter 3`。
- 复核 live web / worker 实际加载的二进制版本；若已确认加载 10:05 CST 修复，则下一步应对完整 SQL bind 列表逐项加编码回归，而不是继续只猜 JSONB / timestamptz 两个字段。

## 验证

- `cargo test -p hone-core llm_audit_record_payload_encodes_as_jsonb_parameter --lib -- --nocapture`
- `cargo test -p hone-core cloud_cron_send_failed_backstop_ --lib -- --nocapture`
- `cargo check -p hone-core --tests`
- `git diff --check`

## 最新运行态复核（2026-06-28 03:00 CST）

- `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-06-27 23:01-2026-06-28 03:00 CST。
  - 本窗继续出现 666 条同类告警：`[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`。
  - 告警与 heartbeat / function-calling 工具调用同窗持续出现，说明不是单次启动瞬态。
- `data/runtime/logs/acp-events.log`
  - 同窗可重构 5 次 `session/prompt`、6 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍有收口。
- 本轮判断
  - 最新证据继续支持 cloud LLM audit 持久化链路仍未止血，状态维持 `New`、严重等级维持 `P2`。
  - 由于未阻断用户回复或 scheduler 出站本身，且没有用户可见原始错误，本轮不升级为 P1。

## 最新运行态复核（2026-06-28 07:02 CST）

- `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-06-28 03:02-07:02 CST。
  - 本窗继续出现 645 条同类告警：`[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`。
  - 告警与 heartbeat / function-calling 工具调用同窗持续出现，说明不是单次启动瞬态。
- `data/runtime/logs/acp-events.log`
  - 同窗可重构 2 次 `session/prompt`、2 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍有收口。
- 本轮判断
  - 最新证据继续支持 cloud LLM audit 持久化链路仍未止血，状态维持 `New`、严重等级维持 `P2`。
  - 由于未阻断用户回复或 scheduler 出站本身，且没有用户可见原始错误，本轮不升级为 P1。

## 最新运行态复核（2026-06-28 23:02 CST）

- `data/runtime/logs/web.log.2026-06-28` / `data/runtime/logs/hone_cli_screen.log`
  - 巡检窗口：2026-06-28 19:02-23:02 CST。
  - 本窗继续出现 666 条同类告警：`[LlmAudit] failed to persist function_calling audit: 配置错误: Postgres LLM audit 写入失败: error serializing parameter 3`。
  - 告警与 heartbeat / function-calling 工具调用同窗持续出现，说明不是单次启动瞬态。
- `data/runtime/logs/acp-events.log`
  - 同窗可重构 30 次 `session/prompt`、30 次 `stopReason=end_turn`、0 个 response error；用户回复主链路仍有收口。
- 本轮判断
  - 最新证据继续支持 cloud LLM audit 持久化链路仍未止血，状态维持 `New`、严重等级维持 `P2`。
  - 由于未阻断用户回复或 scheduler 出站本身，且没有用户可见原始错误，本轮不升级为 P1。
