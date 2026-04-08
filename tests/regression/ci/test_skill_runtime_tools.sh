#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

cargo test -p hone-tools \
  discover_skills::tests::execute_searches_real_skill_files_with_path_filter_and_limit \
  -- --exact

cargo test -p hone-tools \
  skill_tool::tests::execute_persists_invoked_skill_into_real_session_storage \
  -- --exact

echo "skill runtime tool regression passed"
