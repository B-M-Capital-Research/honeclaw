# Bug: Web direct compact 后把旧 ETF 话题混入当前 follow-up

## 发现时间

2026-07-22 19:02 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-07-22 15:02-19:02 CST。
  - `session_id=Actor_web__direct__web-user-f40ae1caa720`。
  - `2026-07-22T15:36:22.539320+08:00`，用户请求为 `159797` 制定 ETF 做 T 策略。
  - `2026-07-22T15:36:43.384961+08:00` 到 `15:36:43.385034+08:00`，同一轮自动 compact 写入旧话题 summary，并追加多条 skill context。
  - `2026-07-22T15:37:39.405496+08:00`，assistant 首轮正确围绕 `159797` 回答，识别为医疗器械 ETF。
  - `2026-07-22T15:39:29.360724+08:00`，用户补充“我已经有底仓”。
  - `2026-07-22T15:40:26.589753+08:00`，assistant follow-up 却写成“承接前面对 510880（华泰柏瑞上证红利ETF）做 T 策略的讨论”，把旧 compact summary 中的 `510880` 混入当前 `159797` 话题。
  - `2026-07-22T16:07:16.169448+08:00`，用户继续补充底仓数量、盯盘时间和收益目标。
  - `2026-07-22T16:07:32.218241+08:00`，assistant 又要求用户确认“510880、159797 还是其他”，虽然当前会话最近有效标的已经是 `159797`。
  - `2026-07-22T16:08:14.544635+08:00`，用户被迫再次输入 `159797`，`16:08:51` 后 assistant 才重新回到正确标的。
- 本窗质量对照：
  - 同窗 `session_messages` 新增 15 条 user / 12 条 assistant / 4 条 system compact，覆盖 6 个更新 session。
  - 未见长期 user-only 残留、错投、空回复、本机绝对路径、provider 原始错误、panic、`<think>` 或 raw tool JSON 进入 ordinary assistant final。
  - 因此本缺陷不是 Web direct 全链路不可用，而是 compact / skill context 恢复后的当前话题选择质量问题。
- 去重检查：
  - `web_direct_consecutive_user_turn_drops_previous_request.md` 覆盖的是同一 direct session 某个 user turn 没有 assistant 终态并被下一轮静默跳过，当前样本每轮都有 assistant 收口。
  - `feishu_direct_compact_retry_still_cannot_answer_new_topic.md` 覆盖的是 Feishu 旧会话 compact retry 后仍返回上下文过长失败，当前样本没有超窗失败，且发生在 Web direct。
  - 当前缺陷的独立症状是 compact summary / 旧话题在 follow-up 阶段压过了当前用户刚确认的标的。

## 端到端链路

1. Web direct 用户围绕 `159797` 请求做 T 策略。
2. 系统自动 compact 并恢复旧会话摘要，其中包含另一个 ETF `510880`。
3. assistant 首轮能正确回答 `159797`。
4. 后续用户补充底仓约束时，assistant 把旧 `510880` 话题当成当前上下文，或要求用户在 `510880` / `159797` 中重新确认。
5. 用户必须重复输入 `159797` 才能继续当前任务。

## 期望效果

- compact 后的会话恢复应优先保留最近用户显式指定的标的和当前任务目标。
- 当同一会话存在多个相近 ETF 主题时，follow-up 应使用最近已确认的 `159797`，不能被旧 summary 中的 `510880` 抢占。
- 如果确实存在歧义，应说明“我看到最近有 159797 和更早的 510880，默认继续 159797”，而不是直接按旧标的作答或要求用户重复确认。

## 当前实现效果

- assistant 在同一主题连续 follow-up 中先把 `159797` 漂移成 `510880`，随后又声称用户没有点名具体 ETF。
- 主消息链路正常收口，但回答没有可靠消费最近上下文，导致用户需要重复提供已给出的关键信息。

## 用户影响

- 用户本来在补充底仓和盯盘约束，期望继续完善 `159797` 做 T 策略；assistant 的上下文漂移打断了任务推进。
- 该问题不影响 Web direct 主功能链路：消息正常落库、assistant 正常回复，用户重新确认后可继续；未见错投、数据破坏、系统级未回复、内部路径或 raw tool 外泄。
- 因此这是质量性缺陷；由于不影响功能链路，定级为 `P3`。

## 根因判断

- 初步判断是 compact summary、skill context 恢复和最近对话窗口之间缺少“当前任务实体优先级”规则。
- 旧 summary 中的 `510880` 在 follow-up answer 阶段仍被视作强上下文，而最近 user turn 和上一轮 assistant 已确认的 `159797` 没有被稳定固定为当前 topic anchor。

## 下一步建议

- 在 Web direct 会话恢复后，为最近显式证券/基金代码建立短期 topic anchor，优先级高于 compact summary 中的历史实体。
- 对 compact 后相邻金融话题增加回归样本：旧 summary 包含 `510880`，当前用户问 `159797` 并补充“我有底仓”时，assistant 应继续 `159797`。
- 最终回复可在存在多标的历史时采用默认当前标的并轻量确认，避免把用户带回旧话题。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`data/sessions.sqlite3` 2026-07-22 15:02-19:02 CST Web direct 会话、`docs/bugs/` 既有文档去重检索。
