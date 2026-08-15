#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TMP_ROOT="$(mktemp -d)"
SERVER_PID=""
PG_SCHEMA_READY=0

command -v psql >/dev/null 2>&1 || {
  printf '[FAIL] psql is required for the PostgreSQL billing E2E\n' >&2
  exit 1
}

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

cleanup_billing_rows() {
  "${PSQL[@]}" >/dev/null <<'SQL'
DELETE FROM billing_entitlements WHERE user_id = 'web_billing_ci';
DELETE FROM billing_webhook_events WHERE event_id LIKE 'evt_ci_%';
DELETE FROM cloud_web_auth_sessions WHERE user_id = 'web_billing_ci';
DELETE FROM cloud_web_user_external_state WHERE user_id = 'web_billing_ci';
DELETE FROM cloud_web_invite_users WHERE user_id = 'web_billing_ci';
SQL
}

cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
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

cargo build -p hone-console-page --quiet

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
    HONE_STRIPE_SECRET_KEY=sk_test_ci_only_not_a_secret \
    HONE_STRIPE_WEBHOOK_SECRET=whsec_ci_only_not_a_secret \
    HONE_STRIPE_PRODUCT_ID=prod_ci_stripe \
    HONE_STRIPE_SUBSCRIPTION_PRICE_ID=price_ci_subscription \
    HONE_STRIPE_FIXED_TERM_PRICE_ID=price_ci_fixed \
    HONE_STRIPE_PUBLIC_BASE_URL="http://127.0.0.1:$PUBLIC_PORT/" \
    HONE_BILLING_GRACE_DAYS=7 \
    "$REPO_ROOT/target/debug/hone-console-page"
) > "$TMP_ROOT/server.log" 2>&1 &
SERVER_PID=$!

ready=0
for _ in {1..100}; do
  if curl -fsS "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/config" >/dev/null 2>&1; then
    ready=1
    break
  fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    break
  fi
  sleep 0.1
done
if [[ "$ready" != 1 ]]; then
  printf '[FAIL] isolated billing backend did not become ready\n' >&2
  tail -n 80 "$TMP_ROOT/server.log" >&2 || true
  exit 1
fi

PG_SCHEMA_READY=1
cleanup_billing_rows
"${PSQL[@]}" >/dev/null <<'SQL'
INSERT INTO cloud_web_invite_users(user_id, phone_number, is_admin, record)
VALUES (
  'web_billing_ci',
  '',
  FALSE,
  '{"user_id":"web_billing_ci","invite_code":"HONE-CI-BILLING-ISOLATED","phone_number":"","created_at":"2026-08-03T00:00:00+00:00","last_login_at":"2026-08-03T00:00:00+00:00","revoked_at":null,"password_hash":null,"password_set_at":null,"tos_accepted_at":"2026-08-03T00:00:00+00:00","tos_version":"2.4","api_key_prefix":null,"api_key_created_at":null,"api_key_last_used_at":null}'::jsonb
);
INSERT INTO cloud_web_user_external_state(
  user_id, email_address, email_verified_at, identity_kind
) VALUES (
  'web_billing_ci',
  'billing-ci@hone-claw.invalid',
  '2026-08-03T00:00:00+00:00',
  'international_email'
);
INSERT INTO cloud_web_auth_sessions(session_hash, user_id, record, expires_at)
VALUES (
  'd655851b3d441ba1635432b543392fc7a568da56ae8a7805e93860bee11bae4f',
  'web_billing_ci',
  '{"session_hash":"d655851b3d441ba1635432b543392fc7a568da56ae8a7805e93860bee11bae4f","user_id":"web_billing_ci","created_at":"2026-08-03T00:00:00+00:00","expires_at":"2099-08-03T00:00:00+00:00","last_seen_at":"2026-08-03T00:00:00+00:00"}'::jsonb,
  '2099-08-03T00:00:00+00:00'::timestamptz
);
SQL

