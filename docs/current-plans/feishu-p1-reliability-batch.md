# Plan

- title: Feishu P1 直聊与定时任务可靠性修复批次
- status: in_progress
- created_at: 2026-04-17 10:05 CST
- updated_at: 2026-04-29 18:02 CST
- owner: Codex
- related_files:
  - `bins/hone-feishu/src/handler.rs`
  - `bins/hone-feishu/src/outbound.rs`
  - `bins/hone-feishu/src/client.rs`
  - `bins/hone-feishu/src/scheduler.rs`
  - `crates/hone-channels/src/agent_session.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `docs/bugs/README.md`
  - `docs/bugs/feishu_direct_empty_reply_false_success.md`
  - `docs/bugs/feishu_direct_cron_job_iteration_exhaustion_no_reply.md`
  - `docs/bugs/feishu_direct_placeholder_without_agent_run.md`
  - `docs/bugs/feishu_scheduler_send_failed_http_400_after_generation.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/bugs/README.md`
  - `docs/handoffs/2026-04-16-feishu-direct-busy-placeholder-gap.md`

## Goal

收口当前活跃的 Feishu `P1` 缺陷，优先保证直聊与 scheduler 在“产出为空 / 失败兜底 / 多段发送”场景下至少能稳定给到用户可见结果，不再出现 placeholder 后静默、空回复伪成功或生成成功但投递失败。

## Scope

- 修复 Feishu 直聊链路把空成功或被净化后的空正文继续当成成功完成的问题
- 修复 Feishu 直聊在 search 迭代耗尽、placeholder 更新失败或 handler panic 时仍可能整轮无回复的问题
- 修复 Feishu scheduler 直达消息在多段发送/回复链路上持续 `HTTP 400 Bad Request` 的问题
- 为 Feishu 发送失败补更具体的日志与回退路径，便于后续继续定位未完全覆盖的场景

## Validation

- `cargo test -p hone-channels`
- `cargo test -p hone-feishu`
- 定向回归：
  - Feishu 直聊失败分支至少会落成可见 assistant 文本，而不是只剩 placeholder
  - 空成功 / 内部文本被净化为空后不会再持久化空 assistant
  - scheduler 多段发送在 direct `open_id` 目标上不再依赖脆弱的 reply 链路

## Current Progress

- 已完成：
  - `AgentSession` 对“净化后为空”的成功结果补 fallback，避免空 assistant 持久化
  - Feishu `update_message` / `reply_message` 返回 `HTTP 400` 时改走 standalone send 回退
  - direct scheduler 无 placeholder 的多段发送不再默认使用 reply 链路
  - Feishu handler 增加 join/panic 兜底与 `handler.session_run` 边界日志
  - Feishu client 为 `tenant_access_token/internal`、send/reply/update message 补 3 次短重试，吸收传输错误、`429` 与 `5xx`
  - `hone-channels` scheduler 将 `EMPTY_SUCCESS_FALLBACK_MESSAGE` 识别为失败信号，避免通用 fallback 继续记为 `completed + sent`
  - Feishu scheduler 触发入口立即写入 `running + pending` 台账，避免 agent run 卡住时 `cron_job_runs` 完全缺失
  - `empty_success_exhausted` 改为 `success=false + fallback error`，直聊和 scheduler 都不再把空回复 fallback 记成正常完成
  - Feishu 失败 partial stream 会丢弃工具/进度轨迹，idle timeout/state migration 后只给用户产品化失败文案
  - 共享输出净化层重新剥离独立 `Context compacted` / `Conversation compacted` marker 行，保留后续真实正文
  - multi-agent search guidance 现在显式要求“我的定时任务 / 提醒 / 更新任务”等请求优先调用 `cron_job`，避免先误入 `data_fetch` / `web_search`
  - multi-agent 现在允许 `cron_job` / `portfolio` 这类可信本地状态结果在搜索阶段直接短路返回，避免已经拿到本地答案后再进入容易产出空/过渡句的 answer 阶段
  - multi-agent search guidance 对 `这个` / `那个` / `上一条` 这类短澄清补充了“直接答或只问一个澄清问题”的约束，减少 `planning_sentence_suppressed`
- 已验证：
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-channels`
  - `cargo test -p hone-channels scheduler::tests`
  - `cargo test -p hone-feishu failed_reply_text`
  - `cargo test -p hone-channels sanitize_user_visible_output`
  - `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries`
  - `cargo test -p hone-channels runners::multi_agent::tests -- --nocapture`
  - `cargo check -p hone-channels`
- 待验证：
  - 下一条真实 Feishu 直聊“我的定时任务 / 我现在有哪些定时任务”样本
  - 下一条真实 Feishu 直聊短澄清（如“这个”）样本
  - 下一轮真实 Feishu scheduler 直达任务送达窗口
  - 下一轮真实 `tenant_access_token/internal` 或 `im/v1/messages` 传输抖动是否被短重试吸收
  - 下一条真实 compact 后回复是否还会以 `Context compacted` 开头
  - 下一条长耗时 scheduler 是否只停在 `running/pending`，以及是否需要继续补 watchdog 终结器

## Documentation Sync

- 更新 `docs/current-plan.md`
- 根据修复结果更新 `docs/bugs/README.md`
- 回写四个活跃 P1 bug 文档的状态、修复情况与验证结论
- 若本轮结束后任务退出活跃态，再补 handoff 或归档；未完成则保留本计划页继续推进

## Risks / Open Questions

- 部分活跃症状可能共享同一发送链路根因，也可能同时存在 handler panic、placeholder update 失败和 multi-agent 空结果收口不完整三类问题
- 若 scheduler 的 `HTTP 400` 来自 Feishu 平台对特定 payload 的校验而非 reply 链路，仅靠回退到 standalone send 可能只能部分止血
- 若 `cron_job` 迭代耗尽来自 prompt/tool 选择策略本身，本轮更现实的目标是“失败时给出用户态结果”，而不是一次性完全消除循环
