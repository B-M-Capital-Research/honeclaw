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

printf '%s\n' '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"financials","ticker":"NBIS"}}}' >&3
read -r FINANCIALS_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"search","query":"INTL"}}}' >&3
read -r INTL_SEARCH_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"quote","ticker":"INTL"}}}' >&3
read -r INTL_QUOTE_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"profile","ticker":"INTL"}}}' >&3
read -r INTL_PROFILE_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"etf_holdings","ticker":"INTL"}}}' >&3
read -r INTL_HOLDINGS_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"financials","ticker":"INTL"}}}' >&3
read -r INTL_FINANCIALS_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"search","query":"BTCUSD"}}}' >&3
read -r BTC_SEARCH_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"quote","ticker":"BTCUSD"}}}' >&3
read -r BTC_QUOTE_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"profile","ticker":"BTCUSD"}}}' >&3
read -r BTC_PROFILE_LINE <&4

python3 - \
  "$SEARCH_LINE" \
  "$QUOTE_LINE" \
  "$FINANCIALS_LINE" \
  "$INTL_SEARCH_LINE" \
  "$INTL_QUOTE_LINE" \
  "$INTL_PROFILE_LINE" \
  "$INTL_HOLDINGS_LINE" \
  "$INTL_FINANCIALS_LINE" \
  "$BTC_SEARCH_LINE" \
  "$BTC_QUOTE_LINE" \
  "$BTC_PROFILE_LINE" <<'PY'
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
financials = structured(sys.argv[3])
intl_search = structured(sys.argv[4])
intl_quote = structured(sys.argv[5])
intl_profile = structured(sys.argv[6])
intl_holdings = structured(sys.argv[7])
intl_financials = structured(sys.argv[8])
btc_search = structured(sys.argv[9])
btc_quote = structured(sys.argv[10])
btc_profile = structured(sys.argv[11])

candidates = search.get("data") or []
exact = [item for item in candidates if str(item.get("symbol", "")).upper() == "NBIS"]
if len(exact) != 1 or not (exact[0].get("name") or exact[0].get("companyName")):
    raise SystemExit("[FAIL] NBIS search did not return one exact named candidate")

quotes = quote.get("data") or []
matching = [item for item in quotes if str(item.get("symbol", "")).upper() == "NBIS"]
if len(matching) != 1 or not isinstance(matching[0].get("price"), (int, float)) or matching[0]["price"] <= 0:
    raise SystemExit("[FAIL] NBIS quote did not return one positive same-symbol price")

financial_data = financials.get("data")
if not isinstance(financial_data, (list, dict)) or not financial_data:
    raise SystemExit("[FAIL] NBIS financials did not return non-empty data")

intl_candidates = intl_search.get("data") or []
intl_exact = [item for item in intl_candidates if str(item.get("symbol", "")).upper() == "INTL"]
if len(intl_exact) != 1 or not (intl_exact[0].get("name") or intl_exact[0].get("companyName")):
    raise SystemExit("[FAIL] INTL search did not return one exact named candidate")

intl_quotes = intl_quote.get("data") or []
intl_matching = [item for item in intl_quotes if str(item.get("symbol", "")).upper() == "INTL"]
if (
    len(intl_matching) != 1
    or not isinstance(intl_matching[0].get("price"), (int, float))
    or intl_matching[0]["price"] <= 0
):
    raise SystemExit("[FAIL] INTL quote did not return one positive same-symbol price")

profiles = intl_profile.get("data") or []
matching_profiles = [item for item in profiles if str(item.get("symbol", "")).upper() == "INTL"]
if len(matching_profiles) != 1:
    raise SystemExit("[FAIL] INTL profile did not return one same-symbol record")
profile = matching_profiles[0]
if profile.get("isEtf") is not True or not isinstance(profile.get("isFund"), bool):
    raise SystemExit("[FAIL] INTL profile did not expose isEtf=true and a boolean isFund flag")

holdings_data = intl_holdings.get("data")
if not isinstance(holdings_data, (list, dict)) or not holdings_data:
    raise SystemExit("[FAIL] INTL ETF holdings did not return non-empty data")

intl_financial_data = intl_financials.get("data")
if not isinstance(intl_financial_data, list) or intl_financial_data:
    raise SystemExit("[FAIL] INTL financials were not a successful empty list as expected for this ETF")

btc_candidates = btc_search.get("data") or []
btc_exact = [item for item in btc_candidates if str(item.get("symbol", "")).upper() == "BTCUSD"]
if len(btc_exact) != 1:
    raise SystemExit("[FAIL] BTCUSD search did not return one exact candidate")
btc_market = str(btc_exact[0].get("exchangeShortName", "")).upper()
btc_exchange = str(btc_exact[0].get("stockExchange", "")).upper()
if btc_market != "CRYPTO" and btc_exchange != "CCC":
    raise SystemExit("[FAIL] BTCUSD exact search did not expose structured CRYPTO/CCC market evidence")

btc_quotes = btc_quote.get("data") or []
btc_matching = [item for item in btc_quotes if str(item.get("symbol", "")).upper() == "BTCUSD"]
if (
    len(btc_matching) != 1
    or not isinstance(btc_matching[0].get("price"), (int, float))
    or btc_matching[0]["price"] <= 0
):
    raise SystemExit("[FAIL] BTCUSD quote did not return one positive same-symbol price")

btc_profile_data = btc_profile.get("data")
if not isinstance(btc_profile_data, list) or btc_profile_data:
    raise SystemExit("[FAIL] BTCUSD stock profile was not a successful empty list")

print("[PASS] live NBIS company, INTL ETF, and BTCUSD crypto entity/data probes succeeded")
print(f"NBIS company={exact[0].get('name') or exact[0].get('companyName')}")
print(f"NBIS quote_price={matching[0]['price']}")
print(f"NBIS financials_shape={type(financial_data).__name__} items={len(financial_data)}")
print(f"INTL fund={intl_exact[0].get('name') or intl_exact[0].get('companyName')}")
print(f"INTL quote_price={intl_matching[0]['price']}")
print(f"INTL isEtf={profile['isEtf']} isFund={profile['isFund']}")
print(f"INTL holdings_shape={type(holdings_data).__name__} items={len(holdings_data)}")
print("INTL financials_shape=list items=0")
print(f"BTCUSD market={btc_market or btc_exchange} quote_price={btc_matching[0]['price']}")
print("BTCUSD stock_profile_shape=list items=0")
PY
