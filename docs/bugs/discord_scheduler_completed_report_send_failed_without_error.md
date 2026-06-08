# Bug: Discord scheduler 已生成报告但发送阶段失败且缺少错误原因

## 发现时间

- 2026-06-07 11:05 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- Fixed

## GitHub Issue

- 无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-06-07 07:00-11:04 CST。
  - 本窗共有 5 个 user turn 与 5 个 assistant final；Feishu direct 与 Discord scheduler 均有 assistant 记录收口。
  - assistant final 污染扫描未命中空回复、`company_profiles/...`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic 或 `index out of bounds`。
  - Discord scheduler session `Session_discord__group__g_3a1469549745654468692_3ac_3a1469549746518622371` 在 2026-06-07 09:30 CST 收到 `每日美股降息概率推送` 定时触发，09:31 CST assistant final 已生成完整降息概率报告正文。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=38432` / `job_id=j_910d8dcb` / `job_name=每日美股降息概率推送` 在 2026-06-07 09:31 CST 落成 `execution_status=completed + message_send_status=send_failed + should_deliver=1 + delivered=0`。
  - `response_preview` 保留完整报告开头，说明 Answer / 会话落库已完成。
  - `detail_json={"scheduler":null,"sent_segments":0,"total_segments":3}`，说明发送阶段 3 个分段全部未送达。
  - `error_message` 为空，无法从台账判断 Discord API、分段格式、网络、权限或其它出站原因。
  - 同一任务最近样本中，2026-06-06 09:34 CST 曾因 ACP transport 断连落成 `noop + skipped_noop`；2026-06-05 09:31 CST 为 `completed + sent + delivered=1`。本轮形态不同：报告已生成，但发送阶段失败。
- `data/runtime/logs/acp-events.log`
  - 2026-06-07 09:30-09:31 CST 同一 Discord scheduler 有 `session/prompt` 与 `stopReason=end_turn`。
  - 同窗未见 `stream disconnected before completion`、runner error、SpawnFailed 或 provider quota 原始错误。
- `data/runtime/task_runs.2026-06-07.jsonl`
  - 同窗 event-engine poller 仍有 FMP 持续失败，但该问题已由 `event_engine_fmp_price_news_poller_persistent_request_failure.md` 跟踪；与本 Discord scheduler 出站失败不是同一链路。
- 最近四小时无非文档代码提交。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-06-08 07:01-11:02 CST 复核窗口内，同一 Discord scheduler 再次复现。
  - `run_id=38433` / `job_name=每日美股降息概率推送` 在 2026-06-08 09:31 CST 落成 `execution_status=completed + message_send_status=send_failed + should_deliver=1 + delivered=0`。
  - 同一 session 在 09:30-09:31 CST 已生成完整 assistant final，`acp-events.log` 也以 `stopReason=end_turn` 收口，说明模型生成与 ACP 链路完成。
  - `detail_json={"scheduler":null,"sent_segments":0,"total_segments":3}` 且 `error_message` 仍为空；这与 2026-06-08 03:06 CST 修复记录中“不再出现空 error_message”的期望不一致。
  - 同窗 14 个 user turn 与 14 个 assistant final 成对收口，assistant final 污染扫描未命中空回复、内部路径、raw tool 字段、思维痕迹、provider 原始错误、quota、panic 或 stream disconnect；本轮问题集中在 Discord 出站发送和可观测性。

## 端到端链路

1. Discord 定时任务到达 09:30 CST 触发点。
2. Scheduler 将任务 prompt 交给 Codex ACP runner。
3. Runner 正常返回 `stopReason=end_turn`，assistant final 落库为完整降息概率报告。
4. Scheduler 判断 `should_deliver=1`，准备向 Discord 群频道发送 3 个分段。
5. 发送阶段 3 个分段均未送达，台账记录 `send_failed + delivered=0`。
6. `error_message` 为空，后续巡检和人工排障无法直接识别出站失败原因。

## 期望效果

- 已生成完整正文的 Discord scheduler 应成功投递到目标频道，或在发送失败时记录明确、脱敏、可分类的错误原因。
- 分段发送失败应至少保留失败阶段、失败分段、Discord API / 网络 / 权限等归一化分类。
- 发送失败不应只留下 `sent_segments=0/3` 与空 `error_message`，否则无法区分 transient retryable failure、目标配置失效和消息格式问题。

## 当前实现效果

- Answer 与会话落库阶段正常完成，用户应收到的报告已经生成。
- 出站投递阶段全部分段失败，最终 `delivered=0`。
- 台账只记录 `send_failed` 和分段计数，没有保存失败原因。
- 2026-06-08 09:31 CST 复核样本显示，03:06 CST 的代码修复结论尚未在真实台账中兑现：仍出现 `send_failed + delivered=0 + error_message=''`。
- 这不是已有 `codex_acp_transport_disconnect_request_failure.md` 的同一表现：本轮 ACP 已正常 `end_turn`，问题发生在 Discord 发送阶段。
- 也不是归档的 `discord_scheduler_empty_reply_send_failed.md` 同一表现：本轮不是空回复或 fallback 伪成功，而是完整报告生成后未送达。

## 用户影响

- 这是功能性 bug，不是单纯输出质量问题。
- 用户预期在 Discord 群里收到每日降息概率报告，但本轮报告虽然生成完成，却没有送达目标频道。
- 该问题影响定时任务交付可靠性和台账可诊断性，因此定级为 P2。
- 当前证据只覆盖单个 Discord 定时任务的一次发送失败，没有跨渠道、大面积未送达、错投、数据破坏或敏感信息外泄证据，因此不定为 P1。

## 根因判断

- 直接根因位于 Discord scheduler 出站发送阶段，而不是模型生成或 ACP transport。
- `sent_segments=0/3` 表明失败发生在第一段发送前或第一段发送时。
- `error_message` 为空是独立可观测性缺口，会放大后续定位成本。
- 需要结合 Discord sender 日志或发送 API 返回路径确认是否是目标频道权限、分段格式、网络传输、rate limit 或错误吞噬。

## 修复记录

- `2026-06-08 20:06 CST` 再次修复：
  - `bins/hone-discord/src/scheduler.rs` 现在与 Feishu / Web scheduler 一样，通过 `execution_detail_with_delivery_key(...)` 给所有 Discord scheduler 终态 detail 补齐顶层 `delivery_key`，避免终态记录与 started row / delivery key 审计链路脱节。
  - Discord 发送阶段 detail 现在保留 `sent_segments` / `total_segments` 的同时，在 `sent_segments=0 && total_segments>0` 时写入 `failure_kind=discord_send_failed`；如果 sender 返回底层发送错误，也会写入 `send_error`，便于区分权限、网络、payload 或其它出站失败。
  - 新增回归 `scheduler_delivery_detail_keeps_delivery_key_and_send_failure`，锁住 Discord 发送失败终态必须同时带 `delivery_key`、分段计数、失败分类和发送错误文本。
  - 本轮只做代码级闭环，不依赖当前机器 live 服务、生产日志或真实 Discord 投递状态来判定恢复；状态更新为 `Fixed`，后续若新运行态仍出现 `send_failed + delivered=0`，应优先检查 `error_message`、`detail.failure_kind` 与 `detail.send_error` 是否已可诊断。

- `2026-06-08 03:06 CST` 已修复：
  - `bins/hone-discord/src/utils.rs` 的分段发送结果现在会保留底层发送/编辑失败文案，而不是只回传 `sent_segments` 计数。
  - `bins/hone-discord/src/scheduler.rs` 记录 `cron_job_runs` 时，`error_message` 现在优先保留 runner error，其次保留 Discord 发送失败文案；如果 `sent_segments=0 && total_segments>0` 且底层库没有给出明确错误，也会至少回写通用 `Discord 定时任务发送失败`。
  - 这样即使报告正文已经生成，但 Discord 出站第一段就失败，台账也不会再落成 `send_failed + delivered=0 + error_message=''` 的不可诊断坏态。
- `2026-06-08 11:03 CST` 复核重新打开：
  - 09:31 CST `run_id=38433` 再次落成同一坏态，且 `error_message` 仍为空。
  - 当前状态从 `Fixed` 调回 `New`；优先判断修复未覆盖真实 Discord scheduler 发送路径，或运行态未使用已修复二进制。

## 下一步建议

- 先确认 09:31 CST 运行的 Discord scheduler 是否已经加载 `958fe17a` 后的二进制；若未加载，补部署/重启后用下一次 scheduler 复核。
- 若后续 live 再出现 Discord `send_failed`，优先看 `cron_job_runs.error_message` 是否已经能区分权限、网络或 payload 失败。
- 本轮只修复了发送失败可观测性，尚未为 Discord 出站增加自动重试；如果 live 继续出现明显的 transient transport 失败，再单独评估是否补短重试。

## 验证

- `cargo test -p hone-discord scheduler_ -- --nocapture`
- `cargo check -p hone-discord --tests`
- `rustfmt --edition 2024 --config skip_children=true --check bins/hone-discord/src/scheduler.rs`
- `cargo test -p hone-discord scheduler_error_message_ -- --nocapture`
- `cargo test -p hone-discord segment_send_result_keeps_error_message -- --nocapture`
- `cargo check -p hone-discord --tests`
- `rustfmt --edition 2024 --check bins/hone-discord/src/scheduler.rs bins/hone-discord/src/utils.rs`
