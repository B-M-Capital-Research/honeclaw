# LLM Profile Runtime Migration

- title: LLM Profile Runtime Migration
- status: done
- created_at: 2026-05-11
- updated_at: 2026-05-11
- owner: codex
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-core/src/config/event_engine.rs`
  - `crates/hone-llm/src/provider.rs`
  - `crates/hone-llm/src/openrouter.rs`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-web-api/src/lib.rs`
  - `bins/hone-desktop/src/sidecar.rs`
  - `packages/app/src/pages/settings.tsx`
  - `crates/hone-channels/src/core/bot_core.rs`
  - `config.example.yaml`
  - `tests/regression/manual/test_llm_profile_poc.sh`
- related_docs:
  - `docs/handoffs/2026-05-11-llm-profile-poc.md`

## Goal

Move the POC `llm.providers` + `llm.profiles` shape into the runtime path so LLM consumers can reference named profiles while profiles own provider, model, reasoning, response format, and generation parameters.

## Scope

- Keep legacy `llm.openrouter` / `llm.auxiliary` fields compatible.
- Add a shared resolver/request-options layer instead of duplicating model and max-token wiring.
- Migrate event-engine OpenRouter construction first, especially classifier, SEC enrichment, earnings review, and global digest/distill paths.
- Migrate channel auxiliary provider creation where it is low risk.
- Do not force runner configs such as `codex_acp`, `opencode`, `gemini_cli`, or `hone_cloud` into this profile model in this task.

## Validation

- `cargo test -p hone-core config::tests`
- `cargo check -p hone-channels --tests`
- `cargo check -p hone-web-api --tests`
- `cargo test -p hone-llm resolver`
- `cargo test -p hone-event-engine global_digest_llm_providers_can_be_wired_per_stage`
- `cargo test -p hone-web-api validate_global_digest`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop --bin hone-desktop sidecar`
- `bun run typecheck:web`
- `bun run test:web`
- `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- Browser DOM check at `http://127.0.0.1:3000/settings?tab=agent` confirmed the `LLM Profile 路由` block renders with profile binding selects and profile parameter fields.
- `git diff --check`

## Documentation Sync

- Updated `config.example.yaml` with the new profile shape.
- Archived this plan from `docs/current-plans/` to `docs/archive/plans/`.
- Appended runtime/frontend migration conclusions to `docs/handoffs/2026-05-11-llm-profile-poc.md`.
- Updated `docs/archive/index.md`.

## Risks / Open Questions

- OpenRouter and OpenAI-compatible APIs do not support identical request fields; typed common fields should be merged with provider-specific `extra_body`.
- Existing configs may only have legacy keys and models, so resolver fallback must be deterministic and non-breaking.
- Token-budget fields currently live in event-engine domain configs; migration should preserve existing caps and avoid accidental 30k completion preauthorization.
