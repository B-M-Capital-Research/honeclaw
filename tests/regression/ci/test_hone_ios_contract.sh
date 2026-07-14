#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
IOS_DIR="$ROOT_DIR/apps/hone-ios"
TEST_BIN="${TMPDIR:-/tmp}/hone-ios-navigation-policy-test"
SWIFTC_LOG="${TEST_BIN}.swiftc.log"
trap 'rm -f "$TEST_BIN" "$SWIFTC_LOG"' EXIT
WORKSPACE_VERSION="$(awk '
  /^\[workspace\.package\]$/ { in_workspace_package = 1; next }
  /^\[/ { in_workspace_package = 0 }
  in_workspace_package && /^version = / {
    gsub(/^[^"]*"|".*$/, "", $0)
    print
    exit
  }
' "$ROOT_DIR/Cargo.toml")"

if [[ -z "$WORKSPACE_VERSION" ]]; then
  echo "failed to read workspace version from Cargo.toml" >&2
  exit 1
fi

if command -v swiftc >/dev/null 2>&1; then
  if swiftc \
    "$IOS_DIR/HONE/NavigationPolicy.swift" \
    "$IOS_DIR/Tests/NavigationPolicyTests.swift" \
    -o "$TEST_BIN" 2>"$SWIFTC_LOG"; then
    "$TEST_BIN"
  elif grep -Fq "redefinition of module 'SwiftBridging'" "$SWIFTC_LOG" \
    && grep -Fq '/Library/Developer/CommandLineTools/usr/include/swift/module.modulemap' "$SWIFTC_LOG"; then
    echo "[WARN] local CommandLineTools Swift module maps are duplicated; running static iOS contract only"
  else
    cat "$SWIFTC_LOG" >&2
    exit 1
  fi
else
  echo "[INFO] swiftc unavailable; running static iOS contract only"
fi

grep -Fq 'https://hone-claw.com/chat' "$IOS_DIR/HONE/NavigationPolicy.swift"
grep -Fq 'com.hone.chat.ios' "$IOS_DIR/HONE.xcodeproj/project.pbxproj"
grep -Fq "MARKETING_VERSION = ${WORKSPACE_VERSION};" "$IOS_DIR/HONE.xcodeproj/project.pbxproj"
if grep -REni 'hone financial|open financial console|hone-mcp|opencode|codex|feishu' "$IOS_DIR/HONE"; then
  echo "forbidden public brand or local runtime dependency found in iOS client" >&2
  exit 1
fi

echo "[PASS] HONE iOS remote-only contract"
