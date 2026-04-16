#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd python3

TMP_ITEMS=()
cleanup() {
  if ((${#TMP_ITEMS[@]} > 0)); then
    rm -rf "${TMP_ITEMS[@]}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

make_tmp_dir() {
  local dir
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_mcp_skill_dir_env.XXXXXX")"
  TMP_ITEMS+=("$dir")
  printf '%s\n' "$dir"
}

ensure_hone_mcp_binary() {
  local bin_path="$ROOT_DIR/target/debug/hone-mcp"
  echo "[INFO] building hone-mcp binary" >&2
  cargo build -p hone-mcp >/dev/null
  if [[ ! -x "$bin_path" ]]; then
    echo "[FAIL] hone-mcp binary missing at $bin_path" >&2
    exit 1
  fi
  printf '%s\n' "$bin_path"
}

BASE_CONFIG="$ROOT_DIR/config.yaml"
if [[ ! -f "$BASE_CONFIG" ]]; then
  BASE_CONFIG="$ROOT_DIR/config.example.yaml"
fi
if [[ ! -f "$BASE_CONFIG" ]]; then
  echo "[FAIL] missing base config.yaml/config.example.yaml" >&2
  exit 1
fi

HONE_MCP_BIN="$(ensure_hone_mcp_binary)"
CASE_ROOT="$(make_tmp_dir)"
DATA_DIR="$CASE_ROOT/data"
SANDBOX_DIR="$CASE_ROOT/sandbox"
mkdir -p "$DATA_DIR" "$SANDBOX_DIR"

run_case() {
  local case_name="$1"
  local pass_skills_dir="$2"
  local expect_success="$3"
  local output_file="$CASE_ROOT/${case_name}.json"

  echo "[INFO] case=$case_name cwd=$SANDBOX_DIR pass_skills_dir=$pass_skills_dir"

  CASE_NAME="$case_name" \
  PASS_SKILLS_DIR="$pass_skills_dir" \
  EXPECT_SUCCESS="$expect_success" \
  OUTPUT_FILE="$output_file" \
  HONE_MCP_BIN="$HONE_MCP_BIN" \
  BASE_CONFIG="$BASE_CONFIG" \
  DATA_DIR="$DATA_DIR" \
  ROOT_DIR="$ROOT_DIR" \
  SANDBOX_DIR="$SANDBOX_DIR" \
  python3 - <<'PY'
import json
import os
import pathlib
import subprocess
import sys

case_name = os.environ["CASE_NAME"]
pass_skills_dir = os.environ["PASS_SKILLS_DIR"] == "1"
expect_success = os.environ["EXPECT_SUCCESS"] == "1"
output_file = pathlib.Path(os.environ["OUTPUT_FILE"])

env = os.environ.copy()
env.update(
    {
        "HONE_CONFIG_PATH": os.environ["BASE_CONFIG"],
        "HONE_DATA_DIR": os.environ["DATA_DIR"],
        "HONE_MCP_ALLOW_CRON": "1",
        "HONE_MCP_ACTOR_CHANNEL": "cli",
        "HONE_MCP_ACTOR_USER_ID": "skill-dir-env-test",
        "HONE_MCP_CHANNEL_TARGET": "cli",
    }
)
if pass_skills_dir:
    env["HONE_SKILLS_DIR"] = str(pathlib.Path(os.environ["ROOT_DIR"]) / "skills")
else:
    env.pop("HONE_SKILLS_DIR", None)

reqs = [
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "clientCapabilities": {}},
    },
    {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "skill_tool",
            "arguments": {"skill_name": "scheduled_task"},
        },
    },
]

proc = subprocess.Popen(
    [os.environ["HONE_MCP_BIN"]],
    cwd=os.environ["SANDBOX_DIR"],
    env=env,
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
)
stdout, stderr = proc.communicate(
    "\n".join(json.dumps(item, ensure_ascii=False) for item in reqs) + "\n",
    timeout=20,
)

responses = [json.loads(line) for line in stdout.splitlines() if line.strip()]
tool_payload = next(
    item["result"]["structuredContent"] for item in responses if item.get("id") == 2
)
output_file.write_text(
    json.dumps(
        {
            "case": case_name,
            "stdout": responses,
            "stderr": stderr,
            "tool_payload": tool_payload,
        },
        ensure_ascii=False,
        indent=2,
    )
)

print(json.dumps(tool_payload, ensure_ascii=False))

if tool_payload.get("success") != expect_success:
    print(
        f"[FAIL] case={case_name} expected success={expect_success} got {tool_payload.get('success')}",
        file=sys.stderr,
    )
    print(stdout, file=sys.stderr)
    print(stderr, file=sys.stderr)
    sys.exit(1)

if expect_success:
    if tool_payload.get("skill_name") != "scheduled_task":
        print(
            f"[FAIL] case={case_name} expected skill_name=scheduled_task got {tool_payload.get('skill_name')!r}",
            file=sys.stderr,
        )
        sys.exit(1)
else:
    error = tool_payload.get("error", "")
    if "不存在或当前未激活" not in error:
        print(
            f"[FAIL] case={case_name} expected missing-skill error, got {error!r}",
            file=sys.stderr,
        )
        sys.exit(1)

print(f"[PASS] case={case_name}")
PY
}

run_case "missing_skills_dir_env" "0" "0"
run_case "absolute_skills_dir_env" "1" "1"

echo "[PASS] hone-mcp skill dir env regression passed"
