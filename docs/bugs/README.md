# Bugs Navigation

最后更新：2026-04-18 23:02 CST

这个文件是 `docs/bugs/` 的导航页，也是后续 agent / 人工协作时优先查看的缺陷台账入口。

## 使用约定

- 开始修 bug 前，先看“活跃待修复”表，再进入对应缺陷文档核对证据、链路和代码位置。
- 新增缺陷文档、更新严重等级、切换状态、确认修复、补充修复提交时，必须在同一次改动里同步更新本页。
- 修复完成后，除了更新单个 bug 文档的状态，也必须同步更新本页的“状态”和“修复情况”列。
- `bug` 自动化负责发现/更新缺陷，并维护本页导航；`bug-2` 自动化负责从本页活跃缺陷中选择修复对象，并在修复后回写本页。
- 新缺陷默认使用标准状态：`New`、`Approved`、`Fixing`、`Fixed`、`Closed`。历史文档若仍保留旧写法，可先在本页做归一化摘要，不必为了统一格式单独重写全文。

## 当前概览

- 活跃待修复：16
- 已修复 / 已关闭：30
- 历史分析 / 部分止血：2
- 当前活跃队列中没有 `P0`；最高待修优先级为 `P1`

## 活跃待修复

| Bug | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| Feishu 直聊 Answer 阶段再次出现空回复伪成功，`reply.chars=0` 仍被记成功并发送空分段 | P1 | Fixing | 2026-04-17 已补 `AgentSession` 的“净化后为空”成功收口，并补 Feishu 发送回退；`cargo test -p hone-channels`、`cargo test -p hone-feishu` 已通过，待真实直聊样本复核 | [feishu_direct_empty_reply_false_success.md](./feishu_direct_empty_reply_false_success.md) |
| Feishu 直聊任务治理 / 定时汇总请求在搜索阶段耗尽迭代后整轮无回复 | P1 | Fixing | 2026-04-18 13:12 同一 Feishu 直聊链路又在 8 次 `data_fetch` 后触顶；这次不再静默，但直接把 `已达最大迭代次数 8` 落成 assistant 文本，说明失败收口从“无回复”变成“原始内部错误外泄” | [feishu_direct_cron_job_iteration_exhaustion_no_reply.md](./feishu_direct_cron_job_iteration_exhaustion_no_reply.md) |
| Feishu 直达定时任务已生成最终播报，但发送阶段持续返回 `HTTP 400 Bad Request` 导致用户收不到提醒 | P1 | Fixing | 2026-04-17 21:32 `Oil_Price_Monitor_Premarket` 在最新真实窗口仍落成 `completed + send_failed`；说明 10:40 的 fallback 修补尚未收口到生产链路 | [feishu_scheduler_send_failed_http_400_after_generation.md](./feishu_scheduler_send_failed_http_400_after_generation.md) |
| Feishu 直聊在工具尚未跑完时提前把过渡句当成最终答复发送，组合评估请求只收到半成品回复 | P3 | New | 2026-04-16 16:00 真实会话复现；`session.persist_assistant/done` 后仍继续启动 `hone/web_search`，但用户侧只收到 55 字过渡句 | [feishu_direct_partial_reply_before_tool_completion.md](./feishu_direct_partial_reply_before_tool_completion.md) |
| Feishu 直聊把歧义股票简称 `lite` 直接猜成 Litecoin，未先澄清实体 | P3 | New | 2026-04-17 07:48 真实会话复现；用户说“分析目前lite价值”后系统直接输出 Litecoin 分析，需用户二次纠正为 `LITE Lumentum` | [feishu_ambiguous_lite_entity_guessed_as_litecoin.md](./feishu_ambiguous_lite_entity_guessed_as_litecoin.md) |
| Feishu 直聊沿用旧证券上下文，用户问 `DRAM` 却被整轮答成 `SNDK` | P3 | New | 2026-04-17 14:53 真实会话复现；当前 user turn 是“美股DRAM详细分析”，但 search 从首个工具调用起就锁定 `SNDK`，最终整轮答成 SanDisk 个股分析 | [feishu_direct_stale_symbol_context_hijacks_new_query.md](./feishu_direct_stale_symbol_context_hijacks_new_query.md) |
| 深度分析链路持续访问不存在的 `company_profiles` 相对路径，长期画像记忆被静默跳过 | P3 | New | 2026-04-18 14:46 `rklb，tem分析下` 真实直聊仍连续两次命中 `company_profiles` 不存在；主链路虽成功返回，但画像记忆继续被静默跳过 | [company_profiles_relative_path_misses_actor_sandbox.md](./company_profiles_relative_path_misses_actor_sandbox.md) |
| Feishu 直聊已拿到行情工具结果，但 Answer 仍谎报链路阻断并退化成空泛建议 | P3 | New | 2026-04-18 08:32 的创新药日报再次复现：4 次 `hone_data_fetch` 全部成功后，最终正文仍声称“港股与A股数据底层链路暂时阻断” | [feishu_direct_quote_tool_result_ignored.md](./feishu_direct_quote_tool_result_ignored.md) |
| Feishu 定时汇总已送达但未执行最新资讯检索，静默退化为非实时摘要 | P3 | New | 2026-04-18 12:00 的 `每日公司资讯与分析总结` 已送达，但 search/answer 全程 `tool_calls=0`，正文还直接承认“未完成最新实时接口校验” | [feishu_scheduler_daily_company_digest_skips_realtime_research.md](./feishu_scheduler_daily_company_digest_skips_realtime_research.md) |
| Feishu 直聊自动 compact 后仍无法稳定完成新话题回答，同一旧会话会在成功与 fallback 间抖动 | P2 | New | 2026-04-18 22:58 同一会话先后答出 `CAI/TEM`、`CRWV/NBIS`，但切到 `Google` 财报预判后又在 compact 重试后回落成统一 fallback | [feishu_direct_compact_retry_still_cannot_answer_new_topic.md](./feishu_direct_compact_retry_still_cannot_answer_new_topic.md) |
| MiniMax 搜索阶段 HTTP 发送失败后缺少自动重试与降级，用户仅收到通用失败提示 | P2 | Fixing | 2026-04-18 当前工作区已出现 provider 级重试补丁与测试草案，但修复尚未以已提交代码进入仓库主线，也未完成最新真实样本复核 | [minimax_search_http_transport_failure_no_retry.md](./minimax_search_http_transport_failure_no_retry.md) |
| Heartbeat 定时任务结构化状态退化后被静默跳过，监控提醒可能长期失效 | P2 | New | 2026-04-18 23:01 最新窗口仍落成 `RKLB异动监控` 未知状态失败；同窗其它任务虽记成 `noop`，但原始输出依旧以 `<think>` 开头，说明结构化契约仍未恢复 | [scheduler_heartbeat_unknown_status_silent_skip.md](./scheduler_heartbeat_unknown_status_silent_skip.md) |
| Heartbeat 已触发提醒偶发向用户投递原始 JSON 载荷 | P3 | New | 2026-04-18 10:31 的 `TEM大事件心跳监控` 已送达成功，但 `response_preview` 与 `deliver_preview` 都直接等于 `{\"trigger\":...}`；11:01 同任务又恢复自然语言 | [scheduler_heartbeat_trigger_json_payload_leak.md](./scheduler_heartbeat_trigger_json_payload_leak.md) |
| Heartbeat 定时任务命中 MiniMax HTTP 发送失败后缺少自动重试与降级，提醒整轮失败 | P2 | Fixing | 2026-04-18 当前工作区已出现共享 provider 重试补丁与测试草案，但尚未以已提交代码进入仓库主线，也未完成 heartbeat 真实样本复核 | [scheduler_heartbeat_minimax_http_transport_failure_no_retry.md](./scheduler_heartbeat_minimax_http_transport_failure_no_retry.md) |
| Heartbeat 监控任务触发 `context window exceeds limit` 后缺少恢复，故障会在不同任务间漂移复现 | P2 | New | 2026-04-16 20:01-20:31 最新窗口中 `RKLB_动态监控` 连续两轮超窗，`TEM_动态监控` 同轮失败后 30 分钟内又恢复，抖动仍在持续 | [scheduler_heartbeat_context_window_limit_no_recovery.md](./scheduler_heartbeat_context_window_limit_no_recovery.md) |
| Feishu 直聊询问 skill 时误报“没有该 skill”，并把内部约束直接当答案返回 | P3 | New | 2026-04-18 21:06 `hone_discover_skills` 已执行成功，但最终答案仍否认存在相关 skill，并直接外泄“底层系统纪律/FOMO 禁令”等内部口径 | [feishu_direct_skill_query_internal_policy_leak.md](./feishu_direct_skill_query_internal_policy_leak.md) |

