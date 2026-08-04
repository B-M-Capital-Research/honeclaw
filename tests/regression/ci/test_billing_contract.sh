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
  local pattern='/activate/whop|whop_membership|registration_policy'

  if command -v rg >/dev/null 2>&1; then
    rg -n "$pattern" \
      packages/app/src \
      crates/hone-web-api/src \
      --glob '!**/*test*' \
      --glob '!memory/src/web_auth.rs'
  else
    find packages/app/src crates/hone-web-api/src \
      -type f \
      ! -path '*test*' \
      ! -path '*/memory/src/web_auth.rs' \
      -print0 \
      | xargs -0 grep -En -- "$pattern"
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
contains 'purchases_allowed_on_this_client' packages/app/src/pages/public-plan.tsx
contains 'management_allowed_on_this_client' packages/app/src/pages/public-me.tsx
contains 'billing_entitlements' memory/src/billing.rs
contains 'billing_webhook_events' memory/src/billing.rs

cargo test -p hone-memory billing::tests --quiet
cargo test -p hone-web-api routes::stripe::tests --quiet
cargo test -p hone-web-api routes::whop::tests --quiet
bun test --preload ./packages/app/happydom.ts \
  packages/app/src/lib/public-membership.test.ts \
  packages/app/src/pages/public-billing-activation-contract.test.ts \
  packages/app/src/pages/public-plan-purchase-contract.test.ts

echo "[PASS] provider-neutral Stripe + Whop billing contract"
