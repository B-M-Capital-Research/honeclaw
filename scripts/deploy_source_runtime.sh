#!/usr/bin/env bash
# Deploy the local source backend and the currently-running Discord listener as
# one revision-bound unit. Frontend Vite processes remain independent.

set -euo pipefail

PROJECT_ROOT=""
CONFIG_PATH=""
DATA_DIR=""
SKILLS_DIR=""
EXPECTED_REVISION=""
STARTUP_TIMEOUT=180
DRAIN_TIMEOUT=300
POLL_INTERVAL=1
SKIP_BUILD=0
ALLOW_UNPUSHED=0
DISCORD_MODE=auto

usage() {
    cat <<'EOF'
Usage: scripts/deploy_source_runtime.sh [options]

  --project-root PATH       Source checkout (default: repository root)
  --config PATH             Runtime config (default: <root>/config.yaml)
  --data-dir PATH           Runtime data (default: <root>/data)
  --skills-dir PATH         Skills directory (default: <root>/skills)
  --revision SHA            Require HEAD to equal this revision
  --startup-timeout SEC     Startup/readiness deadline (default: 180)
  --drain-timeout SEC       Active-chat drain deadline (default: 300)
  --poll-interval SEC       Poll interval; decimals allowed (default: 1)
  --discord auto|yes|no     Restart Discord when previously running, always, or never
  --skip-build              Reuse target/debug binaries
  --allow-unpushed          Permit a revision absent from origin/*
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --project-root) PROJECT_ROOT="${2:?missing project root}"; shift 2 ;;
        --config) CONFIG_PATH="${2:?missing config path}"; shift 2 ;;
        --data-dir) DATA_DIR="${2:?missing data dir}"; shift 2 ;;
        --skills-dir) SKILLS_DIR="${2:?missing skills dir}"; shift 2 ;;
        --revision) EXPECTED_REVISION="${2:?missing revision}"; shift 2 ;;
        --startup-timeout) STARTUP_TIMEOUT="${2:?missing startup timeout}"; shift 2 ;;
        --drain-timeout) DRAIN_TIMEOUT="${2:?missing drain timeout}"; shift 2 ;;
        --poll-interval) POLL_INTERVAL="${2:?missing poll interval}"; shift 2 ;;
        --discord) DISCORD_MODE="${2:?missing discord mode}"; shift 2 ;;
        --skip-build) SKIP_BUILD=1; shift ;;
        --allow-unpushed) ALLOW_UNPUSHED=1; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "[deploy] unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

if [[ -z "$PROJECT_ROOT" ]]; then
    PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
PROJECT_ROOT="$(cd "$PROJECT_ROOT" && pwd)"
CONFIG_PATH="${CONFIG_PATH:-$PROJECT_ROOT/config.yaml}"
DATA_DIR="${DATA_DIR:-$PROJECT_ROOT/data}"
SKILLS_DIR="${SKILLS_DIR:-$PROJECT_ROOT/skills}"

case "$DISCORD_MODE" in auto|yes|no) ;; *) echo "[deploy] invalid --discord: $DISCORD_MODE" >&2; exit 2 ;; esac
for value in "$STARTUP_TIMEOUT" "$DRAIN_TIMEOUT"; do
    [[ "$value" =~ ^[1-9][0-9]*$ ]] || { echo "[deploy] timeout must be a positive integer: $value" >&2; exit 2; }
done
[[ -f "$CONFIG_PATH" ]] || { echo "[deploy] config missing: $CONFIG_PATH" >&2; exit 1; }
[[ -d "$DATA_DIR" ]] || { echo "[deploy] data dir missing: $DATA_DIR" >&2; exit 1; }
[[ -d "$SKILLS_DIR" ]] || { echo "[deploy] skills dir missing: $SKILLS_DIR" >&2; exit 1; }

if [[ "${HONE_DEPLOY_TEST_MODE:-0}" != 1 && "$(uname -s)" != Darwin ]]; then
    echo "[deploy] direct source deployment currently requires macOS launchctl" >&2
    exit 1
fi

WEB_LABEL="${HONE_DEPLOY_WEB_LABEL:-com.honeclaw.source.web}"
DISCORD_LABEL="${HONE_DEPLOY_DISCORD_LABEL:-com.honeclaw.source.discord}"
DOMAIN="gui/$(id -u)"
LOG_DIR="$DATA_DIR/logs"
LOCK_DIR="$DATA_DIR/runtime/locks"
RELEASE_ROOT="$DATA_DIR/releases/source"
WEB_LOG="$LOG_DIR/hone-console-page-source.log"
WEB_ERR="$LOG_DIR/hone-console-page-source.err.log"
DISCORD_LOG="$LOG_DIR/hone-discord-source.log"
DISCORD_ERR="$LOG_DIR/hone-discord-source.err.log"
mkdir -p "$LOG_DIR" "$RELEASE_ROOT"

log() { printf '[deploy] %s %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*"; }

DEPLOY_STATE="preflight"
transition() {
    DEPLOY_STATE="$1"
    log "state=$DEPLOY_STATE"
}

job_pid() {
    launchctl print "$DOMAIN/$1" 2>/dev/null | awk '/^[[:space:]]*pid = / { print $3; exit }'
}

job_exists() {
    launchctl print "$DOMAIN/$1" >/dev/null 2>&1
}

job_running() {
    [[ -n "$(job_pid "$1" || true)" ]]
}

process_binary() {
    local pid="$1"
    lsof -a -p "$pid" -d txt -Fn 2>/dev/null | sed -n 's/^n//p' | head -n 1
}

wait_pid_exit() {
    local pid="$1" deadline=$((SECONDS + STARTUP_TIMEOUT))
    while ps -p "$pid" -o pid= >/dev/null 2>&1; do
        (( SECONDS < deadline )) || return 1
        sleep "$POLL_INTERVAL"
    done
}

wait_locks_release() {
    local deadline=$((SECONDS + STARTUP_TIMEOUT))
    while lsof "$LOCK_DIR/hone-console-page.lock" "$LOCK_DIR/hone-discord.lock" >/dev/null 2>&1; do
        (( SECONDS < deadline )) || return 1
        sleep "$POLL_INTERVAL"
    done
}

active_chat_count() {
    local payload
    payload="$(curl -fsS --max-time 3 http://127.0.0.1:8077/api/runtime/active-chat-runs 2>/dev/null || true)"
    printf '%s' "$payload" | sed -n 's/.*"count"[[:space:]]*:[[:space:]]*\([0-9][0-9]*\).*/\1/p'
}

