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
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_mcp_cron_visibility.XXXXXX")"
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
  local allowed_tools="${3:-}"
  local expect_list_has_cron="$4"
  local expect_call_mode="$5"
  local output_file="$CASE_ROOT/${case_name}.json"

  echo "[INFO] case=$case_name allow_cron=$allow_cron allowed_tools=${allowed_tools:-<none>}"

  CASE_NAME="$case_name" \
  ALLOW_CRON="$allow_cron" \
  ALLOWED_TOOLS="$allowed_tools" \
  EXPECT_LIST_HAS_CRON="$expect_list_has_cron" \
  EXPECT_CALL_MODE="$expect_call_mode" \
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
        "HONE_MCP_ACTOR_USER_ID": "cron-visibility-test",
        "HONE_MCP_CHANNEL_TARGET": "telegram",
        "HONE_MCP_ALLOW_CRON": os.environ["ALLOW_CRON"],
    }
)
allowed_tools = os.environ.get("ALLOWED_TOOLS", "").strip()
if allowed_tools:
    env["HONE_MCP_ALLOWED_TOOLS"] = allowed_tools
else:
    env.pop("HONE_MCP_ALLOWED_TOOLS", None)

reqs = [
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "clientCapabilities": {}},
    },
    {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    {
        "jsonrpc": "2.0",
        "id": 3,
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
tools = next(item["result"]["tools"] for item in responses if item.get("id") == 2)
tool_names = [tool["name"] for tool in tools]
call_payload = next(item["result"] for item in responses if item.get("id") == 3)

pathlib.Path(os.environ["OUTPUT_FILE"]).write_text(
    json.dumps(
        {
            "tools": tool_names,
            "call_payload": call_payload,
            "stderr": stderr,
        },
        ensure_ascii=False,
        indent=2,
    )
)

print(json.dumps({"tools": tool_names, "call_payload": call_payload}, ensure_ascii=False))

expect_list_has_cron = os.environ["EXPECT_LIST_HAS_CRON"] == "1"
if ("cron_job" in tool_names) != expect_list_has_cron:
    print(
        f"[FAIL] expected cron_job visible={expect_list_has_cron}, got tools={tool_names}",
        file=sys.stderr,
    )
    sys.exit(1)

expect_call_mode = os.environ["EXPECT_CALL_MODE"]
text = ""
content = call_payload.get("content")
if isinstance(content, list) and content:
    text = content[0].get("text", "")

if expect_call_mode == "success":
    if call_payload.get("isError") is not False:
        print(f"[FAIL] expected cron_job call success, got {call_payload}", file=sys.stderr)
        sys.exit(1)
elif expect_call_mode == "missing":
    if "工具不存在: cron_job" not in text:
        print(f"[FAIL] expected missing-tool error, got {call_payload}", file=sys.stderr)
        sys.exit(1)
elif expect_call_mode == "restricted":
    if "tool `cron_job` is not allowed in this stage" not in text:
        print(f"[FAIL] expected stage restriction error, got {call_payload}", file=sys.stderr)
        sys.exit(1)
else:
    raise ValueError(f"unknown EXPECT_CALL_MODE={expect_call_mode}")

print(f"[PASS] case={os.environ['CASE_NAME']}")
PY
}

run_case "allow_cron_false" "0" "" "0" "missing"
run_case "allow_cron_true" "1" "" "1" "success"
run_case "allow_cron_true_restricted" "1" "discover_skills,skill_tool" "0" "restricted"

echo "[PASS] hone-mcp cron visibility regression passed"
