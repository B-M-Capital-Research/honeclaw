#!/usr/bin/env bash

set -euo pipefail

if [[ "${HONE_RUN_STRIPE_SANDBOX:-0}" != "1" ]]; then
  echo "[SKIP] set HONE_RUN_STRIPE_SANDBOX=1 after reading docs/runbooks/stripe-billing.md"
  exit 0
fi

for command_name in stripe curl; do
  command -v "$command_name" >/dev/null 2>&1 || {
    echo "[FAIL] missing command: $command_name" >&2
    exit 1
  }
done

required_vars=(
  HONE_STRIPE_PRODUCT_ID
  HONE_STRIPE_SUBSCRIPTION_PRICE_ID
  HONE_STRIPE_FIXED_TERM_PRICE_ID
  HONE_STRIPE_WEBHOOK_URL
)
for variable_name in "${required_vars[@]}"; do
  [[ -n "${!variable_name:-}" ]] || {
    echo "[FAIL] missing environment variable: $variable_name" >&2
    exit 1
  }
done

[[ "$HONE_STRIPE_PRODUCT_ID" == prod_* ]] || {
  echo "[FAIL] HONE_STRIPE_PRODUCT_ID must start with prod_" >&2
  exit 1
}
[[ "$HONE_STRIPE_SUBSCRIPTION_PRICE_ID" == price_* ]] || {
  echo "[FAIL] HONE_STRIPE_SUBSCRIPTION_PRICE_ID must start with price_" >&2
  exit 1
}
[[ "$HONE_STRIPE_FIXED_TERM_PRICE_ID" == price_* ]] || {
  echo "[FAIL] HONE_STRIPE_FIXED_TERM_PRICE_ID must start with price_" >&2
  exit 1
}

product_json="$(stripe products retrieve "$HONE_STRIPE_PRODUCT_ID")"
subscription_price_json="$(stripe prices retrieve "$HONE_STRIPE_SUBSCRIPTION_PRICE_ID")"
fixed_price_json="$(stripe prices retrieve "$HONE_STRIPE_FIXED_TERM_PRICE_ID")"

python3 -c '
import json, sys
payload = json.load(sys.stdin)
assert payload["id"] == sys.argv[1]
assert payload.get("livemode") is False, "product must be in Stripe test mode"
assert payload.get("active") is True, "product must be active"
assert payload.get("name") == "B&M Research Membership — Full Access"
' "$HONE_STRIPE_PRODUCT_ID" <<<"$product_json" >/dev/null

python3 -c '
import json, sys
payload = json.load(sys.stdin)
product = payload.get("product")
if isinstance(product, dict):
    product = product.get("id")
assert payload["id"] == sys.argv[2]
assert product == sys.argv[1]
assert payload.get("livemode") is False, "price must be in Stripe test mode"
assert payload.get("active") is True, "price must be active"
assert payload.get("type") == "recurring"
assert payload.get("currency") == "usd"
assert payload.get("unit_amount") == 19999
assert payload.get("recurring", {}).get("interval") == "year"
assert payload.get("recurring", {}).get("interval_count") == 1
' "$HONE_STRIPE_PRODUCT_ID" "$HONE_STRIPE_SUBSCRIPTION_PRICE_ID" <<<"$subscription_price_json" >/dev/null

python3 -c '
import json, sys
payload = json.load(sys.stdin)
product = payload.get("product")
if isinstance(product, dict):
    product = product.get("id")
assert payload["id"] == sys.argv[2]
assert product == sys.argv[1]
assert payload.get("livemode") is False, "price must be in Stripe test mode"
assert payload.get("active") is True, "price must be active"
assert payload.get("type") == "one_time"
assert payload.get("currency") == "usd"
assert payload.get("unit_amount") == 22999
assert payload.get("recurring") is None
' "$HONE_STRIPE_PRODUCT_ID" "$HONE_STRIPE_FIXED_TERM_PRICE_ID" <<<"$fixed_price_json" >/dev/null

status_code="$(curl -sS -o /dev/null -w '%{http_code}' \
  -X POST \
  -H 'Content-Type: application/json' \
  --data '{}' \
  "$HONE_STRIPE_WEBHOOK_URL")"
[[ "$status_code" == "401" ]] || {
  echo "[FAIL] unsigned webhook must return 401, received $status_code" >&2
  exit 1
}

echo "[PASS] Stripe test catalog has exact recurring and fixed-term prices, and the webhook rejects unsigned input"
echo "[NEXT] run 'stripe listen --forward-to $HONE_STRIPE_WEBHOOK_URL' and complete the signed event matrix in the runbook"
