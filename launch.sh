#!/usr/bin/env bash
set -euo pipefail

cat >&2 <<'EOF'
launch.sh has been retired.

Use the CLI startup path instead:

  # Source checkout: build local CLI/runtime binaries, then start from target/debug
  cargo run -p hone-cli -- start --build

  # Installed users, including Homebrew or the release installer
  hone-cli start

  # Configure before starting
  hone-cli onboard
  hone-cli configure --section agent --section channels --section providers

For Web UI development, run the frontend separately after the backend is up:

  bun run dev:web
  bun run dev:web:public

For desktop development, prepare sidecars and use Tauri directly:

  bun run tauri:prep:dev -- --skip-dev-command
  bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json
EOF

exit 64
