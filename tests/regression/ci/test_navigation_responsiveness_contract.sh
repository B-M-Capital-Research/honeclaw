#!/usr/bin/env bash
#
# Users reported that switching sections had a noticeable delay and that a sent
# message sat silent for a long time. Neither was the navigation itself: every
# route is lazy so the first click paid for a chunk download, the account page
# blanked itself for a round-trip it did not need, and the pre-turn evidence
# pass ran roughly twenty provider calls behind two static status lines.
#
# This locks the three fixes so responsiveness cannot regress silently.

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

WORKSPACE="packages/app/src/components/public-agent-workspace.tsx"
ME_PAGE="packages/app/src/pages/public-me.tsx"
API="packages/app/src/lib/api.ts"
PREFETCH="packages/app/src/lib/route-prefetch.ts"
SESSION_CACHE="packages/app/src/lib/public-session-cache.ts"
GUARD="crates/hone-channels/src/investment_response_guard.rs"
SESSION_CORE="crates/hone-channels/src/agent_session/core.rs"
WEB_CHAT="crates/hone-web-api/src/routes/chat.rs"

fail() {
  echo "[FAIL] $1"
  exit 1
}

# 1. A lazy route's chunk is warmed before the click that needs it.
contains 'import("@/pages/public-me")' "$PREFETCH" || fail "route prefetch lost its me chunk"
contains 'import("@/pages/public-community")' "$PREFETCH" ||
  fail "route prefetch lost its community chunk"
contains "onPointerEnter" "$PREFETCH" || fail "prefetch no longer warms on pointer entry"
contains 'routePrefetchHandlers("me")' "$WORKSPACE" || fail "account controls stopped prefetching"
contains 'routePrefetchHandlers("community")' "$WORKSPACE" ||
  fail "insights controls stopped prefetching"

# 2. A route paints from what is already known and revalidates behind it.
contains "setCachedPublicUser(payload.user)" "$API" ||
  fail "the resolved user is no longer cached for the next route"
contains "createSignal<PublicAuthUserInfo | null>(cachedPublicUser())" "$ME_PAGE" ||
  fail "the account page stopped painting from cache"
contains "createSignal(!hasCachedPublicUser())" "$ME_PAGE" ||
  fail "the account page blanks itself again while it revalidates"
contains "setCachedPublicUser(null)" "$ME_PAGE" ||
  fail "a stale cache can now outlive the session"
contains "export function setCachedPublicUser" "$SESSION_CACHE" ||
  fail "session cache lost its writer"

# 3. The pre-turn pass reports movement instead of one static line.
contains "pub(crate) type PreTurnProgressSink" "$GUARD" ||
  fail "the pre-turn pass can no longer report stages"
contains '"preturn.identity"' "$GUARD" || fail "identity stage is no longer reported"
contains '"preturn.evidence"' "$GUARD" || fail "evidence stage is no longer reported"
contains "progress_rx.recv()" "$SESSION_CORE" ||
  fail "stage updates are no longer drained while the pass runs"
contains "fn public_progress_detail" "$WEB_CHAT" ||
  fail "stage details reach the browser unsanitized"
contains '"preturn.evidence" => Some((' "$WEB_CHAT" ||
  fail "the evidence stage lost its user-facing wording"

# 4. A slow provider degrades one branch, not the whole pass.
contains "async fn bounded_branch" "$GUARD" ||
  fail "evidence branches share one budget again"
contains "PRETURN_IDENTITY_DEADLINE" "$GUARD" ||
  fail "identity resolution lost its own budget"
contains "PRETURN_EVIDENCE_BRANCH_DEADLINE" "$GUARD" ||
  fail "evidence branches lost their own budget"
contains "bounded_branch(futures::future::join_all(pending" "$GUARD" ||
  fail "the quote branch is unbounded again"

# 5. Reopening a section repaints before it revalidates.
contains "cachedCommunityFeed()" "packages/app/src/pages/public-community.tsx" ||
  fail "the community feed stopped painting from cache"
contains "setCachedCommunityFeed(null)" "packages/app/src/pages/public-community.tsx" ||
  fail "a signed-out visitor can keep reading a cached feed"

cargo test -p hone-web-api routes::chat::tests --quiet
if command -v bun >/dev/null 2>&1; then
  bun test --preload ./packages/app/happydom.ts \
    packages/app/src/lib/public-session-cache.test.ts \
    packages/app/src/components/public-navigation-responsiveness.test.ts
else
  echo "[INFO] bun unavailable; frontend-checks owns the complete Web unit suite"
fi

echo "[PASS] navigation and progress responsiveness contract"
