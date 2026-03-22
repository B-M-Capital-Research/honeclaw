# 2026-03-22 Large Files Refactor Handoff

## What Changed

- `bins/hone-feishu/src/main.rs` is now a thin façade over `card.rs`, `handler.rs`, `listener.rs`, `markdown.rs`, and `types.rs`.
- `bins/hone-telegram/src/main.rs` is now a thin façade over `handler.rs`, `listener.rs`, `markdown_v2.rs`, and `types.rs`.
- `bins/hone-desktop/src/main.rs` now delegates to `commands.rs` and `sidecar.rs`, with `tray.rs` reserved as the tray/menu extension point.
- `crates/hone-core/src/config.rs` is now a façade over `config/{agent,channels,server}.rs`.
- `crates/hone-channels/src/attachments.rs` is now a façade over `attachments/ingest.rs`, `attachments/vision.rs`, and `attachments/vector_store.rs`.

## Notes

- `crates/hone-channels/src/agent_session.rs` was intentionally left untouched.
- Public compatibility was preserved for `hone_core::HoneConfig` and `hone_channels::attachments::extract_full_pdf_text`.
- The desktop `generate_handler!` registration lives in `bins/hone-desktop/src/commands.rs`; `sidecar.rs` now only wraps it.

## Verification

- `cargo check -p hone-desktop`
- `cargo check -p hone-core -p hone-channels`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `bash tests/regression/run_ci.sh`
- `bash scripts/ci/check_fmt_changed.sh` via `/opt/homebrew/bin/bash` because the system bash is 3.2 and lacks `mapfile`

## Follow-ups

- `cargo fmt --all --check` still reports pre-existing formatting drift in unrelated files outside this refactor.
- If you want to finish the formatting cleanup, start from the files reported by `cargo fmt --all --check` and confirm with the same command again.
