# Bug: Feishu 直聊在 Answer 阶段触发 idle timeout 后整轮无回复

- **发现时间**: 2026-04-15 23:12 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `sessions`
    - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`
    - `updated_at=2026-04-15T22:45:16.220393+08:00`
    - `last_message_at=2026-04-15T22:45:16.220389+08:00`
    - `last_message_role=user`
    - `last_message_preview=我的持仓情况`
    - 说明 22:45 这轮用户提问后，会话最终没有任何新的 assistant 消息落库
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - 同一 `session_id` 的最新消息只有 `2026-04-15T22:45:16.220389+08:00` 用户消息 `我的持仓情况`
    - 之后没有新的 assistant message，也没有空 assistant 占位消息
    - 搜索阶段已经执行过 `portfolio` / `hone_portfolio` 工具，说明链路不是“未启动”，而是在 Answer 阶段失败
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-15 22:45:16.213` `step=reply.placeholder ... detail=sent`
    - `2026-04-15 22:45:24.715` `runner.tool ... tool=portfolio status=start`
    - `2026-04-15 22:46:20.732` `runner.tool ... tool=hone_portfolio status=start`
    - `2026-04-15 22:49:20.738` `runner.error ... kind=TimeoutPerLine message="opencode acp session/prompt idle timeout (180s)"`
    - `2026-04-15 22:49:20.740` `stage=complete success=false`
    - `2026-04-15 22:49:20.741` `MsgFlow/feishu failed ... error="opencode acp session/prompt idle timeout (180s)"`
    - 同一时间窗内未出现对应的 `session.persist_assistant` 或 `reply.send`
  - 相关历史文档：
    - `docs/bugs/opencode_acp_prompt_timeout.md`
    - 该文档记录的是 2026-04-13 前“固定 300s 总超时”缺陷及其修复；本次证据发生在已切换到 `idle_timeout=180s overall_timeout=1200s` 之后，属于新的活跃失败形态

## 端到端链路

1. Feishu 用户在直聊里发送“我的持仓情况”，这是主问答链路中的正常用户请求。
2. 系统先发送 placeholder，并完成搜索阶段，期间成功执行 `portfolio` 与 `hone_portfolio` 工具。
3. Multi-Agent 进入 Answer 阶段后，`opencode_acp` 长时间没有产出最终答复。
4. 约 180 秒后 runner 触发 `session/prompt idle timeout (180s)`，本轮被标记为 `success=false`。
5. 链路随后直接结束，没有 assistant 消息落库，也没有任何最终回复发送给用户，用户视角等同“机器人开始工作但最后没回”。

## 期望效果

- Feishu 直聊主链路在工具阶段已成功后，应返回可消费的最终答复，至少不能无声结束。
- 如果 Answer 阶段发生 `idle timeout`，链路应向用户返回明确、产品化的失败提示，而不是只留下 placeholder 后静默失败。
- 会话落库应保留足够的失败痕迹，避免最终只看到“最后一条还是用户消息”。

## 当前实现效果

- 这轮真实会话里，工具执行已经开始并持续了数分钟，但最终没有 assistant 消息，也没有发送结果。
- 与 `feishu_direct_empty_reply_false_success` 不同，这次不是“空回复被误判成功”，而是 Answer 阶段直接超时失败，整轮回复缺失。
- 历史 `opencode_acp_prompt_timeout.md` 已标注 300 秒固定总超时问题为已修复，但当前生产链路仍会以新的 `idle timeout (180s)` 形态影响 Feishu 直聊用户。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量问题。用户的主问题没有得到任何最终回复，任务实际失败。
- 之所以定级为 `P1`，是因为问题发生在 Feishu 直聊主链路，且从用户视角看属于明显的“问了但没回”。
- 这不是 `P3` 质量类问题，因为损害不是“答得浅或格式差”，而是整轮问答没有完成。

## 根因判断

- 历史上的“固定 300 秒总超时误杀长任务”问题已修复，但当前 `idle timeout=180s` 仍会在某些真实直聊场景下触发，说明链路还存在新的长尾卡顿或无进展问题。
- Feishu 失败分支当前至少没有把这类超时稳定转成用户可见的产品化失败答复，导致用户只能看到 placeholder，然后等到超时后静默结束。
- 从日志看，问题更像是 Answer 阶段在工具完成后迟迟没有稳定收敛，而不是搜索阶段或消息发送阶段本身失败。

## 下一步建议

- 新建活跃缺陷跟踪 `idle timeout (180s)` 在直聊主链路的影响范围，不要继续沿用 `opencode_acp_prompt_timeout.md` 的“已收口”结论覆盖当前现象。
- 排查 `opencode_acp` 在收到工具结果后为何会出现长时间无进展，重点核对是否存在工具后收束、流式事件消费或模型侧卡住未落盘的问题。
- 为 Feishu 直聊失败分支补上用户态兜底提示，至少在 `TimeoutPerLine` 时返回“处理超时，请重试”的稳定回复。
- 后续修复时补回归验证，覆盖“搜索成功 + Answer idle timeout”场景，确保不再出现只有 placeholder、没有最终消息的静默失败。
