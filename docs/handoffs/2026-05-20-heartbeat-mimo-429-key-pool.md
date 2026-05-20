# Heartbeat Mimo 429 Key-Pool Fallback

- title: Heartbeat Mimo 429 Key-Pool Fallback
- status: done
- created_at: 2026-05-20
- updated_at: 2026-05-20
- owner: Codex
- related_files:
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-llm/src/resolver.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `docs/bugs/scheduler_heartbeat_mimo_429_quota_exhausted.md`
  - `docs/bugs/README.md`
- related_docs:
  - `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- related_prs:
  - GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)

## Summary

Closed the P1 heartbeat `mimo-v2.5-pro` 429 quota exhaustion bug by making OpenAI-compatible providers honor configured key pools for non-streaming requests.

## What Changed

- `OpenAiCompatibleProvider` now builds one client per configured key and tries keys in order for `chat` and `chat_with_tools`.
- Non-OpenRouter profile resolution now passes the full `llm.providers.<name>.api_key/api_keys` pool instead of only the first key.
- Heartbeat runner error classification now maps `HTTP 429`, `rate limit exceeded`, `too many requests`, and `resource exhausted` to `provider_quota_exhausted`.

## Verification

- `cargo test -p hone-llm chat_with_tools_falls_back_to_next_key_after_http_429 -- --nocapture`
- `cargo test -p hone-channels heartbeat_provider_429_quota_error_is_classified --lib -- --nocapture`
- `cargo test -p hone-llm openai_compatible -- --nocapture`
- `cargo test -p hone-llm resolver -- --nocapture`
- `cargo test -p hone-channels heartbeat_provider_ --lib -- --nocapture`
- `cargo check -p hone-llm -p hone-channels --tests`
- `rustfmt --edition 2024 --check crates/hone-llm/src/openai_compatible.rs crates/hone-llm/src/resolver.rs crates/hone-channels/src/scheduler.rs`
- `git diff --check`

## Risks / Follow-ups

- If all configured keys are exhausted, heartbeat still fails by design; this is an external quota state, not a local code path to mask.
- Streaming requests still use the first key, matching the existing OpenRouter behavior because switching keys mid-stream is not safe.

## Next Entry Point

Use `docs/bugs/scheduler_heartbeat_mimo_429_quota_exhausted.md` for future #44 follow-up, and verify deployed configs have multiple valid keys under `llm.providers.<name>.api_keys` when quota resilience is required.