python3 - "$PUBLIC_PORT" <<'PY'
import hashlib
import hmac
import json
import sys
import time
import urllib.error
import urllib.request

public_port = int(sys.argv[1])
base_url = f"http://127.0.0.1:{public_port}"
cookie = "hone_web_session=billing-ci-session"
user_id = "web_billing_ci"
email = "billing-ci@hone-claw.invalid"
stripe_product = "prod_ci_stripe"
stripe_subscription_price = "price_ci_subscription"
stripe_fixed_price = "price_ci_fixed"
stripe_secret = b"whsec_ci_only_not_a_secret"
# This is an isolated loopback server. Ignore workstation/system proxy
# discovery so macOS HTTP proxy settings cannot turn a CI-safe local request
# into an external five-second timeout.
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))


def request(method, path, *, headers=None, body=None):
    merged = dict(headers or {})
    if body is not None:
        merged.setdefault("content-type", "application/json")
    req = urllib.request.Request(
        base_url + path,
        data=body,
        headers=merged,
        method=method,
    )
    try:
        with opener.open(req, timeout=5) as response:
            raw = response.read()
            return response.status, json.loads(raw) if raw else None
    except urllib.error.HTTPError as error:
        raw = error.read()
        return error.code, json.loads(raw) if raw else None


def billing_status():
    code, payload = request(
        "GET",
        "/api/public/billing/status",
        headers={"cookie": cookie},
    )
    assert code == 200, (code, payload)
    return payload["billing"]


def paid_api_code():
    code, _ = request(
        "GET",
        "/api/public/bootstrap",
        headers={"cookie": cookie},
    )
    return code


def wait_status(label, predicate):
    latest = None
    for _ in range(60):
        latest = billing_status()
        if predicate(latest):
            return latest
        time.sleep(0.05)
    raise AssertionError(f"{label}: {latest}")


def stripe_post(payload):
    body = json.dumps(payload, separators=(",", ":")).encode()
    timestamp = str(int(time.time()))
    signature = hmac.new(
        stripe_secret,
        timestamp.encode() + b"." + body,
        hashlib.sha256,
    ).hexdigest()
    return request(
        "POST",
        "/api/public/integrations/stripe/webhook",
        headers={"stripe-signature": f"t={timestamp},v1={signature}"},
        body=body,
    )


def invoice(event_id, event_type, subscription_id, created):
    return {
        "id": event_id,
        "type": event_type,
        "created": created,
        "livemode": False,
        "data": {
            "object": {
                "id": f"in_{event_id}",
                "customer": "cus_ci_old" if subscription_id == "sub_ci_old" else "cus_ci_new",
                "customer_email": email,
                "parent": {
                    "subscription_details": {
                        "subscription": subscription_id,
                        "metadata": {"hone_user_id": user_id},
                    }
                },
                "lines": {
                    "data": [
                        {
                            "pricing": {
                                "price_details": {
                                    "price": stripe_subscription_price,
                                    "product": stripe_product,
                                }
                            },
                            "period": {
                                "start": created - 1000,
                                "end": created + 31536000,
                            },
                        }
                    ]
                },
            }
        },
    }


def subscription(event_id, event_type, subscription_id, status, created, cancel=False):
    return {
        "id": event_id,
        "type": event_type,
        "created": created,
        "livemode": False,
        "data": {
            "object": {
                "id": subscription_id,
                "customer": "cus_ci_old" if subscription_id == "sub_ci_old" else "cus_ci_new",
                "status": status,
                "metadata": {"hone_user_id": user_id},
                "items": {
                    "data": [
                        {
                            "price": {"id": stripe_subscription_price, "product": stripe_product},
                            "current_period_start": created - 1000,
                            "current_period_end": created + 31536000,
                        }
                    ]
                },
                "cancel_at_period_end": cancel,
            }
        },
    }


