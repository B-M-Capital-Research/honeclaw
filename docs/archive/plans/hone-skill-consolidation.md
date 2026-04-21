# Plan Template

- title: Hone built-in skill high-confidence consolidation
- status: archived
- created_at: 2026-04-20
- updated_at: 2026-04-20
- owner: Codex
- related_files:
  - skills/stock_research/SKILL.md
  - skills/stock_selection/SKILL.md
  - skills/valuation/SKILL.md
  - skills/scheduled_task/SKILL.md
  - skills/major_alert/SKILL.md
  - skills/one_sentence_memory/SKILL.md
  - tests/regression/ci/test_finance_automation_contracts.sh
  - docs/current-plan.md
  - docs/handoffs/2026-04-20-hone-skill-consolidation.md
  - docs/archive/index.md
- related_docs:
  - docs/repo-map.md
  - docs/technical-spec.md

## Goal

Reduce obvious overlap in Hone built-in skills, remove the non-functional one-sentence memory prompt, and keep regression/docs aligned with the slimmer skill surface.

## Scope

- Remove `one_sentence_memory`
- Fold `major_alert` behavior into `scheduled_task`
- Consolidate the stock research family around one canonical `stock_research` workflow while preserving valuation / screener entrypoints
- Update tests and skill inventory docs that still assume the old shape

## Validation

- `rtk bash tests/regression/ci/test_finance_automation_contracts.sh`
- `rtk cargo test -p hone-tools load_skill_and_direct_invocation_accept_aliases`
- `rtk cargo fmt --all --check`
- Review `git diff --stat` and key skill files to confirm only intended skills changed

## Documentation Sync

- Update `docs/current-plan.md` while active
- Write a handoff after completion because the skill surface and maintenance guidance change
- Add an archive index entry when the task exits active state

## Risks / Open Questions

- `valuation` / `stock_selection` now rely on alias compatibility through `stock_research`; if future runtime matching changes, those legacy trigger words need a dedicated regression
- Historical handoffs and archive plans intentionally still mention removed skill files because they record past states
