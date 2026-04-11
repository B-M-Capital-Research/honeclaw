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
                "arguments": {"data_type": "snapshot", "symbol": "AAPL"},
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

python3 - <<'PY' "$CALL_LINE"
import json
import sys

payload = json.loads(sys.argv[1])
result = payload.get("result", {})
content = result.get("structuredContent") or {}
data = content.get("data") or {}
quote = (data.get("quote") or [None])[0]
profile = (data.get("profile") or [None])[0]
news = (data.get("news") or [None])[0]

if result.get("isError"):
    print("[FAIL] snapshot live run returned isError=true", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if content.get("data_type") != "snapshot":
    print("[FAIL] data_type is not snapshot", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if not isinstance(quote, dict) or quote.get("symbol") != "AAPL":
    print("[FAIL] quote payload missing AAPL symbol", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if not isinstance(profile, dict) or not profile.get("companyName"):
    print("[FAIL] profile payload missing companyName", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

if not isinstance(news, dict) or not news.get("title"):
    print("[FAIL] news payload missing title", file=sys.stderr)
    print(json.dumps(payload, ensure_ascii=False), file=sys.stderr)
    sys.exit(1)

print("[PASS] snapshot live run succeeded")
print(f"quote_symbol={quote.get('symbol')}")
print(f"quote_price={quote.get('price')}")
print(f"profile_company={profile.get('companyName')}")
print(f"news_title={news.get('title')}")
PY
