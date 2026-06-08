# Bug: Feishu 直聊列出定时任务时外露 `enabled=true` 实现字段

- **发现时间**: 2026-06-08 19:01 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 时间窗：2026-06-08 17:54-17:55 CST
  - session_id: `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
  - 用户消息摘要：用户询问“我有哪些定时任务”。
  - assistant final 摘要：回复成功列出 3 个启用中的定时任务，包括任务名称、ID、频率、上次运行和内容；末尾写出“这 3 个任务目前都是 `enabled=true`。”
- `data/runtime/logs/acp-events.log`
  - 同轮 ACP 事件以 `stopReason=end_turn` 收口。
  - 未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误。
- 本轮 2026-06-08 15:01-19:01 CST 复核：
  - `session_messages` 有 7 个 Feishu user turn 与 7 个 assistant final，均成对收口。
  - `cron_job_runs` 同窗无新增记录。
  - assistant final 污染扫描未命中空回复、本机绝对路径、raw tool 字段、思维痕迹、provider 原始错误、quota、panic 或 stream disconnect。

## 端到端链路

1. Feishu direct 用户在 p2p 会话中询问当前有哪些定时任务。
2. runner 读取任务状态并生成最终回复。
3. 回复通过 ACP stream 正常输出，并以 `end_turn` 收口。
4. 最终用户可见文本正确列出任务，但把内部布尔状态字段 `enabled=true` 原样放到自然语言结尾。

## 期望效果

- 用户查询任务列表时，回复应使用用户可理解的状态表达，例如“这 3 个任务当前均已启用”。
- 不应把 `enabled=true` 这类代码字段、配置字段或存储字段原样暴露给用户。
- 若需要展示任务状态，应保持全中文、业务语义化，并避免混入实现层 key/value。

## 当前实现效果

- 任务列表查询主链路成功，用户能看到 3 个启用中的任务。
- 最终结尾直接写出 `enabled=true`，暴露实现字段，且与前文中文业务表达风格不一致。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 本轮查询没有未回复、空回复、投递失败、错投、状态错乱或工具链失败证据。
- 用户仍能完成“查看定时任务”的主要目标，因此不影响主功能链路，按规则定级为 P3，而不是 P1/P2。
- 影响主要是专业感下降、回复格式不自然，以及把内部状态字段当作用户态文案外露。

## 根因判断

- 直接证据只能证明最终 answer 阶段把任务状态字段原样转述给用户。
- 初步判断是 cron/list 工具结果或上下文中包含 `enabled` 布尔字段，answer 阶段缺少用户态字段映射约束，导致模型把 key/value 直接拼进最终回复。
- 该问题不同于历史“我的定时任务”空回复缺陷：本轮主链路已恢复并正常列出任务，只剩用户态措辞污染。
- 该问题也不同于 scheduler skill 降级措辞外泄：本轮没有“当前运行器 / 技能未加载”等内部降级前言，而是任务状态字段外泄。

## 下一步建议

- 在 cron/list 工具展示层或 answer guidance 中把 `enabled=true/false` 映射为“已启用 / 已停用”，避免模型直接复述布尔字段。
- 对 Feishu direct “列出我的定时任务”增加一条回归样本：最终可见文本不应包含 `enabled=true`、`enabled=false` 或其它裸 key/value 实现字段。
- 后续巡检若只看到内部工具结果包含 `enabled`，但最终用户可见文本已自然化，不应继续补充本缺陷证据。

## 修复记录

- 2026-06-09 已修复：
  - 共享 `sanitize_user_visible_output(...)` 新增 `enabled=true/false` 到“已启用 / 已停用”的用户态映射，并把“这 3 个任务目前都是 `enabled=true`”归一成“这 3 个任务目前均已启用”。
  - 修复落在共享净化层，因此 Feishu/Web direct 与 scheduler 共用同一用户态文案边界。
- 2026-06-09 04:43 CST 追加 prompt 层防线：
  - `DEFAULT_CRON_TASK_POLICY` 已补充用户态任务状态措辞约束，要求列出/说明任务状态时不要直接复述 `enabled=true`、`enabled=false`、`bypass_quiet_hours=true` 等实现层 key/value，而要改写为“已启用 / 已停用 / 遵守勿扰 / 豁免勿扰”等自然语言。
  - 新增 `resolve_prompt_input_maps_cron_enabled_flags_to_user_language` 回归，避免仅依赖出站净化兜底。

## 验证

- `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
- `cargo test -p hone-channels resolve_prompt_input_maps_cron_enabled_flags_to_user_language --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 文档同步

- 已同步更新 `docs/bugs/README.md` 活跃表与已修复表。
- 本修复只调整共享用户态文案净化，不改变模块边界、长期约束或运行工作流，无需更新 `docs/repo-map.md`、`docs/invariants.md` 或新增 handoff。
