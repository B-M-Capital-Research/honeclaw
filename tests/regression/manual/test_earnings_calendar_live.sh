#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd python3
require_cmd rtk

TMP_DIR="$(mktemp -d)"
cleanup() {
  if [[ -n "${MCP_PID:-}" ]]; then
    kill "$MCP_PID" 2>/dev/null || true
    wait "$MCP_PID" 2>/dev/null || true
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

IN_PIPE="$TMP_DIR/in.pipe"
OUT_PIPE="$TMP_DIR/out.pipe"
STDERR_LOG="$TMP_DIR/hone-mcp.stderr.log"
mkfifo "$IN_PIPE" "$OUT_PIPE"

echo "[INFO] building hone-mcp"
rtk cargo build -p hone-mcp >/dev/null

python3 - <<'PY' "$TMP_DIR/init.json" "$TMP_DIR/call.json"
from pathlib import Path
import json
import sys

Path(sys.argv[1]).write_text(
    json.dumps(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": 1, "clientCapabilities": {}},
        }
    )
    + "\n"
)
Path(sys.argv[2]).write_text(
    json.dumps(
        {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "data_fetch",
                "arguments": {"data_type": "earnings_calendar"},
            },
        }
    )
    + "\n"
)
PY

echo "[INFO] starting hone-mcp"
HONE_CONFIG_PATH="$ROOT_DIR/config.yaml" \
HONE_MCP_ACTOR_CHANNEL="cli" \
HONE_MCP_ACTOR_USER_ID="cli_user" \
HONE_MCP_CHANNEL_TARGET="cli" \
HONE_MCP_ALLOW_CRON="0" \
rtk proxy "$ROOT_DIR/target/debug/hone-mcp" <"$IN_PIPE" >"$OUT_PIPE" 2>"$STDERR_LOG" &
MCP_PID=$!

exec 3>"$IN_PIPE"
exec 4<"$OUT_PIPE"

cat "$TMP_DIR/init.json" >&3
read -r INIT_LINE <&4

cat "$TMP_DIR/call.json" >&3
read -r CALL_LINE <&4
printf '%s\n' "$CALL_LINE" >"$TMP_DIR/call_response.json"

python3 - <<'PY' "$TMP_DIR/call_response.json"
import json
import sys
from datetime import date, timedelta

payload = json.loads(open(sys.argv[1]).read())
result = payload.get("result", {})
content = result.get("structuredContent") or {}
window = content.get("request_window") or {}
data = content.get("data")

expected_from = date.today().isoformat()
expected_to = (date.today() + timedelta(days=14)).isoformat()

if result.get("isError"):
    print("[FAIL] earnings_calendar live run returned isError=true", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if content.get("data_type") != "earnings_calendar":
    print("[FAIL] data_type is not earnings_calendar", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if window.get("from") != expected_from:
    print(
        f"[FAIL] request_window.from={window.get('from')} expected {expected_from}",
        file=sys.stderr,
    )
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if window.get("to") != expected_to:
    print(
        f"[FAIL] request_window.to={window.get('to')} expected {expected_to}",
        file=sys.stderr,
    )
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if not isinstance(data, list):
    print("[FAIL] earnings_calendar data is not a list", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

sample = data[0] if data else {}
print("[PASS] earnings_calendar live run succeeded")
print(f"window_from={window.get('from')}")
print(f"window_to={window.get('to')}")
print(f"count={len(data)}")
print(f"sample_date={sample.get('date') if isinstance(sample, dict) else None}")
print(f"sample_symbol={sample.get('symbol') if isinstance(sample, dict) else None}")
PY
