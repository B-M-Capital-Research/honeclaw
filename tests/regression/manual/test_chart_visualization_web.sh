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
require_cmd bun
require_cmd python3

echo "[INFO] Running local chart + Web parser regressions..."
cargo test -p hone-tools chart_visualization_renderer_smoke_writes_png_when_matplotlib_is_available
cargo test -p hone-channels response_content_segments_
cargo test -p hone-web-api history_attachments_
bun run test:web

cat <<'EOF'
[INFO] Manual Web verification steps
1. Start Hone Web locally with the normal dev/runtime entrypoint.
2. Ask a trend-style question that should use the chart skill, for example:
   "Visualize quarterly revenue trend with a line chart using these values: Q1 12, Q2 15, Q3 18, Q4 17."
3. Verify the final assistant reply contains inline text before/after the chart and the UI renders an actual image instead of literal `file://` text.
4. Refresh the page or reopen the conversation history.
5. Verify the same reply still shows the inline chart and history attachment extraction treats the inline local image marker as an image attachment.
EOF

echo "[PASS] chart visualization Web regression checks completed"
