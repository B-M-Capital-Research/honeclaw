#!/usr/bin/env bash

# Skill runtime manual regression via hone-cli.
# Verifies:
# 1. User slash skill expansion (`/skill ...`)
# 2. Model-driven skill selection for an obvious skill-matching task
#
# This script is manual-only because it depends on the locally configured ACP
# runner and model credentials.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd codex
require_cmd codex-acp
require_cmd python3

ensure_hone_mcp_binary() {
  local bin_path="$ROOT_DIR/target/debug/hone-mcp"
  if [[ ! -x "$bin_path" ]]; then
    echo "[INFO] building hone-mcp binary" >&2
    cargo build -p hone-mcp >/dev/null
  else
    echo "[INFO] refreshing hone-mcp binary" >&2
    cargo build -p hone-mcp >/dev/null
  fi
  if [[ ! -x "$bin_path" ]]; then
    echo "[FAIL] hone-mcp binary missing at $bin_path" >&2
    exit 1
  fi
  printf '%s\n' "$bin_path"
}

TMP_ITEMS=()
cleanup() {
  if ((${#TMP_ITEMS[@]} > 0)); then
    rm -rf "${TMP_ITEMS[@]}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

make_tmp_dir() {
  local dir
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_skill_runtime.XXXXXX")"
  TMP_ITEMS+=("$dir")
  printf '%s\n' "$dir"
}

write_override_config() {
  local case_dir="$1"
  local data_dir="$2"
  local runtime_log="$3"
  cat >"$case_dir/config.overrides.yaml" <<EOF
skills_dir: "$ROOT_DIR/skills"
agent:
  runner: "codex_acp"
  max_iterations: 30
  system_prompt_path: "$ROOT_DIR/soul.md"
storage:
  sessions_dir: "$data_dir/sessions"
  session_sqlite_db_path: "$data_dir/sessions.sqlite3"
  session_sqlite_shadow_write_enabled: false
  session_runtime_backend: "json"
  conversation_quota_dir: "$data_dir/conversation_quota"
  llm_audit_db_path: "$data_dir/llm_audit.sqlite3"
  llm_audit_enabled: false
  portfolio_dir: "$data_dir/portfolio"
  reports_dir: "$data_dir/reports"
  cron_jobs_dir: "$data_dir/cron_jobs"
  x_drafts_dir: "$data_dir/x_drafts"
  gen_images_dir: "$data_dir/gen_images"
  kb_dir: "$data_dir/kb"
logging:
  console: false
  file: "$runtime_log"
EOF
}

run_case() {
  local case_name="$1"
  local prompt_text="$2"
  local expected_token="$3"
  local expected_skill="$4"
  local expected_stdout_snippet="$5"

  local case_dir data_dir stdout_file stderr_file session_path runtime_log
  local hone_mcp_bin
  case_dir="$(make_tmp_dir)"
  data_dir="$case_dir/data"
  stdout_file="$case_dir/stdout.log"
  stderr_file="$case_dir/stderr.log"
  runtime_log="$case_dir/runtime.log"
  hone_mcp_bin="$(ensure_hone_mcp_binary)"

  mkdir -p "$data_dir"
  cp "$ROOT_DIR/config.yaml" "$case_dir/config.yaml"
  write_override_config "$case_dir" "$data_dir" "$runtime_log"

  echo "[INFO] running case=$case_name"
  (
    cd "$case_dir"
    HONE_CONFIG_PATH="$case_dir/config.yaml" \
    HONE_DATA_DIR="$data_dir" \
    HONE_MCP_BIN="$hone_mcp_bin" \
    cargo run -q --manifest-path "$ROOT_DIR/Cargo.toml" -p hone-cli >"$stdout_file" 2>"$stderr_file" <<EOF
$prompt_text
quit
EOF
  )

  if [[ -n "$expected_token" ]]; then
    if ! grep -q "$expected_token" "$stdout_file"; then
      echo "[FAIL] case=$case_name missing expected token: $expected_token" >&2
      echo "--- stdout ---" >&2
      cat "$stdout_file" >&2 || true
      echo "--- stderr ---" >&2
      cat "$stderr_file" >&2 || true
      exit 1
    fi
  fi

  if [[ -n "$expected_stdout_snippet" ]]; then
    if ! grep -q "$expected_stdout_snippet" "$stdout_file"; then
      echo "[FAIL] case=$case_name missing expected stdout snippet: $expected_stdout_snippet" >&2
      echo "--- stdout ---" >&2
      cat "$stdout_file" >&2 || true
      echo "--- stderr ---" >&2
      cat "$stderr_file" >&2 || true
      exit 1
    fi
  fi

  session_path="$(find "$data_dir/sessions" -maxdepth 1 -name '*.json' | head -n 1)"
  if [[ ! -f "$session_path" ]]; then
    echo "[FAIL] case=$case_name did not produce any session file under $data_dir/sessions" >&2
    echo "--- stdout ---" >&2
    cat "$stdout_file" >&2 || true
    echo "--- stderr ---" >&2
    cat "$stderr_file" >&2 || true
    exit 1
  fi

  python3 - "$session_path" "$expected_skill" "$case_name" <<'PY'
import json
import sys
from pathlib import Path

session_path = Path(sys.argv[1])
expected_skill = sys.argv[2]
case_name = sys.argv[3]

payload = json.loads(session_path.read_text())
skills = payload.get("metadata", {}).get("skill_runtime.invoked_skills", [])
skill_names = [item.get("skill_name", "") for item in skills]

if expected_skill not in skill_names:
    print(f"[FAIL] case={case_name} expected invoked skill {expected_skill!r}, got {skill_names!r}", file=sys.stderr)
    print(session_path.read_text(), file=sys.stderr)
    sys.exit(1)

print(f"[INFO] case={case_name} invoked_skills={skill_names}")
PY

  echo "[PASS] case=$case_name token=$expected_token skill=$expected_skill"
}

run_case \
  "slash_skill_lookup" \
  "/skill skill manager" \
  "" \
  "skill_manager" \
  ""

run_case \
  "model_auto_skill_selection" \
  "I want to understand how Hone skills are designed and which built-in skill I should use before creating or editing a skill. Choose and invoke the most relevant built-in skill before answering. End your final answer with the exact token HONE_AUTO_SKILL_OK." \
  "HONE_AUTO_SKILL_OK" \
  "skill_manager" \
  ""

echo "[PASS] skill runtime CLI regression passed"
