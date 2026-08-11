#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
backend="$repo_root/crates/hone-web-api/src/routes/research_library.rs"
routes="$repo_root/crates/hone-web-api/src/routes/mod.rs"
frontend="$repo_root/packages/app/src/pages/public-research-library.tsx"

rg -q 'CommunityCandidate' "$backend"
rg -q 'ResearchReviewStatus::Pending' "$backend"
rg -q '只有管理员可以审核社区投稿' "$backend"
rg -q 'fn list_retrievable' "$backend"
if sed -n '/fn list_retrievable/,/^}/p' "$backend" | rg -q 'CommunityCandidate'; then
  echo "candidate material must not enter retrieval before approval" >&2
  exit 1
fi
rg -q '"/research-library/\{id\}/submit"' "$routes"
rg -q '"/research-library/\{id\}/review"' "$routes"
rg -q 'official_skill_export' "$backend"
rg -q 'automatic_sync": false' "$backend"
rg -q '投稿给 HONE' "$frontend"
rg -q '核验并采纳' "$frontend"

echo "research curation contract: ok"
