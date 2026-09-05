# Handoff

- title: Atlas Cloud optional image provider
- status: done locally
- created_at: 2026-08-20 00:52 CST
- updated_at: 2026-08-20 00:52 CST
- owner: Codex
- related_files:
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-integrations/src/nano_banana.rs`
  - `config.example.yaml`
- related_docs:
  - `docs/archive/plans/atlascloud-image-provider-2026-08-20.md`
  - `docs/repo-map.md`
  - `docs/technical-spec.md`
  - `docs/wiki.md`
- related_prs: external PR pending at handoff creation

## Summary

`nano_banana` can now use Atlas Cloud as an optional image backend while preserving OpenRouter
as the default and keeping the existing local image download contract.

## What Changed

- Added `nano_banana.provider` with a backward-compatible `openrouter` default.
- Reused the selected `llm.providers` key pool instead of introducing another credential field.
- Added the Atlas Cloud `generateImage` submission and prediction polling flow. Generation POST
  is attempted once; only GET prediction checks use bounded backoff.
- Added local HTTP contract coverage for request shape, one-time submission, polling, and output
  extraction, plus backward-compatibility coverage for configs without the new field.

## Verification

- `cargo test -p hone-integrations --lib`: 12 passed.
- `cargo test -p hone-core config::tests --lib`: 55 passed.
- `cargo check -p hone-integrations -p hone-core`: passed.
- Direct changed-file `rustfmt --check` and `git diff --check`: passed.
- No paid Atlas Cloud generation request was made; the current
  `google/nano-banana-2/text-to-image` schema was verified before implementation.

## Risks / Follow-ups

- Atlas Cloud model parameters are model-specific. The sample config intentionally documents one
  currently verified text-to-image model and sends only its required fields plus async mode.
- Repository-wide Clippy with `-D warnings` is currently blocked by pre-existing warnings in
  unrelated `hone-core` code.

## Next Entry Point

Start at `NanoBananaClient::generate_atlascloud_images`; keep paid POST submission one-shot and
add model-specific optional parameters only after verifying the target model schema.
