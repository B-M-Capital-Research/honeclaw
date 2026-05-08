# Event Engine Earnings Quality Review

- title: Event Engine Earnings Quality Review
- status: done
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
  - docs/archive/plans/event-engine-earnings-quality-review.md
  - docs/archive/index.md
- related_prs: N/A

## Summary

AAOI / CAI / CRWV 真实样本 POC 显示，EPS-only surprise 对亏损或 EPS 接近 0 的公司会产生误导性百分比：例如 CAI `-0.00` vs `-0.02` 被写成 `+92.5%`，AAOI `-0.19` vs `-0.05` 被写成 `-280.0%`。本轮移除 EPS-only 财报推送：FMP EPS surprise 只作为财报发布触发器和 LLM 输入，只有综合 review 成功应用后才进入事件流。

## What Changed

- `EarningsSurprisePoller` 不再输出单独 EPS 事件；负数或近零 EPS 的安全文案仅作为内部 candidate / LLM 输入，不作为推送兜底。
- 新增 `pollers::earnings_quality`：用 `x-ai/grok-4.1-fast` 风格 prompt 综合收入、指引、backlog、GAAP/non-GAAP、EBIT/EBITA/EBITDA、现金流和风险，输出 `immediate / digest / suppress` JSON。
- review 只在 OpenRouter provider 可用且近期 SEC 8-K 上下文可抓取时调用；LLM、JSON 解析、FMP SEC lookup、SEC HTML fetch、低置信或 invalid route 都会跳过 candidate。
- engine 只有在 earnings quality review provider 装配成功时才启动 `earnings_surprise` poller，避免无输出时白耗 FMP 配额。
- 新增 `event_engine.earnings.quality_review` 配置，并在 web-api 装配独立 max token provider。

## Verification

- Passed: `cargo test -p hone-event-engine pollers::earnings_surprise`
- Passed: `cargo test -p hone-event-engine pollers::earnings_quality`
- Passed: `cargo test -p hone-event-engine --lib`
- Passed: `cargo test -p hone-core --lib`
- Passed: `cargo check -p hone-web-api`
- Passed: changed-file `rustfmt --check --edition 2024`
- Passed: `git diff --check`
- Not run live LLM: 本轮已有 POC 结果；默认验证不消耗 OpenRouter。
- Known repo-wide fmt status: `cargo fmt --all -- --check` 失败在 unrelated formatting debt，本轮未改无关文件。

## Risks / Follow-ups

- `LlmProvider` 仍没有 JSON-mode 参数；当前实现靠 strict prompt + robust parser + fallback。后续如果要提升稳定性，应给 provider 增加 per-call response format。
- SEC 8-K 选择现在是按 FMP 最近 8-K 时间窗口启发式匹配；后续可复用已有 SecFilingsPoller / EventStore cache，减少重复抓取和误配概率。
- 如果 OpenRouter 或 SEC 长时间不可用，财报发布类推送会整体缺席；这是产品选择，避免 EPS-only 噪声。
- 可以在后续真实推送复盘里把稳定样本固化成 earnings quality baseline，但不要存完整新闻稿正文或真实 export。

## Next Entry Point

从 `crates/hone-event-engine/src/pollers/earnings_quality.rs` 的 prompt / parser 开始；从 `crates/hone-event-engine/src/pollers/earnings_surprise.rs` 的 `apply_quality_review` 看 SEC 上下文抓取与 fallback 边界。
