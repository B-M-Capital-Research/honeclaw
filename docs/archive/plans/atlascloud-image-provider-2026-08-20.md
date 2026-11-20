# Plan

- title: Atlas Cloud optional image provider
- status: done locally
- created_at: 2026-08-20 00:28 CST
- updated_at: 2026-08-20 00:52 CST
- owner: Codex
- related_files:
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-integrations/src/nano_banana.rs`
  - `config.example.yaml`
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/technical-spec.md`
  - `docs/wiki.md`

## Goal

Add Atlas Cloud as an optional image-generation transport without changing the existing
OpenRouter default.

## Scope

- Select the image transport through `nano_banana.provider`.
- Read the selected provider's key from `llm.providers`.
- Submit Atlas Cloud image jobs once and poll prediction state with bounded backoff.
- Preserve the current image download and response contract.

## Validation

- `cargo test -p hone-integrations nano_banana`
- `cargo test -p hone-core config::tests`
- `bash scripts/ci/check_fmt_changed.sh`
- `git diff --check`

## Documentation Sync

- Updated the sample config, repo map, and configuration references.
- Archived this plan and added `docs/handoffs/2026-08-20-atlascloud-image-provider.md`.

## Completion

- `cargo test -p hone-integrations --lib`: 12 passed.
- `cargo test -p hone-core config::tests --lib`: 55 passed with the documented local PostgreSQL config environment.
- `cargo check -p hone-integrations -p hone-core`: passed.
- Changed Rust files pass direct `rustfmt --check`; `git diff --check` passed.
- Full `cargo clippy -D warnings` remains blocked by existing warnings in unrelated `hone-core` files.

## Risks / Open Questions

- Atlas Cloud models have model-specific schemas; the sample is limited to the currently
  verified `google/nano-banana-2/text-to-image` schema.
- No paid generation request is part of automated validation; the transport is covered by a
  local HTTP contract test.
