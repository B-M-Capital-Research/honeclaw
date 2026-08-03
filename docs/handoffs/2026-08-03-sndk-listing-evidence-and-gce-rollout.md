- title: SNDK current-listing evidence and GCE rollout
- status: in_progress
- created_at: 2026-08-03
- updated_at: 2026-08-03
- owner: Codex
- related_files: `crates/hone-tools/src/data_fetch.rs`, `agents/function_calling/src/lib.rs`, `crates/hone-channels/src/prompt.rs`, `skills/stock_research/SKILL.md`, `soul.md`, `docs/invariants.md`
- related_docs: `docs/current-plans/ticker-resolution-architecture.md`, `docs/runbooks/backend-deployment.md`
- related_prs: none

## Summary

一次 production-like 的“用户问 SNDK 财报前瞻”会话中，DataFetch 已成功取得 SNDK 当前行情，但回答仍依据陈旧模型记忆声称 SNDK 没有上市，并错误引导用户查看 WDC。根因不是 ticker 没被识别或没有 data fetch，而是结构化结果没有携带明确的“当前上市”结论，Agent prompt 也没有规定当前 provider 证据与历史并购记忆冲突时的优先级。

当前 provider 的 exact search、quote 与 profile 均把该证券识别为 `SNDK` / Sandisk、NASDAQ 上市、正价格且 `isActivelyTrading=true`。历史事实也已变化：Western Digital 在 2025-02 完成 Flash 业务分拆，Sandisk 随后以 `SNDK` 在 Nasdaq 开始 regular-way trading。因此，2016 年收购事实不能作为 2026 年“未上市”的依据。

## What Changed

- `snapshot` 与 `earnings_outlook` 都返回 `hone_security_listing_evidence`，并在财报前瞻中同时抓取 profile。
- 只有同代码 quote/profile、正价格、非空交易所及 provider 明示 active 时才输出 `active_listing`；显式 inactive 仅在无正价格且同代码 profile 明示 inactive 时输出，其余情况一律 `unverified`。
- Agent、channel prompt、soul 和 stock-research skill 明确规定：当前同代码 `active_listing` 证据优先于陈旧并购、退市或改名记忆；若当前官方 filing 与 provider 冲突则继续核验并披露冲突。
- 增加 SNDK snapshot、SNDK earnings-outlook 与 quote/profile 冲突回归，避免用单一弱信号断言上市状态。

## Verification

- Passed: SNDK snapshot/outlook, conflicting listing evidence, snapshot aggregation, partial earnings-outlook coverage, Agent prompt, and channel soul-contract focused Rust tests.
- Passed: `cargo check -p hone-tools -p hone-agent -p hone-channels --tests`.
- Passed: all 44 finance contract checks, changed Rust-file `rustfmt --check`, and `git diff --check`.
- Pending: immutable GCE release build, two zero-active-chat checks, service switch, `/api/meta`, cloud authority, public auth boundary, and SNDK production canary.

## Risks / Follow-ups

- `active_listing` is intentionally conservative and provider-derived; absence of the marker must not be interpreted as delisted.
- A future provider schema change could omit `exchange` or `isActivelyTrading`; the result will degrade to `unverified` instead of making a false claim.
- Rollout must preserve the previous immutable GCE release symlink for immediate rollback.

## Next Entry Point

Run the focused regressions from the isolated worktree, push the exact commit, then follow `docs/runbooks/backend-deployment.md` for a zero-active-chat GCE release switch and canary verification.
