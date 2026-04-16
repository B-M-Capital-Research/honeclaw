# Bug: Feishu 直聊消息在已有同 session 任务处理中时仍先发送 placeholder，但未真正进入 agent 主链路

- **发现时间**: 2026-04-16 13:40 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 最近真实会话：
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-16 13:35:53`、`13:36:06`、`13:39:29` 连续三次只记录到 `step=reply.placeholder ... detail=sent`
    - 对应 message_id:
      - `om_x100b51331da2fcb0b372d4261515e4d`
      - `om_x100b51331af1c8a8b25f3dadee4a13a`
      - `om_x100b51332e157888b351106abb9185b`
  - 最近运行日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - 在上述三条 placeholder 之后，没有出现同 message_id 的：
      - `session.persist_user`
      - `recv`
      - `agent.prepare`
      - `agent.run`
      - `failed`
    - 同时间窗内 Feishu 渠道进程仍在线，说明不是整个 listener 进程退出。
  - 代码线索：
    - `bins/hone-feishu/src/handler.rs` 中 direct / group 共用同一条 placeholder 发送逻辑
    - `crates/hone-channels/src/agent_session.rs` 中 `AgentSession::run()` 会在写 `session.persist_user` 日志前先等待 per-session run lock
    - Feishu handler 当前只对群聊做 `SessionLockRegistry::try_begin_active(...)` busy 短路，私聊没有对应入口级并发保护
  - 2026-04-16 当前源码修复与验证：
    - `bins/hone-feishu/src/handler.rs` 已将 Feishu 私聊也纳入入口层 `SessionLockRegistry` busy 检查，并把 placeholder 发送移动到获得处理权之后
    - 定向回归：`cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture` 通过

## 端到端链路

1. 用户在 Feishu 私聊里连续发送多条消息或附件。
2. 新消息进入 Feishu handler 后，系统先发送“正在思考中...”或附件确认 placeholder。
3. 但如果同一 `session_id` 已有上一条消息仍在处理中，新消息会在更深层的 `AgentSession::run()` 入口等待 session run lock。
4. 因为 Feishu 私聊入口缺少显式 busy 短路，用户侧只看到 placeholder，却没有后续正式处理日志与结果回执。

## 期望效果

- 如果同一 Feishu 私聊 session 已有消息在处理中，应在入口期直接返回明确 busy 提示，而不是先发送 placeholder。
- 只有真正拿到处理权的消息，才应发送 placeholder 并进入 `agent.run`。
- 日志应能清晰区分“真正开始处理”与“因 busy 被短路”。

## 当前实现效果

- 修复前，群聊已经有 busy / pretrigger 策略，但 Feishu 私聊没有同等级入口保护。
- 修复前，私聊用户连续发送消息时，系统会先给 placeholder，随后卡在更深层 session 锁等待，体感上像“处理失败”或“系统没反应”。
- 当前已改为：如果同一私聊 session 已有消息在处理中，入口会直接发送明确 busy 提示，并记录 `direct.busy` 日志；只有拿到处理权的消息才会发送 placeholder。

## 用户影响

- 这是功能性缺陷。用户会误以为消息已经开始处理，但实际没有进入 agent 主链路。
- 之所以定级为 `P1`，是因为它直接影响 Feishu 私聊主链路的可用性与可解释性，且会持续误导用户反复重试。
- 之所以不是 `P0`，是因为当前证据仍集中在单渠道、单 session 并发场景，并非系统全局不可用。

## 根因判断

- 根因不在 Tavily、MiniMax 或 answer provider，而在 Feishu 私聊入口的并发策略缺口。
- 具体来说，placeholder 的发送发生在 `AgentSession::run()` 之前，而 `run()` 内部会先等待 per-session run lock；当同一 session 已有旧任务未完成时，新消息会被深层锁住，却没有入口级 busy 提示。
- 群聊已有 `SessionLockRegistry` 保护，但私聊未复用这一机制，导致“placeholder 假启动”。

## 下一步建议

- 继续观察 `direct.busy` 与同 session 的后续成功率，确认用户不再收到“处理中假启动”。
- 若后续确认深层 `session.run()` 仍存在异常长时间持锁的独立根因，再单独建档追踪。
