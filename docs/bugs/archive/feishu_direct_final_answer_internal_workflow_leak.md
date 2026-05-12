# Bug: Feishu 直聊最终答复混入内部任务计划与文档治理文本

- **发现时间**: 2026-05-06 10:04 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实会话：`data/runtime/logs/acp-events.log`
    - `session_id=Actor_feishu__direct__ou_5fb47bd113e7776b05e7a5c2c56e310652`
    - `2026-05-06T01:22:09.819501+00:00`（北京时间 `2026-05-06 09:22:09`）用户提问：`asts 最近股价跌`
    - 同轮 `tool_call_update` 全部落成 `completed`，最终 `stopReason=end_turn`
    - 但用户可见 final 首段仍写出：`我先把任务计划压缩成当前会话 todo... 文档方面只在结论有长期变化时更新公司画像，否则说明无需更新，不落盘到 current-plan...`
    - 这类 `todo` / `current-plan` 口径属于 agent 内部任务治理文本，不应出现在用户最终答复
  - 最近一小时真实会话：`data/runtime/logs/acp-events.log`
    - `session_id=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`
    - `2026-05-06T01:28:15.516411+00:00`（北京时间 `2026-05-06 09:28:15`）用户提问：`请详细分析下 KOPN`
    - 同轮 `16` 次 `tool_call_update` 均已完成，最终 `stopReason=end_turn`
    - final 首段仍写出：`我会先按当前时间核验 KOPN... 再检查本地是否已有相关公司画像；这类单股深度分析不需要落盘到动态计划...`
    - 后文虽给出完整分析，但首屏先暴露内部执行顺序、画像检查与动态计划治理判断
  - 最近一小时真实会话：`data/runtime/logs/acp-events.log`
    - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`
    - `2026-05-06T01:52:07.704217+00:00`（北京时间 `2026-05-06 09:52:07`）用户提问：`市场为什么一直打压软件公司`
    - 同轮 `6` 次 `tool_call_update` 均已完成，最终 `stopReason=end_turn`
    - final 首段仍写出：`我先对齐今天的市场口径，再把“软件被压”的原因拆成...`
    - 该样本未泄露 `current-plan`，但继续暴露了明显属于内部执行顺序的回答草稿式前言
  - 对照缺陷：
    - [`docs/bugs/feishu_direct_partial_reply_before_tool_completion.md`](../feishu_direct_partial_reply_before_tool_completion.md)
    - 上述旧缺陷已在 `2026-04-27` 修复为“未完成工具不得收口成功”；本轮样本里工具均已完成，说明这是独立的残留质量问题，而不是旧缺陷回归

## 端到端链路

1. Feishu 直聊用户发起正常研究/解释类问题。
2. 搜索、行情、本地画像等工具链正常完成，ACP 事件最终返回 `stopReason=end_turn`。
3. Answer 阶段生成了可交付正文，但把面向 agent 的执行计划、画像维护判断或动态计划治理口径一起拼进 final 首段。
4. 用户虽拿到了主体结论，却先看到 `todo`、`不落盘 current-plan`、`先检查本地画像` 这类内部工作流文本。

## 期望效果

- 用户最终看到的应是面向用户的结论、事实、判断和建议，不应夹带 agent 自用的任务治理或落盘策略说明。
- 若确实需要交代分析方法，也应转译成用户可理解的简洁说明，而不是暴露 `todo`、`current-plan`、`画像落盘` 等内部协作术语。
- 在工具已完成的成功链路里，final 首段应保持产品化表达，而不是工作稿口吻。

## 当前实现效果

- 最近一小时至少 3 条 Feishu 直聊成功会话都把内部执行顺序带入最终答复首段。
- `ASTS` 与 `KOPN` 两个样本最明显，直接出现 `当前会话 todo`、`不落盘到 current-plan`、`检查本地是否已有相关公司画像` 等内部流程词汇。
- `软件公司为何被打压` 样本虽然没有出现文档治理术语，但仍以“我先对齐口径、再拆四条线”的工作稿式前言开头，说明问题不止单条 prompt 偶发。
- 与“半成品提前收口”不同，本轮样本的主体答案基本完整，问题集中在 final 开头混入了不该对外暴露的内部协作语域。

## 用户影响

- 这是质量性 bug，不影响主功能链路，因此定级为 `P3`。
- 用户仍能拿到主体结论，没有出现错投、无回复、链路中断、数据写坏或任务未完成。
- 但内部工作流文本会显著降低产品专业感，也会让用户误以为系统把执行草稿、文档治理规则或维护动作直接当成正式答复的一部分。

## 根因判断

- Answer 阶段缺少对“内部执行计划语句”的最终净化或重写，导致模型把 agent workflow 指令显式复述给用户。
- 这更像成功链路的输出风格污染，而不是工具状态机错误：从 ACP 事件看，相关工具都已完成，没有 `unfinished tool` 或 `send_failed` 迹象。
- `todo`、`current-plan`、`画像落盘` 这类术语出现在用户 final 中，说明内部协作规则与对外答复边界仍未完全隔离。

## 下一步建议

- 在成功 answer 出站前增加一层轻量净化，拦截或改写 `todo`、`current-plan`、`不落盘`、`公司画像` 这类明显面向内部协作的表达。
- 检查触发公司画像维护/动态计划判断的共享提示词，避免模型把“我将如何工作”直接当成给用户的首段。
- 后续巡检重点看：
  - Feishu 直聊长答是否还会以 `我先...`、`当前会话 todo...`、`不落盘...` 开头
  - 同类污染是否已经扩散到 Web / Discord / scheduler 成功链路

## 修复记录（2026-05-06 15:08 CST）

- 状态更新为 `Fixed`。
- 在 `crates/hone-channels/src/runtime.rs` 的共享 `sanitize_user_visible_output(...)` 增加成功答复首段/首句工作流前言剥离：
  - 命中 `todo`、`current-plan`、`动态计划`、`不落盘`、`任务计划`、内部工作流等明显 agent 协作术语时，会移除首段并保留后续正式答案。
  - 对 `我先/我会先/先...再...` 这类执行步骤式开头，若后面还有实质答案，会移除该工作稿前言。
  - 保留 `我先给结论`、`核心判断`、`直接说` 等用户可见结论型开头，避免误删正常表达。
- 该修复位于共享净化层，覆盖 Feishu direct final，也同步保护 Web / Discord / Telegram 等复用同一最终输出净化的成功链路；未改模块边界、入口或长期运行方式。
- 关联 GitHub Issue：无。
- 验证：
  - `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --check crates/hone-channels/src/runtime.rs`
  - `git diff --check`
  - `bash scripts/ci/check_fmt_changed.sh` 未通过：当前 macOS 系统 Bash 3.2 缺少 `mapfile`，脚本在执行前置阶段退出；本轮已用定向 `rustfmt --check` 覆盖改动文件格式。
