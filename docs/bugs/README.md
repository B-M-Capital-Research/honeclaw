# Bugs Navigation

最后更新：2026-04-15 19:05 CST

这个文件是 `docs/bugs/` 的导航页，也是后续 agent / 人工协作时优先查看的缺陷台账入口。

## 使用约定

- 开始修 bug 前，先看“活跃待修复”表，再进入对应缺陷文档核对证据、链路和代码位置。
- 新增缺陷文档、更新严重等级、切换状态、确认修复、补充修复提交时，必须在同一次改动里同步更新本页。
- 修复完成后，除了更新单个 bug 文档的状态，也必须同步更新本页的“状态”和“修复情况”列。
- `bug` 自动化负责发现/更新缺陷，并维护本页导航；`bug-2` 自动化负责从本页活跃缺陷中选择修复对象，并在修复后回写本页。
- 新缺陷默认使用标准状态：`New`、`Approved`、`Fixing`、`Fixed`、`Closed`。历史文档若仍保留旧写法，可先在本页做归一化摘要，不必为了统一格式单独重写全文。

## 当前概览

- 活跃待修复：14
- 已修复 / 已关闭：7
- 历史分析 / 部分止血：2
- 当前活跃队列中没有 `P0`；最高待修优先级为 `P1`

## 活跃待修复

| Bug | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| Release runtime 缺少稳定 supervisor 时会丢失固定 `8077` 端口或整组进程退出，导致 Desktop 周期性掉线 | P1 | New | 未修复；2026-04-15 日志显示 backend 曾漂移到 `56044`，Desktop 仍持续探测 `8077` | [desktop_release_runtime_supervision_gap.md](./desktop_release_runtime_supervision_gap.md) |
| Desktop legacy runtime 会整块覆盖 canonical `agent.opencode` 配置，破坏本机 OpenCode 继承语义 | P1 | New | 未修复；2026-04-15 HEAD 复核仍存在整块覆盖 | [desktop_opencode_legacy_override_gap.md](./desktop_opencode_legacy_override_gap.md) |
| Desktop Agent 设置会把 `multi-agent.answer` 反写到 `agent.opencode`，导致不同 runner 的独立配置互相覆盖 | P1 | New | 未修复；设置页两块配置仍共用同一落盘字段 | [desktop_runner_settings_cross_runner_overwrite.md](./desktop_runner_settings_cross_runner_overwrite.md) |
| Desktop 设置页多入口保存共用同一份配置文件但缺少串行写保护，可能造成 runner 配置被并发保存静默覆盖 | P1 | New | 未修复；配置写入链路仍缺少共享串行锁 | [desktop_runner_settings_write_race.md](./desktop_runner_settings_write_race.md) |
| Desktop 设置页切换 runner 后可能显示已切换，但 bundled runtime 重启失败会被静默吞掉，实际仍跑旧 runner 或未完成切换 | P1 | New | 未修复；runner 切换仍可能出现“UI 成功、runtime 未生效” | [desktop_runner_switch_false_success_gap.md](./desktop_runner_switch_false_success_gap.md) |
| Desktop 设置页重复点击 runner 会触发重入保存与 bundled backend 重启，导致切换过程卡死或表现为“点一下就崩” | P1 | New | 未修复；重复点击仍会排队触发连续重启 | [desktop_runner_switch_reentrant_restart_gap.md](./desktop_runner_switch_reentrant_restart_gap.md) |
| Multi-Agent Answer Agent 在设置页允许 `maxToolCalls=0`，但运行时强制提升为至少 1，用户无法真正禁用补充工具调用 | P1 | New | 未修复；UI 与运行时的 `maxToolCalls` 语义仍不一致 | [multi_agent_answer_max_tool_calls_zero_ignored.md](./multi_agent_answer_max_tool_calls_zero_ignored.md) |
| Multi-Agent Search Agent 在 Desktop 设置页显示可继承 auxiliary key，但真实运行时不使用该 fallback，导致看似已配置却直接失败 | P1 | New | 未修复；UI fallback 与运行时 Search Agent key 语义仍未对齐 | [multi_agent_search_key_fallback_mismatch.md](./multi_agent_search_key_fallback_mismatch.md) |
| 会话压缩摘要会把最后一个新问题误写成完整“用户报告”并以 `Compact Summary` 回灌，正式回答因此引用不存在的报告与伪造价格假设 | P1 | New | 2026-04-15 Rocket Lab 会话已复现；`MiniMax-M2.7-highspeed` 压缩产出了虚假的 RKLB 长报告并被后续回答当成用户材料 | [session_compact_summary_report_hallucination.md](./session_compact_summary_report_hallucination.md) |
| 定时任务链路绕过统一输出净化，向用户投递内部思考与未清洗富文本 | P1 | New | 未修复；普通会话已净化，scheduler 仍直接发送原始输出 | [scheduled_output_sanitization_gap.md](./scheduled_output_sanitization_gap.md) |
| 定时任务达到上限后，Agent 未经用户确认就批量删除已有任务 | P1 | New | 2026-04-15 最近一小时真实会话新增；`add` 失败后同轮连续删除 8 个旧任务再重试创建 | [scheduler_task_limit_auto_cleanup_without_confirmation.md](./scheduler_task_limit_auto_cleanup_without_confirmation.md) |
| Feishu 直聊会话在 Multi-Agent Answer 阶段返回空回复后，链路仍记成功并发送空消息 | P1 | New | 2026-04-15 17:49 最近一小时新增；`reply_chars=0` 后仍 `success=true`，并执行 `reply.send segments.sent=1/1` | [feishu_direct_empty_reply_false_success.md](./feishu_direct_empty_reply_false_success.md) |
| Heartbeat 定时任务遇到 `JsonUnknownStatus` 时静默跳过，监控提醒可能长期失效 | P2 | New | 2026-04-15 19:01 最近一小时仍在复现；`18:01`、`18:31`、`19:01` 三轮连续 heartbeat 再次被当作 `noop` 静默跳过 | [scheduler_heartbeat_unknown_status_silent_skip.md](./scheduler_heartbeat_unknown_status_silent_skip.md) |
| Discord 定时任务在 Answer 阶段返回空回复时被记为成功执行，但最终未向用户送达 | P2 | New | 2026-04-15 最近一小时新增；`reply_chars=0` 但 run 仍记为 `completed`，最终 `send_failed` | [discord_scheduler_empty_reply_send_failed.md](./discord_scheduler_empty_reply_send_failed.md) |

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

## 历史分析 / 部分止血

| 主题 | 严重等级 | 状态 | 修复情况 | 入口 |
| --- | --- | --- | --- | --- |
| opencode ACP `session/prompt timeout (300s)` 问题分析 | - | Fixed | 2026-04-13 已收口到 ACP runners 公共等待逻辑 | [opencode_acp_prompt_timeout.md](./opencode_acp_prompt_timeout.md) |
| opencode ACP 相关的 Prompt 泄露与缓存失效问题分析 | - | Partial | Prompt Echo 已止血；完整多轮 message 级缓存复用仍未实现 | [opencode_prompt_issues.md](./opencode_prompt_issues.md) |
