# Bug: Web direct replies stream to ACP but are not persisted to session history

- 发现时间：2026-06-01 07:02 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无

## 证据来源

- `data/runtime/logs/acp-events.log`
  - `2026-06-01 06:30 CST` Web direct session `Actor_web__direct__web-user-14f4cadb069f` 收到用户组合跟踪请求，ACP runner 在 `06:30:14-06:31:48 CST` 持续输出 `agent_message_chunk`，并在 `06:31:48 CST` 返回 `stopReason=end_turn`。
  - `2026-06-01 06:52 CST` Web direct session `Actor_web__direct__web-user-ba50cb9401c0` 收到 NBIS 深度分析请求，ACP runner 在 `06:52:46-06:56:06 CST` 持续输出 `agent_message_chunk`，并在 `06:56:06 CST` 返回 `stopReason=end_turn`。
- `data/sessions/*.json`
  - `data/sessions/Actor_web__direct__web-user-14f4cadb069f.json` 仍停在 `2026-05-29T23:25:56+08:00`，尾部最后一条仍是旧用户消息，未包含 `2026-06-01 06:30 CST` 这轮组合跟踪请求或 assistant final。
  - `data/sessions/Actor_web__direct__web-user-ba50cb9401c0.json` 仍停在 `2026-05-31T17:09:21+08:00`，未包含 `2026-06-01 06:52 CST` 这轮 NBIS 请求或 assistant final。
- `data/sessions.sqlite3`
  - `session_messages` 中 `Actor_web__direct__web-user-14f4cadb069f` 最新消息仍是 `2026-05-29 15:25:56 UTC`，`message_count=27`。
  - `session_messages` 中 `Actor_web__direct__web-user-ba50cb9401c0` 最新消息仍是 `2026-05-31 09:09:21 UTC`，`message_count=212`。
  - 同一窗口 Feishu direct session `Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8` 能正常推进到 `2026-06-01 03:27 CST` 并写入 JSON / SQLite，说明问题不是 `sessions.sqlite3` 全局停摆。

## 端到端链路

1. Web direct 用户发送投研请求。
2. Web API 启动 Codex ACP runner，并向前端流式发送 `agent_message_chunk`。
3. Runner 返回 `stopReason=end_turn`，说明模型侧已经完成本轮回复。
4. 本轮 user turn 与 assistant final 没有写入 canonical `data/sessions/<session>.json`。
5. SQLite 会话索引也没有对应增量，后续历史恢复、上下文续聊和缺陷巡检均看不到这两轮真实 Web 会话。

## 期望效果

- Web direct 每轮用户输入应先持久化 user turn。
- ACP runner 完成后，应把最终 assistant 回复写入 canonical JSON 会话。
- SQLite 会话索引应随 JSON truth source 同步更新，至少在下一次启动回填后可追平。
- 前端实时流、历史恢复、后续上下文与巡检数据源应看到同一轮对话。

## 当前实现效果

- ACP/SSE 路径能向用户实时输出回复并收到 `end_turn`。
- Canonical JSON 会话没有追加本轮 user / assistant 消息。
- `sessions.sqlite3` 也没有追加这两轮 Web direct 消息。
- Feishu direct 同窗正常落库，说明缺陷更集中在 Web direct streaming / persistence 分支，而不是全局 session storage 不可写。

## 用户影响

- 这是功能性 bug，不是单纯质量问题。
- 用户刷新页面、跨设备打开或后续续聊时，可能看不到刚完成的 Web direct 回复，也无法让下一轮上下文引用该回复。
- 巡检任务默认优先读 `data/sessions.sqlite3`，会漏掉真实 Web direct 会话，降低缺陷发现能力。
- 定级为 `P2`：它影响 Web direct 历史、上下文和排障数据完整性；但当前证据显示实时流本身已完成，且没有跨用户投递、数据破坏或全渠道不可用，因此不定为 `P1`。

## 根因判断

- 初步判断是 Web direct ACP streaming 分支在 runner `end_turn` 后没有进入正常 `session.persist_user` / `session.persist_assistant` 路径，或 persistence 失败后没有把失败提升为可观测运行错误。
- 这不同于既有 `sessions_sqlite_mirror_stalled_after_successful_direct_replies.md`：该旧缺陷关注 JSON truth source 已更新但 SQLite mirror 没追平；本轮证据显示 Web direct JSON truth source 本身也没有写入。
- 这也不同于既有 `web_direct_quota_rejected_without_visible_reply.md`：本轮不是 quota 早退或无可见回复，ACP runner 已产生完整流式回复并 `end_turn`。

## 下一步建议

1. 检查 Web direct request handler 在启动 ACP runner 前后是否调用了 `AgentSession` 的统一持久化入口，尤其是 SSE streaming 成功路径。
2. 在 Web direct 成功 `end_turn` 后增加持久化失败日志和显式错误状态，避免只完成实时流但丢失历史。
3. 增加回归：模拟 Web direct ACP successful end_turn，断言 JSON session 追加 user / assistant，SQLite mirror 能看到同一轮消息。
4. 修复后复核 `Actor_web__direct__web-user-14f4cadb069f` 和 `Actor_web__direct__web-user-ba50cb9401c0` 类 Web direct 会话，确认新轮次能同时进入实时流、JSON history 和 SQLite index。
