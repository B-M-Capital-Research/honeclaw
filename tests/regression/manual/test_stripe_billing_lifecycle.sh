#!/usr/bin/env bash

set -euo pipefail

if [[ "${HONE_RUN_STRIPE_LIFECYCLE:-0}" != "1" ]]; then
  echo "[SKIP] set HONE_RUN_STRIPE_LIFECYCLE=1 after reading docs/runbooks/stripe-billing.md"
  exit 0
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

for command_name in cargo curl jq psql python3 stripe; do
  command -v "$command_name" >/dev/null 2>&1 || {
    echo "[FAIL] missing command: $command_name" >&2
    exit 1
  }
done

[[ "${HONE_STRIPE_SECRET_KEY:-}" == sk_test_* ]] || {
  echo "[FAIL] HONE_STRIPE_SECRET_KEY must be an exported Stripe test key" >&2
  exit 1
}

RUN_ID="$(date -u +%Y%m%dt%H%M%Sz)-$$"
USER_ID="web_stripe_lifecycle_${RUN_ID//[^A-Za-z0-9]/_}"
SESSION_TOKEN="stripe-lifecycle-session-${RUN_ID}"
EMAIL_ADDRESS="stripe-lifecycle-${RUN_ID}@hone-claw.invalid"
TMP_ROOT="$(mktemp -d)"
chmod 700 "$TMP_ROOT"

if [[ -n "${DATABASE_URL:-}" ]]; then
  PSQL=(psql -X --set=ON_ERROR_STOP=1 "$DATABASE_URL")
else
  : "${HONE_POSTGRES_HOST:?HONE_POSTGRES_HOST is required}"
  : "${HONE_POSTGRES_PORT:?HONE_POSTGRES_PORT is required}"
  : "${HONE_POSTGRES_USER:?HONE_POSTGRES_USER is required}"
  : "${HONE_POSTGRES_PASSWORD:?HONE_POSTGRES_PASSWORD is required}"
  : "${HONE_POSTGRES_DATABASE:?HONE_POSTGRES_DATABASE is required}"
  export PGPASSWORD="$HONE_POSTGRES_PASSWORD"
  PSQL=(
    psql -X --set=ON_ERROR_STOP=1
    --host "$HONE_POSTGRES_HOST"
    --port "$HONE_POSTGRES_PORT"
    --username "$HONE_POSTGRES_USER"
    --dbname "$HONE_POSTGRES_DATABASE"
  )
fi

SERVER_PID=""
LISTENER_PID=""
SUBSCRIPTION_CHECKOUT_SESSION_ID=""
SUBSCRIPTION_CHECKOUT_SESSION_EXPIRED=0
FIXED_CHECKOUT_SESSION_ID=""
FIXED_CHECKOUT_SESSION_EXPIRED=0
TEST_CLOCK_ID=""
TEST_CLOCK_DELETED=0
PRODUCT_ID=""
PRODUCT_ARCHIVED=0
SUBSCRIPTION_PRICE_ID=""
SUBSCRIPTION_PRICE_ARCHIVED=0
FIXED_PRICE_ID=""
FIXED_PRICE_ARCHIVED=0
PG_SCHEMA_READY=0

cleanup_billing_rows() {
  "${PSQL[@]}" --set=user_id="$USER_ID" >/dev/null <<'SQL'
DELETE FROM billing_entitlements WHERE user_id = :'user_id';
DELETE FROM billing_webhook_events
WHERE record::text LIKE '%' || :'user_id' || '%';
DELETE FROM cloud_web_auth_sessions WHERE user_id = :'user_id';
DELETE FROM cloud_web_user_external_state WHERE user_id = :'user_id';
DELETE FROM cloud_web_invite_users WHERE user_id = :'user_id';
SQL
}

stripe_api() {
  stripe --color off "$@"
}

redacted_tail() {
  local file="$1"
  [[ -f "$file" ]] || return 0
  tail -n 80 "$file" \
    | sed -E \
      -e 's/sk_test_[A-Za-z0-9_]+/<redacted-test-key>/g' \
      -e 's/whsec_[A-Za-z0-9_]+/<redacted-webhook-secret>/g'
}

