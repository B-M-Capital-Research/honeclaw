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

echo "[INFO] Running Discord outbound regression tests..."
cargo test -p hone-discord
cargo test -p hone-channels response_content_segments_
cargo test -p hone-tools chart_visualization_renderer_smoke_writes_png_when_matplotlib_is_available

cat <<'EOF'
[INFO] Manual Discord verification steps
1. Start `hone-discord` with a real bot token and a reachable channel.
2. Ask a question that should trigger the chart skill.
3. Verify Discord sends:
   - text before the chart when present
   - the chart as a real attachment/image message
   - trailing text after the chart when present
4. Confirm the visible answer never contains the raw local `file:///...` path.
5. In reply-threaded flows, verify text/image/text ordering is preserved across referenced follow-up messages.
EOF

echo "[PASS] chart visualization Discord regression checks completed"
