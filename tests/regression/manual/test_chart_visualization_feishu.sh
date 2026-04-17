#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_cmd cargo

echo "[INFO] Running Feishu outbound regression tests..."
cargo test -p hone-feishu
cargo test -p hone-channels response_content_segments_
cargo test -p hone-tools execute_rejects_artifacts_outside_allowed_roots

cat <<'EOF'
[INFO] Manual Feishu verification steps
1. Start `hone-feishu` with a real app credential and a reachable target chat.
2. Ask a trend-style question that should produce a chart.
3. Verify Feishu receives:
   - leading text if the assistant placed text before the chart
   - a real image message uploaded through Feishu image APIs
   - trailing text after the chart when present
4. Confirm no raw `file:///...` local path is visible in the chat.
5. Repeat once in `p2p` and once in a group chat if both modes are used in production.
EOF

echo "[PASS] chart visualization Feishu regression checks completed"
