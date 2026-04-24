# Handoff Template

- title: Hone built-in skill high-confidence consolidation
- status: done
- created_at: 2026-04-20
- updated_at: 2026-04-20
- owner: Codex
- related_files:
  - skills/stock_research/SKILL.md
  - skills/scheduled_task/SKILL.md
  - crates/hone-tools/src/skill_runtime.rs
  - tests/regression/ci/test_finance_automation_contracts.sh
  - docs/repo-map.md
  - docs/archive/plans/hone-skill-consolidation.md
- related_docs:
  - docs/current-plan.md
  - docs/archive/index.md
- related_prs: N/A

## Summary

Removed the non-functional `one_sentence_memory` and redundant `major_alert` / `valuation` / `stock_selection` skill files, and collapsed the surviving surface around a canonical `stock_research` skill plus an enriched `scheduled_task` skill.

## What Changed

- Deleted `skills/one_sentence_memory/SKILL.md`
- Deleted `skills/major_alert/SKILL.md` and moved its event-reminder behavior into `skills/scheduled_task/SKILL.md`
- Deleted `skills/valuation/SKILL.md` and `skills/stock_selection/SKILL.md`
- Expanded `skills/stock_research/SKILL.md` so it now covers research, valuation, and screening modes with compatibility aliases such as `OWGZ`, `OWXG`, `valuation`, and `stock screener`
- Updated the finance regression script to validate the canonical `stock_research` and `scheduled_task` contracts instead of the removed files
- Repointed the alias-resolution unit test away from the removed one-sentence-memory skill

## Verification

- `bash tests/regression/ci/test_finance_automation_contracts.sh`
- `cargo test -p hone-tools load_skill_and_direct_invocation_accept_aliases`
- `cargo fmt --all --check`

## Risks / Follow-ups

- Alias-based compatibility now carries more weight for legacy valuation / screener trigger words; if slash or direct-invoke semantics tighten later, add a dedicated regression that exercises those aliases through the full runtime
- Several older historical docs still mention the removed skills; they were left intact as historical records rather than rewritten

## Next Entry Point

- `skills/stock_research/SKILL.md`
