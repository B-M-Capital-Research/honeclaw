#!/usr/bin/env bash

set -euo pipefail

TAP_NAME="${HONE_BREW_TAP_NAME:-B-M-Capital-Research/honeclaw}"
TAP_REMOTE="${HONE_BREW_TAP_REMOTE:-https://github.com/B-M-Capital-Research/honeclaw}"
FORMULA_REF="${HONE_BREW_FORMULA_REF:-$TAP_NAME/honeclaw}"
TMP_HOME="$(mktemp -d "${TMPDIR:-/tmp}/hone-brew-smoke.XXXXXX")"
BREW_BIN_DIR="$(brew --prefix)/bin"

cleanup() {
  if [[ -n "${START_PID:-}" ]]; then
    kill "$START_PID" >/dev/null 2>&1 || true
    wait "$START_PID" >/dev/null 2>&1 || true
  fi
  HOMEBREW_NO_AUTO_UPDATE=1 brew uninstall --formula --force "$FORMULA_REF" >/dev/null 2>&1 || true
  brew untap "$TAP_NAME" >/dev/null 2>&1 || true
  rm -rf "$TMP_HOME"
}
trap cleanup EXIT

echo "[INFO] tapping $TAP_NAME from $TAP_REMOTE"
brew untap "$TAP_NAME" >/dev/null 2>&1 || true
HOMEBREW_NO_AUTO_UPDATE=1 brew tap --custom-remote "$TAP_NAME" "$TAP_REMOTE"

echo "[INFO] installing $FORMULA_REF"
HOMEBREW_NO_AUTO_UPDATE=1 brew install "$FORMULA_REF"

export HOME="$TMP_HOME"
export PATH="$BREW_BIN_DIR:$PATH"

echo "[INFO] doctor --json"
hone-cli doctor --json > "$TMP_HOME/doctor.json"

echo "[INFO] config file"
CONFIG_PATH="$(hone-cli config file)"
echo "[INFO] config file => $CONFIG_PATH"
if [[ "$CONFIG_PATH" != "$TMP_HOME/.honeclaw/config.yaml" ]]; then
  echo "[FAIL] hone-cli config file returned unexpected path: $CONFIG_PATH" >&2
  exit 1
fi

echo "[INFO] start smoke"
hone-cli start > "$TMP_HOME/start.log" 2>&1 &
START_PID=$!

READY=0
for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:8077/api/meta >/dev/null 2>&1; then
    READY=1
    break
  fi
  sleep 1
done

if [[ "$READY" -ne 1 ]]; then
  echo "[FAIL] hone-cli start did not become ready" >&2
  cat "$TMP_HOME/start.log" >&2
  exit 1
fi

echo "[PASS] brew install smoke passed"
