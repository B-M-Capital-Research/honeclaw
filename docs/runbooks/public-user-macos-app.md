# Public User macOS App

## Purpose

Build the focused HONE macOS client that opens the production `/chat` login and conversation experience without packaging the local HONE runtime. This is separate from `hone-desktop` and must not contain Codex/OpenCode ACP, `hone-mcp`, channel listeners, skills, config, or user data directories.

## Prerequisites

- macOS 12 or newer
- Rust toolchain with `rustup`
- Bun on `PATH`
- Optional for public distribution: an Apple Developer ID Application signing identity and notarization credentials

## Build

```bash
bash scripts/build_user_app.sh
```

The default build is Universal (`arm64` + `x86_64`) and writes isolated build output under:

```text
~/Library/Caches/honeclaw/user-app-target/universal-apple-darwin/release/bundle/
```

Expected deliverables:

```text
macos/HONE.app
dmg/HONE_<version>_universal.dmg
```

Override the target or output root only when needed:

```bash
HONE_USER_APP_TARGET=aarch64-apple-darwin CARGO_TARGET_DIR=/tmp/hone-user-app-target \
  bash scripts/build_user_app.sh
```

## Verification

```bash
cargo test -p hone-user-app
cargo check -p hone-user-app
bash -n scripts/build_user_app.sh
file ~/Library/Caches/honeclaw/user-app-target/universal-apple-darwin/release/bundle/macos/HONE.app/Contents/MacOS/hone-user-app
codesign -dvv ~/Library/Caches/honeclaw/user-app-target/universal-apple-darwin/release/bundle/macos/HONE.app
find ~/Library/Caches/honeclaw/user-app-target/universal-apple-darwin/release/bundle/macos/HONE.app/Contents -maxdepth 3 -type f -print
```

Launch the packaged app and confirm its logs reach `https://hone-claw.com/chat`. The bundle file list should contain only the app metadata, icon, and `hone-user-app` executable; the startup UI is embedded in the executable.

The app uses a fixed named `WKWebsiteDataStore` on macOS 14+ so the HONE session cookie and Web storage survive normal restarts and version upgrades. macOS 12/13 use WebKit's default persistent store because named stores are unavailable there. Do not change `HONE_WEBKIT_DATA_STORE_ID`, enable incognito mode, or change the bundle identifier during a routine release: any of those changes can move users to a different login store.

## Versioning

Keep `bins/hone-user-app/tauri.conf.json` aligned with the workspace version in `Cargo.toml`. The formal release checklist includes this file, but building this app does not itself create a Git tag or publish a formal repository release.

## Signing And Notarization

Tauri can produce an ad-hoc signed artifact when no Apple signing identity is installed. That artifact is suitable for local/internal validation but may trigger Gatekeeper on another Mac. Before public distribution, configure a `Developer ID Application` identity, sign the Universal app, submit it to Apple notarization, staple the ticket, and verify with `spctl --assess`.

## Troubleshooting

- If startup remains on the offline screen, verify `https://hone-claw.com/logo.svg` and `https://hone-claw.com/chat` are reachable from the Mac.
- If a build unexpectedly starts compiling channel binaries or `hone-mcp`, stop it and ensure the command runs through `scripts/build_user_app.sh`; the script changes into `bins/hone-user-app` before invoking Tauri.
- If `cargo fmt --all -- --check` reports unrelated existing files, run `bash scripts/ci/check_fmt_changed.sh` and separately format/check `bins/hone-user-app/src/main.rs` without rewriting unrelated worktree changes.
