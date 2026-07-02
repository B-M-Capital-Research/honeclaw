# Bug: Feishu 启动恢复向已完成旧会话补发过期失败提示

## 发现时间

2026-07-02 15:01 CST

## Bug Type

System Error

## 严重等级

P2

## 状态

New

## GitHub Issue

无，当前不是 P1。

## 证据来源

- `data/runtime/logs/hone_cli_screen.log`
  - 2026-07-02 13:02:23 CST，Feishu 启动恢复记录发现 1 个中断会话并补发失败提示。
  - 2026-07-02 13:02:25 CST，同一恢复流程记录已向 `session_id=Actor_feishu__direct__ou_5f895bed1573d53053e89bfc382b523a44` 补发失败提示。
- `data/sessions.sqlite3`
  - 同一 session 的 `sessions.last_message_at=2026-06-17T05:01:56.178743+08:00`，`last_message_role=assistant`，`last_message_preview` 是 6 月 17 日 05:00 定时任务的正式 assistant final。
  - `session_messages` 中同一 session 最新真实消息仍是 2026-06-17 05:01 CST assistant final；其后没有 2026-07-02 用户 turn 需要恢复。
  - 这说明恢复扫描命中了一个约 15 天前已经以 assistant final 收口的旧 session，而不是最近 30 分钟内仍悬挂的 direct user turn。
- 查重：
  - `docs/bugs/archive/feishu_direct_answer_idle_timeout_no_reply.md` 记录的是 Feishu direct Answer 阶段失败后无最终回复，并在 2026-06-13 增加中断会话补偿。
  - 本轮不是“无回复”复发，而是补偿链路误选已完成 / 过期会话并发送失败提示；受影响链路、用户表现和修复方向不同，因此单独建档。

## 端到端链路

1. Feishu runtime 启动或恢复任务运行。
2. 恢复扫描尝试查找中断会话并补发产品化失败提示。
3. 扫描结果错误包含已在 2026-06-17 以 assistant final 收口的旧 session。
4. 系统在 2026-07-02 13:02 CST 向该旧 session 补发“服务重启 / 之前消息中断”类失败提示。

## 期望效果

- 恢复扫描只应处理最近 grace window 后仍 `last_message_role=user` 的 Feishu direct 会话。
- 已有 assistant final 收口的 session 不应再次收到失败补偿。
- 过期数天的旧 session 即使存在镜像导入噪声，也不应被识别为可恢复中断会话。

## 当前实现效果

- 2026-07-02 13:02 CST，恢复流程向一个 2026-06-17 已完成的旧 session 补发失败提示。
- SQLite 会话真相显示该 session 最新业务消息是 assistant final，不符合“仍悬挂 user turn”的恢复条件。

## 用户影响

- 用户可能在没有新请求的情况下收到迟到失败提示，误以为当前任务或旧定时任务刚刚失败。
- 这会干扰用户对任务状态和服务稳定性的判断，并可能造成重复追问或误操作。
- 当前证据集中在单个 Feishu session，且不是凭据泄漏、错对象投递、全渠道不可用或用户当前请求批量失败，因此定级为功能性 P2。

## 根因判断

- 启动恢复扫描的候选条件可能没有严格使用权威 `last_message_role=user + 最近时间窗 + active session lock` 组合过滤。
- `sessions.updated_at`、导入时间或旧 session 源文件状态可能被当成恢复候选依据，导致已完成旧会话进入补偿队列。
- 也可能存在 SQLite mirror 与 runtime session source 不一致，恢复流程读取的来源没有反映同一 session 已 assistant final 收口。

## 下一步建议

1. 检查 Feishu 启动恢复候选查询，强制要求 `last_message_role=user` 且 `last_message_at` 位于短时间窗内。
2. 恢复前再次读取 session tail，若最后一条是 assistant / final / failure text，则跳过补发。
3. 为“已完成旧 session 不补发失败提示”和“导入时间推进不等于用户消息悬挂”补回归测试。
4. 修复后复核 runtime 日志，确认启动恢复不会再命中过期已完成 session。
