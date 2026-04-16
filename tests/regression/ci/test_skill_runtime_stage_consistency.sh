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
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_skill_runtime_stage_consistency.XXXXXX")"
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
  local allow_cron="$2"
  local expect_discover_has_skill="$3"
  local expect_list_has_cron="$4"
  local expect_skill_mode="$5"
  local expect_cron_mode="$6"
  local output_file="$CASE_ROOT/${case_name}.json"

  echo "[INFO] case=$case_name allow_cron=$allow_cron"

  CASE_NAME="$case_name" \
  ALLOW_CRON="$allow_cron" \
  EXPECT_DISCOVER_HAS_SKILL="$expect_discover_has_skill" \
  EXPECT_LIST_HAS_CRON="$expect_list_has_cron" \
  EXPECT_SKILL_MODE="$expect_skill_mode" \
  EXPECT_CRON_MODE="$expect_cron_mode" \
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

env = os.environ.copy()
env.update(
    {
        "HONE_CONFIG_PATH": os.environ["BASE_CONFIG"],
        "HONE_DATA_DIR": os.environ["DATA_DIR"],
        "HONE_SKILLS_DIR": str(pathlib.Path(os.environ["ROOT_DIR"]) / "skills"),
        "HONE_MCP_ACTOR_CHANNEL": "telegram",
        "HONE_MCP_ACTOR_USER_ID": "skill-runtime-stage-consistency",
        "HONE_MCP_CHANNEL_TARGET": "telegram",
        "HONE_MCP_ALLOW_CRON": os.environ["ALLOW_CRON"],
    }
)

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
            "name": "discover_skills",
            "arguments": {"query": "scheduled task", "limit": 5},
        },
    },
    {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "skill_tool",
            "arguments": {"skill_name": "scheduled_task"},
        },
    },
    {"jsonrpc": "2.0", "id": 4, "method": "tools/list", "params": {}},
    {
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {"name": "cron_job", "arguments": {"action": "list"}},
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
discover_payload = next(item["result"]["structuredContent"] for item in responses if item.get("id") == 2)
skill_payload = next(item["result"]["structuredContent"] for item in responses if item.get("id") == 3)
tools_payload = next(item["result"]["tools"] for item in responses if item.get("id") == 4)
cron_payload = next(item["result"] for item in responses if item.get("id") == 5)

discover_ids = [skill["id"] for skill in discover_payload.get("skills", [])]
tool_names = [tool["name"] for tool in tools_payload]

pathlib.Path(os.environ["OUTPUT_FILE"]).write_text(
    json.dumps(
        {
            "discover_ids": discover_ids,
            "skill_payload": skill_payload,
            "tool_names": tool_names,
            "cron_payload": cron_payload,
            "stderr": stderr,
        },
        ensure_ascii=False,
        indent=2,
    )
)

print(
    json.dumps(
        {
            "discover_ids": discover_ids,
            "tool_names": tool_names,
            "skill_success": skill_payload.get("success"),
            "cron_is_error": cron_payload.get("isError"),
        },
        ensure_ascii=False,
    )
)

expect_discover_has_skill = os.environ["EXPECT_DISCOVER_HAS_SKILL"] == "1"
if ("scheduled_task" in discover_ids) != expect_discover_has_skill:
    print(
        f"[FAIL] expected discover to include scheduled_task={expect_discover_has_skill}, got {discover_ids}",
        file=sys.stderr,
    )
    sys.exit(1)

expect_list_has_cron = os.environ["EXPECT_LIST_HAS_CRON"] == "1"
if ("cron_job" in tool_names) != expect_list_has_cron:
    print(
        f"[FAIL] expected tools/list to include cron_job={expect_list_has_cron}, got {tool_names}",
        file=sys.stderr,
    )
    sys.exit(1)

expect_skill_mode = os.environ["EXPECT_SKILL_MODE"]
skill_error = skill_payload.get("error", "")
if expect_skill_mode == "success":
    if skill_payload.get("success") is not True or skill_payload.get("skill_name") != "scheduled_task":
        print(f"[FAIL] expected skill_tool success, got {skill_payload}", file=sys.stderr)
        sys.exit(1)
elif expect_skill_mode == "stage-missing":
    if "本阶段缺少工具 cron_job" not in skill_error:
        print(f"[FAIL] expected stage-missing skill error, got {skill_payload}", file=sys.stderr)
        sys.exit(1)
else:
    raise ValueError(f"unknown EXPECT_SKILL_MODE={expect_skill_mode}")

expect_cron_mode = os.environ["EXPECT_CRON_MODE"]
cron_text = ""
content = cron_payload.get("content")
if isinstance(content, list) and content:
    cron_text = content[0].get("text", "")

if expect_cron_mode == "success":
    if cron_payload.get("isError") is not False:
        print(f"[FAIL] expected cron_job success, got {cron_payload}", file=sys.stderr)
        sys.exit(1)
elif expect_cron_mode == "missing":
    if "工具不存在: cron_job" not in cron_text:
        print(f"[FAIL] expected missing cron_job error, got {cron_payload}", file=sys.stderr)
        sys.exit(1)
else:
    raise ValueError(f"unknown EXPECT_CRON_MODE={expect_cron_mode}")

print(f"[PASS] case={os.environ['CASE_NAME']}")
PY
}

run_case "cron_disabled" "0" "0" "0" "stage-missing" "missing"
run_case "cron_enabled" "1" "1" "1" "success" "success"

echo "[PASS] skill runtime stage consistency regression passed"