def fixed_checkout(event_id, event_type, payment_status, created):
    return {
        "id": event_id,
        "type": event_type,
        "created": created,
        "livemode": False,
        "data": {
            "object": {
                "id": "cs_test_ci_fixed",
                "created": created - 1,
                "mode": "payment",
                "payment_status": payment_status,
                "payment_intent": "pi_ci_fixed",
                "customer": "cus_ci_fixed",
                "client_reference_id": user_id,
                "metadata": {
                    "hone_user_id": user_id,
                    "hone_product_id": stripe_product,
                    "hone_price_id": stripe_fixed_price,
                    "hone_entitlement_kind": "fixed_term_purchase",
                    "hone_term_months": "12",
                },
                "customer_details": {"email": email},
            }
        },
    }


def fixed_refund(event_id, amount_refunded, refunded, created):
    return {
        "id": event_id,
        "type": "charge.refunded",
        "created": created,
        "livemode": False,
        "data": {
            "object": {
                "id": "ch_ci_fixed",
                "payment_intent": "pi_ci_fixed",
                "customer": "cus_ci_fixed",
                "amount": 22999,
                "amount_refunded": amount_refunded,
                "refunded": refunded,
                "metadata": {
                    "hone_user_id": user_id,
                    "hone_product_id": stripe_product,
                    "hone_price_id": stripe_fixed_price,
                    "hone_entitlement_kind": "fixed_term_purchase",
                    "hone_term_months": "12",
                },
                "billing_details": {"email": email},
            }
        },
    }


config_code, config = request("GET", "/api/public/billing/config")
assert config_code == 200
assert config == {
    "stripe": {
        "subscription": {
            "enabled": True,
            "amount_minor": 19999,
            "currency": "usd",
            "term_months": 12,
            "auto_renews": True,
            "advertised_payment_methods": {
                "card": True,
                "alipay": False,
                "wechat_pay": False,
            },
        },
        "fixed_term": {
            "enabled": True,
            "amount_minor": 22999,
            "currency": "usd",
            "term_months": 12,
            "auto_renews": False,
            "advertised_payment_methods": {
                "card": True,
                "alipay": False,
                "wechat_pay": False,
            },
        },
    },
    "purchases_allowed_on_this_client": True,
    "management_allowed_on_this_client": True,
}
ios_code, ios_config = request(
    "GET",
    "/api/public/billing/config",
    headers={"user-agent": "HONE-iOS/1.0 WKWebView"},
)
assert ios_code == 200
assert ios_config["purchases_allowed_on_this_client"] is False
assert ios_config["management_allowed_on_this_client"] is False
assert billing_status()["access_granted"] is False
assert paid_api_code() == 402

unsigned_code, _ = request(
    "POST",
    "/api/public/integrations/stripe/webhook",
    body=b"{}",
)
assert unsigned_code == 401
unauthenticated_status_code, _ = request("GET", "/api/public/billing/status")
assert unauthenticated_status_code == 401
missing_origin_code, _ = request(
    "POST",
    "/api/public/billing/checkout/stripe",
    headers={"cookie": cookie},
    body=json.dumps({"offer": "subscription"}).encode(),
)
assert missing_origin_code == 403
ios_checkout_code, _ = request(
    "POST",
    "/api/public/billing/checkout/stripe",
    headers={
        "cookie": cookie,
        "origin": base_url,
        "sec-fetch-site": "same-origin",
        "user-agent": "HONE-iOS/1.0 WKWebView",
    },
    body=json.dumps({"offer": "subscription"}).encode(),
)
assert ios_checkout_code == 403
cross_site_checkout_code, _ = request(
    "POST",
    "/api/public/billing/checkout/stripe",
    headers={
        "cookie": cookie,
        "origin": "https://cross-site.example",
        "sec-fetch-site": "cross-site",
    },
    body=json.dumps({"offer": "fixed_term"}).encode(),
)
assert cross_site_checkout_code == 403

