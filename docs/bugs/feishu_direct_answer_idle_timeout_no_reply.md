# Bug: Feishu 直聊在 Answer 阶段触发 idle timeout 后整轮无最终回复

- **发现时间**: 2026-04-15 23:12 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **修复提交**:
  - `02d01d2 fix channel error message sanitization`
  - `3e769d7 test feishu timeout fallback reply`
- **证据来源**:
  - 2026-04-21 20:25-20:29 最新失败样本：
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
      - `2026-04-21T20:25:49.192603+08:00` 用户输入：`击球区不要只写比率，也要写区间值`
      - `2026-04-21T20:29:14.930893+08:00` assistant 只落库 `抱歉，处理超时了。请稍后再试。`
      - 同一会话在该 user turn 后没有正式完成“补区间值”的确认或更新说明。
    - `data/runtime/logs/web.log`
      - `2026-04-21 20:25:49.214` 记录 `restore_context + build_prompt + create_runner` 后进入 `agent.run`
      - `2026-04-21 20:29:14.922` 记录 `runner.error ... kind=TimeoutPerLine message="codex acp session/prompt idle timeout (180s) ... state_5.sqlite: migration 23 was previously applied but is missing in the resolved migrations"`
      - `2026-04-21 20:29:14.925` 记录 `handler.session_run ... completed success=false reply_chars=0`
    - 这说明 15:14-15:32 的 Codex ACP state DB migration 变体没有自然恢复；到 20:29 仍会让普通短指令在 Answer 阶段失败，用户只得到通用超时文案，主任务未完成。
  - 2026-04-21 15:14-15:32 最新连续失败样本：
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc`
      - `2026-04-21T15:14:27.370207+08:00` 与 `2026-04-21T15:19:50.059361+08:00`，用户连续两次发送 `分析一下ASTS`
      - 对应 assistant 仅在 `15:17:53.101867` 与 `15:23:20.926592` 落库 `抱歉，处理超时了。请稍后再试。`
      - `2026-04-21T15:28:55.973084+08:00` 又出现一条图片类 user turn，`15:32:31.073397` 仍只落库同一通用超时文案
      - 到本轮巡检结束，该会话在 15:14 之后没有任何正式 ASTS 分析答复；用户连续重试也只得到通用失败提示。
    - `data/runtime/logs/sidecar.log`
      - `2026-04-21 15:32:31.066` 记录 `runner.error ... kind=AgentFailed message="codex acp request failed: Internal error stderr=... failed to open state db at <absolute-path>/state_5.sqlite: migration 23 was previously applied but is missing in the resolved migrations ..."`
      - `2026-04-21 15:32:31.067` 同步记录 `MsgFlow/feishu failed ... error="codex acp request failed: Internal error stderr=... state_5.sqlite ... migration 23 ..."`
      - `2026-04-21 15:32:31.068` `handler.session_run ... completed success=false reply_chars=0`
    - 这说明本缺陷最新形态已从单纯 `idle timeout` 扩展到 Codex ACP 本地 state DB migration 不一致导致的 Answer 阶段失败；当前用户侧虽然收到通用“处理超时”，但主任务仍没有被完成。
  - 2026-04-21 14:52-15:00 最新回归样本：
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc`
      - `ordinal=16` user turn，`timestamp=2026-04-21T14:52:33.208137+08:00`
      - 用户输入：`分析一下金风科技`
      - `ordinal=17` assistant turn，`timestamp=2026-04-21T14:58:54.387058+08:00`
      - assistant 内容不是正式分析结论，而是 `正在思考中...当前时间为2026年4月21日14:52（北京时间）。金风科技同时有 A 股和 H 股口径...`，后续夹带 `正在调用 Tool: hone/skill_tool...`、`正在执行：rg --files company_profiles | rg 'goldwind|jinfeng|金风|002202|02208'`、多次工具调用进度等中间轨迹。
      - `ordinal=18` 已是用户下一轮 `分析一下ASTS`，说明 `金风科技` 这一轮没有后续正式 `final` 答复落库。
    - `data/runtime/logs/sidecar.log`
      - `2026-04-21 14:52:33.212` `recv ... input.preview="分析一下金风科技"`
      - `2026-04-21 14:58:54.382` `runner.error ... kind=TimeoutPerLine message="codex acp session/prompt idle timeout (180s) stderr=... failed to open state db ... migration 23 was previously applied but is missing in the resolved migrations ..."`
      - `2026-04-21 14:58:54.385` `MsgFlow/feishu failed ... error="codex acp session/prompt idle timeout (180s) ..."`
      - `2026-04-21 14:58:54.385` `step=handler.session_run ... completed success=false reply_chars=0`
    - 这说明 2026-04-16 的“timeout 失败分支会回填友好文案、不再静默结束”结论已回归失效：当前真实会话虽落库了一条 assistant，但它是半成品进度/工具轨迹，不是用户请求的最终答案；从用户视角仍等同本轮任务失败。
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

