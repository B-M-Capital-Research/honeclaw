# HONE for iOS

Native SwiftUI/WKWebView client for the production HONE user experience. The app opens `https://hone-claw.com/chat`, keeps HONE-owned HTTPS routes in the app, hands unrelated links to iOS, persists sign-in through the default WebKit data store, and provides native loading/offline recovery.

## Open And Build

Open `HONE.xcodeproj` in Xcode 16 or newer, select the `HONE` scheme, and run an iPhone Simulator. The deployment target is iOS 16.

Command-line Simulator build:

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

Device/App Store distribution requires a valid Apple Developer team, bundle identifier provisioning, archive signing, and App Store Connect/TestFlight upload. The GitHub Release workflow intentionally publishes a Simulator build and Xcode source archive when signing credentials are unavailable; it does not mislabel them as an installable device IPA.
