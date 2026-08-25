# Bug: Heartbeat 定时任务命中 MiniMax output sensitive 拒绝后跳过提醒

## 发现时间

- 2026-08-25 14:02 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- Fixed（代码级，待部署复核）

## 最新进展

- `2026-08-25` `bug-2` 已补上 heartbeat 对 provider 内容安全拒绝的单次恢复路径：
  - `scheduler.rs` 现在会把 `output new_sensitive (1027)` 归类为 `provider_content_safety_refusal`，不再混成普通 `runner_error`。
  - `heartbeat_recovery_reason(...)` 新增 `ContentSafetyRefusal` 分支；首次命中该类错误时，会进入现有 heartbeat budget recovery，要求模型只用更中性、更短的表述重试一次，无法安全改写时返回 `noop`，避免直接静默漏发。
  - recovery prompt 明确要求避免渲染血腥、暴力、极端或耸动细节，只保留触发事实、关键数据与检查时间。
- 本轮验证：
  - `cargo test -p hone-channels heartbeat_provider_content_safety_refusal_is_classified --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_recovery_reason_covers_context_iteration_and_transport_failures --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_content_safety_recovery_prompt_mentions_neutral_short_path --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- 当前仍缺 live runtime 自然部署后的运行态复核，因此状态先记为代码级 `Fixed`，不直接关闭。

## 证据来源

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-25 10:02-14:02 CST（UTC 2026-08-25 02:02-06:02）。
  - 14:00 CST `存储板块关键事件心跳提醒` 在 `MiniMax-M2.7-highspeed` function-calling runner 上落成 `success=false`，错误为 `LLM 错误: stream provider error: output new_sensitive (1027)`。
  - 同一 job 随后记录 `failure_kind=runner_error`，Web events 记录 `定时任务执行失败，跳过发送`。
  - 本轮日志中 `output new_sensitive` 相关信号共 3 条，均指向同一次 heartbeat 失败。
- `data/runtime/task_runs.2026-08-25.jsonl`
  - 同窗 event-engine 仍推进：`poller.fmp.price ok=16`、`poller.fmp.extended_hours ok=8`、`poller.fmp.news failed=8`。
- `data/sessions.sqlite3`
  - 本地镜像仍停在 2026-08-01 / 2026-08-02，无法提供近窗真实会话内容；本轮以 source log 和 task_runs 为准。

## 端到端链路

1. Web heartbeat scheduler 触发 `存储板块关键事件心跳提醒`。
2. 调度链路将任务交给 function-calling runner，模型为 `MiniMax-M2.7-highspeed`。
3. 上游流式响应返回 `output new_sensitive (1027)`。
4. scheduler 将该轮记为 runner error，Web events 跳过发送。

## 期望效果

- Provider 内容安全拒绝不应让 heartbeat 监控静默漏发。
- 系统应能用脱敏/降级 prompt 重试，或输出产品化的安全降级结论，并保留可审计失败分类。
- 用户可见侧不应暴露 provider 原始错误。

## 当前实现效果

- 单次 provider 内容安全拒绝直接导致对应 heartbeat 失败并跳过发送。
- 日志中能看到 `runner_error`，但没有证据显示进入更短、更安全的 heartbeat recovery 或 provider fallback。
- 本轮未见原始错误进入用户可见 final。

## 用户影响

- 受影响用户在该轮不会收到本应覆盖的存储板块心跳提醒。
- 如果某些行业监控 prompt 持续触发 provider 内容安全拒绝，相关监控会周期性漏发。
- 同窗仍有其它 heartbeat deliver 和 event-engine ok 样本，因此不是全渠道不可用，不定级为 P1。

## 根因判断

- 这是 provider 内容安全拒绝类失败，不同于既有 MiniMax HTTP 529 / transport failure，也不同于 FMP poller 请求失败。
- 当前 heartbeat recovery 对 `output new_sensitive (1027)` 这类上游安全拒绝缺少独立分类和降级重试策略。

## 后续复核

- 部署后需复看真实 heartbeat 窗口：同类 `output new_sensitive (1027)` 应先进入 `retry_with_budget_recovery reason=content_safety_refusal`，随后收口为合规 JSON 或 `noop`，而不是直接 `runner_error + skipped_error`。
- 若 live 仍直接跳过发送，再继续排查是否需要在 provider / prompt 侧补更窄的 neutralization，而不是回退当前分类与 recovery。
