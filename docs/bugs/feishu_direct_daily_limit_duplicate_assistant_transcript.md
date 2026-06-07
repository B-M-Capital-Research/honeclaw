# Bug: Feishu 直聊今日对话上限提示在会话历史中重复落库

## 发现时间

- 2026-06-07 23:02 CST

## Bug Type

- System Error

## 严重等级

- P3

## 状态

- New

## 证据来源

- `data/sessions.sqlite3`
  - 2026-06-07 20:57:39 CST，session `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3` 在用户发送 `message_id=om_x100b6d62e52c98a0b1cdf6c2a489edc` 后，连续落库两条 assistant 记录：
    - ordinal `680`：`[{"type":"final","text":"已达到今日对话上限（12/12，北京时间 2026-06-07），请明天再试"}]`
    - ordinal `681`：`[{"type":"text","text":"已达到今日对话上限（12/12，北京时间 2026-06-07），请明天再试", ...}]`
  - 两条 assistant 记录使用同一个 Feishu `message_id`，时间间隔约 270ms，内容语义完全相同，仅包装形态不同。
  - 最近四小时窗口 `2026-06-07 19:02-23:02 CST` 中，`session_messages` 有 14 个 Feishu user turn 与 15 个 assistant 记录；多出的 1 条 assistant 正是该 daily-limit final/text 双记录。
- 同表历史样本显示这不是单次孤例：
  - 2026-06-05 22:26 CST 同一 session 出现 ordinal `641` final 与 `642` text 两条同类 daily-limit assistant 记录。
  - 2026-06-03 19:39 CST 同一 session 出现 ordinal `604` final 与 `605` text 两条同类 daily-limit assistant 记录。
  - 2026-06-02 23:21 CST / 23:29 CST，session `Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c` 也出现同类 final/text 双记录。
- `data/runtime/logs/acp-events.log`
  - 最近四小时 Feishu prompt 均有 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、quota 原始错误或 panic。
  - 当前证据没有证明 Feishu 用户实际收到两次消息；已确认的是本地会话历史/镜像重复记录。

## 端到端链路

1. Feishu direct 用户发送普通直聊消息。
2. Hone 自身用户额度检查命中“今日对话上限”。
3. 系统生成一条友好的 daily-limit 提示。
4. 会话持久化同时写入 runner final 形态与 Feishu text 形态两条 assistant 记录，且两条记录绑定同一个用户 `message_id`。
5. 后续会话恢复、巡检统计或上下文拼装可能把同一额度提示视作两条 assistant 回复。

## 期望效果

- daily-limit 短路回复应只在会话历史中保留一条规范 assistant 记录。
- 如果系统需要同时保留“内部 final”和“已发送 Feishu text”两类事件，应在不同表或 metadata 中表达，不应在 `session_messages` 里形成两个用户可见 assistant turn。
- 后续上下文恢复、消息计数、巡检统计应能稳定看到 user/assistant 成对收口。

## 当前实现效果

- daily-limit 命中时，`session_messages` 中同一用户消息后会连续出现两条 assistant 记录。
- 两条记录内容相同但 JSON 包装不同，导致最近四小时统计出现 user/assistant 数量不对齐。
- 该问题不同于 `feishu_direct_codex_usage_limit_generic_failure.md`：本轮不是 Codex runner / upstream usage limit，也没有返回通用失败文案；Hone 自身 daily-limit 文案是清晰的，问题在重复落库。

## 用户影响

- 直接用户请求已经得到清晰的 daily-limit 提示，且当前没有证据证明 Feishu 端实际收到重复消息。
- 但会话历史中重复 assistant turn 会污染后续上下文、自动巡检统计与历史回放，可能让后续 agent 误判为重复回复或增加无意义上下文。
- 因为当前证据只影响会话历史质量和统计准确性，没有阻断 Feishu direct 主功能链路、没有投递失败、没有内部错误外泄、没有数据破坏，所以定级为 P3。

## 根因判断

- daily-limit 短路路径很可能既走了“持久化 final 回复”逻辑，又走了 Feishu 出站消息镜像写入逻辑。
- 普通 runner final 看起来不会普遍重复；本轮重复集中在 Hone 自身 quota/daily-limit 短路提示，说明该路径可能绕过了正常的 assistant turn 去重或规范化。
- 当前证据不足以确认是否同时发生了 Feishu 出站重复投递；需要在发送日志或 Feishu API 回执层继续核对。

## 下一步建议

- 检查 Feishu direct daily-limit / quota 短路分支，确认是否同时调用了 final persistence 与 outbound mirror persistence。
- 若 `session_messages` 是用户可见 transcript，应只保留一条 assistant 记录，并把 Feishu delivery metadata 合并进同一条记录。
- 增加回归：同一用户消息触发 daily-limit 后，`session_messages` 中只新增一条 assistant turn，且后续 user/assistant 计数保持成对。
- 若后续确认 Feishu API 实际发送了两次相同文本，应将严重等级从 P3 提升为 P2，并补充出站投递证据。
