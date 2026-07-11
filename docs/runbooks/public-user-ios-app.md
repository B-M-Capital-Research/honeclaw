# Public User iOS App

## Purpose

Build and validate the standalone HONE iOS client under `apps/hone-ios/`. It is a native SwiftUI/WKWebView shell for `https://hone-claw.com/chat` and owns no local HONE runtime, ACP, MCP, channel listener, config, skill, or user-data directory.

## Requirements

- Xcode 16 or newer with an iOS Simulator runtime
- iOS 16 deployment target
- Apple Developer membership and provisioning only when creating a device/App Store archive

## Local Simulator Build

```bash
xcodebuild \
  -project apps/hone-ios/HONE.xcodeproj \
  -scheme HONE \
  -configuration Release \
  -sdk iphonesimulator \
  -derivedDataPath /tmp/hone-ios-derived \
  CODE_SIGNING_ALLOWED=NO \
  build
```

Expected app:

```text
/tmp/hone-ios-derived/Build/Products/Release-iphonesimulator/HONE.app
```

## Contract Validation Without Xcode

```bash
bash tests/regression/ci/test_hone_ios_contract.sh
swiftc -parse apps/hone-ios/HONE/*.swift
plutil -lint apps/hone-ios/HONE.xcodeproj/project.pbxproj
```

The regression compiles and executes the Foundation-only navigation policy when `swiftc` is available, then always verifies the `/chat` target, bundle identifier, version, and absence of forbidden local runtime or legacy user-brand strings.

## Runtime Boundary

- `WKWebsiteDataStore.default()` persists the authenticated Web session.
- Only HTTPS routes on `hone-claw.com` and `www.hone-claw.com` remain in the embedded browser.
- HTTP(S), email, and telephone links outside the first-party boundary are handed to iOS.
- The native HONE loading/offline surface remains visible until the production page completes loading and supports explicit retry.

## Release Assets

The tag workflow publishes:

- `HONE-iOS-Simulator-vX.Y.Z.zip`: unsigned Simulator app for QA only
- `HONE-iOS-Xcode-vX.Y.Z.zip`: source project for signed device/App Store archives

Do not call the Simulator zip an IPA or imply that it can install on a physical iPhone. Device/TestFlight/App Store delivery requires Apple signing credentials, provisioning, an archive export method, and App Store Connect upload.

## Versioning

Keep `MARKETING_VERSION` in `apps/hone-ios/HONE.xcodeproj/project.pbxproj` aligned with `Cargo.toml` and both Tauri configs. Increment `CURRENT_PROJECT_VERSION` when submitting a second build of the same marketing version.
