# Bugs Navigation

最后更新：2026-04-23 14:05 CST

这个文件是 `docs/bugs/` 的导航页，也是后续 agent / 人工协作时优先查看的缺陷台账入口。

## 使用约定

- 开始修 bug 前，先看"活跃待修复"表，再进入对应缺陷文档核对证据、链路和代码位置。
- 新增缺陷文档、更新严重等级、切换状态、确认修复、补充修复提交时，必须在同一次改动里同步更新本页。
- 修复完成后，除了更新单个 bug 文档的状态，也必须同步更新本页的"状态"和"修复情况"列。
- `bug` 自动化负责发现/更新缺陷，并维护本页导航；`bug-2` 自动化负责从本页活跃缺陷中选择修复对象，并在修复后回写本页。
- 新缺陷默认使用标准状态：`New`、`Approved`、`Fixing`、`Fixed`、`Closed`。历史文档若仍保留旧写法，可先在本页做归一化摘要，不必为了统一格式单独重写全文。

## 当前概览

- 活跃待修复：26
- 已修复 / 已关闭：44
- 历史分析 / 部分止血：4
- 当前活跃队列中没有 `P0`；最高待修优先级为 `P1`

## 活跃待修复

| Bug | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| Feishu 直聊 Answer 阶段持续出现空/无效回复，真实任务被 fallback 遮蔽为“未成功产出完整回复” | P1 | Fixing | 2026-04-23 10:36 用户要求建腾讯控股 ADR 画像，日志显示画像文件已写入，但最终 43 字过渡句被 `transitional planning sentence detected` 判空，用户只收到通用失败 fallback；零字节外发已止血，根因仍活跃 | [feishu_direct_empty_reply_false_success.md](./feishu_direct_empty_reply_false_success.md) |
| Feishu 直达定时任务已生成最终播报，但发送阶段持续返回 `HTTP 400 Bad Request` 导致用户收不到提醒 | P1 | Fixing | 2026-04-21 21:02 `OWALERT_PreMarket` 再次落成 `completed + send_failed`，错误体仍是 `code=99992361 / open_id cross app`；正文已落库但用户侧未送达 | [feishu_scheduler_send_failed_http_400_after_generation.md](./feishu_scheduler_send_failed_http_400_after_generation.md) |
| Feishu scheduler 发送前统一卡在 `tenant_access_token` 请求失败，生成完成的日报与 heartbeat 告警都无法送达 | P1 | New | 2026-04-21 08:04-09:04 至少 11 条 Feishu 定时任务跨多个目标统一落成 `send_failed`；11:22 用户明确反馈“今天你的指令工作怎么没发”，对应 08:34-08:49 多条早报/盘前任务仍卡死在 `tenant_access_token/internal` | [feishu_scheduler_tenant_access_token_request_failure.md](./feishu_scheduler_tenant_access_token_request_failure.md) |
| Feishu 出站 `send/update message` 请求传输失败，定时任务和直聊回复都已生成但无法送达 | P1 | New | 2026-04-21 15:37 直聊 `AI工业革命下一个爆发板块` 已生成 3561 字并落库，但 placeholder update 端点 `im/v1/messages/{message_id}` 传输失败；15:00 定时任务 `send message` 端点也仍失败 | [feishu_send_message_request_transport_failure.md](./feishu_send_message_request_transport_failure.md) |
| Feishu 直聊在 Answer 阶段触发 idle timeout / Codex state migration 错误后整轮无最终回复 | P1 | New | 2026-04-21 20:25 用户要求日报击球区补区间值，20:29 仅收到“处理超时”；日志仍是 `codex acp session/prompt idle timeout (180s)` + `state_5.sqlite migration 23 ... missing`，说明 15:14-15:32 的失败形态继续活跃 | [feishu_direct_answer_idle_timeout_no_reply.md](./feishu_direct_answer_idle_timeout_no_reply.md) |
| 渠道失败分支再次把底层 LLM/传输报错直接拼进用户回复 | P1 | New | 2026-04-23 巡检未找到已提交修复覆盖 Codex WebSocket/HTTPS 回退、`wss://chatgpt.com/backend-api/codex/responses`、`cf-ray`、`unexpected status 403` 等内部传输残留；不能维持 Fixed | [channel_raw_llm_error_exposure.md](./channel_raw_llm_error_exposure.md) |
| 会话压缩摘要曾以 `role=user` 的 `Compact Summary` 回灌真实 transcript，且压缩标记会进入最终可见文本 | P1 | Fixing | 2026-04-23 04:00 最新 auto compact 仍生成语义错误 summary：把 03:40 已回答的“如果有新增订单呢”标为“尚未回答的新问题”，说明 compact summary 可信边界仍未闭环 | [session_compact_summary_report_hallucination.md](./session_compact_summary_report_hallucination.md) |
| 原油定时播报把未核验地缘叙述当作油价事实送达用户 | P2 | New | 2026-04-23 12:00 `全天原油价格3小时播报` 再次 `completed + sent + delivered=1`；raw preview 承认没有精确实时价格，却仍把 WTI 约 99-101 美元、布伦特 101.91 美元和航运/伊朗谈判风险作为确定性播报送达 | [oil_price_scheduler_geopolitical_hallucination.md](./oil_price_scheduler_geopolitical_hallucination.md) |
| 深度分析链路持续访问不存在的 `company_profiles` 相对路径，长期画像记忆被静默跳过 | P3 | New | 2026-04-21 21:00 ACP 事件仍记录 `工具执行错误: 目录不存在: company_profiles`，且 assistant chunk 对用户解释“本地没有现成的 company_profiles/ 目录”，说明 2026-04-20 修复未覆盖当前生产路径 | [company_profiles_relative_path_misses_actor_sandbox.md](./company_profiles_relative_path_misses_actor_sandbox.md) |
| Feishu 直聊在工具尚未跑完时提前把过渡句或内部 todo 当成最终答复发送，且任务治理变更可能未生效 | P2 | New | 2026-04-23 13:27 用户要求“携程，价值分析”，日志显示本轮已调用 `data_fetch`/`web_search`/本地工具，但最终只把 96 字“已校验到 TCOM...”过程性片段记为 `success=true` 并发送，正式价值分析缺失；同根因仍活跃 | [feishu_direct_partial_reply_before_tool_completion.md](./feishu_direct_partial_reply_before_tool_completion.md) |
| Feishu 每日动态监控在“无新增催化应跳过”时仍照常推送长文 | P3 | New | 2026-04-23 00:03/00:04 `RKLB 每日动态监控` 与 `TEM 每日动态监控` 正文均写“按规则可跳过正式推送 / 不触发正式推送”，但台账仍是 `completed + sent + delivered=1`，说明 2026-04-20 的止血已回归失效 | [feishu_scheduler_daily_monitor_skip_rule_broken.md](./feishu_scheduler_daily_monitor_skip_rule_broken.md) |
| Heartbeat 定时任务结构化状态退化后被静默跳过，监控提醒可能长期失效 | P2 | New | 2026-04-23 12:00-13:00 三批 heartbeat 仍普遍 `starts_with_json=false`；13:00 `持仓重大事件心跳检测` 触发型输出也先吐 `<think>` 再由解析器提取并发送，12:30 ORCL 超窗虽已降级为 noop，但结构化契约未恢复 | [scheduler_heartbeat_unknown_status_silent_skip.md](./scheduler_heartbeat_unknown_status_silent_skip.md) |
| Web 直聊把投研过程句当成最终回复，用户需要二次追问才拿到正式答案 | P3 | New | 2026-04-22 21:19 Web 用户问“最近BABA值不值得加一些”，assistant final 只返回“下一步补财务和估值质量”；21:20 用户追问“不需要，直接告诉我吧”后才拿到正式判断，不影响投递链路因此定级 P3 | [web_direct_partial_reply_before_tool_completion.md](./web_direct_partial_reply_before_tool_completion.md) |
| Heartbeat 将日内高点/区间振幅误判为涨跌幅阈值并发送错误触发提醒 | P2 | New | 2026-04-23 06:31 `ASTS 重大异动心跳监控` 再次 `JsonTriggered + sent`：当前/收盘价相对昨收仅 `+5.81%`，raw preview 也先判低于 8%，最终仍用日内高点相对昨收 `+9.71%` 判定“盘中涨跌幅超8%”；同根因曾在 ORCL 上把高低点振幅误判为涨跌幅 | [scheduler_heartbeat_orcl_intraday_range_false_trigger.md](./scheduler_heartbeat_orcl_intraday_range_false_trigger.md) |
| Heartbeat 已触发事件在无新增增量时跨窗口重复提醒，同一催化会在半小时轮询里反复送达 | P3 | New | 2026-04-23 12:00 `持仓重大事件心跳检测` 又发送 ASTS/FCC/BlueBird 旧事件；`TEM大事件心跳监控` 同时把 AACR 4月17-22日会议与旧合作新闻重新包装成利好触发，说明去重/增量基线仍不稳定 | [scheduler_heartbeat_retrigger_duplicate_alerts.md](./scheduler_heartbeat_retrigger_duplicate_alerts.md) |
| Heartbeat 重大事件监控触发 `已达最大迭代次数 6` 后整轮跳过，用户收不到应发提醒 | P2 | New | 2026-04-23 01:00 `Monitor_Watchlist_11` 再次落成 `execution_failed + skipped_error`，`error=max_iterations_exceeded:6` 且 `delivered=0`；heartbeat 触顶仍无用户态降级 | [scheduler_heartbeat_iteration_exhaustion_skips_alert.md](./scheduler_heartbeat_iteration_exhaustion_skips_alert.md) |
| 一次性定时任务丢失绝对日期，提前执行并禁用原本未来提醒 | P2 | New | 2026-04-23 08:30 `ADTN财报后总结` 的 prompt 明确写“2026年5月5日早上执行”，但配置只保留 `hour=8/minute=30/repeat=once`，在 2026-04-23 被提前触发并置为 disabled | [scheduler_once_absolute_date_lost.md](./scheduler_once_absolute_date_lost.md) |
| Heartbeat 定时任务命中 MiniMax HTTP 发送失败后仍整轮失败，09:00 到 12:00 多个窗口大面积静默失效 | P2 | Fixing | 2026-04-21 19:30 `Monitor_Watchlist_11` 继续命中 `https://api.minimaxi.com/v1/chat/completions` 发送失败；20:00 同批主要漂移到 `JsonUnknownStatus`，传输吸震仍未稳定收口 | [scheduler_heartbeat_minimax_http_transport_failure_no_retry.md](./scheduler_heartbeat_minimax_http_transport_failure_no_retry.md) |
| Telegram update listener 持续不可用，近一个月没有新消息入库 | P2 | New | 2026-04-23 04:03 与 06:03 `GetUpdates` 仍连接中断；listener 只记录 error 并重试，缺少持久健康状态；最近 Telegram 会话仍停留在 2026-03-18 | [telegram_update_listener_connection_refused.md](./telegram_update_listener_connection_refused.md) |
| Event-engine price poller 单次 FMP quote 抓取失败 | P3 | New | 2026-04-22 12:03 quote 批量请求连接被关闭；后续 poller 恢复且 `fmp.quote` 近 24h 有记录，暂按单 tick 丢失跟踪 | [event_engine_price_poller_transient_fetch_failure.md](./event_engine_price_poller_transient_fetch_failure.md) |
| Event-engine high stock-news events lack sink delivery evidence | P2 | New | 2026-04-22 18:52 UTC 新增 `FLYYQ` high MarketWatch 事件仍无 `delivery_log` sink 行；同窗口也无 `sink delivered` 日志 | [event_engine_high_news_no_sink_delivery.md](./event_engine_high_news_no_sink_delivery.md) |
| Event-engine marks legal-ad style stock news as high severity | P2 | New | 2026-04-22 巡检确认 `class action` / `shareholder alert` 等律所模板在 24h high 事件中持续占比过高，可能消耗 high cap 并污染即时提醒 | [event_engine_legal_news_high_severity_noise.md](./event_engine_legal_news_high_severity_noise.md) |
| Event-engine news classifier 403 errors downgraded uncertain-source review | P2 | New | 2026-04-22 OpenRouter 403 / 反序列化失败让 uncertain-source 新闻 LLM 仲裁返回 `None`，重要新闻可能退回低优先级 digest 路径 | [event_engine_news_classifier_403_fallback.md](./event_engine_news_classifier_403_fallback.md) |
| Event-engine social/event-source pollers repeat decode failures | P3 | New | 2026-04-23 08:37/09:37 CST generic event-source poller 继续 2 次 JSON decode 失败；日志仍缺 poller 字段，最新窗口未见新 `telegram.watcherguru` 入库 | [event_engine_social_source_decode_failures.md](./event_engine_social_source_decode_failures.md) |
| Event-engine window convergence upgrade bursts crowd digest quality | P3 | New | 2026-04-23 08:22:42 CST 单秒再次出现 25 条 `Low→Medium` 窗口收敛提级，超过巡检阈值；poller 与 sink 正常，问题集中在路由降噪 | [event_engine_window_convergence_upgrade_burst.md](./event_engine_window_convergence_upgrade_burst.md) |
| Event-engine high macro events are stored but not routed | P2 | New | 2026-04-23 已确认是工程规则问题：先将 high macro 样本从 77 收敛到预计 15；即时路由因会提前推未来 7 天日历而暂不打开 | [event_engine_high_macro_events_unrouted.md](./event_engine_high_macro_events_unrouted.md) |

