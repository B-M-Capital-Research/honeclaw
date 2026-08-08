#!/usr/bin/env bash
#
# One token vocabulary, one theme switch.
#
# The survey page was written against `--hone-surface`, `--hone-text`,
# `--hone-border` and `--hone-accent`, none of which are defined anywhere. Every
# reference fell through to a hardcoded fallback written for the opposite
# theme, so in dark mode its heading rendered at a contrast ratio of 1.01 —
# invisible — while its option chips stayed bright white on a dark page. A
# phantom token fails silently: the CSS parses, the page renders, and only the
# colour is wrong. Nothing but an explicit check surfaces it.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

contains() {
  local pattern="$1"
  local file="$2"

  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

FOUNDATION="packages/app/src/pages/public-foundation.css"
PREFS="packages/app/src/lib/public-prefs.ts"

contains '[data-theme="dark"]' "$FOUNDATION" ||
  { echo "[FAIL] the palette no longer flips at the token layer"; exit 1; }
contains 'return "auto"' "$PREFS" ||
  { echo "[FAIL] a first visit no longer follows the system theme"; exit 1; }

if command -v bun >/dev/null 2>&1; then
  bun test --preload ./packages/app/happydom.ts \
    packages/app/src/pages/public-design-token-contract.test.ts \
    packages/app/src/lib/public-prefs.test.ts
else
  echo "[INFO] bun unavailable; frontend-checks owns the complete Web unit suite"
fi

echo "[PASS] design system token and theme contract"
