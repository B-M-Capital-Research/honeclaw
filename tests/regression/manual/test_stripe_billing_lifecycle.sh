#!/usr/bin/env bash

set -euo pipefail

if [[ "${HONE_RUN_STRIPE_LIFECYCLE:-0}" != "1" ]]; then
  echo "[SKIP] set HONE_RUN_STRIPE_LIFECYCLE=1 after reading docs/runbooks/stripe-billing.md"
  exit 0
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

for command_name in cargo curl jq python3 stripe; do
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

SERVER_PID=""
LISTENER_PID=""
CHECKOUT_SESSION_ID=""
CHECKOUT_SESSION_EXPIRED=0
TEST_CLOCK_ID=""
TEST_CLOCK_DELETED=0
PRODUCT_ID=""
PRODUCT_ARCHIVED=0
PRICE_ID=""
PRICE_ARCHIVED=0

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

  if [[ -n "$CHECKOUT_SESSION_ID" && "$CHECKOUT_SESSION_EXPIRED" != 1 ]]; then
    stripe_api checkout sessions expire "$CHECKOUT_SESSION_ID" --confirm >/dev/null 2>&1
  fi
  if [[ -n "$TEST_CLOCK_ID" && "$TEST_CLOCK_DELETED" != 1 ]]; then
    stripe_api test_helpers test_clocks delete "$TEST_CLOCK_ID" --confirm >/dev/null 2>&1
  fi
  if [[ -n "$PRICE_ID" && "$PRICE_ARCHIVED" != 1 ]]; then
    stripe_api prices update "$PRICE_ID" --active=false --confirm >/dev/null 2>&1
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
PRICE_ID="$(jq -er '.id' <<<"$price_json")"
jq -e \
  --arg product_id "$PRODUCT_ID" \
  '.livemode == false and .active == true and .product == $product_id and .recurring.interval == "year"' \
  >/dev/null <<<"$price_json" \
  || fail "disposable annual price was not created correctly"

WEBHOOK_SECRET="$(stripe listen --print-secret --skip-update --color off)"
[[ "$WEBHOOK_SECRET" == whsec_* ]] || fail "Stripe CLI did not return a test listener secret"

stripe listen \
  --skip-update \
  --color off \
  --events checkout.session.completed,checkout.session.async_payment_succeeded,checkout.session.async_payment_failed,invoice.paid,invoice.payment_failed,customer.subscription.created,customer.subscription.updated,customer.subscription.deleted \
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
    HONE_CLOUD_MODE=local \
    HONE_WEB_DIST_DIR="$TMP_ROOT/web-admin" \
    HONE_PUBLIC_WEB_DIST_DIR="$TMP_ROOT/web-public" \
    HONE_PUBLIC_ALLOWED_ORIGINS="http://127.0.0.1:$PUBLIC_PORT" \
    HONE_PUBLIC_SECURE_COOKIE=false \
    HONE_BILLING_PRIMARY_PROVIDER=stripe \
    HONE_WHOP_NEW_PURCHASES_ENABLED=true \
    HONE_STRIPE_CHECKOUT_ENABLED=true \
    HONE_STRIPE_MODE=test \
    HONE_STRIPE_SECRET_KEY="$HONE_STRIPE_SECRET_KEY" \
    HONE_STRIPE_WEBHOOK_SECRET="$WEBHOOK_SECRET" \
    HONE_STRIPE_PRODUCT_ID="$PRODUCT_ID" \
    HONE_STRIPE_PRICE_ID="$PRICE_ID" \
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
python3 - \
  "$TMP_ROOT/data/sessions.sqlite3" \
  "$USER_ID" \
  "$SESSION_TOKEN" \
  "$EMAIL_ADDRESS" \
  "$NOW_RFC3339" <<'PY'
import sqlite3
import sys