base = int(time.time()) - 100
missing_mode = invoice("evt_ci_missing_mode", "invoice.paid", "sub_ci_missing_mode", base)
missing_mode.pop("livemode")
assert stripe_post(missing_mode)[0] == 422
wrong_mode = invoice("evt_ci_wrong_mode", "invoice.paid", "sub_ci_wrong_mode", base)
wrong_mode["livemode"] = True
assert stripe_post(wrong_mode)[0] == 422
assert billing_status()["access_granted"] is False
wrong = invoice("evt_ci_wrong_catalog", "invoice.paid", "sub_ci_wrong", base + 1)
wrong["data"]["object"]["lines"]["data"][0]["pricing"]["price_details"] = {
    "price": "price_wrong",
    "product": "prod_wrong",
}
wrong_code, wrong_body = stripe_post(wrong)
assert wrong_code == 200
assert wrong_body == {"ignored": True, "ok": True, "reason": "catalog_mismatch"}
assert billing_status()["access_granted"] is False

checkout = {
    "id": "evt_ci_checkout",
    "type": "checkout.session.completed",
    # Real Stripe delivery can assign checkout.session.completed a later
    # envelope timestamp than the invoice/subscription events it triggers.
    "created": base + 20,
    "livemode": False,
    "data": {
        "object": {
            "id": "cs_test_ci",
            "created": base + 2,
            "mode": "subscription",
            "payment_status": "paid",
            "subscription": "sub_ci_old",
            "customer": "cus_ci_old",
            "client_reference_id": user_id,
            "metadata": {
                "hone_user_id": user_id,
                "hone_product_id": stripe_product,
                "hone_price_id": stripe_subscription_price,
                "hone_entitlement_kind": "recurring_subscription",
            },
            "customer_details": {"email": email},
        }
    },
}
assert stripe_post(checkout)[0] == 202
pending = wait_status(
    "checkout must remain pending",
    lambda value: len(value["entitlements"]) == 1
    and value["entitlements"][0]["access_state"] == "pending",
)
assert pending["access_granted"] is False
assert paid_api_code() == 402

paid = invoice("evt_ci_paid", "invoice.paid", "sub_ci_old", base + 10)
assert stripe_post(paid)[0] == 202
wait_status(
    "invoice.paid must activate",
    lambda value: value["access_granted"]
    and any(item["access_state"] == "active" for item in value["entitlements"]),
)
assert paid_api_code() == 200
assert stripe_post(paid)[0] == 202
active_checkout_code, _ = request(
    "POST",
    "/api/public/billing/checkout/stripe",
    headers={
        "cookie": cookie,
        "origin": base_url,
        "sec-fetch-site": "same-origin",
    },
    body=json.dumps({"offer": "subscription"}).encode(),
)
assert active_checkout_code == 409

assert stripe_post(
    subscription(
        "evt_ci_out_of_order_cancel",
        "customer.subscription.deleted",
        "sub_ci_old",
        "canceled",
        base + 5,
    )
)[0] == 202
wait_status(
    "older delivered-late event must not revoke",
    lambda value: value["access_granted"]
    and any(item["access_state"] == "active" for item in value["entitlements"]),
)

assert stripe_post(invoice("evt_ci_failed", "invoice.payment_failed", "sub_ci_old", base + 30))[0] == 202
grace = wait_status(
    "failed renewal must enter bounded grace",
    lambda value: any(item["access_state"] == "grace" for item in value["entitlements"]),
)
grace_row = next(item for item in grace["entitlements"] if item["access_state"] == "grace")
assert grace_row["grace_expires_at"]
assert grace["access_granted"] is True

assert stripe_post(invoice("evt_ci_restore", "invoice.paid", "sub_ci_old", base + 40))[0] == 202
wait_status(
    "restored payment must reactivate",
    lambda value: value["access_granted"]
    and any(item["raw_status"] == "active" and item["access_state"] == "active" for item in value["entitlements"]),
)

