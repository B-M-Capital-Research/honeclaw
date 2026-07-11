# HONE Client Brand And iOS Release

- title: HONE Client Brand And iOS Release
- status: done
- created_at: 2026-07-11
- updated_at: 2026-07-11
- owner: Codex
- related_files: `packages/app/src/`, `bins/hone-user-app/`, `apps/hone-ios/`, `.github/workflows/apple-clients.yml`, `.github/workflows/release.yml`
- related_docs: `docs/releases/v0.13.0.md`, `docs/archive/plans/hone-client-brand-ios-release.md`, `docs/runbooks/public-user-macos-app.md`, `docs/runbooks/public-user-ios-app.md`
- related_prs: `main` commits `e33a467a`, `dc889ffa`, `aa32c818`, and `6a14e3e7`; tag `v0.13.0`

## Summary

The public Web client, focused macOS client, and new standalone iOS client now share one uppercase HONE identity, canonical mark, and refined navigation language. v0.13.0 is published at `https://github.com/B-M-Capital-Research/honeclaw/releases/tag/v0.13.0` with CLI packages and four Apple user-client assets.

## What Changed

- Replaced legacy public financial-console naming with HONE and centralized the public logo/wordmark treatment.
- Rebuilt desktop/mobile public navigation and menu presentation while preserving push, account, language, contact, and chat actions.
- Added the remote-only SwiftUI/WKWebView iOS app under `apps/hone-ios/`; it opens production `/chat`, persists WebKit login state, constrains first-party navigation, and hands external links to iOS.
- Added path-filtered Apple CI and formal Release packaging for the Universal macOS DMG, iOS Simulator app, complete Xcode project, and Apple SHA256 manifest.

## Verification

- Local Web, Rust, regression, Swift parse/contract, Universal bundle, architecture render, and browser visual checks passed.
- Apple Clients run `29139331210` passed, including the Xcode 15.4 Release Simulator compile.
- Release run `29139409377` passed every job, including Homebrew formula publication.
- GitHub Release contains `HONE-macOS-v0.13.0-universal.dmg`, `HONE-iOS-Simulator-v0.13.0.zip`, `HONE-iOS-Xcode-v0.13.0.zip`, and `HONE-Apple-SHASUMS256.txt`.

## Risks / Follow-ups

- A signed device IPA/TestFlight upload needs an Apple Developer team, signing certificate, provisioning profile, and release secrets; none were fabricated or embedded.
- The macOS DMG remains ad-hoc signed and should receive Developer ID signing/notarization before broad public distribution.
- Upgrade `mozilla-actions/sccache-action` when a Node.js 24-native release is available to remove the current non-blocking deprecation annotation.

## Next Entry Point

Use `docs/runbooks/public-user-ios-app.md` for iOS signing/device distribution and `docs/runbooks/public-user-macos-app.md` for Developer ID signing/notarization. Product branding changes should continue through `packages/app/src/components/hone-brand.tsx` and `packages/app/src/components/public-nav.tsx`.
