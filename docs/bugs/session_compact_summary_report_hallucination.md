# Bug: 会话压缩摘要幻觉生成“用户报告”并回灌正式回答

- **发现时间**: 2026-04-15
- **Bug Type**: Context Corruption / Answer Quality
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 会话: `Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb`
  - 最近一小时复现会话: `Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
  - Prompt audit: `data/runtime/prompt-audit/feishu/20260415-171407-Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb.json`
  - LLM audit: `data/llm_audit.sqlite3`
  - 运行日志: `data/runtime/logs/web.log`

## 端到端链路

1. 这条 Feishu direct session 在 2026-04-15 17:14:07 命中自动压缩。
2. `SessionCompactor` 没有只总结“旧消息”，而是把整个 active window 连同最后一条新的 Rocket Lab 用户问题一起送给压缩模型。
3. 压缩模型没有输出“历史摘要”，而是直接生成了一整篇 `Rocket Lab (RKLB) 全面深度分析` 长文，并在文中编入 `22至25 美元`、`2025 年底首飞`、`FY2025 收入 9.5-10 亿美元` 等未验证数字。
4. 系统随后把这段压缩结果以 `role=user` 的 `【Compact Summary】...` 写回会话。
5. 17:15:10 与 17:16:57，回答链路又因为 `context window exceeds limit (2013)` 触发 `context_overflow_recovery` 强制压缩，进一步把这份伪摘要固化进会话上下文。
6. 最终 17:21:59 的正式回答阶段把该伪摘要当成“用户已提供的报告/原始请求”，于是出现“报告中假设的 22至25 美元”“报告遗漏了……”这类错误引用。

## 期望效果

- 会话压缩只应总结已有历史，不应把本轮最后一个未回答问题当作自由发挥的答题对象。
- `Compact Summary` 应被明确标识为系统内部压缩产物，而不是长得像“用户提供的材料”。
- 回答阶段不应把压缩摘要解释为用户上传报告、用户笔记或外部附件。

## 当前实现效果（问题发现时）

- 压缩模型实际使用的是 `llm.auxiliary.model = MiniMax-M2.7-highspeed`，而不是主对话模型。
- 2026-04-15 17:14:07 的自动压缩记录显示 `active_messages=26`、`trigger=auto`，已经满足 direct session 自动压缩条件。
- 2026-04-15 17:15:47 的恢复压缩记录显示 `trigger=context_overflow_recovery`、`forced=true`，会在上下文溢出后再次强制压缩并重试。
- 压缩结果被写回为 `role=user` 的 `【Compact Summary】...`，后续 prompt 组装与 multi-agent answer 会直接看到这段内容。
- 最终回答引用了压缩摘要中的伪“报告假设”，但用户本轮没有上传任何报告文件，也没有在真实历史里提供 `22至25 美元` 这一数字。

## 当前实现效果（2026-04-15 23:56-23:58 最近一小时复核）

- 同类问题已在另一条 Feishu direct 会话再次复现，说明这不是单次压缩偶发，而是当前 active window 压缩链路仍会持续污染后续回答：
  - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
  - `2026-04-15T23:56:57.340991+08:00` 用户真实输入只有：`rklb呢 马上SpaceX上市 那rklb估值是不是应该会高一些`
  - `2026-04-15 23:56:57.348` `web.log` 记录：`Compressing session ... with 27 messages (~59763 bytes)`
  - `2026-04-15 23:57:23.713` 会话被自动 compact，随后同一时刻写回一条 `role=user` 的 `【Compact Summary】...`
- 这次 `Compact Summary` 仍然不是对旧历史的中性摘要，而是直接生成了一整段带明确结论的 RKLB 投研文本，例如：
  - `SpaceX IPO对RKLB的估值拉动效果有限且逻辑存在误区，不建议以"SpaceX影子股"逻辑建仓RKLB`
  - `Rocket Lab是美国纳斯达克上市的商业火箭公司，专注中小型卫星发射`
- 后续 answer 阶段没有重新完成独立判断，而是继续沿着这段伪摘要输出结论：
  - `2026-04-15T23:58:43.545035+08:00` assistant 回复直接从 `SpaceX的IPO不会系统性抬高Rocket Lab（RKLB）的合理估值` 起笔
  - 同轮日志显示搜索阶段 `tool_calls=0`，但 answer 阶段仍额外执行了 `hone_data_fetch`，说明它是在被 compact summary 污染后的上下文里继续补证，而不是纠正 compact summary 的语义
- 最近一小时这次复现和 17:14 那次事故虽然会话不同，但症状完全一致：新问题进入压缩窗口后，被系统以 `role=user` 的“摘要”形式提前回答，随后正式回答把它当成可信上下文继续展开。

## 已确认事实

- 本次事故里没有用户上传的 PDF / 图片 / 附件报告。
- 根目录 `data/uploads/feishu` 未发现这条消息对应上传物。
- actor sandbox 下 `data/agent-sandboxes/feishu/direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb/uploads/Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb/` 为空。
- `22至25 美元` 只出现在压缩摘要和最终污染后的回答里，未在这条会话之前的真实输入中找到来源。

## 触发条件

1. direct session active messages 超过 20 条，或 active 内容字节超过 80,000，触发自动压缩。
2. provider 返回 `context window exceeds limit` / `too many tokens` 等错误时，`AgentSession` 会额外触发一次 `context_overflow_recovery` 强制压缩并自动重试。
3. 当 active window 末尾恰好是一个新的深度分析请求时，压缩模型更容易从“总结历史”漂移成“直接回答最后的问题”。

## 用户影响

- 用户会被误导为“系统看到了一个我上传过的报告”，从而破坏对回答可信度的判断。
- 正式回答会把压缩幻觉当成事实背景继续扩散，导致二次污染。
- 在金融分析场景里，这类伪上下文会直接引入错误估值、错误时间线和错误事件判断，属于高风险质量故障。
- 之所以不是 `P3`，是因为问题并不只是“回答写得不够好”，而是系统内部压缩产物污染了真实会话上下文，后续工具调用与正式结论都会围绕伪上下文继续执行，已经影响主回答链路的正确性。

## 根因判断

1. `SessionCompactor` 当前总结的是整个 active window，而不是“将被裁掉的旧上下文”。
2. 压缩结果被存成 `role=user` 消息，语义上过于像用户自己提供的材料。
3. 回答链路没有对 `session.compact_summary` 做足够强的隔离或降权，导致 multi-agent search / answer 会把它理解成原始用户请求的一部分。
4. 压缩提示词只要求“总结历史”，但没有显式禁止“回答最后一个问题”或“生成新的投研报告”。
5. 最近一小时的第二次复现证明，即使没有再次命中 `context_overflow_recovery`，仅靠一次普通 auto compact 就足以把伪结论写回会话并污染后续 answer 阶段。

## 建议修复方向

1. 压缩时只喂给模型“准备被压掉的旧消息”，不要把最后一个未回答问题放进摘要输入。
2. `Compact Summary` 不再以 `role=user` 写回；至少应改为独立的系统内部消息类型，或在 prompt 组装层显式标记为非用户材料。
3. 在 answer/search 阶段对 `session.compact_summary` 做强提示：它只代表系统摘要，不代表用户上传报告、用户自述或外部证据。
4. 强化压缩提示词，明确禁止输出完整回答、完整报告、投资建议正文，要求只保留摘要结构。
5. 为“压缩摘要包含大量新事实/估值数字/完整报告标题”增加回归测试，覆盖 `role=user` 回灌与 `context_overflow_recovery` 双阶段路径。
