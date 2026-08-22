- title: 财报数字时效与准确性软核对
- status: archived
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-tools/src/data_fetch.rs`
- related_docs:
  - `docs/invariants.md`
  - `docs/handoffs/2026-08-22-market-data-source-priority.md`
  - `docs/archive/index.md`
- related_prs: none; local uncommitted change set

## Goal

当 Agent 准备在回答中引用营收、净利润、EBIT、EBITA、EBITDA、利润率、现金流等财报数字时，先核对最新已披露报告期、发布日期/截至日、年度/季度/TTM/forward 口径和关键数字一致性，必要时针对性查公司 IR 或监管文件。

## Completed Scope

- 全局投研提示、严格 function-calling 终稿提示、证券预加载财务上下文和 DataFetch 财务返回策略加入一致的软引导。
- 明确 EBITA、EBITDA、EBIT、营业利润不得互相代替；数字必须与报告期和口径一起使用。
- provider 窗口可能滞后或数字冲突时，优先执行一次定向官方来源复核；不可得时披露截至日、来源层级和具体缺口后继续回答。
- 未增加逐数字双来源规则、内容 validator、缺项拒答、反复重试或自动改写门禁。

## Verification

- `cargo test -p hone-tools financials_return_quarterly_statements_with_a_trailing_window`: passed.
- `cargo test -p hone-agent agent_owned_prompts_use_lowercase_tickers_and_natural_final_without_rewrite`: passed.
- `cargo test -p hone-channels finance_policy_prioritizes_structured_market_data_without_a_completion_gate`: passed.
- `cargo test -p hone-channels financial_report_guidance_checks_period_and_metric_without_a_refusal_gate`: passed.
- `cargo test -p hone-agent`: 153 passed.
- `cargo check -p hone-channels`: passed.
- Rust files formatted with repository rustfmt settings; `git diff --check`: passed.

## Documentation Sync

- 长期约束记录在 `docs/invariants.md`。
- 同日执行和验证结果追加到既有 `docs/handoffs/2026-08-22-market-data-source-priority.md`，没有创建碎片 handoff。
- 历史入口并入 `docs/archive/index.md` 的结构化行情优先条目。

## Risks / Follow-up

- “最新”必须指最新已披露且工具可见的报告期，不得暗示未发布季度。
- 官方二次核对不可得时应标明结构化 provider 口径与截至日，不因缺少第二来源拒绝整个回答。
- 上线后通过真实财报问答 canary 观察 Agent 是否同时给出数字、报告期和口径，以及是否只在必要时做定向复核。
