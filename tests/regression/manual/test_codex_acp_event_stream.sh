#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

CONFIG_PATH="${HONE_CONFIG_PATH:-$ROOT_DIR/config.yaml}"
CHANNEL="${HONE_MCP_ACTOR_CHANNEL:-telegram}"
ACTOR_USER_ID="${HONE_MCP_ACTOR_USER_ID:-acp_probe_user}"
CHANNEL_SCOPE="${HONE_MCP_ACTOR_SCOPE:-chat:-1009000000000}"
CHANNEL_TARGET="${HONE_MCP_CHANNEL_TARGET:-chat:-1009000000000}"
ALLOW_CRON="${HONE_MCP_ALLOW_CRON:-0}"
PROMPT="${HONE_ACP_PROBE_PROMPT:-请先用一句中文说明你接下来要做什么，然后立刻调用 MCP 工具 hone/skill_tool 执行 Hone skill skill_manager，参数 skill_name=skill_manager。工具完成后，最终回答只输出 ACP_EVENT_PROBE_OK。}"
SHOW_RAW="${HONE_ACP_PROBE_SHOW_RAW:-0}"

usage() {
  cat <<'EOF'
Usage:
  tests/regression/manual/test_codex_acp_event_stream.sh [options]

Options:
  --prompt TEXT         Override the prompt sent to session/prompt.
  --channel NAME        MCP actor channel. Default: telegram
  --user-id ID          MCP actor user id. Default: acp_probe_user
  --scope SCOPE         MCP actor scope. Default: chat:-1009000000000
  --target TARGET       MCP channel target. Default: chat:-1009000000000
  --config PATH         Config path. Default: config.yaml
  --show-raw            Print the raw JSONL session/update log after the summary.
  -h, --help            Show this help.

Environment overrides:
  HONE_CONFIG_PATH
  HONE_MCP_ACTOR_CHANNEL
  HONE_MCP_ACTOR_USER_ID
  HONE_MCP_ACTOR_SCOPE
  HONE_MCP_CHANNEL_TARGET
  HONE_MCP_ALLOW_CRON
  HONE_ACP_PROBE_PROMPT
  HONE_ACP_PROBE_SHOW_RAW=1
EOF
}

