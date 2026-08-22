- title: 明确公司投研的行情优先取证
- status: done
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-tools/src/web_search.rs`
  - `crates/hone-tools/src/registry.rs`
- related_docs:
  - `docs/invariants.md`
  - `docs/archive/plans/market-data-source-priority.md`
  - `docs/archive/plans/financial-report-data-verification-guidance.md`
- related_prs: none; local uncommitted change set

## Summary

明确公司或证券的交互式投研现在先把结构化行情作为第一事实来源：实体解析后优先读取 `snapshot`，不适用时组合 `quote/profile`，涉及盘前盘后时补 `extended_hours`；开放 Web 搜索随后补公告、关系、事件和因果。该顺序是 Agent 软引导和工具展示信号，不是缺数据即拒答的内容门禁。

## What Changed

- 全局金融提示、严格 function-calling 的发现/取证/自然收口提示，以及 DataFetch/WebSearch 工具描述统一表达 `search → snapshot/quote → Web` 的优先顺序。
- `ToolRegistry::get_tools_schema` 只调整相对优先级：`data_fetch` 在首位，`web_search` 在普通业务工具之后，其它工具之间保留原顺序；所有工具仍完整暴露，执行能力不变。
- 明确 provider 无覆盖或失败时不得反复补取、拒绝终稿或停止回答，继续用已有证据和公开来源并披露具体缺口。
- 严格 function-calling 的市场涨跌幅证据从 provider `changesPercentage` 改为服务端 `hone_change_basis.pct`；AAOI 回归固定 `129.10 → 124.82 = -3.32%`，拒绝把原始 `-3.46%` 当权威值。

## Follow-up: 财报数字时效与准确性软核对

- 回答引用营收、净利润、EPS、EBIT、EBITA、EBITDA、利润率、现金流或资产负债表数字前，Agent 先确认最新已披露的 `date` / `period`，并明确使用单季、`hone_ttm.period_ends` 还是 `hone_forward.forward_period_ends`。"最新" 不得指向尚未披露的季度。
- EBIT、EBITA、EBITDA 与营业利润视为不同指标，不得互相替代，也不得在来源没有给出时自行补算。季度、年度、TTM 与 forward 不得混用。
- 关键数字的 provider 窗口疑似滞后或来源冲突时，只做一次针对性的公司 IR、财报公告或监管文件复核。官方二次来源不可得时，标明截至日、来源层级和具体缺口后继续回答。
- 该行为通过全局提示、严格终稿提示、证券预取上下文和 `hone_financials_policy` 共同引导；没有新增逐数字双来源、内容 validator、缺项拒答、反复搜索或自动重写门禁。

## Verification

- `cargo test -p hone-tools data_fetch::tests`: 57 passed.
- `cargo test -p hone-tools web_search::tests`: 19 passed.
- `cargo test -p hone-tools registry::tests`: 5 passed.
- `cargo test -p hone-agent`: 153 passed.
- `cargo test -p hone-channels finance_policy_prioritizes_structured_market_data_without_a_completion_gate`: 1 passed.
- `cargo check -p hone-channels`: passed.
- Financial-report guidance targeted tests: DataFetch bundle 1/1, function-calling prompt 1/1, global channel policy 1/1, pre-turn financial guidance 1/1.
- `rustfmt --edition 2024 --config skip_children=true ...` and `git diff --check`: passed.
- Broader PostgreSQL-backed test subsets could not complete on this host: `scripts/dev_pg.sh up` reported Docker unavailable. Before the environment failure, `hone-tools` reported 169 passing / 26 PostgreSQL-dependent failures, and the broad `hone-channels prompt` filter reported 47 passing / 26 PostgreSQL-dependent failures.

## Risks / Follow-ups

- Stronger source ordering can add one model/tool round; preferring the aggregate `snapshot` is intended to contain that cost.
- Schema order is a model hint. It does not guarantee every provider/model will choose the preferred tool, so production acceptance should inspect actual traces rather than infer success from prompt text.
- Run three fresh canaries after deployment: a named-company overview, a two-company relationship question, and an AAOI-style current/after-hours move question. Confirm each trace resolves identity, attempts structured market data before Web, and still answers naturally when one provider component is unavailable.

## Next Entry Point

Start with `crates/hone-channels/src/prompt.rs` for cross-runner policy and `agents/function_calling/src/lib.rs` for strict fallback behavior. Do not convert the priority into a missing-data validator or forced retry loop.
