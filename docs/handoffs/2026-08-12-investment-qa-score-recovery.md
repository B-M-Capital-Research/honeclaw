- title: HONE 投资问答五题提分与 Luna 能力审计交接
- status: done locally; not deployed
- created_at: 2026-08-12
- updated_at: 2026-08-12
- owner: Codex

## Outcome

同一五题、同一评分卡复测为 432/500，平均 86.4，最低 80，5/5 通过。没有把用户提到的 70 分改为验收线。

## Implementation

- 修复 OpenAI-compatible 分片/并行工具调用归属和流结束规范化。
- 对投资问答服务端预载 `hari-invest` 与 `company-thesis-ratings`，并保留可审计加载记录。
- 无 FMP 时用 Nasdaq 精确代码行情 + SEC Company Facts；财报题额外前置读取最新 SEC 8-K/6-K/10-Q 及财报附件。
- 最新财报附件优先于滞后的 Company Facts；附件正文抓取覆盖到现金流、资本开支与资产负债表区域。
- 未经 IR/SEC 原文支持的第三方目标价、预测值和情景估值不得进入终稿。
- 研究资料库只有 ticker/topic 强匹配或用户明确要求检索资料库时才注入，普通“最新/分析”等词不再触发无关长报告。
- Luna 网关在尚未收到 HTTP 响应时的瞬时传输故障安全重试一次；响应或流已经开始后不重放。

## Verification

- `hone-llm`: 40 passed.
- `hone-tools`: 189 passed, 1 ignored.
- `hone-agent`: 152 passed.
- `hone-channels`: 798 passed, 1 ignored.
- 研究资料弱匹配回归：passed.
- Hari conversation contract 与 company research dialogue contract：passed.
- fixture JSON、参考脚本语法、Rust format 和 `git diff --check`：passed.

## Model conclusion

Luna 不是唯一根因。数据缺失、旧数据优先、工具分片、无关资料污染和网关断线共同造成首次低分。编排修复后 Luna 可达到当前 80 分门槛，但复杂多公司估值仍表现出更高延迟和工具冗余；生产可保留 Luna 作为成本档，并将高复杂度估值/跨公司决策路由到更稳定的高能力模型。
