#!/usr/bin/env bash
set -euo pipefail

# Manual regression for Discord group reply mechanism:
# 1) window batching
# 2) mention fast-path
# 3) backlog degradation policy

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

echo "[INFO] Running Discord group-reply regression (unit-level)..."

cargo test -p hone-discord --all-targets tests::collect_group_batch_collects_messages_within_window
cargo test -p hone-discord --all-targets tests::mention_fast_path_tightens_deadline
cargo test -p hone-discord --all-targets tests::backlog_policy_keeps_latest_and_summarizes_old

echo "[PASS] Discord group-reply core regression passed."
echo "[INFO] For live Discord behavior, run hone-discord with a real bot token and verify:"
echo "       - normal question burst in 45s window => single merged reply"
echo "       - direct @ => fast response path"
echo "       - high-volume burst => keeps latest N with summarized backlog"
