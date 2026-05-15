# Release Smoke And Oil Guard Fix

- title: Release Smoke And Oil Guard Fix
- status: done
- created_at: 2026-05-15 08:07 CST
- updated_at: 2026-05-15 08:07 CST
- owner: Codex
- related_files:
  - `bins/hone-desktop/src/commands.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `docs/bugs/daily_macos_build_release_app_api_not_persistent.md`
  - `docs/bugs/oil_price_scheduler_geopolitical_hallucination.md`
  - `docs/bugs/README.md`
  - `docs/runbooks/desktop-release-app-runtime.md`
- related_docs:
  - `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- related_prs:
  - GitHub Issue [#42](https://github.com/B-M-Capital-Research/honeclaw/issues/42)

## Summary

Closed the two active bugs in `docs/bugs/README.md`: Daily macOS release app API smoke now has a headless packaged-binary mode, and the latest oil scheduler contract-month recurrence is locked by a precise commodity guard regression.

## What Changed

- `hone-desktop` now recognizes `HONE_DESKTOP_SMOKE_SERVER=1` before building the Tauri app. In that mode it starts `hone_web_api::start_server(...)`, uses fixed env ports/config/data overrides, and stays alive until Ctrl-C.
- `scheduler.rs` gained `commodity_guard_covers_oil_scheduler_contract_months_and_tail_risk_claim`, covering the latest `Brent Jul 2026` / `WTI Jun 2026` ordinary scheduler sample and ensuring unsafe price/causality text is rewritten to the safety notice.
- Bug docs and navigation were updated to `Fixed`; `docs/runbooks/desktop-release-app-runtime.md` now documents the headless smoke command.

## Verification

- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop desktop_smoke -- --nocapture`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop --tests`
- `HONE_DESKTOP_SMOKE_SERVER=1 HONE_WEB_PORT=18077 HONE_PUBLIC_WEB_PORT=18088 HONE_USER_CONFIG_PATH=... HONE_DESKTOP_DATA_DIR=... target/debug/hone-desktop` with `curl /api/meta`, `curl :18088/`, and `curl /api/channels`
- `cargo test -p hone-channels commodity_guard_covers_oil_scheduler_contract_months_and_tail_risk_claim --lib -- --nocapture`
- `cargo test -p hone-channels commodity_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs bins/hone-desktop/src/commands.rs`

## Risks / Follow-ups

- The smoke fix intentionally does not claim LaunchServices window startup is fixed; it provides a deterministic release build smoke path.
- The oil bug was marked fixed based on current HEAD behavior and local regression proof. Current-machine live samples without guard metadata should be treated as old/non-production runtime evidence unless a fresh local run of current code reproduces them.

## Next Entry Point

Start with `docs/bugs/README.md` for the active bug queue, then use `docs/runbooks/desktop-release-app-runtime.md` for the next packaged app smoke verification.
