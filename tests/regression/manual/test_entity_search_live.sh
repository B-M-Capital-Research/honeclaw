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

printf '%s\n' '{"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"search","query":"RMBS"}}}' >&3
read -r RMBS_SEARCH_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"quote","ticker":"RMBS"}}}' >&3
read -r RMBS_QUOTE_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"profile","ticker":"RMBS"}}}' >&3
read -r RMBS_PROFILE_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"financials","ticker":"RMBS"}}}' >&3
read -r RMBS_FINANCIALS_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"news","ticker":"RMBS"}}}' >&3
read -r RMBS_NEWS_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"search","query":"RKLB"}}}' >&3
read -r RKLB_SEARCH_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"quote","ticker":"RKLB"}}}' >&3
read -r RKLB_QUOTE_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"profile","ticker":"RKLB"}}}' >&3
read -r RKLB_PROFILE_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"financials","ticker":"RKLB"}}}' >&3
read -r RKLB_FINANCIALS_LINE <&4

printf '%s\n' '{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{"name":"data_fetch","arguments":{"data_type":"quote","ticker":"000001.SS,ASHR,KBA,^GSPC,^IXIC,^DJI"}}}' >&3
read -r MIXED_MARKET_QUOTE_LINE <&4

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
  "$BTC_PROFILE_LINE" \
  "$RMBS_SEARCH_LINE" \
  "$RMBS_QUOTE_LINE" \
  "$RMBS_PROFILE_LINE" \
  "$RMBS_FINANCIALS_LINE" \
  "$RMBS_NEWS_LINE" \
  "$RKLB_SEARCH_LINE" \
  "$RKLB_QUOTE_LINE" \
  "$RKLB_PROFILE_LINE" \
  "$RKLB_FINANCIALS_LINE" \
  "$MIXED_MARKET_QUOTE_LINE" <<'PY'
import json
import sys
import time


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
rmbs_search = structured(sys.argv[12])
rmbs_quote = structured(sys.argv[13])
rmbs_profile = structured(sys.argv[14])
rmbs_financials = structured(sys.argv[15])
rmbs_news = structured(sys.argv[16])
rklb_search = structured(sys.argv[17])
rklb_quote = structured(sys.argv[18])
rklb_profile = structured(sys.argv[19])
rklb_financials = structured(sys.argv[20])
mixed_market_quote = structured(sys.argv[21])


def matching_fresh_quote(payload, symbol):
    quotes = payload.get("data") or []
    matching = [item for item in quotes if str(item.get("symbol", "")).upper() == symbol]
    if len(matching) != 1:
        raise SystemExit(f"[FAIL] {symbol} quote did not return one same-symbol record")
    quote = matching[0]
    if not isinstance(quote.get("price"), (int, float)) or quote["price"] <= 0:
        raise SystemExit(f"[FAIL] {symbol} quote did not return a positive price")
    if not isinstance(quote.get("changesPercentage"), (int, float)):
        raise SystemExit(f"[FAIL] {symbol} quote did not return numeric changesPercentage")
    timestamp = quote.get("timestamp")
    if not isinstance(timestamp, (int, float)) or not (time.time() - 5 * 86400 <= timestamp <= time.time() + 300):
        raise SystemExit(f"[FAIL] {symbol} quote timestamp is absent or stale")
    return quote

candidates = search.get("data") or []
exact = [item for item in candidates if str(item.get("symbol", "")).upper() == "NBIS"]
if len(exact) != 1 or not (exact[0].get("name") or exact[0].get("companyName")):
    raise SystemExit("[FAIL] NBIS search did not return one exact named candidate")

matching_quote = matching_fresh_quote(quote, "NBIS")

financial_data = financials.get("data")
if not isinstance(financial_data, (list, dict)) or not financial_data:
    raise SystemExit("[FAIL] NBIS financials did not return non-empty data")

intl_candidates = intl_search.get("data") or []
intl_exact = [item for item in intl_candidates if str(item.get("symbol", "")).upper() == "INTL"]
if len(intl_exact) != 1 or not (intl_exact[0].get("name") or intl_exact[0].get("companyName")):
    raise SystemExit("[FAIL] INTL search did not return one exact named candidate")

intl_matching_quote = matching_fresh_quote(intl_quote, "INTL")

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

btc_matching_quote = matching_fresh_quote(btc_quote, "BTCUSD")

btc_profile_data = btc_profile.get("data")
if not isinstance(btc_profile_data, list) or btc_profile_data:
    raise SystemExit("[FAIL] BTCUSD stock profile was not a successful empty list")

