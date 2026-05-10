# LLM Profile Registry POC

- title: LLM Profile Registry POC
- status: done
- created_at: 2026-05-11
- updated_at: 2026-05-11
- owner: codex
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-core/src/config/tests.rs`
  - `crates/hone-llm/examples/llm_profile_poc.rs`
  - `tests/fixtures/llm/profile_poc.yaml`
  - `tests/regression/manual/test_llm_profile_poc.sh`
- related_docs:
  - `docs/archive/index.md`
- related_prs: N/A

## Summary

Validated that Honeclaw can support a named LLM registry shape where `llm.providers` owns transport and credentials, and `llm.profiles` owns model plus generation parameters. The POC covers `max_tokens`, `temperature`, `reasoning`, `response_format`, and provider-specific `extra_body` passthrough without changing production runtime routing.

## What Changed

- Added serde config structs for `llm.providers`, `llm.profiles`, profile params, reasoning params, and provider options.
- Added a Rust POC example that resolves a profile, builds an OpenAI-compatible chat body, loads secrets from env or local `config.yaml`, and sends an optional live OpenRouter request.
- Added a manual regression wrapper and a non-secret fixture profile using `x-ai/grok-4.1-fast`.

## Verification

- `cargo test -p hone-core config::tests`
- `cargo run -p hone-llm --example llm_profile_poc`
- `RUN_LLM_PROFILE_POC=1 cargo run -p hone-llm --example llm_profile_poc`
- `bash tests/regression/manual/test_llm_profile_poc.sh`
- `rustfmt --edition 2024 --check crates/hone-core/src/config/agent.rs crates/hone-core/src/config/mod.rs crates/hone-core/src/config/tests.rs crates/hone-llm/examples/llm_profile_poc.rs`
- `git diff --check`

Live OpenRouter result accepted the profile-derived request with `reasoning_present=true`, `finish_reason=stop`, and nonzero usage.

## Risks / Follow-ups

- Production callers still use legacy `llm.openrouter` / `llm.auxiliary` fields; this POC only proves config parsing and upstream request compatibility.
- Next implementation step should add a shared resolver and request-options type before migrating event-engine call sites.
- `codex_acp`, `opencode`, `gemini_cli`, and `hone_cloud` runner configs should remain separate until a later runner-specific design pass.

## Next Entry Point

Start with `crates/hone-llm/examples/llm_profile_poc.rs` and migrate the duplicated event-engine OpenRouter provider construction in `crates/hone-web-api/src/lib.rs` to a shared profile resolver.

## Runtime Migration Addendum

- status: done
- updated_at: 2026-05-11
- related_plan: `docs/archive/plans/llm-profile-runtime-migration.md`

### Summary

The POC shape is now wired into runtime consumers. `llm.providers` owns provider transport/credential metadata, while `llm.profiles` owns model, token budget, reasoning, response format, and generation parameters. Legacy `llm.openrouter` and `llm.auxiliary` fields remain compatible fallback paths.

### What Changed

- Added `hone_llm::LlmResolver`, `CreatedLlmProvider`, and request-options support for OpenRouter/OpenAI-compatible chat calls.
- Migrated event-engine LLM construction for renderer polish, news classifier, SEC enrichment, earnings quality review, global digest pass1/pass2/event-dedupe, and mainline distill.
- Migrated channel auxiliary provider creation to prefer `llm.auxiliary_profile` while keeping old auxiliary/OpenRouter fallback behavior.
- Added desktop sidecar read/write support for `llmProfiles`, including profile bindings and per-profile provider/model/max_tokens/temperature/top_p/reasoning/JSON fields.
- Added Settings UI controls under the OpenAI-compatible card so the desktop config page can edit profile routing and profile parameters.
- Updated `config.example.yaml` with default profiles: `main`, `aux`, `news_classifier`, `filing_summary`, `earnings_quality`, `digest_fast`, `digest_strong`, and `mainline_short`.

### Verification

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
- Browser DOM check at `http://127.0.0.1:3000/settings?tab=agent` confirmed the `LLM Profile 路由` controls render.
- `git diff --check`

The live OpenRouter smoke accepted a profile-derived request with `reasoning_present=true`, `finish_reason=stop`, and nonzero usage.

### Risks / Follow-ups

- Streaming paths still use the existing SDK request shape and do not yet apply every profile request option.
- Runner-specific configuration (`codex_acp`, `opencode`, `gemini_cli`, `hone_cloud`) remains intentionally separate from this profile registry.
- The desktop UI edits the default profile set; custom hand-written profile IDs are preserved in the loaded draft but there is not yet an add/remove UI for arbitrary profiles.

## Config-Only Credential Addendum

- status: done
- updated_at: 2026-05-11
- related_plan: `docs/archive/plans/llm-config-env-removal.md`
- related_decision: `docs/decisions.md#d-2026-05-11-01-make-llm-credentials-config-only`

### Summary

LLM credentials are now config-only. `api_key_env` is no longer part of the LLM config structs, resolver, OpenRouter provider construction, auxiliary route, or Gemini ACP user config. Missing keys now point users to `config.yaml` instead of suggesting env fallback.

### What Changed

- `llm.providers.openrouter.api_key/api_keys` is the preferred OpenRouter credential path; legacy `llm.openrouter.api_key/api_keys` remains readable only as config fallback.
- `hone-cli configure/onboard` and desktop OpenRouter settings write `llm.providers.openrouter.api_keys` and clear legacy OpenRouter fields on save.
- `llm.auxiliary.api_key` is the only auxiliary key source; `MINIMAX_API_KEY` is ignored at runtime.
- `agent.gemini_acp.api_key` replaces `api_key_env`; if set, Hone bridges that config-owned key to the Gemini child process.
- Default news/global-digest example models have been aligned to `x-ai/grok-4.1-fast`.

### Verification

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
- `cargo run -p hone-cli -- config validate --json`
- `cargo run -p hone-cli -- status --json`
- `cargo run -p hone-cli -- config --config /tmp/hone-cli-config-smoke.<id>/config.yaml set llm.providers.openrouter.api_keys '["sk-test-a","sk-test-b"]'`
- `cargo run -p hone-cli -- config --config /tmp/hone-cli-config-smoke.<id>/config.yaml get llm.providers.openrouter.api_keys`
- `cargo run -p hone-cli -- status --config /tmp/hone-cli-config-smoke.<id>/config.yaml --json`
- `cargo run -p hone-cli -- probe --channel cli --user-id cli_smoke --query '只输出 HONE_CLI_LLM_OK' --show-events false`
- `cargo fmt --all --check`
- `git diff --check`

### Risks / Follow-ups

- Existing configs with only `api_key_env` will parse but fail when a real LLM provider is needed; the fix is to copy the key into `config.yaml`.
- `opencode_acp` still passes a config-owned OpenRouter key to the child process as `OPENROUTER_API_KEY` because opencode does not expose an equivalent inline provider key in the generated session config.
