#!/usr/bin/env bash
set -euo pipefail

# Hone backend launcher (breaking change)
#
# Usage:
#   ./launch.sh                    # start main backend + channel listeners
#   ./launch.sh --web              # start backend + channel listeners + Vite frontend
#   ./launch.sh --desktop          # start Tauri desktop dev (desktop manages bundled backend + channels)
#   ./launch.sh --desktop --remote # start backend + channel listeners + Tauri dev (desktop connects to remote backend)
#   ./launch.sh --release          # start desktop in release mode (no Rust hot reload)
#   ./launch.sh --desktop --all    # same as above, but rebuild frontend dist first
#   ./launch.sh stop               # stop launched processes

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
cd "$PROJECT_ROOT" || exit 1

TARGET_DIR="${CARGO_TARGET_DIR:-$PROJECT_ROOT/target}"
default_target_dir() {
  if [[ "$(uname -s)" == "Darwin" ]]; then
    echo "$HOME/Library/Caches/hone-financial/target"
  else
    echo "${XDG_CACHE_HOME:-$HOME/.cache}/hone-financial/target"
  fi
}

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(default_target_dir)}"
TARGET_DIR="$CARGO_TARGET_DIR"
case "$TARGET_DIR" in
  /*) ;;
  *) TARGET_DIR="$PROJECT_ROOT/$TARGET_DIR" ;;
esac

RUNTIME_DIR="$PROJECT_ROOT/data/runtime"
mkdir -p "$RUNTIME_DIR"

BACKEND_PID=""
FRONTEND_PID=""
DESKTOP_PID=""
IMESSAGE_PID=""
DISCORD_PID=""
FEISHU_PID=""
TELEGRAM_PID=""
START_WEB="0"
START_DESKTOP="0"
START_RELEASE="0"
REBUILD_ALL="0"
DESKTOP_REMOTE="0"

pid_file() {
  echo "$RUNTIME_DIR/$1.pid"
}

bin_path() {
  echo "$TARGET_DIR/debug/$1"
}

pid_is_running() {
  local pid="$1"
  [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null
}

wait_for_exit() {
  local pid="$1"
  local timeout="${2:-6}"
  local waited=0
  local max_loops=$((timeout * 5))
  while pid_is_running "$pid"; do
    if (( waited >= max_loops )); then
      return 1
    fi
    sleep 0.2
    waited=$((waited + 1))
  done
  return 0
}

terminate_pid() {
  local pid="$1"
  local name="$2"
  local timeout="${3:-6}"
  if pid_is_running "$pid"; then
    echo "[INFO] stopping ${name} (pid=${pid})..."
    kill "$pid" 2>/dev/null || true
    if ! wait_for_exit "$pid" "$timeout"; then
      echo "[WARN] force killing ${name} (pid=${pid})..."
      kill -9 "$pid" 2>/dev/null || true
      wait_for_exit "$pid" 3 || true
    fi
  fi
}

ensure_port_available() {
  local port="$1"
  local label="$2"
  local pids=""

  if command -v lsof >/dev/null 2>&1; then
    pids="$(lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true)"
  fi

  [[ -z "$pids" ]] && return 0

  echo "[WARN] ${label} port ${port} is already occupied; cleaning stale listeners..."
  for pid in $pids; do
    terminate_pid "$pid" "${label}-port-${port}"
  done
}

stop_pid_file() {
  local name="$1"
  local file
  file="$(pid_file "$name")"
  if [[ -f "$file" ]]; then
    local pid
    pid="$(cat "$file" 2>/dev/null || true)"
    terminate_pid "$pid" "$name"
    rm -f "$file"
  fi
}

build_runtime_binaries() {
  echo "[INFO] building Hone runtime binaries..."
  cargo build \
    -p hone-mcp \
    -p hone-console-page \
    -p hone-imessage \
    -p hone-discord \
    -p hone-feishu \
    -p hone-telegram
}

build_frontend() {
  echo "[INFO] building frontend (packages/app)..."
  (cd "$PROJECT_ROOT/packages/app" && bun run build)
  echo "[INFO] frontend build done."
}

build_desktop_release() {
  echo "[INFO] building desktop release artifacts..."
  build_frontend
  bun run tauri:prep:build
  cargo build -p hone-mcp -p hone-desktop --release
}

desktop_config_dir() {
  echo "${HONE_DESKTOP_CONFIG_DIR:-$RUNTIME_DIR/desktop-config}"
}

write_desktop_backend_config_bundled() {
  local config_dir
  local config_file
  config_dir="$(desktop_config_dir)"
  config_file="$config_dir/backend.json"
  mkdir -p "$config_dir"
  printf '{\n  "mode": "bundled",\n  "baseUrl": "",\n  "bearerToken": ""\n}\n' > "$config_file"
  echo "[INFO] desktop backend config -> bundled"
}

write_desktop_backend_config_remote() {
  local backend_url="$1"
  local config_dir
  local config_file
  config_dir="$(desktop_config_dir)"
  config_file="$config_dir/backend.json"
  mkdir -p "$config_dir"
  printf '{\n  "mode": "remote",\n  "baseUrl": "%s",\n  "bearerToken": ""\n}\n' "$backend_url" > "$config_file"
  echo "[INFO] desktop backend config -> remote (${backend_url})"
}

CHANNEL_DISABLED_EXIT_CODE=20

start_hone_bin() {
  local bin_name="$1"
  local service_name="$2"
  local pid_var="$3"
  local path
  local pid

  path="$(bin_path "$bin_name")"
  if [[ ! -x "$path" ]]; then
    echo "[FAIL] missing binary: $path"
    exit 1
  fi

  "$path" &
  pid=$!
  printf -v "$pid_var" '%s' "$pid"
  echo "$pid" > "$(pid_file "$service_name")"

  sleep 1
  if ! pid_is_running "$pid"; then
    local status=0
    wait "$pid" || status=$?
    if [[ "$status" -eq "$CHANNEL_DISABLED_EXIT_CODE" ]]; then
      echo "[INFO] ${service_name} skipped by runtime config."
      rm -f "$(pid_file "$service_name")"
      printf -v "$pid_var" '%s' ""
      return 0
    fi
    echo "[FAIL] ${service_name} exited during startup (status=${status}). Check the error output above for the real cause (for example missing config/credentials, port or lock conflicts, or another startup failure)."
    exit 1
  fi
}

stop_all() {
  stop_pid_file frontend
  stop_pid_file desktop
  stop_pid_file telegram
  stop_pid_file feishu
  stop_pid_file discord
  stop_pid_file imessage
  stop_pid_file backend
  rm -f "$(pid_file current)"
}

cleanup() {
  local exit_code="${1:-0}"
  trap - INT TERM EXIT
  if [[ -n "$FRONTEND_PID" ]]; then
    terminate_pid "$FRONTEND_PID" "frontend"
  fi
  if [[ -n "$DESKTOP_PID" ]]; then
    terminate_pid "$DESKTOP_PID" "desktop"
  fi
  if [[ -n "$BACKEND_PID" ]]; then
    terminate_pid "$BACKEND_PID" "backend"
  fi
  if [[ -n "$IMESSAGE_PID" ]]; then
    terminate_pid "$IMESSAGE_PID" "imessage"
  fi
  if [[ -n "$DISCORD_PID" ]]; then
    terminate_pid "$DISCORD_PID" "discord"
  fi
  if [[ -n "$FEISHU_PID" ]]; then
    terminate_pid "$FEISHU_PID" "feishu"
  fi
  if [[ -n "$TELEGRAM_PID" ]]; then
    terminate_pid "$TELEGRAM_PID" "telegram"
  fi
  rm -f "$(pid_file frontend)" "$(pid_file desktop)" "$(pid_file backend)" "$(pid_file imessage)" "$(pid_file discord)" "$(pid_file feishu)" "$(pid_file telegram)"
  exit "$exit_code"
}

trap 'cleanup 130' INT
trap 'cleanup 143' TERM
trap 'cleanup $?' EXIT

for arg in "$@"; do
  case "$arg" in
    stop)        stop_all; echo "[INFO] stopped."; exit 0 ;;
    --web)       START_WEB="1" ;;
    --desktop)   START_DESKTOP="1" ;;
    --release)   START_RELEASE="1" ;;
    --remote)    DESKTOP_REMOTE="1" ;;
    --all)       REBUILD_ALL="1" ;;
    *)
      echo "Usage: ./launch.sh [--web|--desktop|--release] [--remote] [--all] [stop]"
      exit 1
      ;;
  esac
done

mode_count=0
[[ "$START_WEB" == "1" ]] && mode_count=$((mode_count + 1))
[[ "$START_DESKTOP" == "1" ]] && mode_count=$((mode_count + 1))
[[ "$START_RELEASE" == "1" ]] && mode_count=$((mode_count + 1))
if (( mode_count > 1 )); then
  echo "[FAIL] --web, --desktop, and --release are mutually exclusive"
  exit 1
fi

if [[ "$DESKTOP_REMOTE" == "1" && "$START_DESKTOP" != "1" ]]; then
  echo "[FAIL] --remote currently only supports --desktop"
  exit 1
fi

if [[ ! -f "$PROJECT_ROOT/config.yaml" ]]; then
  echo "[FAIL] missing config.yaml. Run: cp config.example.yaml config.yaml"
  exit 1
fi

if [[ -f "$PROJECT_ROOT/soul.md" && ! -f "$RUNTIME_DIR/soul.md" ]]; then
  cp "$PROJECT_ROOT/soul.md" "$RUNTIME_DIR/soul.md"
fi

if [[ "$START_WEB" == "1" || "$START_DESKTOP" == "1" || "$START_RELEASE" == "1" ]]; then
  if [[ -x "$HOME/.bun/bin/bun" ]]; then
    export BUN_INSTALL="$HOME/.bun"
    export PATH="$BUN_INSTALL/bin:$PATH"
  fi
  if ! command -v bun >/dev/null 2>&1; then
    echo "[FAIL] bun not found in PATH"
    exit 1
  fi
  if [[ ! -d "$PROJECT_ROOT/node_modules" ]]; then
    echo "[INFO] installing frontend dependencies..."
    bun install
  fi
fi

if [[ "$START_RELEASE" != "1" ]]; then
  build_runtime_binaries
fi

if [[ "$REBUILD_ALL" == "1" ]]; then
  build_frontend
fi

echo "[INFO] restarting processes..."
stop_all

export HONE_DISABLE_AUTO_OPEN="1"
export HONE_CONFIG_PATH="${HONE_CONFIG_PATH:-$PROJECT_ROOT/config.yaml}"
export HONE_USER_CONFIG_PATH="${HONE_USER_CONFIG_PATH:-$PROJECT_ROOT/config.yaml}"
export HONE_DATA_DIR="${HONE_DATA_DIR:-$PROJECT_ROOT/data}"
export HONE_DESKTOP_DATA_DIR="${HONE_DESKTOP_DATA_DIR:-$HONE_DATA_DIR}"
export HONE_SKILLS_DIR="${HONE_SKILLS_DIR:-$PROJECT_ROOT/skills}"
export HONE_WEB_PORT="${HONE_WEB_PORT:-8077}"
export HONE_DESKTOP_CONFIG_DIR="${HONE_DESKTOP_CONFIG_DIR:-$RUNTIME_DIR/desktop-config}"

if [[ "$START_DESKTOP" == "1" ]]; then
  if [[ "$DESKTOP_REMOTE" == "1" ]]; then
    echo "[INFO] desktop remote mode: keeping backend/channel listeners outside the desktop host."
    echo "[INFO] starting backend (hone-console-page)..."
    start_hone_bin hone-console-page backend BACKEND_PID

    echo "[INFO] waiting backend readiness..."
    for _ in $(seq 1 60); do
      if curl -fsS "http://127.0.0.1:${HONE_WEB_PORT}/api/meta" >/dev/null 2>&1; then
        break
      fi
      if ! pid_is_running "$BACKEND_PID"; then
        echo "[FAIL] backend exited unexpectedly."
        exit 1
      fi
      sleep 0.5
    done

    echo "[INFO] backend ready: http://127.0.0.1:${HONE_WEB_PORT}"

    echo "[INFO] starting channel listeners..."
    start_hone_bin hone-imessage imessage IMESSAGE_PID
    start_hone_bin hone-discord discord DISCORD_PID
    start_hone_bin hone-feishu feishu FEISHU_PID
    start_hone_bin hone-telegram telegram TELEGRAM_PID

    write_desktop_backend_config_remote "http://127.0.0.1:${HONE_WEB_PORT}"
  else
    echo "[INFO] desktop mode: external backend/channel listeners disabled; desktop will manage its bundled runtime."
    write_desktop_backend_config_bundled
  fi
  echo "[INFO] preparing Tauri sidecar binaries..."
  bun run tauri:prep:dev -- --skip-dev-command
  ensure_port_available 3000 "desktop-frontend"
  echo "[INFO] starting frontend (vite)..."
  bun run dev:web &
  FRONTEND_PID=$!
  echo "$FRONTEND_PID" > "$(pid_file frontend)"
  echo "[INFO] waiting frontend readiness..."
  for _ in $(seq 1 60); do
    if curl -fsS "http://127.0.0.1:3000" >/dev/null 2>&1; then
      break
    fi
    if ! pid_is_running "$FRONTEND_PID"; then
      echo "[FAIL] frontend exited unexpectedly."
      exit 1
    fi
    sleep 0.5
  done
  if ! curl -fsS "http://127.0.0.1:3000" >/dev/null 2>&1; then
    echo "[FAIL] frontend did not become ready on http://127.0.0.1:3000"
    exit 1
  fi
  if [[ "$DESKTOP_REMOTE" == "1" ]]; then
    echo "[INFO] starting desktop app (tauri dev, remote backend=http://127.0.0.1:${HONE_WEB_PORT})..."
  else
    echo "[INFO] starting desktop app (tauri dev)..."
  fi
  bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json &
  DESKTOP_PID=$!
  echo "$DESKTOP_PID" > "$(pid_file desktop)"
  echo "[INFO] desktop starting… (Vite: http://127.0.0.1:3000)"
  echo "[INFO] press Ctrl-C to stop."
  wait "$DESKTOP_PID"
elif [[ "$START_RELEASE" == "1" ]]; then
  echo "[INFO] release mode: starting desktop without dev hot reload."
  build_desktop_release
  export HONE_DESKTOP_DATA_DIR="${HONE_DESKTOP_DATA_DIR:-$HONE_DATA_DIR}"
  export HONE_DESKTOP_CONFIG_DIR="${HONE_DESKTOP_CONFIG_DIR:-$RUNTIME_DIR/desktop-config}"
  mkdir -p "$HONE_DESKTOP_CONFIG_DIR"
  write_desktop_backend_config_bundled
  RELEASE_DESKTOP_BIN="$TARGET_DIR/release/hone-desktop"
  if [[ ! -x "$RELEASE_DESKTOP_BIN" ]]; then
    echo "[FAIL] missing desktop release binary: $RELEASE_DESKTOP_BIN"
    exit 1
  fi
  echo "[INFO] desktop release data dir: $HONE_DESKTOP_DATA_DIR"
  echo "[INFO] desktop release config dir: $HONE_DESKTOP_CONFIG_DIR"
  "$RELEASE_DESKTOP_BIN" &
  DESKTOP_PID=$!
  echo "$DESKTOP_PID" > "$(pid_file desktop)"
  echo "[INFO] desktop release running (pid=$DESKTOP_PID)"
  echo "[INFO] press Ctrl-C to stop."
  wait "$DESKTOP_PID"
elif [[ "$START_WEB" == "1" ]]; then
  echo "[INFO] starting backend (hone-console-page)..."
  start_hone_bin hone-console-page backend BACKEND_PID

  echo "[INFO] waiting backend readiness..."
  for _ in $(seq 1 60); do
    if curl -fsS "http://127.0.0.1:${HONE_WEB_PORT}/api/meta" >/dev/null 2>&1; then
      break
    fi
    if ! pid_is_running "$BACKEND_PID"; then
      echo "[FAIL] backend exited unexpectedly."
      exit 1
    fi
    sleep 0.5
  done

  echo "[INFO] backend ready: http://127.0.0.1:${HONE_WEB_PORT}"

  echo "[INFO] starting channel listeners..."
  start_hone_bin hone-imessage imessage IMESSAGE_PID
  start_hone_bin hone-discord discord DISCORD_PID
  start_hone_bin hone-feishu feishu FEISHU_PID
  start_hone_bin hone-telegram telegram TELEGRAM_PID

  ensure_port_available 3000 "web-frontend"
  echo "[INFO] starting frontend (vite)..."
  bun run dev:web &
  FRONTEND_PID=$!
  echo "$FRONTEND_PID" > "$(pid_file frontend)"
  echo "[INFO] frontend ready: http://127.0.0.1:3000"
  echo "[INFO] press Ctrl-C to stop."
  wait "$FRONTEND_PID"
else
  echo "[INFO] starting backend (hone-console-page)..."
  start_hone_bin hone-console-page backend BACKEND_PID

  echo "[INFO] waiting backend readiness..."
  for _ in $(seq 1 60); do
    if curl -fsS "http://127.0.0.1:${HONE_WEB_PORT}/api/meta" >/dev/null 2>&1; then
      break
    fi
    if ! pid_is_running "$BACKEND_PID"; then
      echo "[FAIL] backend exited unexpectedly."
      exit 1
    fi
    sleep 0.5
  done

  echo "[INFO] backend ready: http://127.0.0.1:${HONE_WEB_PORT}"

  echo "[INFO] starting channel listeners..."
  start_hone_bin hone-imessage imessage IMESSAGE_PID
  start_hone_bin hone-discord discord DISCORD_PID
  start_hone_bin hone-feishu feishu FEISHU_PID
  start_hone_bin hone-telegram telegram TELEGRAM_PID

  echo "[INFO] frontend disabled. pass --web or --desktop to start it."
  echo "[INFO] press Ctrl-C to stop."
  wait "$BACKEND_PID"
fi