## 已修复 / 已关闭

| Bug | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| 公开面认证与限流安全审计发现多个高/中风险问题 | P1/P2 | Fixed | 2026-04-20 已修复公开登录限流维度、Secure Cookie 配置、workflow runner `validateCode`、邀请码熵和认证态闪烁问题 | [public_auth_security_audit_2026_04_20.md](./public_auth_security_audit_2026_04_20.md) |
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
| 成功会话仍把原始 multi-agent transcript 落库到 assistant 历史，污染后续上下文 | P2 | Fixed | 2026-04-16 已让 assistant 持久化只写 `final` 文本，并把工具调用改存到 metadata，避免污染会话索引与 sqlite runtime 预览 | [session_persist_assistant_transcript_pollution.md](./session_persist_assistant_transcript_pollution.md) |
| Feishu 定时任务在 Answer 阶段返回空回复后，调度台账仍记为 `completed + sent` | P1 | Fixed | 2026-04-16 已通过共享空成功判定修复收口，scheduler 不再发送或记录零字节正文 | [feishu_scheduler_empty_reply_false_success.md](./feishu_scheduler_empty_reply_false_success.md) |
| Discord 定时任务在 Answer 阶段返回空回复时被记为成功执行，但最终未向用户送达 | P2 | Fixed | 2026-04-16 已通过共享空成功判定修复收口，不再因为只剩搜索工具调用而把空 answer 视为成功 | [discord_scheduler_empty_reply_send_failed.md](./discord_scheduler_empty_reply_send_failed.md) |
| Feishu 定时任务目标校验长期失败，任务生成内容后仍无法送达 | P1 | Fixed | 2026-04-16 已让 direct scheduler 优先使用绑定 actor 的 `open_id`，并收紧 mobile 识别避免把 `open_id` 误判成手机号 | [feishu_scheduler_target_resolution_failed.md](./feishu_scheduler_target_resolution_failed.md) |
| Feishu 图片附件会向用户发送内部 skill transcript，并夹带未清洗的中间协议 | P1 | Fixed | 2026-04-16 已让成功持久化统一只写最终可见文本与 tool-call metadata，不再把 runner `context_messages` 原样落库成 transcript | [feishu_attachment_internal_transcript_leak.md](./feishu_attachment_internal_transcript_leak.md) |
| Feishu 直聊任务治理 / 定时汇总请求在搜索阶段耗尽迭代后整轮无回复 | P1 | Fixed | 2026-04-20 已将 `已达最大迭代次数 N` 改为机器可读 key `max_iterations_exceeded:N`，并在净化层过滤 | [feishu_direct_cron_job_iteration_exhaustion_no_reply.md](./feishu_direct_cron_job_iteration_exhaustion_no_reply.md) |
| Feishu 直聊在处理中遭遇 runtime 重启风暴，placeholder 发出后整轮无最终回复 | P1 | Fixed | 2026-04-20 已在 `run()` 启动时扫描 30 分钟内 `last_message_role=user` 的直聊会话并补发失败提示，同时落库 assistant 失败消息防止重复 | [feishu_direct_runtime_restart_interrupts_inflight_reply.md](./feishu_direct_runtime_restart_interrupts_inflight_reply.md) |
| MiniMax 搜索阶段 HTTP 发送失败后缺少自动重试与降级，用户仅收到通用失败提示 | P2 | Fixed | 2026-04-20 已在 `openai_compatible.rs` 的 `chat` / `chat_with_tools` 中对传输层错误补一次自动重试（2 秒间隔） | [minimax_search_http_transport_failure_no_retry.md](./minimax_search_http_transport_failure_no_retry.md) |
| Feishu 每日动态监控遇到 `codex acp stream closed before response` 后台账仍记为已发送 | P2 | Fixed | 2026-04-20 已将 `codex acp`/`stream closed before response`/`acp stream` 加入 `looks_internal_error_detail`，发送时自动替换为通用失败文案 | [feishu_scheduler_codex_acp_stream_closed_false_sent.md](./feishu_scheduler_codex_acp_stream_closed_false_sent.md) |
| Feishu 直聊消息在已有同 session 任务处理中时仍先发送 placeholder，但未真正进入 agent 主链路 | P1 | Fixed | 2026-04-18 19:01 最新真实 busy 样本已只发送 `direct.busy` 并跳过 placeholder，live 复核通过 | [feishu_direct_placeholder_without_agent_run.md](./feishu_direct_placeholder_without_agent_run.md) |
| Release runtime 缺少稳定 supervisor 时会丢失固定 `8077` 端口或整组进程退出，导致 Desktop 周期性掉线 | P1 | Fixed | `ea5229b` 已为 release helper 收口到 `.app` 启动形态、统一 `honeclaw/target` cache、并让 `launch.sh` 持续写入 `data/runtime/current.pid` 供重启链路可靠接管 | [desktop_release_runtime_supervision_gap.md](./desktop_release_runtime_supervision_gap.md) |
| OpenAI-compatible 搜索阶段出现 tool-call 协议错位，`invalid params` 失败被统一收口成通用失败提示 | P1 | Fixed | 2026-04-16 已补齐搜索上下文清洗：同时移除历史 `tool` 与残留 assistant `tool_calls`，定向回归测试与 desktop release build 已通过 | [openai_compatible_tool_call_protocol_mismatch_invalid_params.md](./openai_compatible_tool_call_protocol_mismatch_invalid_params.md) |
| Feishu 定时汇总旧会话在自动 compact 后仍无法完成日报，最终退化为"当前会话上下文过长"失败提示 | P2 | Fixed | 2026-04-20 在 context overflow compact 重试后改用更小的 restore limit（6 条消息），给 search 阶段留出足够上下文预算 | [feishu_scheduler_compact_retry_still_cannot_finish_company_digest.md](./feishu_scheduler_compact_retry_still_cannot_finish_company_digest.md) |
| Feishu 直聊自动 compact 后仍无法稳定完成新话题回答，同一旧会话会在成功与 fallback 间抖动 | P2 | Fixed | 2026-04-20 同上，compact 重试路径统一使用 CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT=6 | [feishu_direct_compact_retry_still_cannot_answer_new_topic.md](./feishu_direct_compact_retry_still_cannot_answer_new_topic.md) |
| Heartbeat 监控任务触发 `context window exceeds limit` 后缺少恢复，故障会在不同任务间漂移复现 | P2 | Fixed | 2026-04-20 heartbeat context overflow 改为 ContextOverflowNoop（skipped_noop），本轮跳过下轮正常重试 | [scheduler_heartbeat_context_window_limit_no_recovery.md](./scheduler_heartbeat_context_window_limit_no_recovery.md) |
| ASTS 发射链路把预告与停牌前行情误报成已发射后的实时结果 | P2 | Fixed | 2026-04-20 heartbeat prompt 补加时间一致性、价格时间口径、重复事件三条约束规则 | [asts_launch_schedule_misread_as_completed_event.md](./asts_launch_schedule_misread_as_completed_event.md) |
| Heartbeat 已触发提醒偶发向用户投递原始 JSON 载荷 | P3 | Fixed | 2026-04-20 在 JsonTriggered 分支补 `unwrap_nested_json_message`，将 `{"trigger":"..."}` 等嵌套 JSON 对象字段自动提取为纯文本 | [scheduler_heartbeat_trigger_json_payload_leak.md](./scheduler_heartbeat_trigger_json_payload_leak.md) |
| Feishu 直聊把歧义股票简称 `lite` 直接猜成 Litecoin，未先澄清实体 | P3 | Fixed | 2026-04-20 在 DEFAULT_FINANCE_DOMAIN_POLICY 补实体歧义约束：多候选资产时必须先列出候选请用户确认，不允许直接猜测 | [feishu_ambiguous_lite_entity_guessed_as_litecoin.md](./feishu_ambiguous_lite_entity_guessed_as_litecoin.md) |
| Feishu 直聊沿用旧证券上下文，用户问 `DRAM` 却被整轮答成 `SNDK` | P3 | Fixed | 2026-04-20 在 DEFAULT_FINANCE_DOMAIN_POLICY 补旧上下文漂移约束：工具调用目标必须由当前 user turn 推导，禁止套用旧 ticker | [feishu_direct_stale_symbol_context_hijacks_new_query.md](./feishu_direct_stale_symbol_context_hijacks_new_query.md) |
| Feishu 直聊个股分析把同一风险点在多段结构里重复展开，用户需额外指出"很多信息数据都是重复的" | P3 | Fixed | 2026-04-20 在 DEFAULT_COMPANY_PROFILE_POLICY 补长答去重约束：同一关键事实/风险点只在最相关章节展开一次，后续章节可引用不得重复 | [feishu_direct_analysis_redundant_risk_repetition.md](./feishu_direct_analysis_redundant_risk_repetition.md) |
| Feishu 直聊纯文本 15 支股票池请求误触 `image_understanding`，最终只分析 9 支并要求用户补 6 支代码 | P3 | Fixed | 2026-04-20 在 image_understanding SKILL.md 补 when_to_use 约束（仅在有图片附件时触发）；在 SkillTool 系统提示补全局约束：纯文本请求禁止调用图片/PDF 附件类 skill | [feishu_direct_watchlist_text_request_misfires_image_skill.md](./feishu_direct_watchlist_text_request_misfires_image_skill.md) |
| Feishu 直聊已拿到行情工具结果，但 Answer 仍谎报链路阻断并退化成空泛建议 | P3 | Fixed | 2026-04-20 在 multi-agent handoff 文本中添加 CRITICAL 约束：search transcript 中有成功 data_fetch/quote 结果时，answer 禁止输出"链路阻断/数据未完成校验"等降级文案 | [feishu_direct_quote_tool_result_ignored.md](./feishu_direct_quote_tool_result_ignored.md) |
| Feishu 直聊询问 skill 时误报"没有该 skill"，并把内部约束直接当答案返回 | P3 | Fixed | 2026-04-20 扩展 handoff CRITICAL 约束覆盖 discover_skills/skill_tool 结果；在 DEFAULT_FINANCE_DOMAIN_POLICY 补内部策略外泄约束：禁止以「系统纪律」口吻暴露内部规则 | [feishu_direct_skill_query_internal_policy_leak.md](./feishu_direct_skill_query_internal_policy_leak.md) |
| Feishu 定时汇总已送达但未执行最新资讯检索，静默退化为非实时摘要 | P3 | Closed | 2026-04-19 12:00 同一任务已不再复现 `tool_calls=0 + completed`；本轮改为执行 15 次 `data_fetch` 后触发 overflow fallback，旧伪完成形态关闭并转由新缺陷跟踪 | [feishu_scheduler_daily_company_digest_skips_realtime_research.md](./feishu_scheduler_daily_company_digest_skips_realtime_research.md) |

