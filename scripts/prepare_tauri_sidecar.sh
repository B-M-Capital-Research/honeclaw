#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec bun "$ROOT_DIR/scripts/prepare_tauri_sidecar.mjs" "${1:-debug}"