database_path, user_id, session_token, email_address, now = sys.argv[1:]
database = sqlite3.connect(database_path)
with database:
    database.execute(
        """
        INSERT INTO web_invite_users(
            user_id, invite_code, phone_number, created_at, last_login_at,
            tos_accepted_at, tos_version
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (user_id, f"HONE-STRIPE-LIFECYCLE-{user_id}", "", now, now, now, "2.3"),
    )
    database.execute(
        """
        INSERT INTO web_user_external_state(
            user_id, email_address, email_verified_at, identity_kind
        ) VALUES (?, ?, ?, ?)
        """,
        (user_id, email_address, now, "international_email"),
    )
    database.execute(
        """
        INSERT INTO web_auth_sessions(
            session_token, user_id, created_at, expires_at, last_seen_at
        ) VALUES (?, ?, ?, ?, ?)
        """,
        (session_token, user_id, now, "2099-08-03T00:00:00Z", now),
    )
PY

wait_for_billing \
  "fresh international account did not start fail-closed" \
  '.billing.access_granted == false and (.billing.entitlements | length) == 0'
[[ "$(paid_api_status)" == 402 ]] || fail "paid API did not start at 402"

echo "[INFO] verifying real Checkout API creation without completing payment"
checkout_json="$(curl -fsS \
  -X POST \
  -H "Cookie: hone_web_session=$SESSION_TOKEN" \
  -H "Origin: http://127.0.0.1:$PUBLIC_PORT" \
  -H 'Sec-Fetch-Site: same-origin' \
  "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/checkout/stripe")"
checkout_url="$(jq -er '.checkout_url' <<<"$checkout_json")"
[[ "$checkout_url" == https://checkout.stripe.com/* ]] \
  || fail "HONE returned an untrusted Checkout URL"
CHECKOUT_SESSION_ID="$(grep -Eo 'cs_test_[A-Za-z0-9_]+' <<<"$checkout_url" | head -n 1)"
[[ "$CHECKOUT_SESSION_ID" == cs_test_* ]] || fail "could not identify the test Checkout Session"
checkout_object="$(stripe_api checkout sessions retrieve "$CHECKOUT_SESSION_ID")"
jq -e \
  --arg user_id "$USER_ID" \
  --arg product_id "$PRODUCT_ID" \
  --arg price_id "$PRICE_ID" \
  '.livemode == false
    and .mode == "subscription"
    and .client_reference_id == $user_id
    and .metadata.hone_user_id == $user_id
    and .metadata.hone_product_id == $product_id
    and .metadata.hone_price_id == $price_id' \
  >/dev/null <<<"$checkout_object" \
  || fail "Checkout Session metadata did not preserve the HONE binding"
stripe_api checkout sessions expire "$CHECKOUT_SESSION_ID" --confirm >/dev/null
CHECKOUT_SESSION_EXPIRED=1

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
  -d "items[0][price]=$PRICE_ID" \
  -d "metadata[hone_user_id]=$USER_ID" \
  -d "metadata[hone_product_id]=$PRODUCT_ID" \
  -d "metadata[hone_price_id]=$PRICE_ID" \
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
  -d "items[0][price]=$PRICE_ID" \
  -d "metadata[hone_user_id]=$USER_ID" \
  -d "metadata[hone_product_id]=$PRODUCT_ID" \
  -d "metadata[hone_price_id]=$PRICE_ID" \
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

python3 - "$TMP_ROOT/data/sessions.sqlite3" <<'PY'
import sqlite3
import sys

database = sqlite3.connect(sys.argv[1])
event_count = database.execute(
    "SELECT COUNT(*) FROM billing_webhook_events WHERE provider = 'stripe'"
).fetchone()[0]
unfinished = database.execute(
    """
    SELECT COUNT(*)
    FROM billing_webhook_events
    WHERE provider = 'stripe' AND processing_state != 'processed'
    """
).fetchone()[0]
failed_attempts = database.execute(
    """
    SELECT COUNT(*)
    FROM billing_webhook_events
    WHERE provider = 'stripe' AND (attempt_count != 1 OR last_error IS NOT NULL)
    """
).fetchone()[0]
active_rows = database.execute(
    """
    SELECT COUNT(*)
    FROM billing_entitlements
    WHERE provider = 'stripe' AND access_state = 'active'
    """
).fetchone()[0]
inactive_rows = database.execute(
    """
    SELECT COUNT(*)
    FROM billing_entitlements
    WHERE provider = 'stripe' AND access_state = 'inactive'
    """
).fetchone()[0]

assert event_count >= 8, event_count
assert unfinished == 0, unfinished
assert failed_attempts == 0, failed_attempts
assert active_rows == 1, active_rows
assert inactive_rows >= 1, inactive_rows
print(f"[INFO] processed Stripe events: {event_count}; active rows: {active_rows}; inactive rows: {inactive_rows}")
PY

echo "[INFO] removing disposable Stripe customer/subscriptions and archiving the test catalog"
stripe_api test_helpers test_clocks delete "$TEST_CLOCK_ID" --confirm >/dev/null
TEST_CLOCK_DELETED=1

archive_price_json="$(stripe_api prices update "$PRICE_ID" --active=false --confirm)"
jq -e '.active == false' >/dev/null <<<"$archive_price_json" \
  || fail "disposable Stripe Price was not archived"
PRICE_ARCHIVED=1

archive_product_json="$(stripe_api products update "$PRODUCT_ID" --active=false --confirm)"
jq -e '.active == false' >/dev/null <<<"$archive_product_json" \
  || fail "disposable Stripe Product was not archived"
PRODUCT_ARCHIVED=1

echo "[PASS] real Stripe test-mode lifecycle: Checkout, paid access, Portal, failure/grace, recovery, cancellation, and repurchase"
echo "[PASS] all created customer/subscription objects were deleted with the Test Clock; the disposable catalog was archived"
