# HONE Client Brand And iOS Release

- title: HONE Client Brand And iOS Release
- status: archived
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files:
  - `packages/app/src/components/public-nav.tsx`
  - `packages/app/src/pages/public-site.css`
  - `packages/app/src/pages/chat.tsx`
  - `bins/hone-user-app/`
  - `apps/hone-ios/`
  - `.github/workflows/release.yml`
- related_docs:
  - `docs/releases/v0.13.0.md`
  - `docs/runbooks/public-user-macos-app.md`
  - `docs/runbooks/public-user-ios-app.md`
  - `docs/handoffs/2026-07-11-hone-client-brand-ios-release.md`

## Goal

Unify every public user-client surface around the HONE name and one visual identity, refine navigation/menu interaction across screen sizes, add a standalone native iOS shell for `/chat`, and ship both Apple clients in the v0.13.0 GitHub Release.

## Scope

- Removed user-facing `OPEN FINANCIAL CONSOLE`, `Financial Console`, and `Hone Financial` brand copy from public Web/macOS client surfaces.
- Introduced one canonical HONE mark/wordmark treatment across public navigation, footer, chat rail, startup shells, and Apple app icons.
- Refined desktop/mobile navigation hierarchy, active states, menu layout, spacing, and motion without changing authenticated backend behavior.
- Added `apps/hone-ios/` as an independent SwiftUI/WKWebView app with persistent login, first-party navigation policy, external-link handoff, offline recovery, and `/chat` default.
- Extended the tag release workflow to upload the Universal macOS DMG plus iOS Simulator and Xcode-source assets.
- Published v0.13.0 with bilingual release notes without claiming a signed iOS device IPA.

## Validation

- `bun run typecheck:web`, `bun run test:web` (205 passed), and `bun run build:web:public` passed; desktop/mobile browser visual QA passed.
- `cargo test -p hone-user-app`, `cargo check -p hone-user-app`, Universal macOS build, bundle audit, signature inspection, and packaged `/chat` launch smoke passed.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` and `bash tests/regression/run_ci.sh` passed.
- Apple Clients run `29139331210` passed the macOS checks, iOS contract, and Xcode 15.4 Release Simulator build.
- Release run `29139409377` passed all jobs and published eight assets to `v0.13.0`, including all four Apple user-client assets.

## Documentation Sync

- Updated `docs/repo-map.md`, `docs/decisions.md`, Apple client runbooks, formal-release references, and `resources/architecture.svg` / `.html`.
- Added the completion handoff, archived this plan, updated `docs/archive/index.md`, and removed the task from the active index.

## Risks / Open Questions

- The iOS Release asset is a Simulator build plus complete Xcode source, not a device IPA or TestFlight build; Apple Developer signing credentials are still required for device distribution.
- The macOS DMG is ad-hoc signed rather than Developer ID signed/notarized.
- GitHub reports a non-blocking Node.js 20 deprecation warning for `mozilla-actions/sccache-action@v0.0.9`; it did not affect v0.13.0.
