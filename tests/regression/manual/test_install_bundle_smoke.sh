#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hone-install-smoke.XXXXXX")"
INSTALL_ROOT="$TMP_ROOT/.honeclaw"
CURRENT_ROOT="$INSTALL_ROOT/current"
BIN_DIR="$TMP_ROOT/bin"

cleanup() {
  if [[ -n "${START_PID:-}" ]]; then
    kill "$START_PID" >/dev/null 2>&1 || true
    wait "$START_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

cd "$ROOT_DIR"

echo "[INFO] building local install smoke binaries..."
cargo build -p hone-cli -p hone-console-page -p hone-mcp -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram

echo "[INFO] assembling install-like layout under $TMP_ROOT"
mkdir -p "$CURRENT_ROOT/bin" "$CURRENT_ROOT/share/honeclaw" "$BIN_DIR" "$INSTALL_ROOT/data/runtime"
for bin in hone-cli hone-console-page hone-mcp hone-imessage hone-discord hone-feishu hone-telegram; do
  cp "target/debug/$bin" "$CURRENT_ROOT/bin/$bin"
done
cp config.example.yaml "$CURRENT_ROOT/share/honeclaw/config.example.yaml"
cp soul.md "$CURRENT_ROOT/share/honeclaw/soul.md"
cp -R skills "$CURRENT_ROOT/share/honeclaw/skills"
cp "$CURRENT_ROOT/share/honeclaw/config.example.yaml" "$INSTALL_ROOT/config.yaml"
cp "$CURRENT_ROOT/share/honeclaw/soul.md" "$INSTALL_ROOT/soul.md"

cat > "$BIN_DIR/hone-cli" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

HONE_HOME="${HONE_HOME:-$HOME/.honeclaw}"
CURRENT_ROOT="$HONE_HOME/current"

export HONE_HOME
export HONE_INSTALL_ROOT="${HONE_INSTALL_ROOT:-$CURRENT_ROOT}"
export HONE_USER_CONFIG_PATH="${HONE_USER_CONFIG_PATH:-$HONE_HOME/config.yaml}"
export HONE_DATA_DIR="${HONE_DATA_DIR:-$HONE_HOME/data}"
export HONE_SKILLS_DIR="${HONE_SKILLS_DIR:-$CURRENT_ROOT/share/honeclaw/skills}"

exec "$CURRENT_ROOT/bin/hone-cli" "$@"
EOF
chmod +x "$BIN_DIR/hone-cli"

export PATH="$BIN_DIR:$PATH"
export HOME="$TMP_ROOT"

echo "[INFO] doctor --json"
HONE_HOME="$INSTALL_ROOT" "$BIN_DIR/hone-cli" doctor --json > "$TMP_ROOT/doctor.json"

echo "[INFO] config file"
CONFIG_PATH="$(HONE_HOME="$INSTALL_ROOT" "$BIN_DIR/hone-cli" config file)"
echo "[INFO] config file => $CONFIG_PATH"
if [[ "$CONFIG_PATH" != "$INSTALL_ROOT/config.yaml" ]]; then
  echo "[FAIL] hone-cli config file returned unexpected path: $CONFIG_PATH" >&2
  exit 1
fi

echo "[INFO] start smoke"
HONE_HOME="$INSTALL_ROOT" "$BIN_DIR/hone-cli" start > "$TMP_ROOT/start.log" 2>&1 &
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
  cat "$TMP_ROOT/start.log" >&2
  exit 1
fi

echo "[PASS] install-layout smoke passed"
