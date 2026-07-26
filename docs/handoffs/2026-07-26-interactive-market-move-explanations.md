# Interactive Market-Move Explanation Reliability

- title: Interactive market-move explanation reliability
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-tools/src/data_fetch.rs`, `crates/hone-web-api/src/routes/public_finance_calendar.rs`, `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs: `docs/current-plans/ticker-resolution-architecture.md`, `docs/decisions.md#d-2026-07-26-03-hold-market-move-bodies-behind-a-mechanical-consistency-boundary`, `docs/decisions.md#d-2026-07-26-05-let-representative-broad-market-quotes-satisfy-a-narrow-evidence-floor`, `docs/invariants.md`, `docs/bugs/web_direct_terminal_prefix_mismatch_commits_generic_failure.md`
- related_prs: none; commits `27ea2f53`, `cd78375`, `f0281adb`, `e46a6bf4`, `ec06485a`, `12f1a924`, `4139e12c`, `84ca1f21`

## Summary

Recent Web and Feishu questions such as `美股为什么大跌`, `周五美股为什么暴跌`, and `HIMS周五为什么大跌` failed in several distinct ways: a valid answer was replaced by a generic research failure; the requested Friday was changed to another date; the civil weekday was wrong; absolute price change was presented as a percentage; ordinary quote metadata was described as a close or exchange; snippet-only secondary results were promoted into verified same-day causes; and repeated equivalent source searches exceeded the live latency target.

Exact production commit `84ca1f2114c059a157cd893c84067638c7618e84` closes this subphase. It preserves the requested date/scope, grounds broad-market premise checks in full structured quotes from multiple representative groups, validates quote fields mechanically, refuses unsupported definitive causes, and bounds the final research round. Four new live actors passed content, transport, persistence, latency, and cleanup acceptance.

## What Changed

- The exact finance header remains an irreversible visible boundary but no longer acts as an early stop. A later tool batch proceeds only after the complete registered/parseable/structurally-valid/read-only check; invalid or persistent work remains zero-execution fail-closed.
- Non-search DataFetch calls accept `query` as a ticker compatibility alias only on the same call as `identity_match=exact_symbol`. `ticker`, then `symbol`, retain precedence, and malformed higher-precedence values never fall through.
- A server-anchored broad-US-market move question can open its structural floor only through full `quote` or `snapshot` attempts for at least two distinct S&P 500, Nasdaq, Dow, or Russell groups. `quote_short` is excluded because it may omit percentage, exchange, and provider-time fields.
- The market-move consistency boundary now checks that percentage claims use `changesPercentage`, exchange claims match structured exchange evidence, and ordinary quote/provider times are not called closing prices or closing sessions.
- Snippet-only Web results (`full_page_content=false` or `kind=search_snippets`) cannot support a definitive same-day cause. When eligible original-page evidence is unavailable, the answer preserves verified move facts and says `原因本轮未完全核验`.
- Once two representative groups have actual verified full quotes and one source search has been attempted, the same Agent moves to a tools-disabled bounded final rather than repeating equivalent research.
- `sanitize_fmp_error` in the public finance calendar route no longer loops on its own `apikey=<redacted>` replacement. Before this fix, the loop consumed a Tokio worker at 100% CPU and could exhaust Web request capacity; the sanitizer now advances its cursor and has a mixed-key redaction regression.

No module boundary or primary data-flow boundary changed, so `docs/repo-map.md` did not require an update. The umbrella ticker plan remains active only for the separate scheduler task-prose/entity false-positive P2.

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  - Agent `134/134`
  - Channels `680/680`
  - Web API `133 passed, 2 ignored` credentialed cases
  - Hone Tools `143 passed, 1 ignored` optional matplotlib case
- `bun run test:web`: `286/286`
- `cd workers/public-community-edge && bun run typecheck && bun run test`: `45/45`
- `bash tests/regression/ci/test_finance_automation_contracts.sh`: `42/42`
- `bash tests/regression/run_ci.sh`
- changed-Rust formatting and `git diff --check`
- Exact immutable runtime `target/deploy-84ca1f21`:
  - source commit `84ca1f2114c059a157cd893c84067638c7618e84`
  - 504 payload files plus manifest metadata; every recorded hash verified
  - Web and Feishu processes run from the exact package
  - `/api/meta` reports authoritative cloud mode with healthy PostgreSQL and object storage
  - `/api/runtime/active-chat-runs` returned `0` after each canary
- Fresh production actors:
  - `codex-canary-84ca1f21-rumor-20260726103012`: `47.221s`; one source attempt plus full SPY/QQQ/DIA quotes; corrected the broad-crash premise and disclosed the cause gap
  - `codex-canary-84ca1f21-broad-20260726103012`: `54.458s`; full SPY/QQQ/IWM quotes; answered `美股为什么大跌` without a generic failure or invented cause
  - `codex-canary-84ca1f21-friday-20260726103012`: `45.597s`; preserved `2026-07-24（周五）` and corrected the false broad-crash premise
  - `codex-canary-84ca1f21-hims-20260726103012`: `58.917s`; preserved the single-stock scope and verified HIMS `-14.20%` without treating it as a broad-market move
- Every actor produced one start and one successful terminal, no reset/error/partial event, exact visible SSE/assistant-history equality, exactly two persisted history rows, and zero active chats after completion.

## Risks / Follow-ups

- The cause boundary can verify locality and field consistency, but it intentionally does not decide economic causality. When fetched same-day primary evidence is unavailable, an explicit evidence gap is the correct result.
- The configured Discord token is rejected by the gateway (`InvalidAuthentication`). Discord remains offline and isolated; Web and Feishu use separate processes from the same exact build so the invalid channel cannot terminate healthy roles.
- `target/deploy-4139e12c` is retained as the immediate generated rollback package. Older generated `target/deploy-ec06485a` and `target/deploy-12f1a924` directories were removed to recover disk space and can be recovered only by rebuilding their exact commits.
- Continue ordinary production sampling. Reopen the fixed bug if the same exact build again emits a generic failure, changes the requested date/scope, confuses quote fields, promotes snippet-only causes, or exceeds the bounded source-search behavior.

## Next Entry Point

For this completed subphase, start with `agents/function_calling/src/lib.rs` market-move consistency tests and this handoff. For remaining umbrella work, continue `docs/current-plans/ticker-resolution-architecture.md` at the scheduler `800G` / `NAND` / `AST` / `SEC` entity-guard P2; do not reopen this market-move work merely because that separate scheduler issue remains active.
