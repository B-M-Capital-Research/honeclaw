# Standalone Public User macOS App

- title: Standalone Public User macOS App
- status: archived
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files:
  - `Cargo.toml`
  - `bins/hone-user-app/`
  - `scripts/build_user_app.sh`
  - `.github/workflows/ci.yml`
- related_docs:
  - `docs/runbooks/public-user-macos-app.md`
  - `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
  - `docs/handoffs/2026-07-11-standalone-public-user-macos-app.md`

## Goal

Ship the complete public Hone user experience as a focused macOS release app that opens directly to `/chat` login/conversation without bundling or initializing the full local desktop runtime.

## Scope

- Added a dedicated Tauri crate with no Hone runtime, ACP, MCP, channel, skill, config, or local-data dependencies.
- Kept first-party Hone navigation inside the app and routed unrelated external links to the system browser.
- Added a polished branded startup/offline screen, dedicated app icon, and persistent WebKit login state.
- Added a reproducible Universal macOS release build producing `.app` and `.dmg` artifacts.
- Kept `hone-desktop` and formal `v*` release/tag behavior unchanged.

## Validation

- `cargo test -p hone-user-app`: 3 passed.
- `cargo check -p hone-user-app`: passed.
- `rustfmt --edition 2024 --check bins/hone-user-app/src/main.rs`: passed.
- `bash -n scripts/build_user_app.sh`, JSON parse, and `git diff --check`: passed.
- Universal release build passed; final executable contains `x86_64` and `arm64`.
- Final app launch logged successful navigation from the embedded shell to `https://hone-claw.com/chat`.
- Bundle inspection found only `hone-user-app`, `icon.icns`, and `Info.plist`; forbidden runtime/sidecar resources were absent.

## Documentation Sync

- Updated `AGENTS.md`, `docs/repo-map.md`, and `docs/decisions.md` for the separate macOS lane and CI boundary.
- Added `docs/runbooks/public-user-macos-app.md` and a completion handoff.
- Removed this task from the active index and archived the plan here.

## Risks / Open Questions

- The app requires network access to `hone-claw.com`; the bundled shell provides explicit offline retry behavior.
- This machine has no valid Apple Developer signing identity. The artifact is ad-hoc signed and not notarized, so public distribution still needs Developer ID signing and Apple notarization.
