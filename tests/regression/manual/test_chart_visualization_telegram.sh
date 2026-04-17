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

echo "[INFO] Running Telegram outbound regression tests..."
cargo test -p hone-telegram
cargo test -p hone-channels response_content_segments_
cargo test -p hone-tools chart_visualization_renderer_smoke_writes_png_when_matplotlib_is_available

cat <<'EOF'
[INFO] Manual Telegram verification steps
1. Start `hone-telegram` with a real bot token and reachable chat.
2. Ask a trend/comparison question that should produce a chart.
3. Verify Telegram delivers the answer in order:
   - text segment before the chart when present
   - chart as a real photo sent via `send_photo`
   - text segment after the chart when present
4. Confirm no raw `file:///...` local path is visible to the user.
5. If a placeholder/progress message exists, verify the first visible content still resolves into the correct ordered chain.
EOF

echo "[PASS] chart visualization Telegram regression checks completed"
