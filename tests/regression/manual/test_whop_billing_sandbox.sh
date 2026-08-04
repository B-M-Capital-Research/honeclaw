#!/usr/bin/env bash

set -euo pipefail

if [[ "${HONE_RUN_WHOP_SANDBOX:-0}" != "1" ]]; then
  echo "[SKIP] set HONE_RUN_WHOP_SANDBOX=1 after reading docs/runbooks/whop-hone-activation.md"
  exit 0
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

for command_name in cargo cloudflared curl jq python3; do
  command -v "$command_name" >/dev/null 2>&1 || {
    echo "[FAIL] missing command: $command_name" >&2
    exit 1
  }
done

[[ -n "${HONE_WHOP_SANDBOX_API_KEY:-}" ]] || {
  echo "[FAIL] HONE_WHOP_SANDBOX_API_KEY must be exported" >&2
  exit 1
}
[[ "${HONE_WHOP_SANDBOX_COMPANY_ID:-}" == biz_* ]] || {
  echo "[FAIL] HONE_WHOP_SANDBOX_COMPANY_ID must be an exported sandbox biz_ ID" >&2
  exit 1
}

RUN_ID="$(date -u +%Y%m%dt%H%M%Sz)-$$"
SESSION_TOKEN="whop-sandbox-session-$RUN_ID"
TMP_ROOT="$(mktemp -d)"
chmod 700 "$TMP_ROOT"

WHOP_API_BASE="https://sandbox-api.whop.com/api/v1"
WHOP_CURL_CONFIG="$TMP_ROOT/whop-curl.conf"
SERVER_PID=""
TUNNEL_PID=""
PRODUCT_ID=""
PLAN_ID=""
WEBHOOK_ID=""
PRODUCT_ARCHIVED=0
PLAN_ARCHIVED=0
WEBHOOK_DELETED=0

python3 - "$WHOP_CURL_CONFIG" <<'PY'
import os
import pathlib
import sys

api_key = os.environ["HONE_WHOP_SANDBOX_API_KEY"].strip()
if not api_key or any(char in api_key for char in "\r\n\""):
    raise SystemExit("invalid HONE_WHOP_SANDBOX_API_KEY")
path = pathlib.Path(sys.argv[1])
path.write_text(
    f'header = "Authorization: Bearer {api_key}"\n'
    'header = "Content-Type: application/json"\n',
    encoding="utf-8",
)
path.chmod(0o600)
PY

whop_api() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  if [[ -n "$body" ]]; then
    curl -fsS \
      --config "$WHOP_CURL_CONFIG" \
      --request "$method" \
      --data "$body" \
      "$WHOP_API_BASE$path"
  else
    curl -fsS \
      --config "$WHOP_CURL_CONFIG" \
      --request "$method" \
      "$WHOP_API_BASE$path"
  fi
}

redacted_tail() {
  local file="$1"
  [[ -f "$file" ]] || return 0
  tail -n 80 "$file" \
    | sed -E \
      -e 's/ws_[A-Za-z0-9_-]+/<redacted-webhook-secret>/g' \
      -e 's/whsec_[A-Za-z0-9_-]+/<redacted-webhook-secret>/g' \
      -e 's/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/<redacted-email>/g'
}

database_membership_ids() {
  local database="$TMP_ROOT/data/sessions.sqlite3"
  [[ -f "$database" ]] || return 0
  python3 - "$database" <<'PY'
import sqlite3
import sys

database = sqlite3.connect(sys.argv[1])
for row in database.execute(
    "SELECT DISTINCT provider_subscription_id FROM billing_entitlements WHERE provider = 'whop'"
):
    if row[0]:
        print(row[0])
PY
}

