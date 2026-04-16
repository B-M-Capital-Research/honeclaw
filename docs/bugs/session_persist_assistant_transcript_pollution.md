# Bug: 成功会话仍把原始 multi-agent transcript 落库到 assistant 历史，污染后续上下文

- **发现时间**: 2026-04-16 02:22 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f58ff884640e647a1792f618f45209251`
    - `2026-04-16T01:08:28.857635+08:00` 用户提问：`我想等350的美光，能等到吗`
    - `2026-04-16T01:09:31.998121+08:00` assistant 落库消息不是净化后的最终答复，而是同一条消息内混入 `progress`、`tool_call`、`tool_result`、`final`
    - 同会话 `sessions.last_message_preview` 直接以 `<think> The user is asking about waiting for Micron (MU) at $350...` 开头，说明会话摘要索引也拿到了未净化的内部 transcript
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 01:09:31.992` `AgentRunner/opencode ... stop_reason=end_turn success=true reply_chars=1956`
    - `2026-04-16 01:09:32.024` `MsgFlow/feishu done ... success=true ... reply.chars=752`
    - `2026-04-16 01:09:34.283` `step=reply.send ... segments.sent=1/1`
    - 上述日志说明用户侧实际发送的是 752 字的净化结果，但落库 assistant 仍保存了更长的原始 transcript
  - 对照复现会话：
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-16T01:10:01.999236+08:00` assistant 同样以 `progress/tool_call/tool_result/final` 混合结构落库；这条会话已在 `feishu_attachment_internal_transcript_leak.md` 中证明用户侧也看到了泄露内容
  - 2026-04-16 08:31 最新复核：
    - `session_id=Actor_feishu__direct__ou_5f0a88f4c2105e8388aa2a63ae847f7f28`
    - `2026-04-16T08:31:33.820805+08:00` scheduler 成功会话的 assistant 预览直接以 `<think> The user has a scheduled task triggering the "创新药持仓每日动态推送"...` 开头
    - 同轮 `cron_job_runs.run_id=1837` 记为 `completed + sent + delivered=1`，说明即便任务表面成功，`sessions.last_message_preview` 仍保存了未净化 transcript
  - 相关历史缺陷：
    - `docs/bugs/multi_agent_internal_output_leak.md`
    - `docs/bugs/feishu_attachment_internal_transcript_leak.md`

## 端到端链路

1. Feishu 直聊用户发起普通 multi-agent 问答，搜索阶段正常完成工具调用。
2. Answer 阶段产出了完整的内部 transcript，其中包含 `<think>`、`tool_call`、`tool_result` 和 `final`。
3. 发送链路对用户可见文本做了净化，因此日志显示 `reply_chars=752` 并成功投递。
4. 但 `session.persist_assistant` 写入数据库时，保存的不是净化后的最终答复，而是原始 transcript 整包。
5. `sessions.last_message_preview` 继续基于这条脏 assistant 内容生成摘要，导致后续 restore / compact / 检索入口都会看到内部协议文本。

## 期望效果

- 成功会话落库的 assistant 历史必须与用户最终收到的净化文本保持一致，不能保留 `<think>`、`tool_call`、`tool_result` 等内部结构。
- `sessions.last_message_preview`、compact summary 和后续 prompt restore 只能基于净化后的 assistant 正文，而不是原始运行 transcript。
- 即便发送链路需要保留内部 transcript 供调试，也应写入受控诊断字段，而不是复用用户会话历史。

## 当前实现效果

- 最近一小时的 MU 会话已经证明：即使用户侧收到了正常答复，数据库里的 assistant 消息仍保存完整的原始 transcript。
- 同一条消息既包含 `<think>`，又包含两个 `data_fetch` 的 `tool_call/tool_result`，最后才附带 `final`，说明落库对象不是净化后的最终答案。
- `sessions.last_message_preview` 也直接以 `<think>` 开头，说明这份污染已经进入会话索引层，而不只是明细表内部可见。
- 对照同时间窗的图片会话可见，这种持久化污染并非只存在于失败链路；在失败链路里它会进一步升级成用户侧泄露，在成功链路里则以“历史脏写入”的形式继续存在。
- `08:31` 的 `创新药持仓每日动态推送` 则进一步证明：即使 scheduler 已经 `completed + sent + delivered=1`，会话索引层仍可能把 `<think>` 作为最后一条 assistant 预览保存下来，说明污染范围不止于直聊问答，也覆盖定时任务成功链路。

## 用户影响

- 这是功能性缺陷，不是单纯文案质量问题。会话历史被脏 transcript 污染后，后续多轮问答、compact summary、恢复上下文和质量巡检都会把内部协议误当成真实 assistant 历史。
- 问题会直接影响后续回答正确性与会话可维护性，因此定级为 `P2`，而不是仅影响展示体验的 `P3`。
- 之所以不是 `P1`，是因为本条证据链里用户侧主回答仍成功送达，当前确认到的直接伤害主要集中在历史状态污染与后续链路风险。

## 根因判断

- 发送链路和持久化链路使用了不同的 assistant 文本来源：前者发送净化结果，后者仍把原始运行 transcript 写入 `session_messages`。
- `session.persist_assistant` 或其上游数据结构没有强制要求“仅持久化 sanitized final answer”，导致 `<think>`、工具协议和中间稿穿透到了历史存储。
- `sessions.last_message_preview` 继续从脏 assistant 内容生成摘要，说明会话索引层也没有二次净化。
- 这与 `multi_agent_internal_output_leak.md` 中已修复的“直接发给用户”链路不同，属于成功响应后的历史落库边界缺失。

## 下一步建议

- 追踪 `session.persist_assistant` 的输入来源，确认何处仍在写入原始 transcript 而不是最终净化文本。
- 为“用户收到净化文本后，数据库中 assistant 历史也必须是净化文本”补一条回归测试，覆盖 `progress/tool_call/tool_result/final` 混合输出场景。
- 为 `sessions.last_message_preview` 与 compact/restore 路径增加一致性校验，避免历史索引继续吸收 `<think>` 和工具协议。
- 修复后联动复核 `feishu_attachment_internal_transcript_leak.md`，确认用户可见泄露和历史脏写入两个问题是否都已收口。
