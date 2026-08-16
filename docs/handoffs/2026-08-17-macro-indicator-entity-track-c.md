# Macro Indicator First-Class Entity Track C Handoff

- title: Macro indicator first-class entity Track C
- status: blocked
- created_at: 2026-08-17
- updated_at: 2026-08-17
- owner: Codex
- related_files:
  - `crates/hone-core/src/macro_indicator.rs`
  - `crates/hone-core/src/lib.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-channels/src/agent_session/tests.rs`
- related_docs:
  - `docs/current-plans/macro-indicator-entity-2026-08-17.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
- related_prs: none; local commit is blocked by read-only linked-worktree Git metadata

## Summary

Track C is implemented in the worktree, but its PostgreSQL-backed acceptance and requested local
commit remain blocked by this session's sandbox. The shared macro dictionary lowers confidence
without deleting any security candidate. Macro spans no longer help satisfy the two-symbol cluster
quorum, and every matched plain candidate is tentative unless the user explicitly writes a
`ticker` / `股票代码` label.

## What Changed

- Added the shared `hone-core::macro_indicator` dictionary and independent source-span scanner.
  ASCII aliases are case-insensitive with word boundaries, Chinese aliases are direct substring
  matches, and the longest overlapping alias wins.
- Kept `SecurityIdentifierKind` unchanged. Macro scanning is separate from security syntax.
- Added only two confidence hooks to `plain_ticker_mentions`: macro spans do not count toward
  symbol-cluster quorum, and matched candidates become tentative unless explicitly ticker-labelled.
- Added five Track C regression tests: scanner boundaries/overlap, ADP collision plus explicit label,
  cluster quorum isolation, forced tentative without candidate deletion, and the two production
  prompt paths through `prepare_verified_investment_turn`.

## Verification

- New pure-logic regressions: `passed=4 failed=0`.
- Existing five protected regressions: `passed=5 failed=0`; none were edited.
- Mutation (a) disabled: `passed=0 failed=1`; NVDA was incorrectly settled by the PCE quorum.
- Mutation (b) disabled: `passed=0 failed=1`; PCE survived but was incorrectly non-tentative.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed.
- Required full test command compiled successfully, then stopped in `hone-channels` with cumulative
  `passed=824 failed=173 ignored=1`; the binary's final line was
  `644 passed; 173 failed; 1 ignored`. Every observed failure came from unavailable PostgreSQL.
- Explicit scoped `git add` failed before staging because the linked worktree's real Git index lives
  under the read-only main-worktree `.git/worktrees/honeclaw-l2`; no commit or push occurred.

## Risks / Follow-ups

- Rerun `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` with
  the documented PostgreSQL service live on port 5433. This is the only remaining Track C gate.
- Run the reviewed scoped commit from an environment that can write the shared Git common dir; do
  not use `git add .` and do not include files outside this handoff's `related_files` / `related_docs`.
- The four pre-existing macro keyword tables remain intentionally untouched; convergence belongs to
  a later task.
- Do not broaden this dictionary into a deny-list or use `collides_with_listing` to drop candidates.

## Next Entry Point

Start PostgreSQL per `docs/runbooks/local-postgres-development.md`, rerun the full gate, confirm
`macro_production_prompts_do_not_enter_security_resolution_or_error` passes, then create the scoped
Track C commit before merging.