cleanup() {
  trap - ERR
  set +e

  while IFS= read -r membership_id; do
    [[ "$membership_id" == mem_* ]] || continue
    whop_api POST "/memberships/$membership_id/cancel" \
      '{"cancellation_mode":"immediate"}' >/dev/null 2>&1
  done < <(database_membership_ids)

  if [[ -n "$WEBHOOK_ID" && "$WEBHOOK_DELETED" != 1 ]]; then
    whop_api DELETE "/webhooks/$WEBHOOK_ID" >/dev/null 2>&1
  fi
  if [[ -n "$PLAN_ID" && "$PLAN_ARCHIVED" != 1 ]]; then
    whop_api PATCH "/plans/$PLAN_ID" '{"visibility":"archived"}' >/dev/null 2>&1
  fi
  if [[ -n "$PRODUCT_ID" && "$PRODUCT_ARCHIVED" != 1 ]]; then
    whop_api PATCH "/products/$PRODUCT_ID" '{"visibility":"archived"}' >/dev/null 2>&1
  fi

  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  if [[ -n "$TUNNEL_PID" ]] && kill -0 "$TUNNEL_PID" 2>/dev/null; then
    kill "$TUNNEL_PID" 2>/dev/null || true
    wait "$TUNNEL_PID" 2>/dev/null || true
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
  echo "[FAIL] command failed at Whop sandbox script line $line_number" >&2
  redacted_tail "$TMP_ROOT/server.log" >&2
  redacted_tail "$TMP_ROOT/cloudflared.log" >&2
  exit "$status"
}
trap 'on_error "$LINENO"' ERR

fail() {
  local message="$1"
  echo "[FAIL] $message" >&2
  redacted_tail "$TMP_ROOT/server.log" >&2
  redacted_tail "$TMP_ROOT/cloudflared.log" >&2
  exit 1
}

sqlite_scalar() {
  local sql="$1"
  python3 - "$TMP_ROOT/data/sessions.sqlite3" "$sql" <<'PY'
import sqlite3
import sys

database = sqlite3.connect(sys.argv[1])
row = database.execute(sys.argv[2]).fetchone()
print("" if row is None or row[0] is None else row[0])
PY
}

wait_for_sql_value() {
  local label="$1"
  local sql="$2"
  local expected="$3"
  local value=""

  for _ in {1..900}; do
    if [[ -f "$TMP_ROOT/data/sessions.sqlite3" ]]; then
      value="$(sqlite_scalar "$sql")"
      if [[ "$value" == "$expected" ]]; then
        return 0
      fi
    fi
    sleep 1
  done
  fail "$label (last value: ${value:-<empty>})"
}