wait_active_chats_zero() {
    local deadline=$((SECONDS + DRAIN_TIMEOUT)) count
    while true; do
        count="$(active_chat_count)"
        [[ "$count" == 0 ]] && return 0
        [[ -n "$count" ]] || { echo "[deploy] active-chat endpoint unavailable" >&2; return 1; }
        (( SECONDS < deadline )) || { echo "[deploy] active chats did not drain: $count" >&2; return 1; }
        sleep "$POLL_INTERVAL"
    done
}

submit_job() {
    local label="$1" stdout_log="$2" stderr_log="$3" binary="$4"
    launchctl submit -l "$label" -o "$stdout_log" -e "$stderr_log" -- \
        /bin/zsh -c 'cd "$1" && exec /usr/bin/env HONE_CONFIG_PATH="$2" HONE_DATA_DIR="$3" HONE_SKILLS_DIR="$4" "$5"' \
        hone-source-runtime "$PROJECT_ROOT" "$CONFIG_PATH" "$DATA_DIR" "$SKILLS_DIR" "$binary"
}

stop_job_and_wait() {
    local label="$1" pid
    pid="$(job_pid "$label" || true)"
    job_exists "$label" || return 0
    launchctl remove "$label"
    if [[ -n "$pid" ]]; then
        wait_pid_exit "$pid" || { echo "[deploy] old $label pid=$pid did not exit" >&2; return 1; }
    fi
}

wait_web_ready() {
    local expected_revision="${1:-}" deadline=$((SECONDS + STARTUP_TIMEOUT)) payload
    while (( SECONDS < deadline )); do
        payload="$(curl -fsS --max-time 3 http://127.0.0.1:8077/api/meta 2>/dev/null || true)"
        if [[ -n "$payload" ]] && curl -fsS --max-time 3 http://127.0.0.1:8088/ >/dev/null 2>&1; then
            if [[ -z "$expected_revision" || "$payload" == *"\"git_sha\":\"$expected_revision\""* ]]; then
                return 0
            fi
        fi
        sleep "$POLL_INTERVAL"
    done
    return 1
}

wait_discord_ready() {
    local offset="$1" deadline=$((SECONDS + STARTUP_TIMEOUT))
    while (( SECONDS < deadline )); do
        if tail -c "+$((offset + 1))" "$DISCORD_LOG" 2>/dev/null | grep -q 'Discord 已登录'; then
            return 0
        fi
        sleep "$POLL_INTERVAL"
    done
    return 1
}

ORIGINAL_WEB_RUNNING=0
ORIGINAL_DISCORD_RUNNING=0
ORIGINAL_WEB_BINARY=""
ORIGINAL_DISCORD_BINARY=""
ROLLBACK_ARMED=0
DEPLOY_COMMITTED=0
STAGING_DIR=""

