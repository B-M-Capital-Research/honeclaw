# Event Engine Earnings Quality Review

- title: Event Engine Earnings Quality Review
- status: archived
- created_at: 2026-05-08
- updated_at: 2026-05-08
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/pollers/earnings_surprise.rs
  - crates/hone-event-engine/src/pollers/earnings_quality.rs
  - crates/hone-event-engine/src/engine.rs
  - crates/hone-core/src/config/event_engine.rs
  - crates/hone-web-api/src/lib.rs
  - config.example.yaml
- related_docs:
  - docs/handoffs/2026-05-08-event-engine-earnings-quality-review.md

## Goal

用真实财报样本 POC 的结论改进 `EarningsReleased` 推送：FMP EPS surprise 只作为财报发布触发器和 LLM 输入；当存在近期 SEC 8-K 财报新闻稿上下文且 LLM 可用时，用综合财报指标判断即时推 / digest；LLM / SEC / 低置信失败时跳过 candidate，不再产出 EPS-only 推送。

## Scope

- 新增财报综合质量 review 配置、LLM prompt 与解析逻辑。
- `EarningsSurprisePoller` 在构造 EPS surprise candidate 后，best-effort 拉近期 8-K 上下文并调用 review，综合 GAAP/non-GAAP、EBIT/EBITA/EBITDA、现金流等非 EPS 指标。
- 只改变财报 surprise 事件的标题、摘要、payload 和严重度，不改 router policy、不改 digest 选择器。
- 不再产出单独 EPS 事件；不落地真实 LLM 输出、原始财报全文或临时 POC 脚本。

## Validation

- `cargo test -p hone-event-engine pollers::earnings_surprise`
- `cargo test -p hone-event-engine pollers::earnings_quality`
- `cargo test -p hone-event-engine --lib`
- `cargo test -p hone-core --lib`
- `cargo check -p hone-web-api`
- `rustfmt --check --edition 2024` on changed Rust files
- `git diff --check`

`cargo fmt --all -- --check` 当前仍被 unrelated formatting debt 阻塞，主要在 `bins/hone-cli/*`、`crates/hone-core/src/quiet.rs`、`crates/hone-event-engine/src/global_digest/fetcher.rs`、`crates/hone-event-engine/src/router/policy.rs`。本任务未格式化这些无关文件。

## Documentation Sync

- 本计划已从 `docs/current-plan.md` 移除并归档。
- 完成交接写入 `docs/handoffs/2026-05-08-event-engine-earnings-quality-review.md`。
- 历史入口写入 `docs/archive/index.md`。

## Risks / Open Questions

- `LlmProvider` 当前没有 JSON-mode 参数，生产 review 必须能解析失败并安全回退。
- FMP surprise 与 SEC 8-K 的时间关联是启发式，后续可复用 `SecFilingsPoller` 或 store cache 降低重复抓取。
- 真实财报质量判断仍依赖 press release 披露完整度；没有上下文时 candidate 会被跳过。
