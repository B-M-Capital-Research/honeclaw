# Bug: Web direct replies stream to ACP but are not persisted to session history

- 发现时间：2026-06-01 07:02 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：Closed
- GitHub Issue：无

## 2026-06-01 12:10 CST 复核结论

- 本缺陷关闭为“证据不足 / 不成立”，本轮不改业务代码。
- 代码侧 `crates/hone-web-api/src/routes/chat.rs` 的 Web direct SSE 路径调用统一的 `AgentSession::run(...)`；`AgentSession::run(...)` 会先持久化 user turn，成功后持久化 assistant turn。
- 当前仓库长期文档已明确：cloud mode 下 `SessionStorage::new_cloud` 直接使用 PG `cloud_sessions` 作为 session read / write 后端；`docs/repo-map.md` 也说明云模式不再以本地 `data/sessions/*.json` 或 `data/sessions.sqlite3` 作为权威会话存储。
- 原证据只对比了 `acp-events.log` 与本地 JSON / SQLite，没有查询 PG `cloud_sessions` 或通过 Web API history/read path 验证，因此不能证明 Web direct 成功回复未进入真实 session history。
- 该问题不关联 GitHub Issue；本轮只同步 bug 台账，避免继续把云迁移后的本地镜像缺口误判为 Web direct 持久化缺陷。

## 2026-06-01 15:03 CST 复核结论

- 15:00 CST Web direct session `Actor_web__direct__web-user-f40ae1caa720` 在 `acp-events.log` 中已流式输出并返回 `stopReason=end_turn`，但本地 `sessions.sqlite3` 仍停在 2026-05-30 20:32 CST。
- 该观测只证明本地 SQLite / JSON 镜像没有追踪到 cloud mode Web direct 新会话；结合 12:10 CST 的代码与文档复核，仍不能证明 Web API history read path 或 PG `cloud_sessions` 丢失该轮回复。
- 本轮不重新打开缺陷，状态保持 `Closed`；后续若要重开，必须补充 PG `cloud_sessions` 查询或 Web API history/read path 返回缺失的证据。

## 2026-06-05 03:02 CST 复核结论

- 02:26 CST 与 02:32 CST Web direct session `Actor_web__direct__web-user-d77177fe4502` 在 `data/runtime/logs/acp-events.log` 中均有 `session/prompt`、工具/搜索更新和 `stopReason=end_turn`。
- 本地 `data/sessions.sqlite3` 仍显示该 session 最新消息停在 `2026-05-22T22:23:29.723477+08:00`，且 2026-06-04 23:01-2026-06-05 03:02 CST 窗口内 `Actor_web__direct__%` 的 `session_messages` 计数为 0。
- 这与 2026-06-01 的复核一致：只能证明本地 SQLite 镜像未追踪 cloud mode Web direct 新会话，不能证明云端 PG `cloud_sessions` 或 Web API history/read path 丢失该轮回复。
- 本轮没有 PG `cloud_sessions` 查询、Web API history 缺失、用户刷新后丢历史、续聊上下文丢失或前端历史为空的证据，因此状态保持 `Closed`，不重新打开；若后续要重开，仍需补充权威 history/read path 缺失证据。

## 影响范围

- 无需回滚或迁移数据。
- 后续缺陷巡检若运行在 cloud mode，应以 `SessionStorage` 抽象、Web API history route 或 PG `cloud_sessions` 为会话真相源；本地 JSON / SQLite 只能作为 local mode 或迁移/回退证据。

## 验证

- 代码复核：`crates/hone-web-api/src/routes/chat.rs` -> `build_chat_sse(...)` -> `AgentSession::run(...)`。
- 代码复核：`crates/hone-channels/src/agent_session/core.rs` 的 `AgentSession::run(...)` 包含 `session.persist_user` 与成功路径 `session.persist_assistant`。
- 文档复核：`docs/repo-map.md` 记录 cloud mode 下 `SessionStorage::new_cloud` 使用 PG `cloud_sessions`；`docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md` 记录 `sessions_dir` / `sessions.sqlite3` 已从云模式本地 durable dependency 中移除。
- 未运行业务测试：本轮没有业务代码变更；原缺陷无法在不访问云 PG 权威会话存储的前提下本地复现。

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
