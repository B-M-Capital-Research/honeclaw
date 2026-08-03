#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TMP_ROOT="$(mktemp -d)"
SERVER_PID=""

cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
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
    HONE_CLOUD_MODE=local \
    HONE_WEB_DIST_DIR="$TMP_ROOT/web-admin" \
    HONE_PUBLIC_WEB_DIST_DIR="$TMP_ROOT/web-public" \
    HONE_PUBLIC_ALLOWED_ORIGINS="http://127.0.0.1:$PUBLIC_PORT" \
    HONE_PUBLIC_SECURE_COOKIE=false \
    HONE_BILLING_PRIMARY_PROVIDER=stripe \
    HONE_WHOP_NEW_PURCHASES_ENABLED=true \
    HONE_WHOP_WEBHOOK_SECRET=ws_ci_only_not_a_secret \
    HONE_WHOP_COMPANY_ID=biz_ci_billing \
    HONE_WHOP_PRODUCT_ID=prod_ci_whop \
    HONE_WHOP_PLAN_ID=plan_ci_whop \
    HONE_STRIPE_CHECKOUT_ENABLED=true \
    HONE_STRIPE_MODE=test \
    HONE_STRIPE_SECRET_KEY=sk_test_ci_only_not_a_secret \
    HONE_STRIPE_WEBHOOK_SECRET=whsec_ci_only_not_a_secret \
    HONE_STRIPE_PRODUCT_ID=prod_ci_stripe \
    HONE_STRIPE_PRICE_ID=price_ci_stripe \
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

python3 - "$TMP_ROOT/data/sessions.sqlite3" <<'PY'
import sqlite3
import sys

