# Bug: Web / Feishu direct 回复开头外露 Codex 模型元数据 fallback 警告

## 发现时间

- 2026-06-20 15:03 CST

## Bug Type

- Business Error

## 严重等级

- P3

## 状态

- New

## GitHub Issue

- 无，非 P1

## 最新进展（2026-06-30 07:03 CST）

- 本轮 2026-06-30 03:00-07:03 CST 真实运行态确认同根复发，状态从 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3` 只读快照仍停在 2026-06-17，最近真实会话继续以 `data/runtime/logs/acp-events.log` 重构。
  - ACP 本窗可见 11 次 `session/prompt`、11 次 `stopReason=end_turn`、0 个未收口会话；用户可见 chunk 聚合里有 2 条 assistant final 开头直接拼入 `Model metadata for gpt-5.5 not found... Defaulting to fallback metadata...`。
  - 样本覆盖 Web direct session `Actor_web__direct__web-user-266454c88ed6`（2026-06-30 03:02 CST 左右）和 Feishu direct session `Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3`（2026-06-30 07:00 CST 左右），后续业务正文均正常收口。
- 查重结论：该问题与本文档既有根因一致；本轮多出 Web direct 样本，但受影响边界仍是 direct 用户可见输出净化，不新建重复文档，仅扩大标题范围。
- 用户影响：回复主体正常完成，没有错投、空回复、投递失败、数据破坏或主功能链路阻断证据。该警告暴露内部 runner / 模型元数据状态，降低专业感；因此仍为质量性 `P3`，非 P1，不创建 GitHub Issue。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-20 11:02-15:02 CST。
  - `data/sessions.sqlite3` 在同窗仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`，因此本轮以 ACP 流式事件重构用户可见 final。
  - ACP 本窗可重构 9 次 `session/prompt`、8 次 `stopReason=end_turn`、0 个 ACP response error；另有 1 条 13:52 CST Feishu direct prompt 在日志窗口内尚未看到终态。
  - 14:43 CST Feishu direct session `Actor_feishu__direct__ou_5fce891d255ae588dde3bd7b1494a28d1e` 的 MLCC 涨价周期回复以 `stopReason=end_turn` 收口。
  - 该 final 开头直接拼入 `Model metadata for gpt-5.5 not found. Defaulting to fallback metadata; this can degrade performance and cause issues.`，随后才进入“我先按 2018 年 MLCC 被动元件涨价周期核公开资料”的业务正文。
  - 同窗未见该样本包含空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径、思维痕迹、ACP transport trace 或 provider 原始错误。

## 端到端链路

1. 用户在 Feishu direct 里继续追问 2018 年 MLCC 涨价周期相关问题。
2. Codex ACP runner 初始化并完成回复生成。
3. runner 或客户端侧模型元数据 lookup 产生 fallback warning。
4. warning 没有留在内部日志，而是被拼到 assistant final 开头。
5. 后续业务正文正常输出，并以 `stopReason=end_turn` 收口。

## 期望效果

- 模型元数据缺失、fallback metadata、性能降级风险等 runner 内部警告应只进入日志 / 诊断台账。
- 用户可见回复应直接呈现 MLCC 涨价周期核验、价格单位口径、供需逻辑和投资风险，不应暴露模型目录或运行器元数据状态。

## 当前实现效果

- 用户可见 final 第一段是英文内部警告，业务正文被放在警告之后。
- 回复主体仍完成了 2018 年 MLCC 涨价周期分析、单位口径说明、渠道价与合约价区分和投资风险提示。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 本轮回复正常 `end_turn` 收口，用户问题得到回答，没有证据显示投递失败、会话错乱、数据破坏或主功能链路阻断。
- 该警告会降低回复专业感，并让用户接触到不应外露的 runner / 模型元数据状态。
- 因此按规则定级为 `P3`：不影响主功能链路，只影响用户可见输出边界和产品感，非 `P1/P2`。

## 根因判断

- 直接根因是 Codex ACP / runner 的模型元数据 fallback warning 被写入了 assistant 用户可见文本流。
- 该问题不同于 `feishu_scheduler_acp_transport_trace_exposed.md`：本轮不是 transport fallback 或 stream disconnect，而是模型元数据 fallback warning。
- 该问题也不同于 `feishu_direct_internal_runtime_progress_exposed.md`：本轮不是模型自然语言复述内部研究流程，而是英文运行器警告被拼接到 final。

## 修复记录

- 2026-06-21 19:09 CST 修复：
  - 共享 `sanitize_user_visible_output(...)` 新增模型元数据 fallback warning 剥离，覆盖 `Model metadata for ... not found`、`Defaulting to fallback metadata`、`this can degrade performance...` 等句族。
  - 回归样本确认警告删除后仍保留 MLCC 正文，不影响后续业务回答。
  - 验证：`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。

## 下一步建议

- 在 ACP runner 输出聚合或共享用户可见净化层中拦截 `Model metadata for ... not found` / `Defaulting to fallback metadata` 句族。
- 同时检查类似 runner warnings 是否可能以独立 chunk 进入 final，例如模型 registry、metadata fallback、performance degradation 或 capability fallback。
- 增加 Feishu direct 出站净化回归：当 final 以模型元数据 fallback warning 开头时，用户可见文本应剥离该 warning 并保留后续业务正文。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码，未运行代码测试。
- 已验证范围：SQLite 会话上界、ACP final 重构、用户可见污染扫描、web.log 错误类别复核和最近四小时非文档代码提交检查。
