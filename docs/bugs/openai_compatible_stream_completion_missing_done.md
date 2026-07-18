# Bug: OpenAI-compatible stream completion 未正常收口导致用户只收到通用失败

## 发现时间

2026-07-19 03:01 CST

## Bug Type

System Error

## 严重等级

P2

## 状态

Fixed

## GitHub Issue

无，修复提交已在同窗落地；当前不是活跃 P1。

## 证据来源

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-18 23:01-2026-07-19 03:01 CST。
  - `session_messages` 同窗新增 2 条 user / 2 条 assistant；两个 Web direct session 均以 assistant 收口，`last_message_role=user` 为 0。
  - `session_id=Actor_web__direct__codex-canary-crwv-nvda-rel-1784399897` 在 2026-07-19 02:38 CST 收到用户询问 `crwv和英伟达有什么关系`。
  - 该轮 assistant final 只返回产品化失败文案 `抱歉，这次处理失败了。请稍后再试。`，没有回答用户的关系说明。
- `data/runtime/logs/web.log.2026-07-18`
  - 2026-07-19 02:38 CST 同一 session 记录 `MsgFlow/web failed`，底层错误为 `LLM 错误: chat_with_tools stream ended before Done`。
  - 2026-07-19 03:00 CST heartbeat 窗口又有多条 Feishu / Web heartbeat 任务以同类 `chat_with_tools stream ended before Done` 落成 `runner_error` 并跳过发送；同批另有 1 条上游 `HTTP 529` provider 错误。
  - 同窗未见该原始错误进入用户可见 final；错误净化层仍生效。
- 最近非文档代码提交
  - 2026-07-19 02:56 CST `f959cecb fix: normalize compatible stream completion` 修改 `crates/hone-llm/src/openai_compatible.rs`、`crates/hone-llm/src/provider.rs` 与 `tests/regression/ci/test_finance_automation_contracts.sh`。
  - 该提交发生在 02:38 用户可见失败之后，标题与改动范围均指向 OpenAI-compatible stream completion 收口修复，因此本轮按代码级 `Fixed` 登记，后续仍需运行态复核是否不再复发。

## 端到端链路

1. Web direct 用户发起普通金融关系问答。
2. Function-calling runner 调用 OpenAI-compatible provider 执行带工具的回答流程。
3. provider 流式链路在完成前结束，runner 记录 `chat_with_tools stream ended before Done`。
4. Web 消息流把内部错误净化为通用失败 final。
5. 用户没有拿到原问题答案，只能稍后重试。

## 期望效果

- OpenAI-compatible stream 结束时应正确识别可用的 completion / final 状态，不应把已可收口或可重试的流式结束误判成整轮失败。
- 即使 provider stream 异常结束，也应有一次受控恢复或清晰失败分类，避免普通用户问题只得到通用失败。
- 原始 provider / runner 错误继续不得进入用户可见 final。

## 当前实现效果

- 修复前，Web direct canary 用户请求没有得到业务答案，只收到通用失败文案。
- 错误净化有效，用户没有看到 `chat_with_tools stream ended before Done` 原文。
- 同窗 03:00 heartbeat 批量出现相同错误，说明该问题不仅影响单条 Web direct，也可能让 heartbeat 覆盖跳过发送。
- 02:56 CST 已有代码提交针对 compatible stream completion 做规范化修复，本轮不将该缺陷列入活跃待修复。

## 用户影响

- 这是功能性缺陷，不是单纯文案问题：用户明确提出的问题没有被完成。
- 当前证据显示同窗仍有 Web direct 成功样本，且原始错误未外泄、无错投、无数据破坏、无全渠道不可用；因此定级为 P2，而不是 P1。

## 根因判断

- 直接根因是 OpenAI-compatible provider 的 tool stream completion 边界没有被稳定归一化，导致 runner 认为流式请求在 `Done` 前中断。
- 与 `openai_compatible_tool_call_protocol_mismatch_invalid_params.md` 同属 OpenAI-compatible provider 协议 / 收口缺陷族，但本轮底层错误不是 `tool call result does not follow tool call (2013)`，而是 stream completion 未正常闭合，因此单独建档。
- 与 `codex_acp_transport_disconnect_request_failure.md` 不同：本轮不是 Codex ACP transport 断连，而是 Hone 自有 OpenAI-compatible provider 流式收口失败。

## 下一步建议

- 后续巡检继续检索 `chat_with_tools stream ended before Done`，确认 `f959cecb` 后 live 运行态是否收敛。
- 若仍复发，优先补 provider 层流式事件状态机回归，覆盖 final chunk、finish reason、tool call 收尾与 Done 缺失的组合。
- 维持现有错误净化，禁止 runner 原始错误进入用户可见回复。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`data/sessions.sqlite3` 最近四小时消息收口、`data/runtime/logs/web.log.2026-07-18` 运行错误、最近四小时非文档代码提交。
