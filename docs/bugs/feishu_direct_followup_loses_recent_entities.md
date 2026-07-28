# Bug: Feishu direct 连续追问丢失上一轮明确实体上下文

- 发现时间：2026-07-29 02:01 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-07-28 22:00-2026-07-29 02:01 CST。
  - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`。
  - `ordinal=19` / `2026-07-28T23:46:38.893732+08:00`：用户明确询问 `META`、`MSFT`、`STX`、`BE` 的财报日期和盘前 / 盘后时段。
  - `ordinal=20` / `2026-07-28T23:47:33.790431+08:00`：assistant 已复述并确认这四个标的，但没有给出具体时间。
  - `ordinal=21` / `2026-07-28T23:49:05.501059+08:00`：用户追问“上面四家公司的最近一次财报发布分别是什么时间，精确到小时”。
  - `ordinal=22` / `2026-07-28T23:49:36.252765+08:00`：assistant final 却回复“当前消息中未提及具体公司名称或代码，无法确认您指的是哪四家公司”，要求用户重复说明。
- 同窗其它 Feishu direct / scheduler 仍有正常收口样本；未见全渠道不可用、错对象投递、敏感凭据泄露或 P1 级链路停摆。

## 端到端链路

1. Feishu direct 用户在同一会话中围绕四个明确 ticker 连续追问财报时间。
2. assistant 上一轮已经确认并复述 `META`、`MSFT`、`STX`、`BE`。
3. 用户使用自然语言指代“上面四家公司”继续追问。
4. 系统没有消费最近一轮 assistant / user 上下文，把 follow-up 当成缺少实体的新问题。
5. 用户没有得到财报精确时间，被迫重复 ticker。

## 期望效果

- Feishu direct 应在同一会话连续追问中保留最近明确实体集合。
- 当用户说“上面四家公司”时，应解析为上一轮已确认的 `META`、`MSFT`、`STX`、`BE`，继续查询或给出可用答案。
- 如果上下文不确定，也应引用已看到的候选实体并请求确认，而不是直接声称当前消息没有公司名称。

## 当前实现效果

- 系统丢失或没有使用最近 turn 中已确认的四个 ticker。
- assistant 要求用户重复输入已经在上一轮明确过的信息。
- 这不是单纯措辞偏好：用户的连续追问没有完成，业务链路被一次无效澄清打断。

## 用户影响

- 用户需要重复输入 ticker 才能继续同一个财报查询任务。
- 对短间隔 follow-up 的可靠性下降，尤其影响财报、持仓和行情这类多轮精确查询。
- 定级为 P2：这是 Feishu direct 主链路中的任务完成问题，但当前证据是单会话单轮复发，同窗其它会话可收口，未见全渠道停摆、错投、数据破坏或敏感信息泄露，因此不升 P1。

## 根因判断

- 初步判断是 Feishu direct 当前轮 prompt / 历史拼装没有稳定保留最近实体指代，或 answer 层在工具失败 / 查询降级后只看当前用户文本。
- 该问题不同于 `web_direct_consecutive_user_turn_drops_previous_request.md`：本轮 user turn 之间有 assistant 收口，症状不是上一条 user 静默悬空，而是当前 follow-up 无法解析上一轮已确认实体。
- 该问题也不同于 `feishu_direct_partial_reply_before_tool_completion.md`：本轮 `ordinal=22` 是完整澄清回复，但业务判断错误；后续 `ordinal=24` 的半成品工具进度另归入该既有缺陷。

## 下一步建议

1. 为 Feishu direct 增加最近实体 / 最近答复摘要的回归：上一轮确认 ticker 后，下一轮“上面几家 / 这几只”必须解析到相同实体集合。
2. 检查自动 compact、skill snapshot 和 scheduler 注入消息是否会稀释最近普通对话 turn。
3. 对无法解析指代的澄清文案增加防御：先列出最近候选实体并让用户确认，避免直接否认上下文。