assert stripe_post(
    subscription(
        "evt_ci_cancel_at_end",
        "customer.subscription.updated",
        "sub_ci_old",
        "active",
        base + 50,
        True,
    )
)[0] == 202
wait_status(
    "period-end cancellation must retain access",
    lambda value: value["access_granted"]
    and any(item["cancel_at_period_end"] and item["access_state"] == "active" for item in value["entitlements"]),
)

assert stripe_post(
    subscription(
        "evt_ci_deleted",
        "customer.subscription.deleted",
        "sub_ci_old",
        "canceled",
        base + 60,
    )
)[0] == 202
wait_status(
    "subscription deletion must revoke its row",
    lambda value: not value["access_granted"]
    and all(item["access_state"] == "inactive" for item in value["entitlements"]),
)
assert paid_api_code() == 402

assert stripe_post(invoice("evt_ci_repurchase", "invoice.paid", "sub_ci_new", base + 70))[0] == 202
assert stripe_post(
    subscription(
        "evt_ci_old_subscription_late",
        "customer.subscription.deleted",
        "sub_ci_old",
        "canceled",
        base + 80,
    )
)[0] == 202
wait_status(
    "old subscription event must not revoke repurchase",
    lambda value: value["access_granted"]
    and sum(item["access_state"] == "active" for item in value["entitlements"]) == 1,
)

assert stripe_post(invoice("evt_ci_second_active", "invoice.paid", "sub_ci_second", base + 90))[0] == 202
two_active = wait_status(
    "two Stripe subscriptions must produce duplicate warning",
    lambda value: value["has_duplicate_active_subscriptions"],
)
assert two_active["access_granted"] is True

assert stripe_post(
    subscription(
        "evt_ci_delete_repurchase",
        "customer.subscription.deleted",
        "sub_ci_new",
        "canceled",
        base + 91,
    )
)[0] == 202
one_active = wait_status(
    "one active Stripe subscription must retain access after the other is canceled",
    lambda value: value["access_granted"]
    and not value["has_duplicate_active_subscriptions"]
    and sum(item["provider"] == "stripe" and item["access_state"] == "active" for item in value["entitlements"]) == 1,
)
assert one_active["access_granted"] is True

assert stripe_post(
    subscription(
        "evt_ci_delete_second",
        "customer.subscription.deleted",
        "sub_ci_second",
        "canceled",
        base + 92,
    )
)[0] == 202
wait_status(
    "all inactive must deny access",
    lambda value: not value["access_granted"]
    and all(item["access_state"] == "inactive" for item in value["entitlements"]),
)
assert paid_api_code() == 402

assert stripe_post(
    fixed_checkout(
        "evt_ci_fixed_pending",
        "checkout.session.completed",
        "unpaid",
        base + 100,
    )
)[0] == 202
wait_status(
    "unpaid fixed-term checkout must remain pending",
    lambda value: not value["access_granted"]
    and any(
        item["entitlement_kind"] == "fixed_term_purchase"
        and item["access_state"] == "pending"
        and not item["grants_access"]
        for item in value["entitlements"]
    ),
)

fixed_paid = fixed_checkout(
    "evt_ci_fixed_paid",
    "checkout.session.async_payment_succeeded",
    "paid",
    base + 110,
)
assert stripe_post(fixed_paid)[0] == 202
fixed_active = wait_status(
    "paid fixed-term checkout must activate for a fixed period",
    lambda value: value["access_granted"]
    and any(
        item["entitlement_kind"] == "fixed_term_purchase"
        and item["access_state"] == "active"
        and item["grants_access"]
        and item["current_period_end"]
        for item in value["entitlements"]
    ),
)
fixed_period_end = next(
    item["current_period_end"]
    for item in fixed_active["entitlements"]
    if item["entitlement_kind"] == "fixed_term_purchase"
)
assert stripe_post(fixed_paid)[0] == 202
assert wait_status(
    "replayed paid event must not extend the fixed period",
    lambda value: any(
        item["entitlement_kind"] == "fixed_term_purchase"
        and item["current_period_end"] == fixed_period_end
        for item in value["entitlements"]
    ),
)["access_granted"]