paid_api_status() {
  curl -sS -o /dev/null -w '%{http_code}' \
    -H "Cookie: hone_web_session=$SESSION_TOKEN" \
    "http://127.0.0.1:$PUBLIC_PORT/api/public/bootstrap"
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

echo "[INFO] opening an ephemeral HTTPS tunnel to the isolated public port"
cloudflared tunnel \
  --no-autoupdate \
  --url "http://127.0.0.1:$PUBLIC_PORT" \
  > "$TMP_ROOT/cloudflared.log" 2>&1 &
TUNNEL_PID=$!

TUNNEL_URL=""
for _ in {1..120}; do
  TUNNEL_URL="$(grep -Eo 'https://[a-z0-9-]+\.trycloudflare\.com' \
    "$TMP_ROOT/cloudflared.log" | tail -n 1 || true)"
  if [[ -n "$TUNNEL_URL" ]]; then
    break
  fi
  if ! kill -0 "$TUNNEL_PID" 2>/dev/null; then
    break
  fi
  sleep 0.25
done
[[ "$TUNNEL_URL" == https://*.trycloudflare.com ]] \
  || fail "Cloudflare quick tunnel did not become ready"

echo "[INFO] creating disposable Whop sandbox product and annual plan"
product_json="$(whop_api POST /products "$(jq -cn \
  --arg company_id "$HONE_WHOP_SANDBOX_COMPANY_ID" \
  --arg title "HONE Billing Sandbox $RUN_ID" \
  '{company_id:$company_id,title:$title,visibility:"hidden",description:"Disposable HONE billing regression; archive after the run"}')")"
PRODUCT_ID="$(jq -er '.id' <<<"$product_json")"
[[ "$PRODUCT_ID" == prod_* ]] || fail "Whop sandbox product ID is invalid"

plan_json="$(whop_api POST /plans "$(jq -cn \
  --arg company_id "$HONE_WHOP_SANDBOX_COMPANY_ID" \
  --arg product_id "$PRODUCT_ID" \
  '{company_id:$company_id,product_id:$product_id,title:"Annual sandbox",description:"Disposable HONE billing lifecycle",visibility:"hidden",plan_type:"renewal",release_method:"buy_now",currency:"usd",billing_period:365,initial_price:0,renewal_price:1,unlimited_stock:true}')")"
PLAN_ID="$(jq -er '.id' <<<"$plan_json")"
PURCHASE_URL="$(jq -er '.purchase_url' <<<"$plan_json")"
[[ "$PLAN_ID" == plan_* ]] || fail "Whop sandbox plan ID is invalid"
[[ "$PURCHASE_URL" == https://sandbox.whop.com/* ]] \
  || fail "Whop sandbox returned a non-sandbox purchase URL"

webhook_url="$TUNNEL_URL/api/public/integrations/whop/webhook"
webhook_json="$(whop_api POST /webhooks "$(jq -cn \
  --arg url "$webhook_url" \
  '{url:$url,api_version:"v1",enabled:true,events:["membership.activated","membership.deactivated","membership.cancel_at_period_end_changed"]}')")"
WEBHOOK_ID="$(jq -er '.id' <<<"$webhook_json")"
WEBHOOK_SECRET="$(jq -er '.webhook_secret' <<<"$webhook_json")"
[[ "$WEBHOOK_ID" == hook_* ]] || fail "Whop sandbox webhook ID is invalid"
[[ "$WEBHOOK_SECRET" == ws_* ]] \
  || fail "Whop sandbox returned an unsupported webhook secret format"

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
    HONE_BILLING_PRIMARY_PROVIDER=whop \
    HONE_WHOP_NEW_PURCHASES_ENABLED=true \
    HONE_WHOP_WEBHOOK_SECRET="$WEBHOOK_SECRET" \
    HONE_WHOP_COMPANY_ID="$HONE_WHOP_SANDBOX_COMPANY_ID" \
    HONE_WHOP_PRODUCT_ID="$PRODUCT_ID" \
    HONE_WHOP_PLAN_ID="$PLAN_ID" \
    HONE_STRIPE_CHECKOUT_ENABLED=false \
    HONE_BILLING_GRACE_DAYS=7 \
    "$REPO_ROOT/target/debug/hone-console-page"
) > "$TMP_ROOT/server.log" 2>&1 &
SERVER_PID=$!
unset WEBHOOK_SECRET

server_ready=0
for _ in {1..120}; do
  if curl -fsS "http://127.0.0.1:$PUBLIC_PORT/api/public/billing/config" >/dev/null 2>&1; then
    server_ready=1
    break
  fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    break
  fi
  sleep 0.25
done
[[ "$server_ready" == 1 ]] || fail "isolated HONE billing runtime did not become ready"

printf '[ACTION] complete the sandbox checkout at:\n%s\n' "$PURCHASE_URL"
printf '[INFO] waiting up to 15 minutes for a real signed membership.activated webhook\n'
wait_for_sql_value \
  "Whop sandbox activation did not grant HONE access" \
  "SELECT COUNT(*) FROM billing_entitlements WHERE provider = 'whop' AND access_state = 'active'" \
  "1"

MEMBERSHIP_ID="$(sqlite_scalar "SELECT provider_subscription_id FROM billing_entitlements WHERE provider = 'whop' AND access_state = 'active' ORDER BY updated_at DESC LIMIT 1")"
USER_ID="$(sqlite_scalar "SELECT user_id FROM billing_entitlements WHERE provider = 'whop' AND access_state = 'active' ORDER BY updated_at DESC LIMIT 1")"
[[ "$MEMBERSHIP_ID" == mem_* ]] || fail "active Whop membership ID is invalid"
[[ -n "$USER_ID" ]] || fail "active HONE user ID is missing"

NOW_RFC3339="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
python3 - \
  "$TMP_ROOT/data/sessions.sqlite3" \
  "$SESSION_TOKEN" \
  "$USER_ID" \
  "$NOW_RFC3339" <<'PY'
import sqlite3
import sys

database_path, session_token, user_id, now = sys.argv[1:]
database = sqlite3.connect(database_path)
with database:
    database.execute(
        """
        INSERT INTO web_auth_sessions(
            session_token, user_id, created_at, expires_at, last_seen_at
        ) VALUES (?, ?, ?, ?, ?)
        """,
        (session_token, user_id, now, "2099-08-03T00:00:00Z", now),
    )
PY

[[ "$(paid_api_status)" == 200 ]] || fail "paid API did not change from 402 to 200"

echo "[INFO] requesting period-end cancellation through the Whop sandbox API"
whop_api POST "/memberships/$MEMBERSHIP_ID/cancel" \
  '{"cancellation_mode":"at_period_end"}' >/dev/null
wait_for_sql_value \
  "period-end cancellation did not retain active access" \
  "SELECT COUNT(*) FROM billing_entitlements WHERE provider = 'whop' AND provider_subscription_id = '$MEMBERSHIP_ID' AND access_state = 'active' AND cancel_at_period_end = 1" \
  "1"
[[ "$(paid_api_status)" == 200 ]] || fail "period-end cancellation revoked access early"

echo "[INFO] requesting immediate cancellation through the Whop sandbox API"
whop_api POST "/memberships/$MEMBERSHIP_ID/cancel" \
  '{"cancellation_mode":"immediate"}' >/dev/null
wait_for_sql_value \
  "immediate cancellation did not revoke HONE access" \
  "SELECT COUNT(*) FROM billing_entitlements WHERE provider = 'whop' AND provider_subscription_id = '$MEMBERSHIP_ID' AND access_state = 'inactive'" \
  "1"
[[ "$(paid_api_status)" == 402 ]] || fail "paid API did not change from 200 to 402"

printf '[ACTION] repurchase the same sandbox plan at:\n%s\n' "$PURCHASE_URL"
printf '[INFO] waiting up to 15 minutes for the repurchase activation\n'
wait_for_sql_value \
  "Whop sandbox repurchase did not restore HONE access" \
  "SELECT COUNT(*) FROM billing_entitlements WHERE provider = 'whop' AND access_state = 'active'" \
  "1"
[[ "$(paid_api_status)" == 200 ]] || fail "repurchase did not restore paid API access"

unfinished_rows="$(sqlite_scalar "SELECT COUNT(*) FROM billing_webhook_events WHERE provider = 'whop' AND processing_state != 'processed'")"
duplicate_attempts="$(sqlite_scalar "SELECT COUNT(*) FROM billing_webhook_events WHERE provider = 'whop' AND attempt_count != 1")"
processed_rows="$(sqlite_scalar "SELECT COUNT(*) FROM billing_webhook_events WHERE provider = 'whop' AND processing_state = 'processed'")"
[[ "$unfinished_rows" == 0 ]] || fail "Whop sandbox left unfinished webhook rows"
[[ "$duplicate_attempts" == 0 ]] || fail "Whop sandbox events were not processed exactly once"
[[ "$processed_rows" -ge 4 ]] || fail "Whop sandbox lifecycle produced fewer than four provider events"

whop_api DELETE "/webhooks/$WEBHOOK_ID" >/dev/null
WEBHOOK_DELETED=1
whop_api PATCH "/plans/$PLAN_ID" '{"visibility":"archived"}' >/dev/null
PLAN_ARCHIVED=1
whop_api PATCH "/products/$PRODUCT_ID" '{"visibility":"archived"}' >/dev/null
PRODUCT_ARCHIVED=1

printf '[PASS] real Whop sandbox activation/cancel/deactivate/repurchase lifecycle (%s events)\n' \
  "$processed_rows"
