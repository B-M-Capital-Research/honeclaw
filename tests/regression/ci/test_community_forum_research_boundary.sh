#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
cd "$repo_root"

backend=crates/hone-web-api/src/routes/community_forum.rs
frontend=packages/app/src/components/community-forum.tsx

rg -q 'require_public_user' "$backend"
rg -q 'forum_content_is_research: false' "$backend"
rg -q 'pending_review' "$backend"
rg -q 'AUTO_HIDE_REPORTS' "$backend"
rg -q '不会自动进入投资助手、评级或每日产品' "$frontend"
rg -q '等待管理员核验与采纳' "$frontend"

if rg -q 'community_forum' \
  crates/hone-channels/src/prompt.rs \
  crates/hone-web-api/src/routes/company_ratings.rs \
  crates/hone-web-api/src/routes/daily_signals.rs \
  crates/hone-web-api/src/routes/key_event_chain.rs \
  crates/hone-web-api/src/routes/portfolio_news.rs \
  crates/hone-web-api/src/routes/research_library.rs; then
  echo "community forum leaked into a research or prompt path" >&2
  exit 1
fi

echo "community forum research boundary: ok"