assert stripe_post(
    invoice("evt_ci_coexist_subscription", "invoice.paid", "sub_ci_coexist", base + 120)
)[0] == 202
coexisting = wait_status(
    "one recurring and one fixed-term entitlement may coexist without duplicate warning",
    lambda value: value["access_granted"]
    and not value["has_duplicate_active_subscriptions"]
    and sum(item["grants_access"] for item in value["entitlements"]) == 2,
)
assert coexisting["has_duplicate_active_subscriptions"] is False

partial_code, partial_body = stripe_post(
    fixed_refund("evt_ci_fixed_partial_refund", 1000, False, base + 130)
)
assert partial_code == 200
assert partial_body == {"ignored": True, "ok": True, "reason": "partial_refund"}
assert billing_status()["access_granted"] is True

assert stripe_post(
    fixed_refund("evt_ci_fixed_full_refund", 22999, True, base + 140)
)[0] == 202
after_refund = wait_status(
    "full refund must revoke only the matching fixed-term entitlement",
    lambda value: value["access_granted"]
    and any(
        item["entitlement_kind"] == "fixed_term_purchase"
        and item["access_state"] == "inactive"
        and not item["grants_access"]
        for item in value["entitlements"]
    )
    and any(
        item["entitlement_kind"] == "recurring_subscription"
        and item["grants_access"]
        for item in value["entitlements"]
    ),
)
assert after_refund["has_duplicate_active_subscriptions"] is False

assert stripe_post(
    subscription(
        "evt_ci_delete_coexist",
        "customer.subscription.deleted",
        "sub_ci_coexist",
        "canceled",
        base + 150,
    )
)[0] == 202
wait_status(
    "revoking the remaining recurring subscription must deny access",
    lambda value: not value["access_granted"]
    and all(not item["grants_access"] for item in value["entitlements"]),
)
assert paid_api_code() == 402

PY

read -r paid_replay_rows wrong_catalog_rows unfinished_rows duplicate_attempts < <(
  "${PSQL[@]}" --tuples-only --no-align --field-separator=' ' <<'SQL'
SELECT
  COUNT(*) FILTER (WHERE provider = 'stripe' AND event_id = 'evt_ci_paid'),
  COUNT(*) FILTER (
    WHERE event_id IN ('evt_ci_missing_mode', 'evt_ci_wrong_mode', 'evt_ci_wrong_catalog')
  ),
  COUNT(*) FILTER (WHERE processing_state != 'processed'),
  COALESCE(MAX(attempt_count) FILTER (
    WHERE provider = 'stripe' AND event_id = 'evt_ci_paid'
  ), -1)
FROM billing_webhook_events
WHERE event_id LIKE 'evt_ci_%';
SQL
)
[[ "$paid_replay_rows" == 1 ]] || {
  printf '[FAIL] expected one persisted evt_ci_paid row, got %s\n' "$paid_replay_rows" >&2
  exit 1
}
[[ "$wrong_catalog_rows" == 0 ]] || {
  printf '[FAIL] invalid/catalog-mismatch events were persisted: %s\n' "$wrong_catalog_rows" >&2
  exit 1
}
[[ "$unfinished_rows" == 0 ]] || {
  printf '[FAIL] unfinished billing events remain: %s\n' "$unfinished_rows" >&2
  exit 1
}
[[ "$duplicate_attempts" == 1 ]] || {
  printf '[FAIL] replay changed attempt_count: %s\n' "$duplicate_attempts" >&2
  exit 1
}

printf '[PASS] isolated signed Stripe-only HTTP billing lifecycle\n'