database = sqlite3.connect(sys.argv[1])
with database:
    database.execute(
        """
        INSERT INTO web_invite_users(
            user_id, invite_code, phone_number, created_at, last_login_at,
            tos_accepted_at, tos_version
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "web_billing_ci",
            "HONE-CI-BILLING-ISOLATED",
            "",
            "2026-08-03T00:00:00+00:00",
            "2026-08-03T00:00:00+00:00",
            "2026-08-03T00:00:00+00:00",
            "2.3",
        ),
    )
    database.execute(
        """
        INSERT INTO web_user_external_state(
            user_id, email_address, email_verified_at, identity_kind
        ) VALUES (?, ?, ?, ?)
        """,
        (
            "web_billing_ci",
            "billing-ci@hone-claw.invalid",
            "2026-08-03T00:00:00+00:00",
            "international_email",
        ),
    )
    database.execute(
        """
        INSERT INTO web_auth_sessions(
            session_token, user_id, created_at, expires_at, last_seen_at
        ) VALUES (?, ?, ?, ?, ?)
        """,
        (
            "billing-ci-session",
            "web_billing_ci",
            "2026-08-03T00:00:00+00:00",
            "2099-08-03T00:00:00+00:00",
            "2026-08-03T00:00:00+00:00",
        ),
    )
PY

python3 - "$PUBLIC_PORT" "$TMP_ROOT/data/sessions.sqlite3" <<'PY'
import base64
import hashlib
import hmac
import json
import sqlite3
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone

public_port = int(sys.argv[1])
database_path = sys.argv[2]
base_url = f"http://127.0.0.1:{public_port}"
cookie = "hone_web_session=billing-ci-session"
user_id = "web_billing_ci"
email = "billing-ci@hone-claw.invalid"
stripe_product = "prod_ci_stripe"
stripe_price = "price_ci_stripe"
stripe_secret = b"whsec_ci_only_not_a_secret"
whop_secret = b"ws_ci_only_not_a_secret"


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
        with urllib.request.urlopen(req, timeout=5) as response:
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


def whop_post(payload):
    body = json.dumps(payload, separators=(",", ":")).encode()
    timestamp = str(int(time.time()))
    signature = base64.b64encode(
        hmac.new(
            whop_secret,
            payload["id"].encode() + b"." + timestamp.encode() + b"." + body,
            hashlib.sha256,
        ).digest()
    ).decode()
    return request(
        "POST",
        "/api/public/integrations/whop/webhook",
        headers={
            "webhook-id": payload["id"],
            "webhook-timestamp": timestamp,
            "webhook-signature": f"v1,{signature}",
        },
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
                                    "price": stripe_price,
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
                            "price": {"id": stripe_price, "product": stripe_product},
                            "current_period_start": created - 1000,
                            "current_period_end": created + 31536000,
                        }
                    ]
                },
                "cancel_at_period_end": cancel,
            }
        },
    }


def whop_event(event_id, event_type, membership_id, status, created, cancel=False, product="prod_ci_whop"):
    event_time = datetime.fromtimestamp(created, tz=timezone.utc).isoformat().replace("+00:00", "Z")
    period_end = datetime.fromtimestamp(created + 31536000, tz=timezone.utc).isoformat().replace("+00:00", "Z")
    return {
        "id": event_id,
        "api_version": "v1",
        "timestamp": event_time,
        "type": event_type,
        "company_id": "biz_ci_billing",
        "data": {
            "id": membership_id,
            "status": status,
            "user": {"id": "user_ci_whop", "email": email},
            "product": {"id": product},
            "plan": {"id": "plan_ci_whop"},
            "manage_url": f"https://whop.com/billing/manage/{membership_id}",
            "renewal_period_end": period_end,
            "cancel_at_period_end": cancel,
        },
    }


config_code, config = request("GET", "/api/public/billing/config")
assert config_code == 200
assert config == {
    "primary_provider": "stripe",
    "stripe_checkout_enabled": True,
    "whop_new_purchases_enabled": True,
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
            "subscription": "sub_ci_old",
            "customer": "cus_ci_old",
            "client_reference_id": user_id,
            "metadata": {
                "hone_user_id": user_id,
                "hone_product_id": stripe_product,
                "hone_price_id": stripe_price,
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

assert whop_post(
    whop_event(
        "msg_ci_whop_active",
        "membership.activated",
        "mem_ci_old",
        "active",
        base + 90,
    )
)[0] == 202
both_active = wait_status(
    "two providers must produce duplicate warning",
    lambda value: value["has_duplicate_active_subscriptions"],
)
assert both_active["access_granted"] is True

assert whop_post(
    whop_event(
        "msg_ci_whop_cancel_at_end",
        "membership.cancel_at_period_end_changed",
        "mem_ci_old",
        "canceling",
        base + 91,
        True,
    )
)[0] == 202
wait_status(
    "Whop period-end cancellation must retain access",
    lambda value: any(
        item["provider"] == "whop"
        and item["access_state"] == "active"
        and item["cancel_at_period_end"]
        for item in value["entitlements"]
    ),
)

assert stripe_post(
    subscription(
        "evt_ci_delete_repurchase",
        "customer.subscription.deleted",
        "sub_ci_new",
        "canceled",
        base + 92,
    )
)[0] == 202
stripe_inactive = wait_status(
    "Whop must retain access after Stripe cancellation",
    lambda value: value["access_granted"]
    and not value["has_duplicate_active_subscriptions"]
    and any(item["provider"] == "whop" and item["access_state"] == "active" for item in value["entitlements"]),
)
assert stripe_inactive["access_granted"] is True

assert whop_post(
    whop_event(
        "msg_ci_whop_deactivated",
        "membership.deactivated",
        "mem_ci_old",
        "canceled",
        base + 93,
    )
)[0] == 202
wait_status(
    "all inactive must deny access",
    lambda value: not value["access_granted"]
    and all(item["access_state"] == "inactive" for item in value["entitlements"]),
)
assert paid_api_code() == 402

wrong_whop_code, _ = whop_post(
    whop_event(
        "msg_ci_whop_wrong_catalog",
        "membership.activated",
        "mem_ci_wrong",
        "active",
        base + 94,
        product="prod_wrong",
    )
)
assert wrong_whop_code == 422
assert billing_status()["access_granted"] is False

assert whop_post(
    whop_event(
        "msg_ci_whop_repurchase",
        "membership.activated",
        "mem_ci_new",
        "active",
        base + 95,
    )
)[0] == 202
assert whop_post(
    whop_event(
        "msg_ci_whop_old_late",
        "membership.deactivated",
        "mem_ci_old",
        "canceled",
        base + 96,
    )
)[0] == 202
wait_status(
    "old Whop membership event must not revoke repurchase",
    lambda value: value["access_granted"]
    and sum(item["provider"] == "whop" and item["access_state"] == "active" for item in value["entitlements"]) == 1,
)
assert paid_api_code() == 200

database = sqlite3.connect(database_path)
paid_replay_rows = database.execute(
    "SELECT COUNT(*) FROM billing_webhook_events WHERE provider = 'stripe' AND event_id = 'evt_ci_paid'"
).fetchone()[0]
wrong_catalog_rows = database.execute(
    """
    SELECT COUNT(*)
    FROM billing_webhook_events
    WHERE event_id IN (
        'evt_ci_missing_mode',
        'evt_ci_wrong_mode',
        'evt_ci_wrong_catalog',
        'msg_ci_whop_wrong_catalog'
    )
    """
).fetchone()[0]
unfinished_rows = database.execute(
    "SELECT COUNT(*) FROM billing_webhook_events WHERE processing_state != 'processed'"
).fetchone()[0]
duplicate_attempts = database.execute(
    "SELECT attempt_count FROM billing_webhook_events WHERE provider = 'stripe' AND event_id = 'evt_ci_paid'"
).fetchone()[0]
assert paid_replay_rows == 1
assert wrong_catalog_rows == 0
assert unfinished_rows == 0
assert duplicate_attempts == 1
PY

printf '[PASS] isolated signed Stripe + Whop HTTP billing lifecycle\n'
