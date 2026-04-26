# Non-P1 Fixing Bug Batch

- title: Non-P1 Fixing Bug Batch
- status: in_progress
- created_at: 2026-04-26
- updated_at: 2026-04-26
- owner: codex
- related_files:
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/runtime.rs`
  - `bins/hone-discord/src/scheduler.rs`
- related_docs:
  - `docs/bugs/README.md`
  - `docs/bugs/oil_price_scheduler_geopolitical_hallucination.md`
  - `docs/bugs/feishu_direct_compact_retry_still_cannot_answer_new_topic.md`
  - `docs/bugs/feishu_scheduler_compact_retry_still_cannot_finish_company_digest.md`
  - `docs/bugs/discord_scheduler_empty_reply_send_failed.md`
  - `docs/bugs/scheduler_heartbeat_unknown_status_silent_skip.md`
  - `docs/bugs/scheduler_heartbeat_minimax_http_transport_failure_no_retry.md`

## Goal

收口活跃队列中非 P1 且状态为 `Fixing` 的缺陷，优先修复共享根因，避免同类问题在 Feishu / Discord / heartbeat / 金融播报链路间反复回归。

## Scope

- 已将 agent session 的空成功重试耗尽路径改为 `success=false + error`，使 Discord scheduler 与 Feishu scheduler 共享失败态，不再把通用 fallback 记为完成。
- 已在共享净化层剥离独立 compact marker，并清洗失败 partial stream，降低 compact fallback/进度轨迹进入用户可见文本的概率。
- 已确认 OpenAI-compatible provider 对 `error sending request`、连接重置、超时等传输错误执行一次短重试，覆盖 MiniMax heartbeat 调用的主要传输失败形态。
- 已补 heartbeat `PlainTextSuppressed` 执行层失败化，避免长篇自然语言坏态继续静默落成 `noop + skipped_noop`。
- 已补金融 prompt 的原油与大宗商品归因约束，要求地缘/供给/库存等原因归因必须有本轮来源、发布时间与可追溯事实；无法核验时只报价格并明确暂不归因。

## Validation

- `cargo test -p hone-channels scheduler::tests`
- `cargo test -p hone-channels prompt::tests`
- `cargo test -p hone-channels sanitize_user_visible_output`
- `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries`
- `cargo test -p hone-feishu failed_reply_text`
- `cargo test -p hone-feishu`
- `cargo test -p hone-channels`
- 待真实 scheduler / heartbeat 窗口继续复核是否还有线上样本复现。

## Documentation Sync

- 同步更新 `docs/bugs/README.md` 与相关 bug 文档的修复进展。
- 由于缺陷仍需真实窗口复核，本计划保持 `in_progress`，暂不归档。

## Risks / Open Questions

- prompt 约束只能降低油价归因幻觉概率，仍需生产窗口验证模型是否遵守。
- heartbeat `JsonEmptyStatus` 仍保留为兼容 `{}` noop，若后续证明其也主要来自坏态，需要再单独提升为失败或增加更细粒度判定。
- MiniMax provider 级重试已存在，但真实网络/上游抖动是否足够被一次重试吸收仍需继续观察。
