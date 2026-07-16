# Investment Response Template Regression Repair

- title: Investment response template, current-data, and stream recovery repair
- status: in_progress
- created_at: 2026-07-17
- updated_at: 2026-07-17
- owner: Codex
- related_files: `soul.md`, `skills/stock_research/SKILL.md`, `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-web-api/src/routes/chat.rs`
- related_docs: `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`

## Goal

Restore the long investment prompt as an enforced runtime contract: server-owned Beijing data time first, entity-first exact security resolution, current same-symbol DataFetch quote and timestamp, asset-appropriate evidence, and the complete prior single-security / fund / crypto / market / sector response templates. Eliminate false current-data denial, fragile whole-answer retries, duplicate terminal streams, and refresh-time run loss.

## Scope

- Audit the regression commits and production RMBS / NBIS / INTL evidence.
- Make time, resolved entity, canonical quote, quote timestamp, and fact labels deterministic server output.
- Preserve a valid draft during format repair and prevent persistent operations from re-executing.
- Remove conflicting profile prices, entity-mismatched news, and ambiguous raw financial evidence.
- Cover lowercase/common tickers and broad market/sector queries without promoting theme acronyms.
- Rebuild, restart all runtime services, and run isolated real-data E2E checks.

## Validation

- Focused Rust unit tests for entity resolution, evidence routing, template validation, retry safety, and stream recovery.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `bash tests/regression/run_ci.sh`
- Live DataFetch and isolated Web E2E cases for RMBS, NBIS, INTL, crypto, market, and sector prompts.
- Full runtime restart plus `/api/meta` and active-run health checks.

## Documentation Sync

- Update `docs/current-plan.md`, `docs/invariants.md`, `docs/decisions.md`, and `docs/repo-map.md` while active.
- On completion, write one handoff, archive this plan, update `docs/archive/index.md`, and remove the active index entry.

## Risks / Open Questions

- Raw provider payloads can contain conflicting snapshot fields or entity-ambiguous news; canonical facts must win.
- Format checks must not discard a substantively correct answer or trigger a second persistent write.
- Broad market and sector discovery must fail closed when evidence is insufficient, without treating a common ticker as an acronym.
