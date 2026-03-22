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

require_cmd opencode
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
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_opencode_acp.XXXXXX")"
  TMP_ITEMS+=("$dir")
  printf '%s\n' "$dir"
}

ensure_hone_mcp_binary() {
  local bin_path="$ROOT_DIR/target/debug/hone-mcp"
  if [[ ! -x "$bin_path" ]]; then
    echo "[INFO] building hone-mcp binary"
    cargo build -p hone-mcp >/dev/null
  fi
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
  local deadline shell_now
  deadline=$((SECONDS + timeout_seconds))
  while (( SECONDS < deadline )); do
    if IFS= read -r -t 1 line <&4; then
      printf '%s\n' "$line" >>"$STDOUT_LOG"
      if [[ "$line" == *"$pattern"* ]]; then
        return 0
      fi
    fi
  done
  return 1
}

wait_for_any_pattern() {
  local timeout_seconds="$1"
  shift
  local deadline pattern line
  deadline=$((SECONDS + timeout_seconds))
  while (( SECONDS < deadline )); do
    if IFS= read -r -t 1 line <&4; then
      printf '%s\n' "$line" >>"$STDOUT_LOG"
      for pattern in "$@"; do
        if [[ "$line" == *"$pattern"* ]]; then
          return 0
        fi
      done
    fi
  done
  return 1
}

HONE_MCP_BIN="$(ensure_hone_mcp_binary)"
WORK_DIR="$(make_tmp_dir)"
IN_PIPE="$WORK_DIR/in.pipe"
OUT_PIPE="$WORK_DIR/out.pipe"
STDOUT_LOG="$WORK_DIR/opencode_acp.stdout.log"
STDERR_LOG="$WORK_DIR/opencode_acp.stderr.log"
mkfifo "$IN_PIPE" "$OUT_PIPE"

echo "[INFO] starting opencode acp"
opencode acp --cwd "$ROOT_DIR" --print-logs <"$IN_PIPE" >"$OUT_PIPE" 2>"$STDERR_LOG" &
ACP_PID=$!

exec 3>"$IN_PIPE"
exec 4<"$OUT_PIPE"

echo "[INFO] initialize ACP session"
send_jsonrpc '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{}}}'
if ! wait_for_pattern '"id":1,"result"' 20; then
  echo "[FAIL] opencode acp initialize did not return a result" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

MCP_ENV=$(cat <<EOF
[{"name":"HONE_CONFIG_PATH","value":"$ROOT_DIR/data/runtime/config_runtime.yaml"},{"name":"HONE_MCP_ACTOR_CHANNEL","value":"cli"},{"name":"HONE_MCP_ACTOR_USER_ID","value":"cli_user"},{"name":"HONE_MCP_CHANNEL_TARGET","value":"cli"},{"name":"HONE_MCP_ALLOW_CRON","value":"0"}]
EOF
)

echo "[INFO] creating opencode ACP session with Hone MCP bridge"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"session/new\",\"params\":{\"cwd\":\"$ROOT_DIR\",\"mcpServers\":[{\"name\":\"hone\",\"command\":\"$HONE_MCP_BIN\",\"args\":[],\"env\":$MCP_ENV}]}}"
if ! wait_for_pattern '"id":2,"result"' 30; then
  echo "[FAIL] opencode acp session/new did not succeed" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

if ! grep -q 'toolCount=8 create() successfully created client' "$STDERR_LOG"; then
  echo "[FAIL] opencode stderr missing Hone MCP registration evidence" >&2
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

SESSION_ID="$(LC_ALL=C sed -n 's/.*"id":2,"result":{"sessionId":"\([^"]*\)".*/\1/p' "$STDOUT_LOG" | tail -n 1)"
if [[ -z "$SESSION_ID" ]]; then
  echo "[FAIL] could not extract sessionId from session/new response" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

PROMPT='Use the hone_kb_search tool with action=search and query=Rocket Lab. Do not use any other search tool. Include the exact token HONE_OPENCODE_ACP_MCP_OK in the final answer.'
echo "[INFO] prompting opencode ACP to call Hone tool"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"session/prompt\",\"params\":{\"sessionId\":\"$SESSION_ID\",\"prompt\":[{\"type\":\"text\",\"text\":\"$PROMPT\"}]}}"

if ! wait_for_pattern '"id":3,"result":{"stopReason":"end_turn"' 60; then
  echo "[FAIL] opencode acp prompt did not complete successfully" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

if ! grep -Eq '"title":"(Tool: hone/kb_search|hone_kb_search)"' "$STDOUT_LOG"; then
  echo "[FAIL] opencode acp did not emit hone/kb_search tool_call" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

ASSISTANT_TEXT="$(
  perl -ne '
    while (/"sessionUpdate":"agent_message_chunk".*"text":"((?:\\.|[^"])*)"/g) {
      my $text = $1;
      $text =~ s/\\"/"/g;
      $text =~ s/\\\\/\\/g;
      $text =~ s/\\n/\n/g;
      $text =~ s/\\r/\r/g;
      $text =~ s/\\t/\t/g;
      print $text;
    }
  ' "$STDOUT_LOG"
)"

if [[ "$ASSISTANT_TEXT" != *'HONE_OPENCODE_ACP_MCP_OK'* ]]; then
  echo "[FAIL] opencode acp final output missing expected token" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  exit 1
fi

echo "[PASS] opencode_acp Hone MCP e2e passed"
