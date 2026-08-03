#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

if rg -n '/activate/whop|whop_membership|registration_policy' \
  packages/app/src \
  crates/hone-web-api/src \
  --glob '!**/*test*' \
  --glob '!memory/src/web_auth.rs'; then
  echo "[FAIL] legacy provider-specific billing surface remains" >&2
  exit 1
fi

rg -Fq 'POST /api/public/integrations/stripe/webhook' docs/runbooks/stripe-billing.md
rg -Fq 'HONE_STRIPE_CHECKOUT_ENABLED=false' .env.example
rg -Fq 'HONE_STRIPE_MODE=test' .env.example
rg -Fq 'window.location.assign(checkout_url)' packages/app/src/pages/public-activate.tsx
rg -Fq 'purchases_allowed_on_this_client' packages/app/src/pages/public-plan.tsx
rg -Fq 'management_allowed_on_this_client' packages/app/src/pages/public-me.tsx
rg -Fq 'billing_entitlements' memory/src/billing.rs
rg -Fq 'billing_webhook_events' memory/src/billing.rs

cargo test -p hone-memory billing::tests --quiet
cargo test -p hone-web-api routes::stripe::tests --quiet
cargo test -p hone-web-api routes::whop::tests --quiet
bun test --preload ./packages/app/happydom.ts \
  packages/app/src/lib/public-membership.test.ts \
  packages/app/src/pages/public-billing-activation-contract.test.ts \
  packages/app/src/pages/public-plan-purchase-contract.test.ts

echo "[PASS] provider-neutral Stripe + Whop billing contract"
