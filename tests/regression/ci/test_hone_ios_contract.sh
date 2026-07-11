#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
IOS_DIR="$ROOT_DIR/apps/hone-ios"
TEST_BIN="${TMPDIR:-/tmp}/hone-ios-navigation-policy-test"

if command -v swiftc >/dev/null 2>&1; then
  swiftc \
    "$IOS_DIR/HONE/NavigationPolicy.swift" \
    "$IOS_DIR/Tests/NavigationPolicyTests.swift" \
    -o "$TEST_BIN"
  "$TEST_BIN"
else
  echo "[INFO] swiftc unavailable; running static iOS contract only"
fi

grep -Fq 'https://hone-claw.com/chat' "$IOS_DIR/HONE/NavigationPolicy.swift"
grep -Fq 'com.hone.chat.ios' "$IOS_DIR/HONE.xcodeproj/project.pbxproj"
grep -Fq 'MARKETING_VERSION = 0.13.0' "$IOS_DIR/HONE.xcodeproj/project.pbxproj"
if grep -REni 'hone financial|open financial console|hone-mcp|opencode|codex|feishu' "$IOS_DIR/HONE"; then
  echo "forbidden public brand or local runtime dependency found in iOS client" >&2
  exit 1
fi

echo "[PASS] HONE iOS remote-only contract"
