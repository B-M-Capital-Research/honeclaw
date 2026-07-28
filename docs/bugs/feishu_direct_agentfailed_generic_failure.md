# Bug: Feishu direct 追问失败后只返回通用失败文案

- 发现时间：2026-07-28 10:02 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-07-28 06:02-10:02 CST。
  - `session_id=Actor_feishu__direct__ou_5f49e2e252460a05eee0ff98f685cf9f16`
  - `2026-07-28T09:10:15.947442+08:00` 用户给出富途新闻链接并要求分析开元大模型发展、性能、收入和估值；assistant 于 `09:11:27.779750+08:00` 正常返回长文对比。
  - `2026-07-28T09:35:59.446539+08:00` 用户继续追问 `Moonshot AI 商业模式和商业价值`。
  - `2026-07-28T09:36:46.837926+08:00` assistant final 只返回 `抱歉，这次处理失败了。请稍后再试。`。
  - 同条 assistant `metadata_json` 标记 `error_kind=AgentFailed`、`run_failed=true`。
- 2026-07-28 10:01-14:02 CST 复核继续确认同类可见失败：
  - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
  - `2026-07-28T10:30:34.750477+08:00` 用户问 `GLW应该现在加仓？还是等财报出来，结果分别是怎样的，股票怎么动？`
  - `2026-07-28T10:33:15.769625+08:00` assistant final 只返回 `抱歉，这次处理失败了。请稍后再试。`，同条 `metadata_json` 标记 `error_kind=AgentFailed`、`run_failed=true`。
  - 用户 10:35 重试后又触发 `feishu_direct_partial_reply_before_tool_completion.md` 记录的原始 `<function_calls>` 外泄；本单只记录首次 `AgentFailed` 通用失败表现，不重复登记 raw tool final。
- 本轮对照：
  - 同窗按真实 `timestamp` 有 33 条 user、20 条 assistant、8 条 system compact。
  - 最近 assistant 到 `2026-07-28T10:01:08.570449+08:00`，且普通 scheduler / Feishu direct 有多条正常收口样本。
  - 运行日志当前可用尾部停在 2026-07-10，未能从日志确认更细的 runner 错误根因；本单仅登记用户可见失败表现和元数据状态。

## 端到端链路

1. Feishu 用户在已有直聊会话中先完成一轮 AI 公司对比问题。
2. 用户继续追问 Moonshot AI 的商业模式与商业价值。
3. 直聊 runner 进入 agent 执行链路并失败，落库元数据标记 `AgentFailed`。
4. finalizer 只给出通用失败文案，没有消费上一轮已建立的上下文，也没有返回可用的部分分析。
5. 用户当前追问未完成，只能重试或改写问题。

## 期望效果

- 对同一主题的追问，应基于上一轮上下文和必要检索继续回答 Moonshot AI 的商业模式、收入路径、估值支撑和风险。
- 如果 runner 失败，应尽可能基于已有上下文给出受控的部分答案，或明确让用户缩小问题；不应只返回无信息量的通用失败。
- 原始内部错误仍应脱敏，不得外露底层 runner / provider 细节。

## 当前实现效果

- 用户收到的 final 只有通用失败文案，没有任何业务内容。
- 错误净化有效，未外露底层报错、路径、工具中间稿或思维痕迹。
- 但任务功能没有完成，且缺少可操作降级说明。

## 用户影响

- 用户明确提出的追问没有得到回答，当前会话需要重试。
- 这是功能性问答链路缺陷：不是单纯表达质量问题，而是 Feishu direct 当前任务失败。
- 未升级为 P1 的原因：本轮只确认单个 Feishu direct 追问失败，同窗其它 direct / scheduler 有正常 assistant 收口，未见全渠道不可用、错对象投递、数据破坏或敏感信息泄露。

## 根因判断

- 直接症状是 `AgentFailed` 后只进入通用失败 final，没有利用上一轮已有上下文进行受控恢复。
- 由于当前 runtime 日志未覆盖 2026-07-28 09:36 窗口，暂不能确认是 provider stream、tool-call protocol、context overflow、max iteration，还是其它 runner 失败。
- 与 `feishu_direct_internal_only_output_fallback.md` 不同：本轮没有外露 `internal-only output` 内部占位短语，而是脱敏通用失败；与 `feishu_function_calling_max_iterations_generic_failure.md` 相邻但暂缺 `max_iterations_exceeded` 证据，因此单独登记为 Feishu direct `AgentFailed` 通用失败恢复缺口。

## 下一步建议

- 优先补齐 direct `AgentFailed` 的细粒度失败分类落库，避免只能从用户可见 final 反推。
- 在 Feishu direct 失败 finalizer 中增加上下文内恢复：若上一轮同主题已形成结构化结论，追问失败时至少返回可用的短版续答或请用户缩小范围。
- 后续巡检若拿到更具体根因，应把本单归并到对应 provider / runner 缺陷，或在本单补充 root-cause 证据后继续修复。
