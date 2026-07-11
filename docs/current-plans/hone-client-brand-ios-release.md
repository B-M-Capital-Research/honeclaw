# HONE Client Brand And iOS Release

- title: HONE Client Brand And iOS Release
- status: in_progress
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

## Goal

Unify every public user-client surface around the HONE name and one visual identity, refine navigation/menu interaction across screen sizes, add a standalone native iOS shell for `/chat`, and ship both Apple clients in the v0.13.0 GitHub Release.

## Scope

- Remove user-facing `OPEN FINANCIAL CONSOLE`, `Financial Console`, and `Hone Financial` brand copy from public Web/macOS client surfaces.
- Introduce one canonical HONE mark/wordmark treatment across public navigation, footer, chat rail, startup shells, and Apple app icons.
- Refine desktop/mobile navigation hierarchy, active states, menu layout, spacing, and motion without changing authenticated backend behavior.
- Add `apps/hone-ios/` as an independent SwiftUI/WKWebView app with persistent login, first-party navigation policy, external-link handoff, offline recovery, and `/chat` default.
- Extend the tag release workflow to upload the Universal macOS DMG plus iOS Simulator and Xcode-source assets.
- Publish v0.13.0 with bilingual release notes; do not claim a signed iOS device IPA without Apple signing credentials.

## Validation

- Public frontend typecheck, unit tests, production build, and desktop/mobile browser visual QA.
- macOS user-app unit tests, crate check, Universal build, bundle audit, and `/chat` launch smoke.
- Swift syntax/static contract checks locally; iOS Simulator build on GitHub macOS runner because this machine only has Command Line Tools, not full Xcode.
- Release-note preparation, workflow syntax review, workspace Rust check, and final GitHub Release asset inspection.

## Documentation Sync

- Update `docs/repo-map.md`, `docs/decisions.md`, Apple client runbooks, formal-release references, and `resources/architecture.svg` / `.html` for the new client boundary.
- Add a completion handoff, archive this plan, update `docs/archive/index.md`, and remove this task from the active index after the tag workflow succeeds.

## Risks / Open Questions

- This machine cannot run `xcodebuild`; iOS compile proof must come from the GitHub macOS runner before release completion.
- No Apple Developer signing identities or secrets are currently available. Release assets can include an iOS Simulator build and source project, but not a distributable device IPA/TestFlight upload.
- Public navigation changes must preserve the existing mobile push indicator, account actions, and chat overlay stacking behavior.
