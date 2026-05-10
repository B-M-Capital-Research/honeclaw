# LLM Config Env Removal

- title: LLM Config Env Removal
- status: done
- created_at: 2026-05-11
- updated_at: 2026-05-11
- owner: codex
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-llm/src/resolver.rs`
  - `crates/hone-llm/src/openrouter.rs`
  - `crates/hone-llm/examples/llm_profile_poc.rs`
  - `bins/hone-cli/src/{configure.rs,onboard.rs,mutations.rs,reports.rs}`
  - `bins/hone-desktop/src/sidecar.rs`
  - `bins/hone-desktop/src/sidecar/settings.rs`
  - `packages/app/src/{lib/types.ts,pages/settings-model.ts,pages/settings.tsx}`
  - `config.example.yaml`
- related_docs:
  - `docs/decisions.md`
  - `docs/invariants.md`
  - `docs/handoffs/2026-05-11-llm-profile-poc.md`

## Goal

Make LLM provider/profile/auxiliary/runner credentials config-only. Runtime LLM paths should no longer read `*_API_KEY` or `api_key_env`; missing keys should fail with a migration message that points users to `config.yaml`.

## Scope

- Remove LLM `api_key_env` fields and env fallback from OpenRouter, auxiliary LLM, provider registry, Kimi, and Gemini ACP config.
- Keep non-LLM runtime env controls and push-channel tokens out of scope.
- Update CLI/Desktop OpenRouter writes to target `llm.providers.openrouter.*` while keeping config-only legacy `llm.openrouter.*` fallback readable.
- Keep profile params such as `reasoning`, `response_format`, and `extra_body` working.

## Validation

- `cargo test -p hone-core config::tests`
- `cargo test -p hone-llm resolver`
- `cargo test -p hone-cli mutations`
- `cargo check -p hone-cli --tests`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop --bin hone-desktop sidecar`
- `cargo check -p hone-channels -p hone-integrations -p hone-web-api --tests --examples`
- `cargo test -p hone-web-api validate_global_digest`
- `bun run test:web`
- `bun run typecheck:web`
- `cargo run -p hone-llm --example llm_profile_poc`
- `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- `cargo fmt --all --check`
- `git diff --check`

## Documentation Sync

- Updated `config.example.yaml`.
- Updated `docs/decisions.md`, `docs/invariants.md`, and `docs/repo-map.md`.
- Updated active context references that still described OpenRouter env-based manual runs.
- Archived this plan and appended `docs/archive/index.md`.
- Appended the existing LLM profile handoff with the config-only credential migration result.

## Risks / Open Questions

- `opencode_acp` still needs `OPENROUTER_API_KEY` as a child-process bridge when Hone injects a config key into opencode; this is not a user env fallback.
- Existing user configs may still carry `api_key_env`; serde should ignore those fields after removal, but runtime must not consume them.
