#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

for command in cargo python3; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $command" >&2
    exit 1
  fi
done

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/hone-entity-search.XXXXXX")"
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
cargo build -q -p hone-mcp

TARGET_ROOT="${CARGO_TARGET_DIR:-$ROOT_DIR/target}"
case "$TARGET_ROOT" in
  /*) ;;
  *) TARGET_ROOT="$ROOT_DIR/$TARGET_ROOT" ;;
esac

CONFIG_PATH="${HONE_CONFIG_PATH:-$ROOT_DIR/config.yaml}"
HONE_CONFIG_PATH="$CONFIG_PATH" \
HONE_MCP_ACTOR_CHANNEL="cli" \
HONE_MCP_ACTOR_USER_ID="entity_search_probe" \
HONE_MCP_CHANNEL_TARGET="cli" \
HONE_MCP_ALLOW_CRON="0" \
"$TARGET_ROOT/debug/hone-mcp" <"$IN_PIPE" >"$OUT_PIPE" 2>"$STDERR_LOG" &
MCP_PID=$!

exec 3>"$IN_PIPE"
exec 4<"$OUT_PIPE"

printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{}}}' >&3
read -r _INIT_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"search","query":"NBIS"}}}' >&3
read -r SEARCH_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"quote","ticker":"NBIS"}}}' >&3
read -r QUOTE_LINE <&4

python3 - "$SEARCH_LINE" "$QUOTE_LINE" <<'PY'
import json
import sys


def structured(line):
    payload = json.loads(line)
    result = payload.get("result", {})
    if result.get("isError"):
        raise SystemExit("[FAIL] data_fetch returned isError=true")
    content = result.get("structuredContent") or {}
    if content.get("error"):
        raise SystemExit("[FAIL] data_fetch returned a provider error")
    return content


search = structured(sys.argv[1])
quote = structured(sys.argv[2])

candidates = search.get("data") or []
exact = [item for item in candidates if str(item.get("symbol", "")).upper() == "NBIS"]
if len(exact) != 1 or not (exact[0].get("name") or exact[0].get("companyName")):
    raise SystemExit("[FAIL] NBIS search did not return one exact named candidate")

quotes = quote.get("data") or []
matching = [item for item in quotes if str(item.get("symbol", "")).upper() == "NBIS"]
if len(matching) != 1 or not isinstance(matching[0].get("price"), (int, float)) or matching[0]["price"] <= 0:
    raise SystemExit("[FAIL] NBIS quote did not return one positive same-symbol price")

print("[PASS] live entity search and same-symbol quote succeeded")
print(f"symbol=NBIS company={exact[0].get('name') or exact[0].get('companyName')}")
print(f"quote_price={matching[0]['price']}")
PY
