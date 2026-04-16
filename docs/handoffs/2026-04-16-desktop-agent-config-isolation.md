- title: Desktop agent config isolation fixes
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: Codex
- related_files:
  - crates/hone-core/src/config.rs
  - bins/hone-desktop/src/sidecar.rs
  - docs/bugs/README.md
  - docs/bugs/desktop_opencode_legacy_override_gap.md
  - docs/bugs/desktop_runner_settings_cross_runner_overwrite.md
  - docs/current-plans/canonical-config-runtime-apply.md
- related_docs:
  - docs/archive/index.md
  - docs/current-plans/canonical-config-runtime-apply.md
- related_prs:
  - N/A

## Summary

Closed two P1 desktop agent-config bugs in the canonical config / desktop settings lane:

- legacy runtime migration no longer overwrites canonical `agent.opencode` just because `api_key` is blank
- desktop settings save no longer lets `multi-agent.answer` overwrite `agent.opencode`

## What Changed

- `crates/hone-core/src/config.rs`
  - added `canonical_opencode_block_is_blank(...)`
  - changed legacy `agent.opencode` promotion so a fully blank canonical block still migrates as a whole, but partially configured canonical state only backfills blank `api_base_url` / `model` / `variant`
  - explicitly preserves a blank canonical `agent.opencode.api_key` so local OpenCode inheritance semantics survive desktop upgrade
- `bins/hone-desktop/src/sidecar.rs`
  - extracted `build_agent_setting_updates(...)`
  - removed the later save-step that rewrote `agent.opencode.*` from `multi_agent.answer.*`
  - kept `agent.opencode.*` and `agent.multi_agent.answer.*` as separate persisted targets
- updated the two bug docs plus `docs/bugs/README.md` to mark them `Fixed`
- updated the active canonical-config plan with this round’s focus

## Verification

- `cargo test -p hone-core promote_legacy_runtime_agent_settings`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop build_agent_setting_updates_keeps_opencode_and_multi_agent_answer_isolated`
- `rustfmt --edition 2024 --check crates/hone-core/src/config.rs bins/hone-desktop/src/sidecar.rs`

## Risks / Follow-ups

- `multi-agent.answer` and `agent.opencode` are now isolated on save, but the product contract is still visually adjacent in the desktop UI; further UX cleanup may still be worth doing
- `desktop_runner_settings_write_race.md` remains open and is adjacent to the same settings persistence lane
- `multi_agent_search_key_fallback_mismatch.md` is still open in the same desktop/config area and is a likely next candidate

## Next Entry Point

- `docs/bugs/desktop_runner_settings_write_race.md`