while (($# > 0)); do
  case "$1" in
    --prompt)
      shift
      [[ $# -gt 0 ]] || { echo "[FAIL] --prompt requires a value" >&2; exit 1; }
      PROMPT="$1"
      ;;
    --channel)
      shift
      [[ $# -gt 0 ]] || { echo "[FAIL] --channel requires a value" >&2; exit 1; }
      CHANNEL="$1"
      ;;
    --user-id)
      shift
      [[ $# -gt 0 ]] || { echo "[FAIL] --user-id requires a value" >&2; exit 1; }
      ACTOR_USER_ID="$1"
      ;;
    --scope)
      shift
      [[ $# -gt 0 ]] || { echo "[FAIL] --scope requires a value" >&2; exit 1; }
      CHANNEL_SCOPE="$1"
      ;;
    --target)
      shift
      [[ $# -gt 0 ]] || { echo "[FAIL] --target requires a value" >&2; exit 1; }
      CHANNEL_TARGET="$1"
      ;;
    --config)
      shift
      [[ $# -gt 0 ]] || { echo "[FAIL] --config requires a value" >&2; exit 1; }
      CONFIG_PATH="$1"
      ;;
    --show-raw)
      SHOW_RAW="1"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "[FAIL] unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_cmd python3
require_cmd cargo

agent_scalar() {
  local key="$1"
  awk -v wanted="$key" '
    /^agent:/ { in_agent=1; next }
    in_agent && /^[^[:space:]]/ { exit }
    in_agent && $0 ~ ("^  " wanted ":") {
      sub("^  " wanted ":[[:space:]]*", "", $0)
      sub(/[[:space:]]+#.*$/, "", $0)
      gsub(/^"/, "", $0)
      gsub(/"$/, "", $0)
      print
      exit
    }
  ' "$CONFIG_PATH"
}

codex_scalar() {
  local key="$1"
  awk -v wanted="$key" '
    /^agent:/ { in_agent=1; next }
    in_agent && /^[^[:space:]]/ { exit }
    in_agent && /^  codex_acp:/ { in_codex=1; next }
    in_codex && /^  [^[:space:]][^:]*:/ { exit }
    in_codex && $0 ~ ("^    " wanted ":") {
      sub("^    " wanted ":[[:space:]]*", "", $0)
      sub(/[[:space:]]+#.*$/, "", $0)
      gsub(/^"/, "", $0)
      gsub(/"$/, "", $0)
      print
      exit
    }
  ' "$CONFIG_PATH"
}

[[ -f "$CONFIG_PATH" ]] || { echo "[FAIL] missing config: $CONFIG_PATH" >&2; exit 1; }

RUNNER="$(agent_scalar runner)"
[[ "$RUNNER" == "codex_acp" ]] || {
  echo "[FAIL] current runtime runner is '$RUNNER'; this probe currently supports codex_acp only." >&2
  exit 1
}

CODEX_ACP_COMMAND="$(codex_scalar command)"
CODEX_COMMAND="$(codex_scalar codex_command)"
MODEL="$(codex_scalar model)"
VARIANT="$(codex_scalar variant)"

[[ -n "$CODEX_ACP_COMMAND" ]] || { echo "[FAIL] could not read agent.codex_acp.command from $CONFIG_PATH" >&2; exit 1; }
[[ -n "$CODEX_COMMAND" ]] || { echo "[FAIL] could not read agent.codex_acp.codex_command from $CONFIG_PATH" >&2; exit 1; }

require_cmd "$CODEX_COMMAND"
require_cmd "$CODEX_ACP_COMMAND"

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
  dir="$(mktemp -d "${TMPDIR:-/tmp}/hone_codex_event_probe.XXXXXX")"
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

prompt_payload_json() {
  python3 - "$PROMPT" <<'PY'
import json
import sys

print(json.dumps([{"type": "text", "text": sys.argv[1]}], ensure_ascii=False))
PY
}

HONE_MCP_BIN="$(ensure_hone_mcp_binary)"
WORK_DIR="$(make_tmp_dir)"
IN_PIPE="$WORK_DIR/in.pipe"
OUT_PIPE="$WORK_DIR/out.pipe"
STDOUT_LOG="$WORK_DIR/codex_acp.stdout.jsonl"
STDERR_LOG="$WORK_DIR/codex_acp.stderr.log"
SUMMARY_LOG="$WORK_DIR/summary.txt"
PROBE_DATA_DIR="$WORK_DIR/data"
mkfifo "$IN_PIPE" "$OUT_PIPE"
touch "$STDOUT_LOG" "$STDERR_LOG"

MODEL_ID="$MODEL"
if [[ -n "$MODEL" && -n "$VARIANT" && "$MODEL" != */"$VARIANT" ]]; then
  MODEL_ID="${MODEL}/${VARIANT}"
fi

MCP_ENV="$(python3 - "$CONFIG_PATH" "$CHANNEL" "$ACTOR_USER_ID" "$CHANNEL_TARGET" "$CHANNEL_SCOPE" "$ALLOW_CRON" "$PROBE_DATA_DIR" <<'PY'
import json
import sys

config_path, channel, user_id, target, scope, allow_cron, data_dir = sys.argv[1:]
entries = [
    {"name": "HONE_CONFIG_PATH", "value": config_path},
    {"name": "HONE_MCP_ACTOR_CHANNEL", "value": channel},
    {"name": "HONE_MCP_ACTOR_USER_ID", "value": user_id},
    {"name": "HONE_MCP_CHANNEL_TARGET", "value": target},
    {"name": "HONE_MCP_ACTOR_SCOPE", "value": scope},
    {"name": "HONE_MCP_ALLOW_CRON", "value": allow_cron},
    {"name": "HONE_MCP_SESSION_ID", "value": "ACP_EVENT_PROBE_SESSION"},
    {"name": "HONE_DATA_DIR", "value": data_dir},
]
print(json.dumps(entries, ensure_ascii=False))
PY
)"

PROMPT_JSON="$(prompt_payload_json)"

echo "[INFO] config: $CONFIG_PATH"
echo "[INFO] runner: $RUNNER"
echo "[INFO] codex command: $CODEX_COMMAND"
echo "[INFO] codex-acp command: $CODEX_ACP_COMMAND"
echo "[INFO] model id: ${MODEL_ID:-<default>}"
echo "[INFO] actor: channel=$CHANNEL user_id=$ACTOR_USER_ID scope=$CHANNEL_SCOPE target=$CHANNEL_TARGET"
echo "[INFO] probe workspace: $WORK_DIR"

echo "[INFO] starting codex ACP"
"$CODEX_ACP_COMMAND" \
  -c 'sandbox_mode="workspace-write"' \
  -c 'approval_policy="never"' \
  <"$IN_PIPE" >"$OUT_PIPE" 2>"$STDERR_LOG" &
ACP_PID=$!

exec 3>"$IN_PIPE"
exec 4<"$OUT_PIPE"

sleep 1

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

if [[ -n "$MODEL_ID" ]]; then
  echo "[INFO] setting codex ACP model: $MODEL_ID"
  send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"session/set_model\",\"params\":{\"sessionId\":\"$SESSION_ID\",\"modelId\":\"$MODEL_ID\"}}"
  if ! wait_for_pattern '"id":3,"result"' 30; then
    echo "[FAIL] codex acp session/set_model did not succeed" >&2
    echo "--- stdout ---" >&2
    cat "$STDOUT_LOG" >&2 || true
    echo "--- stderr ---" >&2
    cat "$STDERR_LOG" >&2 || true
    exit 1
  fi
fi

echo "[INFO] prompting codex ACP to emit a pre-tool chunk, call hone/skill_tool, and finish cleanly"
send_jsonrpc "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"session/prompt\",\"params\":{\"sessionId\":\"$SESSION_ID\",\"prompt\":$PROMPT_JSON}}"

if ! wait_for_pattern '"id":4,"result"' 180; then
  echo "[FAIL] codex acp prompt did not complete successfully" >&2
  echo "--- stdout ---" >&2
  cat "$STDOUT_LOG" >&2 || true
  echo "--- stderr ---" >&2
  cat "$STDERR_LOG" >&2 || true
  exit 1
fi

python3 - "$STDOUT_LOG" "$SUMMARY_LOG" <<'PY'
import json
import sys
from pathlib import Path

log_path = Path(sys.argv[1])
summary_path = Path(sys.argv[2])

event_sequence = []
assistant_chunks = []
tool_starts = []
tool_done = []
tool_failed = []
prompt_result = None

for raw in log_path.read_text().splitlines():
    raw = raw.strip()
    if not raw:
        continue
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError:
        continue

    if payload.get("id") == 4 and isinstance(payload.get("result"), dict):
        prompt_result = payload["result"]

    update = payload.get("params", {}).get("update")
    if not isinstance(update, dict):
        continue

    kind = update.get("sessionUpdate")
    if kind == "agent_message_chunk":
        text = None
        content = update.get("content")
        if isinstance(content, dict):
            text = content.get("text")
        if not isinstance(text, str):
            text = update.get("text") or update.get("delta")
        if isinstance(text, str):
            assistant_chunks.append(text)
        event_sequence.append("agent_message_chunk")
    elif kind == "tool_call":
        title = update.get("title") or update.get("kind") or "tool"
        event_sequence.append(f"tool_call:{title}")
        tool_starts.append(title)
    elif kind == "tool_call_update":
        title = update.get("title") or update.get("kind") or "tool"
        status = update.get("status") or "unknown"
        event_sequence.append(f"tool_call_update:{status}:{title}")
        if status == "completed":
            tool_done.append(title)
        elif status == "failed":
            tool_failed.append(title)
    elif kind:
        event_sequence.append(kind)

assistant_text = "".join(assistant_chunks)
stop_reason = None
if isinstance(prompt_result, dict):
    stop_reason = prompt_result.get("stopReason")

pre_tool_chunk = False
first_tool_index = next((i for i, item in enumerate(event_sequence) if item.startswith("tool_call:")), None)
first_chunk_index = next((i for i, item in enumerate(event_sequence) if item == "agent_message_chunk"), None)
if first_tool_index is not None and first_chunk_index is not None and first_chunk_index < first_tool_index:
    pre_tool_chunk = True

lines = [
    "=== ACP Event Probe Summary ===",
    f"log_path={log_path}",
    f"stop_reason={stop_reason}",
    f"assistant_chunk_count={len(assistant_chunks)}",
    f"tool_call_count={len(tool_starts)}",
    f"tool_completed_count={len(tool_done)}",
    f"tool_failed_count={len(tool_failed)}",
    f"pre_tool_chunk_seen={str(pre_tool_chunk).lower()}",
    f"final_token_present={'ACP_EVENT_PROBE_OK' in assistant_text}",
    "",
    "event_sequence:",
]
for item in event_sequence:
    lines.append(f"- {item}")

lines.extend(
    [
        "",
        "assistant_text:",
        assistant_text if assistant_text else "<empty>",
        "",
        "what_success_looks_like:",
        "1. session/new returns a sessionId.",
        "2. optional session/set_model returns result.",
        "3. session/update notifications may stream agent_message_chunk and tool_call/tool_call_update events.",
        "4. the terminal success marker is the id=4 session/prompt result with stopReason=end_turn.",
        "5. any earlier agent_message_chunk is only intermediate content, not the terminal result by itself.",
    ]
)

summary_path.write_text("\n".join(lines) + "\n")
print(summary_path.read_text(), end="")

if stop_reason != "end_turn":
    print("[FAIL] session/prompt did not finish with stopReason=end_turn", file=sys.stderr)
    sys.exit(1)
if "ACP_EVENT_PROBE_OK" not in assistant_text:
    print("[FAIL] assistant chunks did not contain ACP_EVENT_PROBE_OK", file=sys.stderr)
    sys.exit(1)
PY

echo "[INFO] stderr log: $STDERR_LOG"
echo "[INFO] raw stdout log: $STDOUT_LOG"

if [[ "$SHOW_RAW" == "1" ]]; then
  echo
  echo "=== Raw JSONL ==="
  cat "$STDOUT_LOG"
fi

echo "[PASS] codex_acp event-stream probe passed"