cleanup_staging() {
    [[ -n "$STAGING_DIR" && -d "$STAGING_DIR" ]] || return 0
    rm -f "$STAGING_DIR/hone-console-page" "$STAGING_DIR/hone-discord" \
        "$STAGING_DIR/hone-mcp" "$STAGING_DIR/manifest.json"
    rmdir "$STAGING_DIR" 2>/dev/null || true
    STAGING_DIR=""
}

rollback() {
    local failure=0 pid offset
    log "rollback begin failed_state=$DEPLOY_STATE"
    stop_job_and_wait "$DISCORD_LABEL" || failure=1
    stop_job_and_wait "$WEB_LABEL" || failure=1
    wait_locks_release || failure=1
    if (( ORIGINAL_WEB_RUNNING )); then
        submit_job "$WEB_LABEL" "$WEB_LOG" "$WEB_ERR" "$ORIGINAL_WEB_BINARY" || failure=1
        wait_web_ready "" || failure=1
    fi
    if (( ORIGINAL_DISCORD_RUNNING )); then
        offset="$(wc -c < "$DISCORD_LOG" 2>/dev/null || printf 0)"
        submit_job "$DISCORD_LABEL" "$DISCORD_LOG" "$DISCORD_ERR" "$ORIGINAL_DISCORD_BINARY" || failure=1
        wait_discord_ready "$offset" || failure=1
    fi
    if (( failure )); then
        echo "[deploy] rollback incomplete; inspect launchctl and logs immediately" >&2
        return 1
    fi
    log "rollback complete"
}

on_exit() {
    local status=$?
    trap - EXIT INT TERM
    if (( ROLLBACK_ARMED && ! DEPLOY_COMMITTED )); then
        rollback || status=1
    fi
    cleanup_staging
    exit "$status"
}
trap on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

cd "$PROJECT_ROOT"
transition preflight
[[ -z "$(git status --porcelain)" ]] || { echo "[deploy] worktree is dirty" >&2; exit 1; }
REVISION="$(git rev-parse HEAD)"
[[ -z "$EXPECTED_REVISION" || "$REVISION" == "$EXPECTED_REVISION" ]] || {
    echo "[deploy] HEAD $REVISION does not match --revision $EXPECTED_REVISION" >&2
    exit 1
}
if (( ! ALLOW_UNPUSHED )); then
    git for-each-ref --format='%(refname)' --contains "$REVISION" refs/remotes/origin/ | grep -q . || {
        echo "[deploy] revision $REVISION is not reachable from origin/*; push it or use --allow-unpushed" >&2
        exit 1
    }
fi

RELEASE_DIR="$RELEASE_ROOT/$REVISION"
if [[ -d "$RELEASE_DIR" ]]; then
    [[ -f "$RELEASE_DIR/manifest.json" ]] || { echo "[deploy] release manifest missing: $RELEASE_DIR" >&2; exit 1; }
    for binary in hone-console-page hone-discord hone-mcp; do
        [[ -x "$RELEASE_DIR/$binary" ]] || { echo "[deploy] release binary missing: $RELEASE_DIR/$binary" >&2; exit 1; }
    done
    WEB_SHA="$(shasum -a 256 "$RELEASE_DIR/hone-console-page" | awk '{print $1}')"
    DISCORD_SHA="$(shasum -a 256 "$RELEASE_DIR/hone-discord" | awk '{print $1}')"
    MCP_SHA="$(shasum -a 256 "$RELEASE_DIR/hone-mcp" | awk '{print $1}')"
    grep -q "\"git_sha\":\"$REVISION\"" "$RELEASE_DIR/manifest.json" \
        && grep -q "\"hone-console-page\":\"$WEB_SHA\"" "$RELEASE_DIR/manifest.json" \
        && grep -q "\"hone-discord\":\"$DISCORD_SHA\"" "$RELEASE_DIR/manifest.json" \
        && grep -q "\"hone-mcp\":\"$MCP_SHA\"" "$RELEASE_DIR/manifest.json" || {
            echo "[deploy] immutable release manifest/hash mismatch: $RELEASE_DIR" >&2
            exit 1
        }
    log "reusing verified immutable release $RELEASE_DIR"
