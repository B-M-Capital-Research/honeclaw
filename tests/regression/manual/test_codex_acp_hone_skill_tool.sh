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

require_cmd codex
require_cmd codex-acp
require_cmd cargo

TMP_ITEMS=()
cleanup() {
  if [[ -n "${ACP_PID:-}" ]]; then
    kill "$ACP_PID" 2>/dev/null || true
    wait "$ACP_PID" 2>/dev/null || true
  fi
  if ((${#TMP_ITEMS[@]} > 0)); then
    rm -rf "${TMP_ITEMS[@]}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

make_tmp_dir() {
  local dir
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_codex_skill.XXXXXX")"
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

send_jsonrpc() {
  printf '%s\n' "$1" >&3
}

wait_for_pattern() {
  local pattern="$1"
  local timeout_seconds="$2"
  local deadline line
  deadline=$((SECONDS + timeout_seconds))
  while ((SECONDS < deadline)); do
    if IFS= read -r -t 1 line <&4; then
      printf '%s\n' "$line" >>"$STDOUT_LOG"
      if [[ "$line" == *"$pattern"* ]]; then
        return 0
      fi
    fi
  done
  return 1
}

HONE_MCP_BIN="$(ensure_hone_mcp_binary)"
WORK_DIR="$(make_tmp_dir)"
IN_PIPE="$WORK_DIR/in.pipe"
OUT_PIPE="$WORK_DIR/out.pipe"
STDOUT_LOG="$WORK_DIR/codex_acp.stdout.log"
STDERR_LOG="$WORK_DIR/codex_acp.stderr.log"
mkfifo "$IN_PIPE" "$OUT_PIPE"
touch "$STDOUT_LOG" "$STDERR_LOG"

echo "[INFO] starting codex ACP"
codex-acp \
  -c 'sandbox_mode="workspace-write"' \
  -c 'approval_policy="never"' \
  <"$IN_PIPE" >"$OUT_PIPE" 2>"$STDERR_LOG" &
ACP_PID=$!

exec 3>"$IN_PIPE"
exec 4<"$OUT_PIPE"

sleep 2

echo "[INFO] initialize ACP session"
send_jsonrpc '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{}}}'
if ! wait_for_pattern '"id":1,"result"' 30; then
  echo "[FAIL] codex acp initialize did not return a result" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

MCP_ENV=$(cat <<EOF
[{"name":"HONE_CONFIG_PATH","value":"$ROOT_DIR/data/runtime/config_runtime.yaml"},{"name":"HONE_MCP_ACTOR_CHANNEL","value":"cli"},{"name":"HONE_MCP_ACTOR_USER_ID","value":"cli_user"},{"name":"HONE_MCP_CHANNEL_TARGET","value":"cli"},{"name":"HONE_MCP_ALLOW_CRON","value":"0"},{"name":"HONE_DATA_DIR","value":"$WORK_DIR/data"}]
EOF
)

echo "[INFO] creating codex ACP session with Hone MCP bridge"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"session/new\",\"params\":{\"cwd\":\"$ROOT_DIR\",\"mcpServers\":[{\"name\":\"hone\",\"command\":\"$HONE_MCP_BIN\",\"args\":[],\"env\":$MCP_ENV}]}}"
if ! wait_for_pattern '"id":2,"result"' 45; then
  echo "[FAIL] codex acp session/new did not succeed" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

SESSION_ID="$(LC_ALL=C sed -n 's/.*"id":2,"result":{"sessionId":"\([^"]*\)".*/\1/p' "$STDOUT_LOG" | tail -n 1)"
if [[ -z "$SESSION_ID" ]]; then
  echo "[FAIL] could not extract sessionId from session/new response" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

PROMPT='Use the MCP tool named hone/skill_tool to execute the Hone skill named skill_manager. Pass the argument skill_name=skill_manager. Do not use any other tool. End your final answer with HONE_CODEX_ACP_SKILL_TOOL_OK.'
echo "[INFO] prompting codex ACP to call hone skill_tool"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"session/prompt\",\"params\":{\"sessionId\":\"$SESSION_ID\",\"prompt\":[{\"type\":\"text\",\"text\":\"$PROMPT\"}]}}"

if ! wait_for_pattern '"id":3,"result":{"stopReason":"end_turn"' 90; then
  echo "[FAIL] codex acp prompt did not complete successfully" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

if ! grep -Eq '"title":"(Tool: hone/skill_tool|hone/skill_tool)"' "$STDOUT_LOG"; then
  echo "[FAIL] codex acp did not emit hone/skill_tool tool_call" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

ASSISTANT_TEXT="$(
  python3 - "$STDOUT_LOG" <<'PY'
import json
import sys
from pathlib import Path

chunks = []
for line in Path(sys.argv[1]).read_text().splitlines():
    line = line.strip()
    if not line:
        continue
    try:
        payload = json.loads(line)
    except json.JSONDecodeError:
        continue
    update = payload.get("params", {}).get("update", {})
    if update.get("sessionUpdate") == "agent_message_chunk":
        text = update.get("content", {}).get("text")
        if isinstance(text, str):
            chunks.append(text)

print("".join(chunks))
PY
)"

if [[ "$ASSISTANT_TEXT" != *'HONE_CODEX_ACP_SKILL_TOOL_OK'* ]]; then
  echo "[FAIL] codex acp final output missing expected token" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

echo "[PASS] codex_acp Hone skill_tool e2e passed"