1. Feishu 用户在直聊里发送正常用户请求，例如 `我的持仓情况` 或 `分析一下金风科技`。
2. 系统先发送 placeholder，并进入 agent 执行链路；部分样本已完成搜索/工具调用，部分样本已把工具调用进度流式写入。
3. Runner 进入 Answer / Codex ACP 阶段后长时间没有产出正式最终答复。
4. 约 180 秒后 runner 触发 `session/prompt idle timeout (180s)`，本轮被标记为 `success=false`。
5. 链路随后没有给出用户请求的最终答案：旧样本表现为没有 assistant 消息落库，新样本表现为只落库半成品进度/工具轨迹，用户视角仍等同“机器人开始工作但最后没完成”。

## 期望效果

- Feishu 直聊主链路在工具阶段已成功后，应返回可消费的最终答复，至少不能无声结束。
- 如果 Answer 阶段发生 `idle timeout`，链路应向用户返回明确、产品化的失败提示，而不是只留下 placeholder 后静默失败。
- 会话落库应保留足够的失败痕迹，避免最终只看到“最后一条还是用户消息”。

## 修复情况（2026-04-16 HEAD 复核）

- 2026-04-21 15:14-15:32 又出现同一用户连续请求 `分析一下ASTS` 却只收到通用超时提示的样本；日志根因变体为 Codex ACP 本地 `state_5.sqlite` migration 不一致，仍属于 Answer 阶段失败后主任务无最终答复，本单继续保持 `New`。
- 2026-04-21 14:52 最新真实样本已推翻本节旧结论：`codex acp session/prompt idle timeout (180s)` 后没有回填完整产品化失败文案，也没有正式回答用户问题，而是把过渡文本和工具调用轨迹作为 assistant 消息落库。本单重新打开为 `New`。
- `02d01d2 fix channel error message sanitization` 已把 Feishu 直聊失败分支改成共享 `user_visible_error_message(...)`，不再把原始 timeout 细节直接拼接，也不会在无流式内容时静默结束。
- 当前 `bins/hone-feishu/src/handler.rs` 的失败分支会在 `response.success=false` 时：
  - 若已有部分流式正文，回填“内容可能不完整”的收尾提示；
  - 若没有可见正文，则把 `opencode acp session/prompt idle timeout (180s)` 映射为“抱歉，处理超时了。请稍后再试。”
- 本轮继续把该失败回复逻辑抽成 `build_failed_reply_text(...)`，并补 handler 级回归测试，防止后续重构再次把 timeout 退化成静默失败或内部报错直出。
- 因此，这份缺陷文档记录的“22:49 超时后既未落库 assistant，也未发送最终回复”现象已不再代表当前 HEAD 的行为；现阶段剩余风险更接近“超时后是否有持久化失败痕迹”，而不是“完全无最终回复”。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量问题。用户的主问题没有得到任何最终回复，任务实际失败。
- 之所以定级为 `P1`，是因为问题发生在 Feishu 直聊主链路，且从用户视角看属于明显的“问了但没回”。
- 这不是 `P3` 质量类问题，因为损害不是“答得浅或格式差”，而是整轮问答没有完成。

## 根因判断

- 历史上的“固定 300 秒总超时误杀长任务”问题已修复，但 `idle timeout=180s` 仍可能在某些真实直聊场景下触发，说明链路还存在新的长尾卡顿或无进展问题。
- 这份缺陷最初暴露出的直接用户痛点，是 Feishu 失败分支当时没有稳定把这类超时转成用户可见的产品化失败答复，导致用户只能看到 placeholder，随后静默结束。
- 从原始日志看，问题更像是 Answer 阶段在工具完成后迟迟没有稳定收敛，而不是搜索阶段或消息发送阶段本身失败。

## 回归验证

- `cargo test -p hone-feishu failed_reply_text_maps_idle_timeout_to_friendly_message`
- `cargo test -p hone-feishu failed_reply_text_keeps_partial_stream_output`
- `cargo test -p hone-feishu failed_reply_text_drops_tool_progress_only_partial_stream`

## 修复进展（2026-04-26）

- 已在 `bins/hone-feishu/src/handler.rs` 的失败回复构造中增加 partial stream 清洗：
  - 如果 partial stream 只是 `Tool: hone/...`、`正在执行：...`、`hone/data_fetch`、`hone/web_search` 等工具/进度轨迹，则丢弃 partial；
  - 失败回复改走 `user_visible_error_message(...)`，例如 idle timeout 映射为 `抱歉，处理超时了。请稍后再试。`；
  - 真正有用户可读阶段性正文时，仍保留“内容可能不完整”的收尾提示。
- 已补回归：`failed_reply_text_drops_tool_progress_only_partial_stream`。
- 状态从 `New` 调整为 `Fixing`：用户可见“半成品工具轨迹”已代码止血；Answer 阶段 idle timeout / state migration 的底层原因仍需继续观察与修复。

## 结论

- 该缺陷在 2026-04-16 曾由 `02d01d2` 方向修复，但 2026-04-21 14:52 真实会话出现回归/变体：超时失败后只留下半成品进度和工具轨迹，没有正式回答。
- 下一步应优先检查 Codex ACP runner 超时失败分支、已有 partial stream 时的失败收尾和 assistant 持久化策略，确保用户至少收到明确失败文案，而不是中间执行轨迹。
