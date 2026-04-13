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
require_cmd curl
require_cmd perl

TMP_ITEMS=()
cleanup() {
  if [[ -n "${ACP_PID:-}" ]]; then
    kill "$ACP_PID" 2>/dev/null || true
    wait "$ACP_PID" 2>/dev/null || true
  fi
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  if ((${#TMP_ITEMS[@]} > 0)); then
    rm -rf "${TMP_ITEMS[@]}" 2>/dev/null || true
  fi
}
trap cleanup EXIT

make_tmp_dir() {
  local dir
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_opencode_skill_toggle.XXXXXX")"
  TMP_ITEMS+=("$dir")
  printf '%s\n' "$dir"
}

ensure_binary() {
  local package="$1"
  local bin_name="$2"
  local path="$ROOT_DIR/target/debug/$bin_name"
  if [[ ! -x "$path" ]]; then
    echo "[INFO] building $package" >&2
    cargo build -p "$package" >/dev/null
  fi
  if [[ ! -x "$path" ]]; then
    echo "[FAIL] missing binary: $path" >&2
    exit 1
  fi
  printf '%s\n' "$path"
}

wait_for_http_ok() {
  local url="$1"
  local timeout_seconds="$2"
  local deadline=$((SECONDS + timeout_seconds))
  while ((SECONDS < deadline)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

send_jsonrpc() {
  printf '%s\n' "$1" >&3
}

wait_for_pattern() {
  local pattern="$1"
  local timeout_seconds="$2"
  local deadline=$((SECONDS + timeout_seconds))
  local line
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

assistant_text_from_log() {
  local path="$1"
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
  ' "$path"
}

start_console() {
  local config_path="$1"
  local data_dir="$2"
  local port="$3"
  local console_bin="$4"
  local log_path="$5"

  HONE_CONFIG_PATH="$config_path" \
  HONE_DATA_DIR="$data_dir" \
  HONE_WEB_PORT="$port" \
  HONE_DISABLE_AUTO_OPEN=1 \
  "$console_bin" >"$log_path" 2>&1 &
  SERVER_PID=$!
}

run_opencode_case() {
  local case_name="$1"
  local config_path="$2"
  local data_dir="$3"
  local prompt="$4"
  local expected_token="$5"
  local expected_tool_pattern="$6"
  local expected_log_snippet="$7"
  local hone_mcp_bin="$8"

  local case_dir in_pipe out_pipe
  case_dir="$(make_tmp_dir)"
  in_pipe="$case_dir/in.pipe"
  out_pipe="$case_dir/out.pipe"
  STDOUT_LOG="$case_dir/$case_name.stdout.log"
  STDERR_LOG="$case_dir/$case_name.stderr.log"
  mkfifo "$in_pipe" "$out_pipe"

  opencode acp --cwd "$ROOT_DIR" --print-logs <"$in_pipe" >"$out_pipe" 2>"$STDERR_LOG" &
  ACP_PID=$!

  exec 3>"$in_pipe"
  exec 4<"$out_pipe"

  send_jsonrpc '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{}}}'
  if ! wait_for_pattern '"id":1,"result"' 20; then
    echo "[FAIL] case=$case_name initialize timeout" >&2
    cat "$STDERR_LOG" >&2 || true
    exit 1
  fi

  local mcp_env
  mcp_env=$(cat <<EOF
[{"name":"HONE_CONFIG_PATH","value":"$config_path"},{"name":"HONE_DATA_DIR","value":"$data_dir"},{"name":"HONE_MCP_ACTOR_CHANNEL","value":"cli"},{"name":"HONE_MCP_ACTOR_USER_ID","value":"cli_user"},{"name":"HONE_MCP_CHANNEL_TARGET","value":"cli"},{"name":"HONE_MCP_ALLOW_CRON","value":"0"}]
EOF
)

  send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"session/new\",\"params\":{\"cwd\":\"$ROOT_DIR\",\"mcpServers\":[{\"name\":\"hone\",\"command\":\"$hone_mcp_bin\",\"args\":[],\"env\":$mcp_env}]}}"
  if ! wait_for_pattern '"id":2,"result"' 30; then
    echo "[FAIL] case=$case_name session/new timeout" >&2
    cat "$STDERR_LOG" >&2 || true
    exit 1
  fi

  local session_id
  session_id="$(LC_ALL=C sed -n 's/.*"id":2,"result":{"sessionId":"\([^"]*\)".*/\1/p' "$STDOUT_LOG" | tail -n 1)"
  if [[ -z "$session_id" ]]; then
    echo "[FAIL] case=$case_name missing session id" >&2
    cat "$STDOUT_LOG" >&2 || true
    exit 1
  fi

  send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"session/prompt\",\"params\":{\"sessionId\":\"$session_id\",\"prompt\":[{\"type\":\"text\",\"text\":\"$prompt\"}]}}"
  if ! wait_for_pattern '"id":3,"result":{"stopReason":"end_turn"' 90; then
    echo "[FAIL] case=$case_name prompt timeout" >&2
    cat "$STDOUT_LOG" >&2 || true
    cat "$STDERR_LOG" >&2 || true
    exit 1
  fi

  if ! grep -Eq "$expected_tool_pattern" "$STDOUT_LOG"; then
    echo "[FAIL] case=$case_name missing expected tool call" >&2
    cat "$STDOUT_LOG" >&2 || true
    exit 1
  fi

  if [[ -n "$expected_log_snippet" ]] && ! grep -q "$expected_log_snippet" "$STDOUT_LOG"; then
    echo "[FAIL] case=$case_name missing expected log snippet: $expected_log_snippet" >&2
    cat "$STDOUT_LOG" >&2 || true
    exit 1
  fi

  local assistant_text
  assistant_text="$(assistant_text_from_log "$STDOUT_LOG")"
  if [[ "$assistant_text" != *"$expected_token"* ]]; then
    echo "[FAIL] case=$case_name missing token: $expected_token" >&2
    echo "--- assistant ---" >&2
    printf '%s\n' "$assistant_text" >&2
    echo "--- stdout ---" >&2
    cat "$STDOUT_LOG" >&2 || true
    exit 1
  fi

  exec 3>&-
  exec 4<&-
  kill "$ACP_PID" 2>/dev/null || true
  wait "$ACP_PID" 2>/dev/null || true
  unset ACP_PID
}

BASE_CONFIG="$ROOT_DIR/config.yaml"
if [[ ! -f "$BASE_CONFIG" ]]; then
  BASE_CONFIG="$ROOT_DIR/config.example.yaml"
fi
if [[ ! -f "$BASE_CONFIG" ]]; then
  echo "[FAIL] missing base config.yaml/config.example.yaml" >&2
  exit 1
fi

HONE_MCP_BIN="$(ensure_binary hone-mcp hone-mcp)"
CONSOLE_BIN="$(ensure_binary hone-console-page hone-console-page)"
WORK_DIR="$(make_tmp_dir)"
DATA_DIR="$WORK_DIR/data"
CONFIG_PATH="$WORK_DIR/config.yaml"
OVERRIDE_PATH="$WORK_DIR/config.overrides.yaml"
CONSOLE_LOG="$WORK_DIR/hone-console-page.log"
PORT="$((20000 + RANDOM % 10000))"
API_BASE="http://127.0.0.1:$PORT/api"

mkdir -p "$DATA_DIR"
cp "$BASE_CONFIG" "$CONFIG_PATH"
cat >"$OVERRIDE_PATH" <<EOF
skills_dir: "$ROOT_DIR/skills"
agent:
  runner: "opencode_acp"
  system_prompt_path: "$ROOT_DIR/soul.md"
storage:
  sessions_dir: "$DATA_DIR/sessions"
  session_sqlite_db_path: "$DATA_DIR/sessions.sqlite3"
  session_sqlite_shadow_write_enabled: false
  session_runtime_backend: "json"
  conversation_quota_dir: "$DATA_DIR/conversation_quota"
  llm_audit_db_path: "$DATA_DIR/llm_audit.sqlite3"
  llm_audit_enabled: false
  portfolio_dir: "$DATA_DIR/portfolio"
  cron_jobs_dir: "$DATA_DIR/cron_jobs"
  gen_images_dir: "$DATA_DIR/gen_images"
web:
  auth_token: ""
logging:
  console: false
EOF

echo "[INFO] starting hone-console-page on port $PORT"
start_console "$CONFIG_PATH" "$DATA_DIR" "$PORT" "$CONSOLE_BIN" "$CONSOLE_LOG"
if ! wait_for_http_ok "$API_BASE/meta" 30; then
  echo "[FAIL] hone-console-page did not start" >&2
  cat "$CONSOLE_LOG" >&2 || true
  exit 1
fi

echo "[INFO] disabling skill_manager via Web API"
disable_response="$(curl -fsS -X PATCH "$API_BASE/skills/skill_manager/state" -H 'Content-Type: application/json' -d '{"enabled":false}')"
if [[ "$disable_response" != *'"enabled":false'* ]]; then
  echo "[FAIL] disable response missing enabled=false" >&2
  printf '%s\n' "$disable_response" >&2
  exit 1
fi

run_opencode_case \
  "disabled" \
  "$CONFIG_PATH" \
  "$DATA_DIR" \
  "Use the MCP tool hone/skill_tool to load the skill named skill_manager. Do not skip the tool call. If the tool reports that the skill is disabled, briefly say so and include the exact token HONE_SKILL_DISABLED_OK." \
  "HONE_SKILL_DISABLED_OK" \
  '"title":"(Tool: hone/skill_tool|hone/skill_tool|hone_skill_tool)"' \
  "已被管理员禁用" \
  "$HONE_MCP_BIN"

echo "[INFO] enabling skill_manager via Web API"
enable_response="$(curl -fsS -X PATCH "$API_BASE/skills/skill_manager/state" -H 'Content-Type: application/json' -d '{"enabled":true}')"
if [[ "$enable_response" != *'"enabled":true'* ]]; then
  echo "[FAIL] enable response missing enabled=true" >&2
  printf '%s\n' "$enable_response" >&2
  exit 1
fi

run_opencode_case \
  "enabled" \
  "$CONFIG_PATH" \
  "$DATA_DIR" \
  "Use the MCP tool hone/skill_tool to load the skill named skill_manager. Do not skip the tool call. After the tool succeeds, reply with the exact token HONE_SKILL_ENABLED_OK." \
  "HONE_SKILL_ENABLED_OK" \
  '"title":"(Tool: hone/skill_tool|hone/skill_tool|hone_skill_tool)"' \
  "" \
  "$HONE_MCP_BIN"

echo "[PASS] opencode_acp skill toggle e2e passed"
