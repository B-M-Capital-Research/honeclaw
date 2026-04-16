# Bugs Navigation

最后更新：2026-04-16 14:06 CST

这个文件是 `docs/bugs/` 的导航页，也是后续 agent / 人工协作时优先查看的缺陷台账入口。

## 使用约定

- 开始修 bug 前，先看“活跃待修复”表，再进入对应缺陷文档核对证据、链路和代码位置。
- 新增缺陷文档、更新严重等级、切换状态、确认修复、补充修复提交时，必须在同一次改动里同步更新本页。
- 修复完成后，除了更新单个 bug 文档的状态，也必须同步更新本页的“状态”和“修复情况”列。
- `bug` 自动化负责发现/更新缺陷，并维护本页导航；`bug-2` 自动化负责从本页活跃缺陷中选择修复对象，并在修复后回写本页。
- 新缺陷默认使用标准状态：`New`、`Approved`、`Fixing`、`Fixed`、`Closed`。历史文档若仍保留旧写法，可先在本页做归一化摘要，不必为了统一格式单独重写全文。

## 当前概览

- 活跃待修复：5
- 已修复 / 已关闭：26
- 历史分析 / 部分止血：2
- 当前活跃队列中没有 `P0`；最高待修优先级为 `P1`

## 活跃待修复

| Bug | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| Feishu 直聊 Answer 阶段再次出现空回复伪成功，`reply.chars=0` 仍被记成功并发送空分段 | P1 | New | 2026-04-16 12:12 与 12:22 两条真实直聊会话回归复现；原“已修复”结论已撤回 | [feishu_direct_empty_reply_false_success.md](./feishu_direct_empty_reply_false_success.md) |
| Feishu 直聊任务配置请求在搜索阶段反复调用 `cron_job` 后耗尽迭代并整轮无回复 | P1 | New | 2026-04-16 12:06 新发现；待为迭代耗尽补用户态兜底与循环收敛 | [feishu_direct_cron_job_iteration_exhaustion_no_reply.md](./feishu_direct_cron_job_iteration_exhaustion_no_reply.md) |
| Feishu 直聊消息再次出现 placeholder 假启动，最新“喂喂喂”与“1”两条都未进入主链路 | P1 | New | 2026-04-16 13:54、13:56、13:58 四次继续复现；原“已修复”结论已撤回，当前仅确认入口 busy 止血不充分 | [feishu_direct_placeholder_without_agent_run.md](./feishu_direct_placeholder_without_agent_run.md) |
| MiniMax 搜索阶段 HTTP 发送失败后缺少自动重试与降级，用户仅收到通用失败提示 | P2 | New | 2026-04-16 13:08 Feishu 直聊 `rklb要不要加` 命中；52 秒后同句重试成功，说明当前缺少对传输抖动的吸震 | [minimax_search_http_transport_failure_no_retry.md](./minimax_search_http_transport_failure_no_retry.md) |
| Heartbeat 定时任务遇到 `JsonUnknownStatus` 时静默跳过，监控提醒可能长期失效 | P2 | New | 2026-04-16 11:30 仍复现；README 已按 bug 文档与日志纠正回活跃队列 | [scheduler_heartbeat_unknown_status_silent_skip.md](./scheduler_heartbeat_unknown_status_silent_skip.md) |

## 已修复 / 已关闭

