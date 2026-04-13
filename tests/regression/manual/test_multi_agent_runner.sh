#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/hone_multi_agent.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd opencode

cat >"$TMP_DIR/config.yaml" <<'EOF'
llm:
  provider: openrouter
  openrouter:
    api_key: ""
    model: "moonshotai/kimi-k2.5"
    sub_model: "moonshotai/kimi-k2.5"

storage:
  base_path: "./data"
  sessions_dir: "./data/sessions"
  session_sqlite_db_path: "./data/sessions.sqlite3"
  session_sqlite_shadow_write_enabled: false
  session_runtime_backend: "json"
  llm_audit_enabled: false
  llm_audit_db_path: "./data/llm_audit.sqlite3"
  llm_audit_retention_days: 7
  portfolio_dir: "./data/portfolio"
  cron_jobs_dir: "./data/cron_jobs"

logging:
  level: "INFO"
  console: true
  colorize: false
  enqueue: false
  file: "./data/logs/hone.log"
  rotation: "20 MB"
  retention: "7 days"
  compression: "zip"

search:
  api_keys: []

fmp:
  api_keys: []

agent:
  runner: "multi-agent"
  system_prompt: "You are Hone."
  opencode:
    command: "opencode"
    args: ["acp"]
    api_base_url: "https://openrouter.ai/api/v1"
    api_key: ""
    model: ""
    variant: ""
    startup_timeout_seconds: 15
    request_timeout_seconds: 300
  multi_agent:
    search:
      base_url: "https://api.minimaxi.com/v1"
      api_key: "REPLACE_ME_MINIMAX_KEY"
      model: "MiniMax-M2.7-highspeed"
      max_iterations: 8
    answer:
      api_base_url: "https://openrouter.ai/api/v1"
      api_key: "REPLACE_ME_OPENROUTER_KEY"
      model: "google/gemini-3.1-pro-preview"
      variant: "high"
      startup_timeout_seconds: 15
      request_timeout_seconds: 300
      max_tool_calls: 1

security:
  kb_actor_isolation: true
  tool_guard:
    enabled: true
    mode: "block"
    apply_tools:
      - "*"
      - "!web_search"
EOF

mkdir -p "$TMP_DIR/data"

echo "[INFO] Edit $TMP_DIR/config.yaml with valid MiniMax/OpenRouter keys before running this test."
echo "[INFO] Then execute from the temp directory:"
echo "       cd \"$TMP_DIR\" && cargo run --manifest-path \"$ROOT_DIR/Cargo.toml\" -q -p hone-cli"
echo "[INFO] Suggested prompts:"
echo "       hi"
echo "       查一下 Rocket Lab 的最新股价和一条相关新闻，然后用 HONE_MULTI_AGENT_OK 结尾。"
