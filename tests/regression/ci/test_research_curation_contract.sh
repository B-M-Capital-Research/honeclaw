#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
backend="$repo_root/crates/hone-web-api/src/routes/research_library.rs"

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

routes="$repo_root/crates/hone-web-api/src/routes/mod.rs"
frontend="$repo_root/packages/app/src/pages/public-research-library.tsx"

contains 'CommunityCandidate' "$backend"
contains 'ResearchReviewStatus::Pending' "$backend"
contains '只有管理员可以审核社区投稿' "$backend"
contains 'fn list_retrievable' "$backend"
if sed -n '/fn list_retrievable/,/^}/p' "$backend" | { if command -v rg >/dev/null 2>&1; then rg -q --fixed-strings 'CommunityCandidate'; else grep -F -q -- 'CommunityCandidate'; fi; }; then
  echo "candidate material must not enter retrieval before approval" >&2
  exit 1
fi
contains '"/research-library/{id}/submit"' "$routes"
contains '"/research-library/{id}/review"' "$routes"
contains 'official_skill_export' "$backend"
contains 'automatic_sync": false' "$backend"
contains '投稿给 HONE' "$frontend"
contains '核验并采纳' "$frontend"

echo "research curation contract: ok"
