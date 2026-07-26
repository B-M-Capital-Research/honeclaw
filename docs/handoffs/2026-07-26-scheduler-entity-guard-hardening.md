- title: Scheduler entity guard hardening for macro and institution terms
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex bug-2 automation
- related_files: `crates/hone-channels/src/investment_response_guard.rs`, `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`, `docs/bugs/README.md`, `docs/current-plans/ticker-resolution-architecture.md`
- related_docs: `docs/current-plan.md`, `docs/archive/index.md`
- related_prs:

## Summary

Code-level hardening is in place for the remaining scheduler/entity P2 under the ticker umbrella. Non-interactive bare `TitleCase` names no longer become deterministic security subjects, institution-name fragments such as `ARK Invest` no longer leak an `ARK` ticker, and macro/regulatory acronyms such as `PCE` / `SEC` now require explicit ticker binding before they can enter securities resolution.

## What Changed

- Tightened `plain_ticker_mentions(...)` for non-interactive turns so bare mixed-case company names fall back to `AgentToolDiscovery` instead of deterministic securities preflight.
- Added `identifier_is_multiword_proper_name_component(...)` to suppress institution-name fragments like `ARK Invest`.
- Expanded `identifier_requires_explicit_security_binding(...)` to cover macro, regulatory, and institution acronyms that previously slipped through clause-subject bindings in scheduler/heartbeat prose.
- Added regression coverage for:
  - macro digest prompts (`PCE/FOMC/GDP`)
  - heartbeat prompts containing `SEC/FDA/NASA`
  - institution/person names (`Nancy Pelosi`, `ARK Invest`)
  - bare company-name heartbeat titles (`Oracle 大事件监控`)

## Verification

- `cargo test -p hone-channels scheduler_and_heartbeat_skip_macro_regulatory_and_name_components --lib -- --nocapture`
- `cargo test -p hone-channels heartbeat_subject_markers_count_as_security_context --lib -- --nocapture`
- `cargo test -p hone-channels scheduled_ticker_subject_is_available_without_parsing_the_envelope --lib -- --nocapture`
- `cargo test -p hone-channels operational_checks_and_scheduler_conditions_do_not_become_tickers --lib -- --nocapture`
- `cargo test -p hone-channels collision_policy_accepts_real_short_tickers_only_with_strong_binding --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## Risks / Follow-ups

- This run did not restart Web/Feishu/heartbeat processes, so there is no fresh live scheduler window proving the fix has loaded in production.
- The broader ticker umbrella may still need follow-up for other task-prose/entity drift samples (`800G`, `NAND`, cross-symbol prompt pollution) even if this specific P2 no longer reproduces.

## Next Entry Point

Start from `crates/hone-channels/src/investment_response_guard.rs` around `plain_ticker_mentions(...)`, `identifier_is_multiword_proper_name_component(...)`, and `identifier_requires_explicit_security_binding(...)`. Use `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md` to decide whether a future live sample is the same fixed class or a new drift pattern.
