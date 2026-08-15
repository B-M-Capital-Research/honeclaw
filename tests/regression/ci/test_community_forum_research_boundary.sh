#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
cd "$repo_root"

backend=crates/hone-web-api/src/routes/community_forum.rs
frontend=packages/app/src/components/community-forum.tsx

# `rg` 不是 GitHub runner 的保证依赖(2026-08-16 实测 CI 上 `rg: command not found`,
# 该失败自 67d292b1 起一直红着),本机上它也可能只是 shell 函数而非二进制。
# 与 test_billing_contract.sh 一致:有 rg 用 rg,没有就回退 grep。
contains() {
  local pattern="$1"
  shift
  if command -v rg >/dev/null 2>&1; then
    rg -q --fixed-strings "$pattern" "$@"
  else
    grep -F -q -- "$pattern" "$@"
  fi
}

contains 'require_public_user' "$backend"
contains 'forum_content_is_research: false' "$backend"
contains 'pending_review' "$backend"
contains 'AUTO_HIDE_REPORTS' "$backend"
contains '不会自动进入投资助手、评级或每日产品' "$frontend"
contains '等待管理员核验与采纳' "$frontend"

if contains 'community_forum' \
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