cleanup() {
  trap - ERR
  set +e

  if [[ -n "$SUBSCRIPTION_CHECKOUT_SESSION_ID" && "$SUBSCRIPTION_CHECKOUT_SESSION_EXPIRED" != 1 ]]; then
    stripe_api checkout sessions expire "$SUBSCRIPTION_CHECKOUT_SESSION_ID" --confirm >/dev/null 2>&1
  fi
  if [[ -n "$FIXED_CHECKOUT_SESSION_ID" && "$FIXED_CHECKOUT_SESSION_EXPIRED" != 1 ]]; then
    stripe_api checkout sessions expire "$FIXED_CHECKOUT_SESSION_ID" --confirm >/dev/null 2>&1
  fi
  if [[ -n "$TEST_CLOCK_ID" && "$TEST_CLOCK_DELETED" != 1 ]]; then
    stripe_api test_helpers test_clocks delete "$TEST_CLOCK_ID" --confirm >/dev/null 2>&1
  fi
  if [[ -n "$SUBSCRIPTION_PRICE_ID" && "$SUBSCRIPTION_PRICE_ARCHIVED" != 1 ]]; then
    stripe_api prices update "$SUBSCRIPTION_PRICE_ID" --active=false --confirm >/dev/null 2>&1
  fi
  if [[ -n "$FIXED_PRICE_ID" && "$FIXED_PRICE_ARCHIVED" != 1 ]]; then
    stripe_api prices update "$FIXED_PRICE_ID" --active=false --confirm >/dev/null 2>&1
  fi
  if [[ -n "$PRODUCT_ID" && "$PRODUCT_ARCHIVED" != 1 ]]; then
    stripe_api products update "$PRODUCT_ID" --active=false --confirm >/dev/null 2>&1
  fi

  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  if [[ -n "$LISTENER_PID" ]] && kill -0 "$LISTENER_PID" 2>/dev/null; then
    kill "$LISTENER_PID" 2>/dev/null || true
    wait "$LISTENER_PID" 2>/dev/null || true
  fi
  if [[ "$PG_SCHEMA_READY" == 1 ]]; then
    cleanup_billing_rows || true
  fi

  case "$TMP_ROOT" in
    /tmp/* | /private/tmp/* | /var/folders/* | /private/var/folders/*)
      rm -rf -- "$TMP_ROOT"
      ;;
    *)
      printf '[WARN] refusing to remove unexpected temp path: %s\n' "$TMP_ROOT" >&2
      ;;
  esac
}
trap cleanup EXIT

on_error() {
  local status="$?"
  local line_number="$1"
  trap - ERR
  echo "[FAIL] command failed at lifecycle script line $line_number" >&2
  redacted_tail "$TMP_ROOT/server.log" >&2
  redacted_tail "$TMP_ROOT/stripe-listener.log" >&2
  exit "$status"
}
trap 'on_error "$LINENO"' ERR

fail() {
  local message="$1"
  echo "[FAIL] $message" >&2
  redacted_tail "$TMP_ROOT/server.log" >&2
  redacted_tail "$TMP_ROOT/stripe-listener.log" >&2
  exit 1
}

wait_for_billing() {
  local label="$1"
  local filter="$2"
  local payload=""

  for _ in {1..360}; do
    payload="$(curl -fsS \
      -H "Cookie: hone_web_session=$SESSION_TOKEN" \
      "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/status" 2>/dev/null || true)"
    if [[ -n "$payload" ]] && jq -e "$filter" >/dev/null 2>&1 <<<"$payload"; then
      return 0
    fi
    sleep 0.5
  done

  if [[ -n "$payload" ]]; then
    jq -c '{
      access_granted: .billing.access_granted,
      entitlements: [
        .billing.entitlements[]? | {
          provider,
          raw_status,
          access_state,
          cancel_at_period_end,
          has_grace_deadline: (.grace_expires_at != null)
        }
      ]
    }' <<<"$payload" >&2 || true
  fi
  fail "$label"
}

paid_api_status() {
  curl -sS -o /dev/null -w '%{http_code}' \
    -H "Cookie: hone_web_session=$SESSION_TOKEN" \
    "http://127.0.0.1:$PUBLIC_PORT/api/public/bootstrap"
}

wait_for_clock_ready() {
  local expected_time="$1"
  local payload=""

  for _ in {1..360}; do
    payload="$(stripe_api test_helpers test_clocks retrieve "$TEST_CLOCK_ID")"
    if jq -e \
      --argjson expected_time "$expected_time" \
      '.status == "ready" and .frozen_time >= $expected_time' \
      >/dev/null <<<"$payload"; then
      return 0
    fi
    sleep 0.5
  done
  fail "Stripe test clock did not become ready"
}

echo "[INFO] building isolated HONE billing runtime"
cargo build -p hone-console-page --quiet

read -r ADMIN_PORT PUBLIC_PORT < <(python3 - <<'PY'
import socket

sockets = []
try:
    for _ in range(2):
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.bind(("127.0.0.1", 0))
        sockets.append(sock)
    print(*(sock.getsockname()[1] for sock in sockets))
finally:
    for sock in sockets:
        sock.close()
PY
)

mkdir -p "$TMP_ROOT/data" "$TMP_ROOT/skills" "$TMP_ROOT/web-admin" "$TMP_ROOT/web-public"
ln -s "$REPO_ROOT/soul.md" "$TMP_ROOT/soul.md"
sed 's/^  udp_port: null$/  udp_port: 0/' \
  "$REPO_ROOT/config.example.yaml" > "$TMP_ROOT/config.yaml"

echo "[INFO] creating disposable Stripe test catalog"
product_json="$(stripe_api products create \
  --name "HONE Billing Lifecycle Regression $RUN_ID" \
  --description 'Disposable test-mode catalog; archive after lifecycle regression' \
  -d "metadata[hone_regression_run]=$RUN_ID" \
  --idempotency "hone-lifecycle-product-$RUN_ID" \
  --confirm)"
PRODUCT_ID="$(jq -er '.id' <<<"$product_json")"
jq -e '.livemode == false and .active == true' >/dev/null <<<"$product_json" \
  || fail "disposable product was not created in test mode"

price_json="$(stripe_api prices create \
  --currency usd \
  --unit-amount 199 \
  --product "$PRODUCT_ID" \
  --recurring.interval year \
  --nickname 'HONE lifecycle regression only' \
  -d "metadata[hone_regression_run]=$RUN_ID" \
  --idempotency "hone-lifecycle-price-$RUN_ID" \
  --confirm)"
SUBSCRIPTION_PRICE_ID="$(jq -er '.id' <<<"$price_json")"
jq -e \
  --arg product_id "$PRODUCT_ID" \
  '.livemode == false and .active == true and .product == $product_id and .recurring.interval == "year"' \
  >/dev/null <<<"$price_json" \
  || fail "disposable annual price was not created correctly"

fixed_price_json="$(stripe_api prices create \
  --currency usd \
  --unit-amount 22999 \
  --product "$PRODUCT_ID" \
  --nickname 'HONE fixed-term lifecycle regression only' \
  -d "metadata[hone_regression_run]=$RUN_ID" \
  --idempotency "hone-lifecycle-fixed-price-$RUN_ID" \
  --confirm)"
FIXED_PRICE_ID="$(jq -er '.id' <<<"$fixed_price_json")"
jq -e \
  --arg product_id "$PRODUCT_ID" \
  '.livemode == false and .active == true and .product == $product_id and .type == "one_time" and .unit_amount == 22999' \
  >/dev/null <<<"$fixed_price_json" \
  || fail "disposable fixed-term price was not created correctly"

WEBHOOK_SECRET="$(stripe listen --print-secret --skip-update --color off)"
[[ "$WEBHOOK_SECRET" == whsec_* ]] || fail "Stripe CLI did not return a test listener secret"

stripe listen \
  --skip-update \
  --color off \
  --events checkout.session.completed,checkout.session.async_payment_succeeded,checkout.session.async_payment_failed,checkout.session.expired,invoice.paid,invoice.payment_failed,customer.subscription.created,customer.subscription.updated,customer.subscription.deleted,charge.refunded \
  --forward-to "http://127.0.0.1:$PUBLIC_PORT/api/public/integrations/stripe/webhook" \
  > "$TMP_ROOT/stripe-listener.log" 2>&1 &
LISTENER_PID=$!

listener_ready=0
for _ in {1..100}; do
  if grep -Fq 'Ready!' "$TMP_ROOT/stripe-listener.log"; then
    listener_ready=1
    break
  fi
  if ! kill -0 "$LISTENER_PID" 2>/dev/null; then
    break
  fi
  sleep 0.1
done
[[ "$listener_ready" == 1 ]] || fail "Stripe CLI listener did not become ready"

(
  cd "$TMP_ROOT"
  exec env \
    HONE_CONFIG_PATH="$TMP_ROOT/config.yaml" \
    HONE_DATA_DIR="$TMP_ROOT/data" \
    HONE_SKILLS_DIR="$TMP_ROOT/skills" \
    HONE_WEB_PORT="$ADMIN_PORT" \
    HONE_PUBLIC_WEB_PORT="$PUBLIC_PORT" \
    HONE_DISABLE_AUTO_OPEN=1 \
    HONE_DEPLOYMENT_MODE=local \
    HONE_WEB_DIST_DIR="$TMP_ROOT/web-admin" \
    HONE_PUBLIC_WEB_DIST_DIR="$TMP_ROOT/web-public" \
    HONE_PUBLIC_ALLOWED_ORIGINS="http://127.0.0.1:$PUBLIC_PORT" \
    HONE_PUBLIC_SECURE_COOKIE=false \
    HONE_STRIPE_CHECKOUT_ENABLED=true \
    HONE_STRIPE_MODE=test \
    HONE_STRIPE_SECRET_KEY="$HONE_STRIPE_SECRET_KEY" \
    HONE_STRIPE_WEBHOOK_SECRET="$WEBHOOK_SECRET" \
    HONE_STRIPE_PRODUCT_ID="$PRODUCT_ID" \
    HONE_STRIPE_SUBSCRIPTION_PRICE_ID="$SUBSCRIPTION_PRICE_ID" \
    HONE_STRIPE_FIXED_TERM_PRICE_ID="$FIXED_PRICE_ID" \
    HONE_STRIPE_PUBLIC_BASE_URL="http://127.0.0.1:$PUBLIC_PORT/" \
    HONE_BILLING_GRACE_DAYS=7 \
    "$REPO_ROOT/target/debug/hone-console-page"
) > "$TMP_ROOT/server.log" 2>&1 &
SERVER_PID=$!
unset WEBHOOK_SECRET

server_ready=0
for _ in {1..100}; do
  if curl -fsS "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/config" >/dev/null 2>&1; then
    server_ready=1
    break
  fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    break
  fi
  sleep 0.1
done
[[ "$server_ready" == 1 ]] || fail "isolated HONE billing runtime did not become ready"

NOW_RFC3339="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
SESSION_HASH="$(python3 - "$SESSION_TOKEN" <<'PY'
import hashlib
import sys

print(hashlib.sha256(sys.argv[1].encode()).hexdigest())
PY
)"
PG_SCHEMA_READY=1
cleanup_billing_rows
"${PSQL[@]}" \
  --set=user_id="$USER_ID" \
  --set=session_hash="$SESSION_HASH" \
  --set=email_address="$EMAIL_ADDRESS" \
  --set=now="$NOW_RFC3339" >/dev/null <<'SQL'
INSERT INTO cloud_web_invite_users(user_id, phone_number, is_admin, record)
VALUES (
  :'user_id',
  '',
  FALSE,
  jsonb_build_object(
    'user_id', :'user_id',
    'invite_code', 'HONE-STRIPE-LIFECYCLE-' || :'user_id',
    'phone_number', '',
    'created_at', :'now',
    'last_login_at', :'now',
    'revoked_at', NULL,
    'password_hash', NULL,
    'password_set_at', NULL,
    'tos_accepted_at', :'now',
    'tos_version', '2.4',
    'api_key_prefix', NULL,
    'api_key_created_at', NULL,
    'api_key_last_used_at', NULL
  )
);
INSERT INTO cloud_web_user_external_state(
  user_id, email_address, email_verified_at, identity_kind
) VALUES (:'user_id', :'email_address', :'now', 'international_email');
INSERT INTO cloud_web_auth_sessions(session_hash, user_id, record, expires_at)
VALUES (
  :'session_hash',
  :'user_id',
  jsonb_build_object(
    'session_hash', :'session_hash',
    'user_id', :'user_id',
    'created_at', :'now',
    'expires_at', '2099-08-03T00:00:00Z',
    'last_seen_at', :'now'
  ),
  '2099-08-03T00:00:00Z'::timestamptz
);
SQL

wait_for_billing \
  "fresh international account did not start fail-closed" \
  '.billing.access_granted == false and (.billing.entitlements | length) == 0'
[[ "$(paid_api_status)" == 402 ]] || fail "paid API did not start at 402"

echo "[INFO] verifying real Checkout API creation without completing payment"
checkout_json="$(curl -fsS \
  -X POST \
  -H 'Content-Type: application/json' \
  -H "Cookie: hone_web_session=$SESSION_TOKEN" \
  -H "Origin: http://127.0.0.1:$PUBLIC_PORT" \
  -H 'Sec-Fetch-Site: same-origin' \
  --data '{"offer":"subscription"}' \
  "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/checkout/stripe")"
checkout_url="$(jq -er '.checkout_url' <<<"$checkout_json")"
[[ "$checkout_url" == https://checkout.stripe.com/* ]] \
  || fail "HONE returned an untrusted Checkout URL"
SUBSCRIPTION_CHECKOUT_SESSION_ID="$(grep -Eo 'cs_test_[A-Za-z0-9_]+' <<<"$checkout_url" | head -n 1)"
[[ "$SUBSCRIPTION_CHECKOUT_SESSION_ID" == cs_test_* ]] || fail "could not identify the test Checkout Session"
checkout_object="$(stripe_api checkout sessions retrieve "$SUBSCRIPTION_CHECKOUT_SESSION_ID")"
jq -e \
  --arg user_id "$USER_ID" \
  --arg product_id "$PRODUCT_ID" \
  --arg price_id "$SUBSCRIPTION_PRICE_ID" \
  '.livemode == false
    and .mode == "subscription"
    and .client_reference_id == $user_id
    and .metadata.hone_user_id == $user_id
    and .metadata.hone_product_id == $product_id
    and .metadata.hone_price_id == $price_id
    and .metadata.hone_entitlement_kind == "recurring_subscription"' \
  >/dev/null <<<"$checkout_object" \
  || fail "Checkout Session metadata did not preserve the HONE binding"
stripe_api checkout sessions expire "$SUBSCRIPTION_CHECKOUT_SESSION_ID" --confirm >/dev/null
SUBSCRIPTION_CHECKOUT_SESSION_EXPIRED=1

echo "[INFO] verifying real one-time Checkout API creation without completing payment"
fixed_checkout_json="$(curl -fsS \
  -X POST \
  -H 'Content-Type: application/json' \
  -H "Cookie: hone_web_session=$SESSION_TOKEN" \
  -H "Origin: http://127.0.0.1:$PUBLIC_PORT" \
  -H 'Sec-Fetch-Site: same-origin' \
  --data '{"offer":"fixed_term"}' \
  "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/checkout/stripe")"
fixed_checkout_url="$(jq -er '.checkout_url' <<<"$fixed_checkout_json")"
[[ "$fixed_checkout_url" == https://checkout.stripe.com/* ]] \
  || fail "HONE returned an untrusted fixed-term Checkout URL"
FIXED_CHECKOUT_SESSION_ID="$(grep -Eo 'cs_test_[A-Za-z0-9_]+' <<<"$fixed_checkout_url" | head -n 1)"
[[ "$FIXED_CHECKOUT_SESSION_ID" == cs_test_* ]] || fail "could not identify the fixed-term Checkout Session"
fixed_checkout_object="$(stripe_api checkout sessions retrieve "$FIXED_CHECKOUT_SESSION_ID")"
jq -e \
  --arg user_id "$USER_ID" \
  --arg product_id "$PRODUCT_ID" \
  --arg price_id "$FIXED_PRICE_ID" \
  '.livemode == false
    and .mode == "payment"
    and .client_reference_id == $user_id
    and .metadata.hone_user_id == $user_id
    and .metadata.hone_product_id == $product_id
    and .metadata.hone_price_id == $price_id
    and .metadata.hone_entitlement_kind == "fixed_term_purchase"
    and .metadata.hone_term_months == "12"' \
  >/dev/null <<<"$fixed_checkout_object" \
  || fail "fixed-term Checkout Session did not preserve the HONE binding"
stripe_api checkout sessions expire "$FIXED_CHECKOUT_SESSION_ID" --confirm >/dev/null
FIXED_CHECKOUT_SESSION_EXPIRED=1

echo "[INFO] creating disposable Stripe Test Clock customer and paid subscription"
clock_json="$(stripe_api test_helpers test_clocks create \
  --frozen-time "$(date -u +%s)" \
  --name "HONE lifecycle $RUN_ID" \
  --confirm)"
TEST_CLOCK_ID="$(jq -er '.id' <<<"$clock_json")"
jq -e '.livemode == false and .status == "ready"' >/dev/null <<<"$clock_json" \
  || fail "Stripe Test Clock was not created in test mode"

customer_json="$(stripe_api customers create \
  --email "$EMAIL_ADDRESS" \
  --name 'HONE Billing Lifecycle Regression' \
  --description "Disposable HONE lifecycle run $RUN_ID" \
  --payment-method pm_card_visa \
  -d 'invoice_settings[default_payment_method]=pm_card_visa' \
  -d "metadata[hone_regression_run]=$RUN_ID" \
  -d "test_clock=$TEST_CLOCK_ID" \
  --idempotency "hone-lifecycle-customer-$RUN_ID" \
  --confirm)"
CUSTOMER_ID="$(jq -er '.id' <<<"$customer_json")"
SUCCESS_PAYMENT_METHOD_ID="$(jq -er '.invoice_settings.default_payment_method' <<<"$customer_json")"
jq -e \
  --arg clock_id "$TEST_CLOCK_ID" \
  '.livemode == false
    and .test_clock == $clock_id
    and (.invoice_settings.default_payment_method | startswith("pm_"))' \
  >/dev/null <<<"$customer_json" \
  || fail "test customer was not attached to the disposable clock"

subscription_json="$(stripe_api subscriptions create \
  --customer "$CUSTOMER_ID" \
  --default-payment-method "$SUCCESS_PAYMENT_METHOD_ID" \
  --payment-behavior error_if_incomplete \
  -d "items[0][price]=$SUBSCRIPTION_PRICE_ID" \
  -d "metadata[hone_user_id]=$USER_ID" \
  -d "metadata[hone_product_id]=$PRODUCT_ID" \
  -d "metadata[hone_price_id]=$SUBSCRIPTION_PRICE_ID" \
  -d "metadata[hone_regression_run]=$RUN_ID" \
  --idempotency "hone-lifecycle-subscription-1-$RUN_ID" \
  --confirm)"
SUBSCRIPTION_ID="$(jq -er '.id' <<<"$subscription_json")"
CURRENT_PERIOD_END="$(jq -er \
  '.current_period_end // .items.data[0].current_period_end // .items.data[0].current_period.end' \
  <<<"$subscription_json")"
jq -e '.livemode == false and .status == "active"' >/dev/null <<<"$subscription_json" \
  || fail "initial test subscription did not become active"

wait_for_billing \
  "real Stripe invoice.paid did not grant HONE access" \
  '.billing.access_granted == true
    and ([.billing.entitlements[] | select(.provider == "stripe" and .access_state == "active")] | length) == 1'
[[ "$(paid_api_status)" == 200 ]] || fail "paid API did not change from 402 to 200"

echo "[INFO] verifying real Customer Portal session creation"
portal_json="$(curl -fsS \
  -X POST \
  -H "Cookie: hone_web_session=$SESSION_TOKEN" \
  -H "Origin: http://127.0.0.1:$PUBLIC_PORT" \
  -H 'Sec-Fetch-Site: same-origin' \
  "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/portal/stripe")"
portal_url="$(jq -er '.portal_url' <<<"$portal_json")"
[[ "$portal_url" == https://billing.stripe.com/* ]] \
  || fail "HONE returned an untrusted Customer Portal URL"

echo "[INFO] forcing a real renewal failure through the Stripe Test Clock"
failure_payment_method_json="$(stripe_api payment_methods attach pm_card_chargeCustomerFail \
  --customer "$CUSTOMER_ID" \
  --confirm)"
FAILURE_PAYMENT_METHOD_ID="$(jq -er '.id' <<<"$failure_payment_method_json")"
stripe_api subscriptions update "$SUBSCRIPTION_ID" \
  --default-payment-method "$FAILURE_PAYMENT_METHOD_ID" \
  --proration-behavior none \
  --confirm >/dev/null
ADVANCE_TO=$((CURRENT_PERIOD_END + 60))
stripe_api test_helpers test_clocks advance "$TEST_CLOCK_ID" \
  --frozen-time "$ADVANCE_TO" \
  --confirm >/dev/null
wait_for_clock_ready "$ADVANCE_TO"

# Stripe first creates the renewal invoice in draft state, then schedules its
# automatic payment attempt roughly an hour later. Advance to Stripe's own
# next_payment_attempt timestamp so the failure is produced by Billing rather
# than by a fabricated webhook.
NEXT_PAYMENT_ATTEMPT=""
for _ in {1..120}; do
  renewal_invoices="$(stripe_api invoices list \
    --subscription "$SUBSCRIPTION_ID" \
    --limit 10)"
  NEXT_PAYMENT_ATTEMPT="$(jq -r \
    '[.data[] | select(.next_payment_attempt != null) | .next_payment_attempt] | max // empty' \
    <<<"$renewal_invoices")"
  if [[ "$NEXT_PAYMENT_ATTEMPT" =~ ^[0-9]+$ ]]; then
    break
  fi
  sleep 0.5
done
[[ "$NEXT_PAYMENT_ATTEMPT" =~ ^[0-9]+$ ]] \
  || fail "Stripe did not schedule the renewal invoice payment attempt"
PAYMENT_ATTEMPT_ADVANCE_TO=$((NEXT_PAYMENT_ATTEMPT + 60))
stripe_api test_helpers test_clocks advance "$TEST_CLOCK_ID" \
  --frozen-time "$PAYMENT_ATTEMPT_ADVANCE_TO" \
  --confirm >/dev/null
wait_for_clock_ready "$PAYMENT_ATTEMPT_ADVANCE_TO"
wait_for_billing \
  "real Stripe renewal failure did not enter bounded grace" \
  '.billing.access_granted == true
    and ([.billing.entitlements[]
      | select(.provider == "stripe" and .access_state == "grace" and .grace_expires_at != null)]
      | length) == 1'

echo "[INFO] recovering the failed invoice with a successful test payment method"
stripe_api subscriptions update "$SUBSCRIPTION_ID" \
  --default-payment-method "$SUCCESS_PAYMENT_METHOD_ID" \
  --proration-behavior none \
  --confirm >/dev/null
open_invoices="$(stripe_api invoices list \
  --subscription "$SUBSCRIPTION_ID" \
  --status open \
  --limit 10)"
FAILED_INVOICE_ID="$(jq -er '.data | sort_by(.created) | reverse | .[0].id' <<<"$open_invoices")"
stripe_api invoices pay "$FAILED_INVOICE_ID" \
  --payment-method "$SUCCESS_PAYMENT_METHOD_ID" \
  --confirm >/dev/null
wait_for_billing \
  "real Stripe invoice recovery did not restore active access" \
  '.billing.access_granted == true
    and ([.billing.entitlements[] | select(.provider == "stripe" and .access_state == "active")] | length) == 1'

echo "[INFO] verifying cancel-at-period-end, immediate end, and repurchase"
stripe_api subscriptions update "$SUBSCRIPTION_ID" \
  --cancel-at-period-end=true \
  --proration-behavior none \
  --confirm >/dev/null
wait_for_billing \
  "period-end cancellation revoked access too early" \
  '.billing.access_granted == true
    and ([.billing.entitlements[]
      | select(.provider == "stripe" and .access_state == "active" and .cancel_at_period_end == true)]
      | length) == 1'

stripe_api subscriptions cancel "$SUBSCRIPTION_ID" --confirm >/dev/null
wait_for_billing \
  "immediate cancellation did not revoke the ended subscription" \
  '.billing.access_granted == false
    and ([.billing.entitlements[] | select(.provider == "stripe" and .access_state == "inactive")] | length) == 1'
[[ "$(paid_api_status)" == 402 ]] || fail "paid API did not return to 402 after cancellation"

repurchase_json="$(stripe_api subscriptions create \
  --customer "$CUSTOMER_ID" \
  --default-payment-method "$SUCCESS_PAYMENT_METHOD_ID" \
  --payment-behavior error_if_incomplete \
  -d "items[0][price]=$SUBSCRIPTION_PRICE_ID" \
  -d "metadata[hone_user_id]=$USER_ID" \
  -d "metadata[hone_product_id]=$PRODUCT_ID" \
  -d "metadata[hone_price_id]=$SUBSCRIPTION_PRICE_ID" \
  -d "metadata[hone_regression_run]=$RUN_ID" \
  --idempotency "hone-lifecycle-subscription-2-$RUN_ID" \
  --confirm)"
REPURCHASE_SUBSCRIPTION_ID="$(jq -er '.id' <<<"$repurchase_json")"
[[ "$REPURCHASE_SUBSCRIPTION_ID" != "$SUBSCRIPTION_ID" ]] \
  || fail "repurchase reused the canceled subscription"
wait_for_billing \
  "real Stripe repurchase did not restore access with a new subscription" \
  '.billing.access_granted == true
    and ([.billing.entitlements[] | select(.provider == "stripe" and .access_state == "active")] | length) == 1
    and ([.billing.entitlements[] | select(.provider == "stripe" and .access_state == "inactive")] | length) >= 1'
[[ "$(paid_api_status)" == 200 ]] || fail "paid API did not return to 200 after repurchase"

read -r event_count unfinished failed_attempts active_rows inactive_rows < <(
  "${PSQL[@]}" \
    --set=user_id="$USER_ID" \
    --tuples-only --no-align --field-separator=' ' <<'SQL'
WITH matching_events AS (
  SELECT *
  FROM billing_webhook_events
  WHERE provider = 'stripe'
    AND record::text LIKE '%' || :'user_id' || '%'
), matching_entitlements AS (
  SELECT *
  FROM billing_entitlements
  WHERE provider = 'stripe' AND user_id = :'user_id'
)
SELECT
  (SELECT COUNT(*) FROM matching_events),
  (SELECT COUNT(*) FROM matching_events WHERE processing_state != 'processed'),
  (SELECT COUNT(*) FROM matching_events WHERE attempt_count != 1 OR last_error IS NOT NULL),
  (SELECT COUNT(*) FROM matching_entitlements WHERE access_state = 'active'),
  (SELECT COUNT(*) FROM matching_entitlements WHERE access_state = 'inactive');
SQL
)
(( event_count >= 8 )) || fail "expected at least 8 processed Stripe events, got $event_count"
[[ "$unfinished" == 0 ]] || fail "unfinished Stripe events remain: $unfinished"
[[ "$failed_attempts" == 0 ]] || fail "Stripe events have failed/repeated attempts: $failed_attempts"
[[ "$active_rows" == 1 ]] || fail "expected one active entitlement, got $active_rows"
(( inactive_rows >= 1 )) || fail "expected at least one inactive entitlement, got $inactive_rows"
echo "[INFO] processed Stripe events: $event_count; active rows: $active_rows; inactive rows: $inactive_rows"

echo "[INFO] removing disposable Stripe customer/subscriptions and archiving the test catalog"
stripe_api test_helpers test_clocks delete "$TEST_CLOCK_ID" --confirm >/dev/null
TEST_CLOCK_DELETED=1

archive_price_json="$(stripe_api prices update "$SUBSCRIPTION_PRICE_ID" --active=false --confirm)"
jq -e '.active == false' >/dev/null <<<"$archive_price_json" \
  || fail "disposable Stripe Price was not archived"
SUBSCRIPTION_PRICE_ARCHIVED=1

archive_fixed_price_json="$(stripe_api prices update "$FIXED_PRICE_ID" --active=false --confirm)"
jq -e '.active == false' >/dev/null <<<"$archive_fixed_price_json" \
  || fail "disposable fixed-term Stripe Price was not archived"
FIXED_PRICE_ARCHIVED=1

archive_product_json="$(stripe_api products update "$PRODUCT_ID" --active=false --confirm)"
jq -e '.active == false' >/dev/null <<<"$archive_product_json" \
  || fail "disposable Stripe Product was not archived"
PRODUCT_ARCHIVED=1

echo "[PASS] real Stripe test-mode lifecycle: Checkout, paid access, Portal, failure/grace, recovery, cancellation, and repurchase"
echo "[PASS] all created customer/subscription objects were deleted with the Test Clock; the disposable catalog was archived"
