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
require_cmd python3

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
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_codex_reload_cron.XXXXXX")"
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

permission_response_json() {
  python3 - "$1" <<'PY'
import json
import sys

try:
    payload = json.loads(sys.argv[1])
except json.JSONDecodeError:
    sys.exit(0)

if payload.get("method") != "session/request_permission":
    sys.exit(0)

request_id = payload.get("id")
params = payload.get("params") or {}
options = params.get("options") or []

selected = None
for kind in ("allow_always", "allow_once"):
    for option in options:
        if option.get("kind") == kind and option.get("optionId"):
            selected = option["optionId"]
            break
    if selected:
        break

if request_id is None or not selected:
    sys.exit(0)

print(
    json.dumps(
        {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "outcome": {
                    "outcome": "selected",
                    "optionId": selected,
                }
            },
        },
        ensure_ascii=False,
    )
)
PY
}

wait_for_pattern() {
  local pattern="$1"
  local timeout_seconds="$2"
  local deadline line response
  deadline=$((SECONDS + timeout_seconds))
  while ((SECONDS < deadline)); do
    if IFS= read -r -t 1 line <&4; then
      printf '%s\n' "$line" >>"$STDOUT_LOG"
      response="$(permission_response_json "$line" || true)"
      if [[ -n "$response" ]]; then
        send_jsonrpc "$response"
        echo "[INFO] auto-approved ACP permission request" >&2
      fi
      if [[ "$line" == *"$pattern"* ]]; then
        return 0
      fi
    fi
  done
  return 1
}

extract_session_id() {
  LC_ALL=C sed -n 's/.*"sessionId":"\([^"]*\)".*/\1/p' "$STDOUT_LOG" | tail -n 1
}

assistant_text_from_log() {
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
}

MCP_ENV_FOR() {
  local allow_cron="$1"
  cat <<EOF
[{"name":"HONE_CONFIG_PATH","value":"$ROOT_DIR/config.yaml"},{"name":"HONE_MCP_ACTOR_CHANNEL","value":"telegram"},{"name":"HONE_MCP_ACTOR_USER_ID","value":"8039067465"},{"name":"HONE_MCP_CHANNEL_TARGET","value":"telegram"},{"name":"HONE_MCP_ALLOW_CRON","value":"$allow_cron"},{"name":"HONE_DATA_DIR","value":"$WORK_DIR/data"}]
EOF
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
  cat "$STDOUT_LOG" >&2 || true
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

echo "[INFO] create session with cron disabled"
MCP_ENV_DISABLED="$(MCP_ENV_FOR 0)"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"session/new\",\"params\":{\"cwd\":\"$ROOT_DIR\",\"mcpServers\":[{\"name\":\"hone\",\"command\":\"$HONE_MCP_BIN\",\"args\":[],\"env\":$MCP_ENV_DISABLED}]}}"
if ! wait_for_pattern '"id":2,"result"' 45; then
  echo "[FAIL] codex acp session/new did not succeed" >&2
  cat "$STDOUT_LOG" >&2 || true
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

SESSION_ID="$(extract_session_id)"
if [[ -z "$SESSION_ID" ]]; then
  echo "[FAIL] could not extract sessionId from session/new response" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

PROMPT_DISABLED='Use the MCP tool hone/discover_skills to search for a scheduled task skill for recurring reminders. Do not invent tools. If no such skill is visible in this stage, end your reply with HONE_STAGE_DISABLED_OK.'
echo "[INFO] prompt disabled stage"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"session/prompt\",\"params\":{\"sessionId\":\"$SESSION_ID\",\"prompt\":[{\"type\":\"text\",\"text\":\"$PROMPT_DISABLED\"}]}}"
if ! wait_for_pattern '"id":3,"result":{"stopReason":"end_turn"' 90; then
  echo "[FAIL] disabled-stage prompt did not complete successfully" >&2
  cat "$STDOUT_LOG" >&2 || true
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

DISABLED_ASSISTANT_TEXT="$(assistant_text_from_log)"
if [[ "$DISABLED_ASSISTANT_TEXT" != *'HONE_STAGE_DISABLED_OK'* ]]; then
  echo "[FAIL] disabled stage final output missing token" >&2
  printf '%s\n' "$DISABLED_ASSISTANT_TEXT" >&2
  exit 1
fi

if ! grep -Eq '"title":"(Tool: hone/discover_skills|hone/discover_skills)"' "$STDOUT_LOG"; then
  echo "[FAIL] disabled stage did not emit hone/discover_skills tool call" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

echo "[INFO] reload same session with cron enabled"
MCP_ENV_ENABLED="$(MCP_ENV_FOR 1)"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"session/load\",\"params\":{\"sessionId\":\"$SESSION_ID\",\"cwd\":\"$ROOT_DIR\",\"mcpServers\":[{\"name\":\"hone\",\"command\":\"$HONE_MCP_BIN\",\"args\":[],\"env\":$MCP_ENV_ENABLED}]}}"
if ! wait_for_pattern '"id":4,"result"' 45; then
  echo "[FAIL] codex acp session/load did not succeed" >&2
  cat "$STDOUT_LOG" >&2 || true
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

PROMPT_ENABLED='Use the MCP tool hone/skill_tool to load the skill named scheduled_task. Then use the MCP tool hone/cron_job with action=list. Do not claim a tool is missing unless a real tool call fails. If both tool calls succeed, end your reply with HONE_STAGE_RELOADED_OK.'
echo "[INFO] prompt reloaded stage"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"session/prompt\",\"params\":{\"sessionId\":\"$SESSION_ID\",\"prompt\":[{\"type\":\"text\",\"text\":\"$PROMPT_ENABLED\"}]}}"
if ! wait_for_pattern '"id":5,"result":{"stopReason":"end_turn"' 120; then
  echo "[FAIL] reloaded-stage prompt did not complete successfully" >&2
  cat "$STDOUT_LOG" >&2 || true
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

if ! grep -Eq '"title":"(Tool: hone/skill_tool|hone/skill_tool)"' "$STDOUT_LOG"; then
  echo "[FAIL] reloaded stage did not emit hone/skill_tool tool call" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

if ! grep -Eq '"title":"(Tool: hone/cron_job|hone/cron_job)"' "$STDOUT_LOG"; then
  echo "[FAIL] reloaded stage did not emit hone/cron_job tool call" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

RELOADED_ASSISTANT_TEXT="$(assistant_text_from_log)"
if [[ "$RELOADED_ASSISTANT_TEXT" != *'HONE_STAGE_RELOADED_OK'* ]]; then
  echo "[FAIL] reloaded stage final output missing token" >&2
  printf '%s\n' "$RELOADED_ASSISTANT_TEXT" >&2
  exit 1
fi

echo "[PASS] codex_acp session/load refreshed cron visibility"