else
    BUILD_TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    if (( ! SKIP_BUILD )); then
        log "building revision $REVISION"
        HONE_BUILD_GIT_SHA="$REVISION" HONE_BUILD_TIMESTAMP="$BUILD_TIMESTAMP" \
            cargo build -p hone-console-page -p hone-discord -p hone-mcp
    fi
    for binary in hone-console-page hone-discord hone-mcp; do
        [[ -x "$PROJECT_ROOT/target/debug/$binary" ]] || { echo "[deploy] binary missing: target/debug/$binary" >&2; exit 1; }
    done
    STAGING_DIR="$(mktemp -d "$RELEASE_ROOT/.staging.XXXXXX")"
    for binary in hone-console-page hone-discord hone-mcp; do
        cp "$PROJECT_ROOT/target/debug/$binary" "$STAGING_DIR/$binary"
        chmod 0755 "$STAGING_DIR/$binary"
    done
    WEB_SHA="$(shasum -a 256 "$STAGING_DIR/hone-console-page" | awk '{print $1}')"
    DISCORD_SHA="$(shasum -a 256 "$STAGING_DIR/hone-discord" | awk '{print $1}')"
    MCP_SHA="$(shasum -a 256 "$STAGING_DIR/hone-mcp" | awk '{print $1}')"
    printf '{"git_sha":"%s","build_timestamp":"%s","binaries":{"hone-console-page":"%s","hone-discord":"%s","hone-mcp":"%s"}}\n' \
        "$REVISION" "$BUILD_TIMESTAMP" "$WEB_SHA" "$DISCORD_SHA" "$MCP_SHA" > "$STAGING_DIR/manifest.json"
    mv "$STAGING_DIR" "$RELEASE_DIR"
    STAGING_DIR=""
fi

[[ "$REVISION" == "$(git rev-parse HEAD)" && -z "$(git status --porcelain)" ]] || {
    echo "[deploy] source changed during build" >&2
    exit 1
}

if job_running "$WEB_LABEL"; then
    ORIGINAL_WEB_RUNNING=1
    pid="$(job_pid "$WEB_LABEL")"
    ORIGINAL_WEB_BINARY="$(process_binary "$pid")"
fi
if job_running "$DISCORD_LABEL"; then
    ORIGINAL_DISCORD_RUNNING=1
    pid="$(job_pid "$DISCORD_LABEL")"
    ORIGINAL_DISCORD_BINARY="$(process_binary "$pid")"
fi
if (( ORIGINAL_WEB_RUNNING )) && [[ -z "$ORIGINAL_WEB_BINARY" ]]; then
    echo "[deploy] cannot resolve current web binary" >&2
    exit 1
fi
if (( ORIGINAL_DISCORD_RUNNING )) && [[ -z "$ORIGINAL_DISCORD_BINARY" ]]; then
    echo "[deploy] cannot resolve current Discord binary" >&2
    exit 1
fi

case "$DISCORD_MODE" in
    auto) START_DISCORD=$ORIGINAL_DISCORD_RUNNING ;;
    yes) START_DISCORD=1 ;;
    no) START_DISCORD=0 ;;
esac

transition drain
log "waiting for active chats to drain"
wait_active_chats_zero
ROLLBACK_ARMED=1
transition stop
stop_job_and_wait "$DISCORD_LABEL"
stop_job_and_wait "$WEB_LABEL"
transition wait_pid_and_lock
wait_locks_release || { echo "[deploy] process locks did not release" >&2; exit 1; }

transition start
log "starting web revision $REVISION"
submit_job "$WEB_LABEL" "$WEB_LOG" "$WEB_ERR" "$RELEASE_DIR/hone-console-page"
transition startup
transition ready
wait_web_ready "$REVISION" || { echo "[deploy] web readiness timeout" >&2; exit 1; }

transition channel_login
if (( START_DISCORD )); then
    offset="$(wc -c < "$DISCORD_LOG" 2>/dev/null || printf 0)"
    log "starting Discord revision $REVISION"
    submit_job "$DISCORD_LABEL" "$DISCORD_LOG" "$DISCORD_ERR" "$RELEASE_DIR/hone-discord"
    wait_discord_ready "$offset" || { echo "[deploy] Discord readiness timeout" >&2; exit 1; }
fi

transition verify
running_web_pid="$(job_pid "$WEB_LABEL")"
running_web_binary="$(process_binary "$running_web_pid")"
[[ "$running_web_binary" == "$RELEASE_DIR/hone-console-page" ]] || {
    echo "[deploy] running web binary mismatch: $running_web_binary" >&2
    exit 1
}
if (( START_DISCORD )); then
    running_discord_pid="$(job_pid "$DISCORD_LABEL")"
    running_discord_binary="$(process_binary "$running_discord_pid")"
    [[ "$running_discord_binary" == "$RELEASE_DIR/hone-discord" ]] || {
        echo "[deploy] running Discord binary mismatch: $running_discord_binary" >&2
        exit 1
    }
fi

ln -sfn "$RELEASE_DIR" "$RELEASE_ROOT/current"
DEPLOY_COMMITTED=1
ROLLBACK_ARMED=0
log "deployment complete revision=$REVISION web_pid=$running_web_pid discord=${START_DISCORD}"