| Bug | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| 飞书渠道消息发错位（跨用户投递） | P0 | Fixed | 2026-03-25 已修复 | [feishu_message_misrouting.md](./feishu_message_misrouting.md) |
| 飞书定时任务重复投递 | P1 | Fixed | 2026-03-25 已修复 | [feishu_scheduler_duplicate_delivery.md](./feishu_scheduler_duplicate_delivery.md) |
| 多代理内部思考与工具协议文本泄漏到用户回复 | P1 | Fixed | `12a5352` 已修复并补齐输出净化 | [multi_agent_internal_output_leak.md](./multi_agent_internal_output_leak.md) |
| 会话上下文溢出未自动恢复且向用户泄露底层报错 | P1 | Fixed | `1a65ce0` 已修复自动 compact 重试与友好报错 | [context_overflow_recovery_gap.md](./context_overflow_recovery_gap.md) |
| Desktop 启动时未完整迁移 legacy runtime 用户配置 | P1 | Fixed | `dfd8a01` 与 `e802582` 已修复主缺陷 | [desktop_legacy_config_migration_gap.md](./desktop_legacy_config_migration_gap.md) |
| Desktop legacy runtime 迁移遗漏 OpenRouter key 池，升级后默认对话链路可能直接失效 | P1 | Fixed | `5404624` 已补齐 key 池迁移 | [desktop_openrouter_key_pool_migration_gap.md](./desktop_openrouter_key_pool_migration_gap.md) |
| Desktop runtime logs 接口曾因坏日志数据或 runtime overlay 漏读而失效，日志面板无法稳定恢复最近运行痕迹 | P1 | Fixed | `d031f16` 已修复日志恢复与 overlay 读取 | [desktop_runtime_logs_recovery_gap.md](./desktop_runtime_logs_recovery_gap.md) |
| Desktop legacy runtime 会整块覆盖 canonical `agent.opencode` 配置，破坏本机 OpenCode 继承语义 | P1 | Fixed | 2026-04-16 已改成字段级补迁；空 `api_key` 的本机 OpenCode 继承语义已保留，并补回归测试 | [desktop_opencode_legacy_override_gap.md](./desktop_opencode_legacy_override_gap.md) |
| Desktop Agent 设置会把 `multi-agent.answer` 反写到 `agent.opencode`，导致不同 runner 的独立配置互相覆盖 | P1 | Fixed | 2026-04-16 已停止 `multi-agent.answer` 反写 `agent.opencode`，并补保存链路回归测试 | [desktop_runner_settings_cross_runner_overwrite.md](./desktop_runner_settings_cross_runner_overwrite.md) |
| Desktop 设置页多入口保存共用同一份配置文件但缺少串行写保护，可能造成 runner 配置被并发保存静默覆盖 | P1 | Fixed | 2026-04-16 已为 desktop 配置写入链路补共享 `config_write_lock`，并补并发保存回归测试 | [desktop_runner_settings_write_race.md](./desktop_runner_settings_write_race.md) |
| Desktop 设置页重复点击 runner 会触发重入保存与 bundled backend 重启，导致切换过程卡死或表现为“点一下就崩” | P1 | Fixed | 2026-04-16 已为 runner 卡片点击补前端重入短路与失败回滚，并为相同 agent payload 增加 sidecar 幂等跳过 | [desktop_runner_switch_reentrant_restart_gap.md](./desktop_runner_switch_reentrant_restart_gap.md) |
| Desktop 设置页切换 runner 后可能显示已切换，但 bundled runtime 重启失败会被静默吞掉，实际仍跑旧 runner 或未完成切换 | P1 | Fixed | 2026-04-16 已让 agent settings 保存回传 `backendStatus` 与重启结论；runtime 未生效时前端会明确报错而非伪装成功 | [desktop_runner_switch_false_success_gap.md](./desktop_runner_switch_false_success_gap.md) |
| Multi-Agent Search Agent 在 Desktop 设置页显示可继承 auxiliary key，但真实运行时不使用该 fallback，导致看似已配置却直接失败 | P1 | Fixed | 2026-04-16 已让 multi-agent 运行时对齐 auxiliary key fallback 语义，并补回归测试锁住显式 search key 优先级 | [multi_agent_search_key_fallback_mismatch.md](./multi_agent_search_key_fallback_mismatch.md) |
| Multi-Agent Answer Agent 在设置页允许 `maxToolCalls=0`，但运行时强制提升为至少 1，用户无法真正禁用补充工具调用 | P1 | Fixed | 2026-04-16 已去掉运行时 `.max(1)` 强制提升，并让 answer-stage handoff 文本与 `0` 配置语义保持一致 | [multi_agent_answer_max_tool_calls_zero_ignored.md](./multi_agent_answer_max_tool_calls_zero_ignored.md) |
| 定时任务达到上限后，Agent 未经用户确认就批量删除已有任务 | P1 | Fixed | 2026-04-16 已为 `cron_job remove` 增加显式确认屏障；未确认前只返回候选任务与确认指引，不再直接删除 | [scheduler_task_limit_auto_cleanup_without_confirmation.md](./scheduler_task_limit_auto_cleanup_without_confirmation.md) |
| 定时任务链路绕过统一输出净化，向用户投递内部思考与未清洗富文本 | P1 | Fixed | 2026-04-16 已为 scheduler 公共出站补统一可见文本净化，并为 Telegram scheduler 补 HTML 公共清洗 | [scheduled_output_sanitization_gap.md](./scheduled_output_sanitization_gap.md) |
| 渠道失败分支会把原始 LLM/provider 报错直接发给用户 | P1 | Fixed | 2026-04-16 已新增共享用户态错误净化层，并接入 outbound、scheduler、Feishu、Discord slash 与 iMessage 失败分支 | [channel_raw_llm_error_exposure.md](./channel_raw_llm_error_exposure.md) |
| 成功会话仍把原始 multi-agent transcript 落库到 assistant 历史，污染后续上下文 | P2 | Fixed | 2026-04-16 已让 assistant 持久化只写 `final` 文本，并把工具调用改存到 metadata，避免污染会话索引与 sqlite runtime 预览 | [session_persist_assistant_transcript_pollution.md](./session_persist_assistant_transcript_pollution.md) |
| Feishu 定时任务在 Answer 阶段返回空回复后，调度台账仍记为 `completed + sent` | P1 | Fixed | 2026-04-16 已通过共享空成功判定修复收口，scheduler 不再发送或记录零字节正文 | [feishu_scheduler_empty_reply_false_success.md](./feishu_scheduler_empty_reply_false_success.md) |
| Discord 定时任务在 Answer 阶段返回空回复时被记为成功执行，但最终未向用户送达 | P2 | Fixed | 2026-04-16 已通过共享空成功判定修复收口，不再因为只剩搜索工具调用而把空 answer 视为成功 | [discord_scheduler_empty_reply_send_failed.md](./discord_scheduler_empty_reply_send_failed.md) |
| Feishu 定时任务目标校验长期失败，任务生成内容后仍无法送达 | P1 | Fixed | 2026-04-16 已让 direct scheduler 优先使用绑定 actor 的 `open_id`，并收紧 mobile 识别避免把 `open_id` 误判成手机号 | [feishu_scheduler_target_resolution_failed.md](./feishu_scheduler_target_resolution_failed.md) |
| 会话压缩摘要会把最后一个新问题误写成完整“用户报告”并以 `Compact Summary` 回灌，正式回答因此引用不存在的报告与伪造价格假设 | P1 | Fixed | 2026-04-16 已让 compactor 只总结将被裁掉的旧消息，并收紧压缩提示词，避免把最新未回答问题提前写成伪摘要 | [session_compact_summary_report_hallucination.md](./session_compact_summary_report_hallucination.md) |
| Feishu 图片附件会向用户发送内部 skill transcript，并夹带未清洗的中间协议 | P1 | Fixed | 2026-04-16 已让成功持久化统一只写最终可见文本与 tool-call metadata，不再把 runner `context_messages` 原样落库成 transcript | [feishu_attachment_internal_transcript_leak.md](./feishu_attachment_internal_transcript_leak.md) |
| Feishu 直聊在 Answer 阶段触发 idle timeout 后整轮无回复 | P1 | Fixed | `02d01d2` 已把失败分支接入共享超时友好文案；2026-04-16 再补 handler 级回归测试，确认 timeout 不会再静默结束 | [feishu_direct_answer_idle_timeout_no_reply.md](./feishu_direct_answer_idle_timeout_no_reply.md) |
| Release runtime 缺少稳定 supervisor 时会丢失固定 `8077` 端口或整组进程退出，导致 Desktop 周期性掉线 | P1 | Fixed | `ea5229b` 已为 release helper 收口到 `.app` 启动形态、统一 `honeclaw/target` cache、并让 `launch.sh` 持续写入 `data/runtime/current.pid` 供重启链路可靠接管 | [desktop_release_runtime_supervision_gap.md](./desktop_release_runtime_supervision_gap.md) |
| OpenAI-compatible 搜索阶段出现 tool-call 协议错位，`invalid params` 失败被统一收口成通用失败提示 | P1 | Fixed | 2026-04-16 已补齐搜索上下文清洗：同时移除历史 `tool` 与残留 assistant `tool_calls`，定向回归测试与 desktop release build 已通过 | [openai_compatible_tool_call_protocol_mismatch_invalid_params.md](./openai_compatible_tool_call_protocol_mismatch_invalid_params.md) |

## 历史分析 / 部分止血

| 主题 | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| opencode ACP `session/prompt timeout (300s)` 问题分析 | - | Fixed | 2026-04-13 已收口到 ACP runners 公共等待逻辑 | [opencode_acp_prompt_timeout.md](./opencode_acp_prompt_timeout.md) |
| opencode ACP 相关的 Prompt 泄露与缓存失效问题分析 | - | Partial | Prompt Echo 已止血；完整多轮 message 级缓存复用仍未实现 | [opencode_prompt_issues.md](./opencode_prompt_issues.md) |
