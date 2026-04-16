# Bug: Feishu 用户触发日对话额度上限后仍只收到通用失败文案，且最新 user turn 不落库

- 发现时间：2026-04-16 15:52 CST
- Bug Type：Business Error
- 严重等级：P1
- 状态：New

## 证据来源

- 会话库：
  - `data/sessions.sqlite3`
  - session_id：`Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
- 会话快照：
  - `data/sessions/Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15.json`
- 额度快照：
  - `data/conversation_quota/feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15/2026-04-16.json`
- 运行日志：
  - `data/runtime/logs/web.log`
  - `data/runtime/logs/hone-feishu.release-restart.log`
  - 最近一小时新增复现：
    - `2026-04-16 22:22:06.869` `web.log` 记录 `step=message.accepted user=+8613121812525 text_chars=16`
    - `2026-04-16 22:22:07.978` 同一 session 紧接着只出现 `step=reply.placeholder`
    - 会话库对应新增的最新消息仍只有 `2026-04-16T22:22:07.994063+08:00` assistant 失败兜底文案，新的 user turn 没有入库
    - `data/conversation_quota/feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15/2026-04-16.json` 仍显示 `success_count = 12`、`in_flight = 0`

## 端到端链路

1. Feishu 直聊用户 `+8613121812525` 在 2026-04-16 15:44 左右连续发送了短文本消息，用户侧观察到“hi”“你好啊”后直接收到“抱歉，这次处理失败了。请稍后再试。”。
2. `web.log` 记录该会话最新一条消息仅到：
   - `2026-04-16 15:44:17` `step=message.accepted`
   - `2026-04-16 15:44:18` `step=reply.placeholder`
3. 同一时间窗里没有出现该 message_id 对应的：
   - `session.persist_user`
   - `agent.prepare`
   - `agent.run`
4. 会话库最终只新增了一条 assistant 失败兜底消息，时间为 `2026-04-16T15:44:18.359632+08:00`，正文为“抱歉，这次处理失败了。请稍后再试。”。
5. 同一 actor 的额度文件显示：
   - `quota_date = 2026-04-16`
   - `success_count = 12`
   - `in_flight = 0`
6. `crates/hone-channels/src/agent_session.rs` 中 `reserve_conversation_quota()` 在超限时会直接返回：
   - `已达到今日对话上限（...），请明天再试`
   且该分支发生在 `session.persist_user` 之前。
7. 最近一小时同一 actor 再次复现相同形态：
   - `2026-04-16 22:22:06` 新 text message 已被 handler 接收
   - 但之后仍只看到 placeholder 与通用失败兜底，既没有新 user turn 落库，也没有进入 `agent.prepare / agent.run`
   - 说明这不是 15:44 的一次性事件，而是 quota 触顶后直到夜间仍会稳定复现

## 期望效果

- 当用户触发当日对话额度上限时，渠道应明确返回“已达到今日对话上限，请明天再试”或等价的明确说明，而不是误导性的“稍后再试”。
- 即使因为 quota 被拒绝，本轮用户输入也应至少保留可审计痕迹，避免支持侧无法从 session 里还原用户到底发了什么。
- 前端/渠道文案应能区分：
  - 真正的临时系统失败
  - 业务规则拒绝（如 quota、权限、白名单）

## 当前实现效果

- 用户命中 quota 后，外层被统一收口成通用失败提示“抱歉，这次处理失败了。请稍后再试。”。
- 这会让用户误以为系统临时故障，并继续重试。
- 由于 quota 检查发生在 `session.persist_user` 之前，最新 user turn 不会进入 session 历史；当前只剩 assistant 失败兜底被落库。
- 支持和排障侧看到的表象会更像“runner / 搜索 / Feishu 链路异常”，而不容易第一时间识别为业务限制。
- 到 `2026-04-16 22:22` 的最新样本，这条缺陷仍未收口：同一用户的新文本消息再次只留下 placeholder 与通用失败文案，session 历史里仍缺失对应 user turn。

## 用户影响

- 受影响用户会被稳定阻断，且不知道真实原因，无法自行判断是等待次日、申请提额还是继续重试。
- 重试只会重复看到同一通用失败文案，造成明显挫败感和误导。
- 由于最新 user turn 未落库，排障与客服支持会丢失关键信息，容易把 quota 问题误归因为系统故障。

## 根因判断

- 根因一：`AgentSession::run()` 在 `reserve_conversation_quota()` 被拒绝时直接 `fail_run()`，该路径发生在 `session.persist_user` 之前，因此用户消息不会写入会话。
- 根因二：Feishu handler 当前对 `response.success == false` 统一使用通用失败文案收口，没有把业务拒绝类错误映射成用户可理解的专用提示。
- 根因三：当前外层可观测性对“系统失败”和“业务规则拒绝”的区分不够，导致真实原因被掩盖。
- 最近一小时的 `22:22` 复现说明问题不依赖特定消息内容或单次 quota 文件脏状态；只要当日额度已达上限，这条链路就会继续稳定误报成通用失败。

## 下一步建议

- 为 quota 拒绝新增用户态专用文案，并在 Feishu/Discord/Telegram 等渠道统一映射，避免继续落到“稍后再试”。
- 在 quota / 权限 / 白名单等前置拒绝分支补最小 user-turn 审计落库，至少保留原始用户输入与拒绝原因。
- 为该类前置拒绝补结构化日志字段，例如 `failure_kind=quota_rejected`，便于监控和 bug 巡检直接聚类。
- 回归验证：
  - 构造已达 `DAILY_CONVERSATION_LIMIT=12` 的 actor
  - 发送短消息如“hi”
  - 断言用户收到明确 quota 提示，而不是通用失败文案
  - 断言本轮 user turn 可在 session storage 中被检索到