## 历史分析 / 部分止血

| 主题 | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| Event-engine logged dryrun high sends while config dryrun was false | P2 | Approved | 2026-04-22 最新进程已装配 `MultiChannelSink` 且无新增 `[dryrun sink]`，但历史高优先级事件可能只打印却被记 `sent`，未自动重放 | [event_engine_dryrun_sink_under_non_dryrun_config.md](./event_engine_dryrun_sink_under_non_dryrun_config.md) |
| Event-engine enabled channel heartbeat write hit ENOSPC | P2 | Approved | 2026-04-22 后续 heartbeat 与磁盘空间恢复，原始 `No space left on device` 写失败仍缺持久 degraded 状态与容量根因 | [event_engine_heartbeat_enospc_write_failure.md](./event_engine_heartbeat_enospc_write_failure.md) |
| opencode ACP `session/prompt timeout (300s)` 问题分析 | - | Fixed | 2026-04-13 已收口到 ACP runners 公共等待逻辑 | [opencode_acp_prompt_timeout.md](./opencode_acp_prompt_timeout.md) |
| opencode ACP 相关的 Prompt 泄露与缓存失效问题分析 | - | Partial | Prompt Echo 已止血；完整多轮 message 级缓存复用仍未实现 | [opencode_prompt_issues.md](./opencode_prompt_issues.md) |
