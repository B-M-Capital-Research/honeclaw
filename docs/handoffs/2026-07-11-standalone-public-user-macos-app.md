# Standalone Public User macOS App

- title: Standalone Public User macOS App
- status: done
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files:
  - `bins/hone-user-app/`
  - `scripts/build_user_app.sh`
  - `.github/workflows/ci.yml`
- related_docs:
  - `docs/runbooks/public-user-macos-app.md`
  - `docs/archive/plans/standalone-public-user-macos-app.md`
  - `docs/decisions.md#d-2026-07-11-01-separate-the-public-macos-app-from-the-local-runtime-desktop`
- related_prs: N/A

## Summary

Hone now has a dedicated Universal macOS user app that embeds a polished startup/offline experience and then opens the production `/chat` login and conversation surface. It is a remote-only client and does not package or start the local Hone runtime.

## What Changed

- Added a minimal Tauri app, dedicated Hone icon, restricted first-party navigation, external-browser routing, and WebKit-persisted login state.
- Added a standalone release build script with an isolated target directory and documented build/signing workflow.
- Kept the app out of the default Linux Rust gate while retaining app-specific macOS tests and packaging verification.
- Updated formal-release version locations so future release version bumps cannot leave the user app behind.

## Verification

- Unit tests: 3 passed; crate check and changed-file formatting passed.
- Built `Hone.app` (16 MB) and `Hone_0.12.4_universal.dmg` (5.7 MB).
- DMG SHA-256: `b986972cb23373fc4c1d62494fcc5a63947c3a80faf7a9d625458079be2f4d21`.
- `file` / `lipo` confirmed `x86_64 arm64` Universal architecture.
- Packaged launch completed `tauri://localhost` then `https://hone-claw.com/chat`.
- Bundle audit found no `hone-mcp`, ACP CLI, channel listener, config, skill, or local-data payload.
- Browser visual QA confirmed the startup transition and production `/chat` login layout.

## Risks / Follow-ups

- The artifact is ad-hoc signed (`TeamIdentifier` absent) because this Mac has zero valid code-signing identities. Complete Developer ID signing and notarization before external distribution.
- The app deliberately depends on production availability. Offline startup remains recoverable but cannot provide chat without network access.

## Next Entry Point

Use `docs/runbooks/public-user-macos-app.md`; build with `bash scripts/build_user_app.sh`. Current artifacts are under `~/Library/Caches/honeclaw/user-app-target/universal-apple-darwin/release/bundle/`.
