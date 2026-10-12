#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

if ! command -v python3 >/dev/null 2>&1; then
    echo "[FAIL] Python 3.9+ is required for API liveness monitor regression tests" >&2
    exit 1
fi

python3 - <<'PY'
import sys

if sys.version_info < (3, 9):
    raise SystemExit("[FAIL] Python 3.9+ is required for API liveness monitor regression tests")
PY

# Only a local ephemeral HTTP server and temporary fixtures; no root, PostgreSQL,
# production endpoints, account credentials, or external packages are needed.
python3 tests/regression/ci/test_diagnose_api_liveness.py
echo "[PASS] API liveness monitor privacy, failure capture and bounded logs"