rmbs_candidates = rmbs_search.get("data") or []
rmbs_exact = [item for item in rmbs_candidates if str(item.get("symbol", "")).upper() == "RMBS"]
if len(rmbs_exact) != 1 or "RAMBUS" not in str(rmbs_exact[0].get("name", "")).upper():
    raise SystemExit("[FAIL] RMBS search did not exact-resolve Rambus Inc.")
rmbs_matching_quote = matching_fresh_quote(rmbs_quote, "RMBS")
rmbs_profiles = rmbs_profile.get("data") or []
rmbs_matching_profiles = [
    item for item in rmbs_profiles if str(item.get("symbol", "")).upper() == "RMBS"
]
if len(rmbs_matching_profiles) != 1 or rmbs_matching_profiles[0].get("isEtf") is not False:
    raise SystemExit("[FAIL] RMBS profile did not confirm an exact-symbol non-ETF equity")
rmbs_financial_data = rmbs_financials.get("data")
if not isinstance(rmbs_financial_data, list) or not rmbs_financial_data:
    raise SystemExit("[FAIL] RMBS financials did not return non-empty annual data")
if any(str(item.get("symbol", "")).upper() != "RMBS" for item in rmbs_financial_data):
    raise SystemExit("[FAIL] RMBS financials contained a different symbol")
rmbs_news_data = rmbs_news.get("data")
if not isinstance(rmbs_news_data, list) or not rmbs_news_data:
    raise SystemExit("[FAIL] RMBS news did not return a provider result for filter regression")

rklb_candidates = rklb_search.get("data") or []
rklb_exact = [item for item in rklb_candidates if str(item.get("symbol", "")).upper() == "RKLB"]
if len(rklb_exact) != 1 or "ROCKET LAB" not in str(rklb_exact[0].get("name", "")).upper():
    raise SystemExit("[FAIL] RKLB search did not exact-resolve Rocket Lab USA, Inc.")
rklb_matching_quote = matching_fresh_quote(rklb_quote, "RKLB")
rklb_profiles = rklb_profile.get("data") or []
rklb_matching_profiles = [
    item for item in rklb_profiles if str(item.get("symbol", "")).upper() == "RKLB"
]
if len(rklb_matching_profiles) != 1 or rklb_matching_profiles[0].get("isEtf") is not False:
    raise SystemExit("[FAIL] RKLB profile did not confirm an exact-symbol non-ETF equity")
rklb_financial_data = rklb_financials.get("data")
if not isinstance(rklb_financial_data, list) or not rklb_financial_data:
    raise SystemExit("[FAIL] RKLB financials did not return non-empty annual data")
if any(str(item.get("symbol", "")).upper() != "RKLB" for item in rklb_financial_data):
    raise SystemExit("[FAIL] RKLB financials contained a different symbol")

mixed_market_symbols = ["000001.SS", "ASHR", "KBA", "^GSPC", "^IXIC", "^DJI"]
for symbol in mixed_market_symbols:
    matching_fresh_quote(mixed_market_quote, symbol)

print("[PASS] live RMBS/RKLB/NBIS equities, INTL ETF, and BTCUSD crypto entity/data probes succeeded")
print(f"NBIS company={exact[0].get('name') or exact[0].get('companyName')}")
print(f"NBIS quote_price={matching_quote['price']} change={matching_quote['changesPercentage']} timestamp={int(matching_quote['timestamp'])}")
print(f"NBIS financials_shape={type(financial_data).__name__} items={len(financial_data)}")
print(f"INTL fund={intl_exact[0].get('name') or intl_exact[0].get('companyName')}")
print(f"INTL quote_price={intl_matching_quote['price']} change={intl_matching_quote['changesPercentage']} timestamp={int(intl_matching_quote['timestamp'])}")
print(f"INTL isEtf={profile['isEtf']} isFund={profile['isFund']}")
print(f"INTL holdings_shape={type(holdings_data).__name__} items={len(holdings_data)}")
print("INTL financials_shape=list items=0")
print(f"BTCUSD market={btc_market or btc_exchange} quote_price={btc_matching_quote['price']}")
print("BTCUSD stock_profile_shape=list items=0")
print(f"RMBS company={rmbs_exact[0].get('name')} quote_price={rmbs_matching_quote['price']} change={rmbs_matching_quote['changesPercentage']} timestamp={int(rmbs_matching_quote['timestamp'])}")
print(f"RMBS financials_shape=list items={len(rmbs_financial_data)} news_items={len(rmbs_news_data)}")
print(f"RKLB company={rklb_exact[0].get('name')} quote_price={rklb_matching_quote['price']} change={rklb_matching_quote['changesPercentage']} timestamp={int(rklb_matching_quote['timestamp'])}")
print(f"RKLB financials_shape=list items={len(rklb_financial_data)}")
print(f"mixed_market_live_quotes={','.join(mixed_market_symbols)}")
PY
