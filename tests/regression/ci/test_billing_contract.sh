#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

contains() {
  local pattern="$1"
  local file="$2"

  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$file"
  else
    grep -F -q -- "$pattern" "$file"
  fi
}

legacy_billing_surfaces() {
  local pattern='whop|/activate/whop|whop_membership|registration_policy|HONE_BILLING_PRIMARY_PROVIDER'

  if command -v rg >/dev/null 2>&1; then
    rg -n -i "$pattern" \
      packages/app/src \
      crates/hone-web-api/src \
      memory/src/lib.rs \
      memory/src/web_auth.rs \
      .env.example \
      --glob '!**/*test*'
    sed '/^#\[cfg(test)\]/,$d' memory/src/billing.rs | rg -n -i "$pattern"
  else
    find packages/app/src crates/hone-web-api/src \
      -type f \
      ! -path '*test*' \
      -print0 \
      | xargs -0 grep -En -- "$pattern"
    grep -Ein -- "$pattern" memory/src/lib.rs memory/src/web_auth.rs .env.example
    sed '/^#\[cfg(test)\]/,$d' memory/src/billing.rs | grep -Ein -- "$pattern"
  fi
}

if legacy_billing_surfaces; then
  echo "[FAIL] legacy provider-specific billing surface remains" >&2
  exit 1
fi

contains 'POST /api/public/integrations/stripe/webhook' docs/runbooks/stripe-billing.md
contains 'HONE_STRIPE_CHECKOUT_ENABLED=false' .env.example
contains 'HONE_STRIPE_MODE=test' .env.example
contains 'window.location.assign(checkout_url)' packages/app/src/pages/public-activate.tsx
contains 'const purchaseAvailable = createMemo' packages/app/src/pages/public-activate.tsx
contains 'configReady()' packages/app/src/pages/public-activate.tsx
contains 'href="/activate"' packages/app/src/pages/public-plan.tsx
contains 'purchases_allowed_on_this_client' packages/app/src/pages/public-plan.tsx
contains 'management_allowed_on_this_client' packages/app/src/pages/public-me.tsx
contains 'billing_entitlements' memory/src/billing.rs
contains 'billing_webhook_events' memory/src/billing.rs
contains '20260804_stripe_only_billing' crates/hone-core/src/cloud_runtime.rs

if [[ -e crates/hone-web-api/src/routes/whop.rs ]]; then
  echo "[FAIL] Whop webhook adapter still exists" >&2
  exit 1
fi

cargo test -p hone-memory billing::tests --quiet
cargo test -p hone-web-api routes::stripe::tests --quiet
if command -v bun >/dev/null 2>&1; then
  bun test --preload ./packages/app/happydom.ts \
    packages/app/src/lib/public-membership.test.ts \
    packages/app/src/pages/public-billing-activation-contract.test.ts \
    packages/app/src/pages/public-plan-purchase-contract.test.ts
else
  echo "[INFO] bun unavailable; frontend-checks owns the complete Web unit suite"
fi

echo "[PASS] Stripe-only billing contract"
