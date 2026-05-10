# Bug: Web direct quota rejection writes user turn without visible reply

- 发现时间：2026-05-10 23:10 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无

## 证据来源

- `data/sessions/Actor_web__direct__web-user-e05f5e5f74a3.json`
  - `2026-05-10T19:05:33.768892+08:00` user：`心跳检测，请简短回复 OK`
  - `2026-05-10T20:04:17.308557+08:00` user：`心跳检测，请简短回复 OK`
  - `2026-05-10T21:05:55.878963+08:00` user：`心跳检测，请简短回复 OK`
  - `2026-05-10T22:04:55.654009+08:00` user：`心跳检测，请简短回复 OK`
  - 这四条 user turn 后都没有对应 assistant final 或 quota 提示；同一 session 中间的 20:00 scheduler 任务仍正常生成并写入 assistant final。
- `data/runtime/logs/desktop_release_app.log`
  - `2026-05-10T14:04:55Z` 记录 Web direct 收到 `心跳检测，请简短回复 OK`。
  - 同一条随后记录 `step=session.persist_user ... detail=quota_rejected` 与 `recv ... input.preview="心跳检测，请简短回复 OK"`，但未见同 session 的 `session.persist_assistant`、`done success=false` 用户可见文案或 `reply.send` 等价收口。

## 端到端链路

1. Web direct 用户发送短心跳消息，期望收到 `OK`。
2. 会话层判定当日对话额度已触顶，走 `quota_rejected` 分支。
3. 系统把 user turn 写入 JSON 会话文件，但没有写入 assistant quota 提示。
4. 用户可见会话历史连续出现多条未回复 user 消息，表现为 Web direct 吞回复。

## 期望效果

- Web direct 额度触顶时，应明确向用户返回“今日额度已用完 / 已达到今日对话上限，请明天再试”等业务拒绝文案。
- 即使请求被 quota 拒绝，也应在会话历史中保留一条 assistant 业务拒绝消息，避免用户误判为系统卡住。
- 前端如果已经禁用发送，也不应让后端接受请求后只落 user turn。

## 当前实现效果

- 最近四小时真实 Web direct 会话里，至少四条短心跳请求只留下 user turn。
- 运行日志能看到 `quota_rejected`，说明系统知道拒绝原因，但用户可见 transcript 没有对应解释。
- 同窗 scheduler 任务仍能正常完成并写 assistant final，说明不是 Web 会话文件整体不可写，也不是全局 agent runner 停摆。

## 用户影响

- 用户发送消息后得不到任何可见反馈，会感知为 Web direct 卡住或吞消息。
- 由于 user turn 已落库但没有 assistant 拒绝文案，后续上下文里会残留多条未回答请求，影响支持排障与会话可读性。
- 定级为 `P2`：这是 direct 主链路的业务拒绝收口问题，会导致用户无法完成当前请求；但当前证据显示 user turn 已保留，且同窗普通 scheduler 任务仍可送达，没有证明跨渠道大面积不可用，因此不定为 `P1`。

## 根因判断

- 初步判断是 Web direct 的 quota 拒绝分支只执行了 `session.persist_user=quota_rejected` 审计落库，没有把业务拒绝文案持久化成 assistant turn 或返回给 Web transcript。
- 这与历史 Feishu quota 缺陷同属业务拒绝收口问题，但受影响渠道不同；Feishu 历史缺陷关注 placeholder 后无最终提示和 user turn 不落库，本单的最新坏态是 Web direct user turn 已落库但没有可见 quota reply。

## 下一步建议

- 复核 Web chat API 在 `AgentSession::run()` 返回 quota rejection 时是否把失败文本写入 response 和 session assistant turn。
- 为 Web direct 增加回归：quota 触顶后应返回明确 quota 文案，并在会话历史中写入 assistant 业务拒绝消息。
- 若前端根据 remaining quota 禁用发送，后端仍需保持一致的保护语义，防止自动心跳或绕过 UI 的请求制造无回复 user turn。