## 已修复 / 已关闭

| Bug | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| 会话压缩摘要仍以 `Compact Summary` 回灌为 `role=user`，导致 scheduler 任务串入上一轮待办与结论 | P1 | Fixed | 2026-04-17 已把 compact summary 迁出普通用户恢复链路：改存 `role=system`、restore 跳过、prompt 统一改读 `session.summary`，并通过 `hone-channels` 全量测试 | [session_compact_summary_report_hallucination.md](./session_compact_summary_report_hallucination.md) |
| Feishu 用户达到当日对话额度上限后仍只收到“稍后再试”，且最新 user turn 不落库 | P1 | Fixed | 2026-04-17 已让 quota 拒绝直接返回用户态额度文案，并在拒绝前补最小 user-turn 落库；20:00 真实会话已再次返回“已达到今日对话上限（12/12）” | [feishu_conversation_quota_masked_as_generic_failure.md](./feishu_conversation_quota_masked_as_generic_failure.md) |
| Release app / 渠道进程仍可被 legacy `config_runtime.yaml` 驱动，导致 runner 改完后 live 服务不立即生效 | P1 | Fixed | 2026-04-16 已让 desktop 忽略 legacy override，并更新 release runbook 到 canonical/effective config 启动方式 | [desktop_release_runner_legacy_config_source.md](./desktop_release_runner_legacy_config_source.md) |
| Desktop Agent 设置页缺少 `codex_acp` runner 入口，实际已切到 Codex ACP 时仍无法一致展示 | P2 | Fixed | 2026-04-16 已补齐 settings/start 两处 runner 可见入口与检测提示，UI 与 live config 重新对齐 | [desktop_codex_acp_runner_ui_gap.md](./desktop_codex_acp_runner_ui_gap.md) |
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
| Feishu 图片附件会向用户发送内部 skill transcript，并夹带未清洗的中间协议 | P1 | Fixed | 2026-04-16 已让成功持久化统一只写最终可见文本与 tool-call metadata，不再把 runner `context_messages` 原样落库成 transcript | [feishu_attachment_internal_transcript_leak.md](./feishu_attachment_internal_transcript_leak.md) |
| Feishu 直聊在 Answer 阶段触发 idle timeout 后整轮无回复 | P1 | Fixed | `02d01d2` 已把失败分支接入共享超时友好文案；2026-04-16 再补 handler 级回归测试，确认 timeout 不会再静默结束 | [feishu_direct_answer_idle_timeout_no_reply.md](./feishu_direct_answer_idle_timeout_no_reply.md) |
| Feishu 直聊消息在已有同 session 任务处理中时仍先发送 placeholder，但未真正进入 agent 主链路 | P1 | Fixed | 2026-04-18 19:01 最新真实 busy 样本已只发送 `direct.busy` 并跳过 placeholder，live 复核通过 | [feishu_direct_placeholder_without_agent_run.md](./feishu_direct_placeholder_without_agent_run.md) |
| Release runtime 缺少稳定 supervisor 时会丢失固定 `8077` 端口或整组进程退出，导致 Desktop 周期性掉线 | P1 | Fixed | `ea5229b` 已为 release helper 收口到 `.app` 启动形态、统一 `honeclaw/target` cache、并让 `launch.sh` 持续写入 `data/runtime/current.pid` 供重启链路可靠接管 | [desktop_release_runtime_supervision_gap.md](./desktop_release_runtime_supervision_gap.md) |
| OpenAI-compatible 搜索阶段出现 tool-call 协议错位，`invalid params` 失败被统一收口成通用失败提示 | P1 | Fixed | 2026-04-16 已补齐搜索上下文清洗：同时移除历史 `tool` 与残留 assistant `tool_calls`，定向回归测试与 desktop release build 已通过 | [openai_compatible_tool_call_protocol_mismatch_invalid_params.md](./openai_compatible_tool_call_protocol_mismatch_invalid_params.md) |

## 历史分析 / 部分止血

| 主题 | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| opencode ACP `session/prompt timeout (300s)` 问题分析 | - | Fixed | 2026-04-13 已收口到 ACP runners 公共等待逻辑 | [opencode_acp_prompt_timeout.md](./opencode_acp_prompt_timeout.md) |
| opencode ACP 相关的 Prompt 泄露与缓存失效问题分析 | - | Partial | Prompt Echo 已止血；完整多轮 message 级缓存复用仍未实现 | [opencode_prompt_issues.md](./opencode_prompt_issues.md) |
