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
TERMINATE_GRACE=20
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
  --terminate-grace SEC     TERM grace before exact-PID KILL escalation (default: 20)
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
        --terminate-grace) TERMINATE_GRACE="${2:?missing terminate grace}"; shift 2 ;;
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
for value in "$STARTUP_TIMEOUT" "$DRAIN_TIMEOUT" "$TERMINATE_GRACE"; do
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
LEGACY_RUNTIME_LABEL="${HONE_DEPLOY_LEGACY_RUNTIME_LABEL:-com.honeclaw.source.runtime}"
DOMAIN="gui/$(id -u)"
USER_HOME_DIR="$(cd && pwd)"
LAUNCH_AGENT_DIR="${HONE_DEPLOY_LAUNCH_AGENT_DIR:-$USER_HOME_DIR/Library/LaunchAgents}"
DEFAULT_SOURCE_RUNTIME_PATH="/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:/usr/local/sbin:$USER_HOME_DIR/.local/bin:$USER_HOME_DIR/.cargo/bin:/Applications/ChatGPT.app/Contents/Resources:/usr/bin:/bin:/usr/sbin:/sbin"
SOURCE_RUNTIME_PATH="${HONE_SOURCE_RUNTIME_PATH:-$DEFAULT_SOURCE_RUNTIME_PATH}"
WEB_PLIST="$LAUNCH_AGENT_DIR/$WEB_LABEL.plist"
DISCORD_PLIST="$LAUNCH_AGENT_DIR/$DISCORD_LABEL.plist"
LEGACY_RUNTIME_PLIST="$LAUNCH_AGENT_DIR/$LEGACY_RUNTIME_LABEL.plist"
LEGACY_RUNTIME_DISABLED_PLIST="$LEGACY_RUNTIME_PLIST.disabled-by-hone-source-deploy"
LOG_DIR="$DATA_DIR/logs"
LOCK_DIR="$DATA_DIR/runtime/locks"
RELEASE_ROOT="$DATA_DIR/releases/source"
WEB_LOG="$LOG_DIR/hone-console-page-source.log"
WEB_ERR="$LOG_DIR/hone-console-page-source.err.log"
DISCORD_LOG="$LOG_DIR/hone-discord-source.log"
DISCORD_ERR="$LOG_DIR/hone-discord-source.err.log"
mkdir -p "$LOG_DIR" "$RELEASE_ROOT"

log() { printf '[deploy] %s %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*"; }

yaml_agent_value() {
    local section="$1" key="$2" default_value="$3" value
    value="$(awk -v section="$section" -v key="$key" '
        function emit_value(line) {
            sub(/^[^:]*:[[:space:]]*/, "", line)
            sub(/[[:space:]]+#.*$/, "", line)
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)
            if ((substr(line, 1, 1) == "\"" && substr(line, length(line), 1) == "\"") ||
                (substr(line, 1, 1) == "\047" && substr(line, length(line), 1) == "\047")) {
                line = substr(line, 2, length(line) - 2)
            }
            print line
            exit
        }
        /^agent:[[:space:]]*($|#)/ { in_agent = 1; next }
        in_agent && /^[^[:space:]#]/ { exit }
        !in_agent { next }
        section == "" && $0 ~ "^[[:space:]]{2}" key ":[[:space:]]*" { emit_value($0) }
        section != "" && $0 ~ "^[[:space:]]{2}" section ":[[:space:]]*($|#)" { in_section = 1; next }
        in_section && /^[[:space:]]{2}[^[:space:]#][^:]*:/ { in_section = 0 }
        in_section && $0 ~ "^[[:space:]]{4}" key ":[[:space:]]*" { emit_value($0) }
    ' "$CONFIG_PATH")"
    printf '%s' "${value:-$default_value}"
}

validate_source_runtime_path() {
    local entry
    [[ -n "$SOURCE_RUNTIME_PATH" && ":$SOURCE_RUNTIME_PATH:" != *::* ]] || {
        echo "[deploy] source runtime PATH must not contain empty entries" >&2
        return 1
    }
    while IFS= read -r entry; do
        [[ "$entry" == /* ]] || {
            echo "[deploy] source runtime PATH entries must be absolute: $entry" >&2
            return 1
        }
        case "$entry" in
            */.codex/tmp/*|*/.cache/codex-runtimes/*)
                echo "[deploy] refusing ephemeral source runtime PATH entry: $entry" >&2
                return 1
                ;;
        esac
    done < <(printf '%s' "$SOURCE_RUNTIME_PATH" | tr ':' '\n')
}

validate_runtime_command() {
    local command_name="$1" resolved
    if [[ "$command_name" == */* ]]; then
        resolved="$command_name"
    else
        resolved="$(PATH="$SOURCE_RUNTIME_PATH" command -v "$command_name" 2>/dev/null || true)"
    fi
    [[ -n "$resolved" && -x "$resolved" ]] || {
        echo "[deploy] runtime command unavailable on persistent PATH: $command_name" >&2
        return 1
    }
    PATH="$SOURCE_RUNTIME_PATH" "$resolved" --version >/dev/null 2>&1 || {
        echo "[deploy] runtime command version probe failed on persistent PATH: $command_name" >&2
        return 1
    }
}

validate_runner_runtime() {
    local runner adapter_command companion_command
    validate_source_runtime_path
    runner="$(yaml_agent_value "" runner codex_acp)"
    case "$runner" in
        codex_acp)
            adapter_command="$(yaml_agent_value codex_acp command codex-acp)"
            companion_command="$(yaml_agent_value codex_acp codex_command codex)"
            validate_runtime_command "$adapter_command"
            validate_runtime_command "$companion_command"
            ;;
        opencode_acp)
            adapter_command="$(yaml_agent_value opencode command opencode)"
            validate_runtime_command "$adapter_command"
            ;;
    esac
}

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

child_pids() {
    local parent_pid="$1"
    ps -axo pid=,ppid= 2>/dev/null | awk -v parent="$parent_pid" '$2 == parent { print $1 }'
}

pid_is_child_of() {
    local candidate="$1" parent_pid="$2"
    child_pids "$parent_pid" | grep -qx "$candidate"
}

listener_pid() {
    local port="$1"
    lsof -nP -iTCP:"$port" -sTCP:LISTEN -Fp 2>/dev/null | sed -n 's/^p//p' | head -n 1
}

wait_pid_exit() {
    local pid="$1" deadline=$((SECONDS + STARTUP_TIMEOUT))
    while ps -p "$pid" -o pid= >/dev/null 2>&1; do
        (( SECONDS < deadline )) || return 1
        sleep "$POLL_INTERVAL"
    done
}

wait_pid_exit_for() {
    local pid="$1" timeout_seconds="$2" deadline
    deadline=$((SECONDS + timeout_seconds))
    while ps -p "$pid" -o pid= >/dev/null 2>&1; do
        (( SECONDS < deadline )) || return 1
        sleep "$POLL_INTERVAL"
    done
}

terminate_exact_process() {
    local pid="$1" expected_binary="$2" current_binary
    ps -p "$pid" -o pid= >/dev/null 2>&1 || return 0
    current_binary="$(process_binary "$pid")"
    if [[ -z "$expected_binary" || "$current_binary" != "$expected_binary" ]]; then
        echo "[deploy] refusing to signal reused/unverified pid=$pid binary=$current_binary" >&2
        return 1
    fi
    "${HONE_DEPLOY_KILL_COMMAND:-/bin/kill}" -TERM "$pid"
    if wait_pid_exit_for "$pid" "$TERMINATE_GRACE"; then
        return 0
    fi
    log "old child pid=$pid ignored TERM; escalating to KILL"
    "${HONE_DEPLOY_KILL_COMMAND:-/bin/kill}" -KILL "$pid"
    wait_pid_exit_for "$pid" "$TERMINATE_GRACE"
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

bootstrap_job() {
    local plist="$1"
    [[ -f "$plist" ]] || {
        echo "[deploy] launch agent plist missing: $plist" >&2
        return 1
    }
    launchctl bootstrap "$DOMAIN" "$plist"
}

restore_legacy_runtime_job() {
    [[ -f "$LEGACY_RUNTIME_PLIST" ]] || {
        echo "[deploy] legacy runtime plist missing during rollback: $LEGACY_RUNTIME_PLIST" >&2
        return 1
    }
    launchctl bootstrap "$DOMAIN" "$LEGACY_RUNTIME_PLIST"
}

wait_legacy_child_binary() {
    local expected_name="$1" deadline=$((SECONDS + STARTUP_TIMEOUT)) legacy_pid child binary
    while (( SECONDS < deadline )); do
        legacy_pid="$(job_pid "$LEGACY_RUNTIME_LABEL" || true)"
        if [[ -n "$legacy_pid" ]]; then
            while IFS= read -r child; do
                [[ -n "$child" ]] || continue
                binary="$(process_binary "$child")"
                [[ "${binary##*/}" == "$expected_name" ]] && return 0
            done < <(child_pids "$legacy_pid")
        fi
        sleep "$POLL_INTERVAL"
    done
    return 1
}

stop_legacy_runtime_and_wait() {
    local supervisor_pid child child_binary child_record child_records=""
    supervisor_pid="$(job_pid "$LEGACY_RUNTIME_LABEL" || true)"
    job_exists "$LEGACY_RUNTIME_LABEL" || return 0
    if [[ -n "$supervisor_pid" ]]; then
        while IFS= read -r child; do
            [[ -n "$child" ]] || continue
            child_binary="$(process_binary "$child")"
            child_records+="$child|$child_binary"$'\n'
        done < <(child_pids "$supervisor_pid")
    fi
    launchctl remove "$LEGACY_RUNTIME_LABEL"
    if [[ -n "$supervisor_pid" ]]; then
        wait_pid_exit "$supervisor_pid" || {
            echo "[deploy] old legacy runtime pid=$supervisor_pid did not exit" >&2
            return 1
        }
    fi
    while IFS= read -r child_record; do
        [[ -n "$child_record" ]] || continue
        child="${child_record%%|*}"
        child_binary="${child_record#*|}"
        terminate_exact_process "$child" "$child_binary" || {
            echo "[deploy] old legacy child pid=$child could not be terminated safely" >&2
            return 1
        }
    done <<< "$child_records"
}

xml_escape() {
    sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e 's/"/\&quot;/g' -e "s/'/\&apos;/g"
}

write_launch_agent_plist() {
    local label="$1" binary="$2" stdout_log="$3" stderr_log="$4" target="$5"
    local escaped_label escaped_binary escaped_root escaped_config escaped_data escaped_skills
    local escaped_stdout escaped_stderr escaped_path sandbox_xml=""
    escaped_label="$(printf '%s' "$label" | xml_escape)"
    escaped_binary="$(printf '%s' "$binary" | xml_escape)"
    escaped_root="$(printf '%s' "$PROJECT_ROOT" | xml_escape)"
    escaped_config="$(printf '%s' "$CONFIG_PATH" | xml_escape)"
    escaped_data="$(printf '%s' "$DATA_DIR" | xml_escape)"
    escaped_skills="$(printf '%s' "$SKILLS_DIR" | xml_escape)"
    escaped_stdout="$(printf '%s' "$stdout_log" | xml_escape)"
    escaped_stderr="$(printf '%s' "$stderr_log" | xml_escape)"
    escaped_path="$(printf '%s' "$SOURCE_RUNTIME_PATH" | xml_escape)"
    if [[ -n "${SOURCE_AGENT_SANDBOX_DIR:-}" ]]; then
        sandbox_xml="<key>HONE_AGENT_SANDBOX_DIR</key><string>$(printf '%s' "$SOURCE_AGENT_SANDBOX_DIR" | xml_escape)</string>"
    fi
    cat > "$target" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>$escaped_label</string>
<key>ProgramArguments</key><array><string>$escaped_binary</string></array>
<key>WorkingDirectory</key><string>$escaped_root</string>
<key>EnvironmentVariables</key><dict>
<key>HONE_CONFIG_PATH</key><string>$escaped_config</string>
<key>HONE_DATA_DIR</key><string>$escaped_data</string>
<key>HONE_SKILLS_DIR</key><string>$escaped_skills</string>
<key>HONE_SOURCE_RUNTIME_MANAGED</key><string>1</string>
<key>NO_PROXY</key><string>localhost,127.0.0.1,::1</string>
<key>PATH</key><string>$escaped_path</string>
$sandbox_xml
</dict>
<key>RunAtLoad</key><true/><key>KeepAlive</key><true/>
<key>ProcessType</key><string>Background</string>
<key>StandardOutPath</key><string>$escaped_stdout</string>
<key>StandardErrorPath</key><string>$escaped_stderr</string>
</dict></plist>
EOF
}

install_launch_agent_plist() {
    local source="$1" target="$2" temporary
    mkdir -p "$LAUNCH_AGENT_DIR"
    temporary="$(mktemp "$LAUNCH_AGENT_DIR/.hone-source-plist.XXXXXX")"
    cp "$source" "$temporary"
    chmod 0644 "$temporary"
    mv "$temporary" "$target"
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
ORIGINAL_LEGACY_RUNTIME_EXISTS=0
ORIGINAL_LEGACY_RUNTIME_RUNNING=0
ORIGINAL_LEGACY_DISCORD_RUNNING=0
ORIGINAL_WEB_BINARY=""
ORIGINAL_DISCORD_BINARY=""
ORIGINAL_WEB_PLIST_EXISTS=0
ORIGINAL_DISCORD_PLIST_EXISTS=0
LEGACY_PLIST_DISABLED_DURING_DEPLOY=0
ROLLBACK_ARMED=0
DEPLOY_COMMITTED=0
STAGING_DIR=""
ROLLBACK_DIR=""
WEB_PLIST_CANDIDATE=""
DISCORD_PLIST_CANDIDATE=""

cleanup_staging() {
    [[ -n "$STAGING_DIR" && -d "$STAGING_DIR" ]] || return 0
    rm -f "$STAGING_DIR/hone-console-page" "$STAGING_DIR/hone-discord" \
        "$STAGING_DIR/hone-mcp" "$STAGING_DIR/manifest.json"
    rmdir "$STAGING_DIR" 2>/dev/null || true
    STAGING_DIR=""
}

cleanup_rollback_dir() {
    [[ -n "$ROLLBACK_DIR" && -d "$ROLLBACK_DIR" ]] || return 0
    rm -f "$ROLLBACK_DIR/web.plist" "$ROLLBACK_DIR/discord.plist" \
        "$ROLLBACK_DIR/web.next.plist" "$ROLLBACK_DIR/discord.next.plist"
    rmdir "$ROLLBACK_DIR" 2>/dev/null || true
    ROLLBACK_DIR=""
}

restore_launch_agent_files() {
    if (( ORIGINAL_WEB_PLIST_EXISTS )); then
        install_launch_agent_plist "$ROLLBACK_DIR/web.plist" "$WEB_PLIST"
    else
        rm -f "$WEB_PLIST"
    fi
    if (( ORIGINAL_DISCORD_PLIST_EXISTS )); then
        install_launch_agent_plist "$ROLLBACK_DIR/discord.plist" "$DISCORD_PLIST"
    else
        rm -f "$DISCORD_PLIST"
    fi
    if (( LEGACY_PLIST_DISABLED_DURING_DEPLOY )); then
        mv "$LEGACY_RUNTIME_DISABLED_PLIST" "$LEGACY_RUNTIME_PLIST"
        LEGACY_PLIST_DISABLED_DURING_DEPLOY=0
    fi
}

rollback() {
    local failure=0 pid offset
    log "rollback begin failed_state=$DEPLOY_STATE"
    stop_job_and_wait "$DISCORD_LABEL" || failure=1
    stop_job_and_wait "$WEB_LABEL" || failure=1
    wait_locks_release || failure=1
    restore_launch_agent_files || failure=1
    if (( ORIGINAL_WEB_RUNNING )); then
        bootstrap_job "$WEB_PLIST" || failure=1
        wait_web_ready "" || failure=1
    fi
    if (( ORIGINAL_DISCORD_RUNNING )); then
        offset="$(wc -c < "$DISCORD_LOG" 2>/dev/null || printf 0)"
        bootstrap_job "$DISCORD_PLIST" || failure=1
        wait_discord_ready "$offset" || failure=1
    fi
    if (( ORIGINAL_LEGACY_RUNTIME_EXISTS )); then
        restore_legacy_runtime_job || failure=1
        wait_web_ready "" || failure=1
        if (( ORIGINAL_LEGACY_DISCORD_RUNNING )); then
            wait_legacy_child_binary "hone-discord" || failure=1
        fi
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
    cleanup_rollback_dir
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
validate_runner_runtime

if job_exists "$LEGACY_RUNTIME_LABEL" && job_exists "$WEB_LABEL"; then
    echo "[deploy] conflicting legacy and managed Web launchd jobs are both loaded" >&2
    exit 1
fi
if job_exists "$WEB_LABEL" && [[ ! -f "$WEB_PLIST" ]]; then
    echo "[deploy] loaded managed Web job has no persistent plist: $WEB_PLIST" >&2
    exit 1
fi
if job_exists "$DISCORD_LABEL" && [[ ! -f "$DISCORD_PLIST" ]]; then
    echo "[deploy] loaded managed Discord job has no persistent plist: $DISCORD_PLIST" >&2
    exit 1
fi
if job_exists "$LEGACY_RUNTIME_LABEL" && [[ ! -f "$LEGACY_RUNTIME_PLIST" ]]; then
    echo "[deploy] loaded legacy runtime has no rollback plist: $LEGACY_RUNTIME_PLIST" >&2
    exit 1
fi
if [[ -f "$LEGACY_RUNTIME_PLIST" && -e "$LEGACY_RUNTIME_DISABLED_PLIST" ]]; then
    echo "[deploy] legacy runtime plist and disabled backup both exist; resolve them before deployment" >&2
    exit 1
fi
current_listener_pid="$(listener_pid 8077 || true)"
if [[ -n "$current_listener_pid" ]]; then
    managed_web_pid="$(job_pid "$WEB_LABEL" || true)"
    legacy_runtime_pid="$(job_pid "$LEGACY_RUNTIME_LABEL" || true)"
    if [[ "$current_listener_pid" != "$managed_web_pid" ]] \
        && { [[ -z "$legacy_runtime_pid" ]] || ! pid_is_child_of "$current_listener_pid" "$legacy_runtime_pid"; }; then
        echo "[deploy] port 8077 is owned by unmanaged pid=$current_listener_pid; refusing takeover" >&2
        exit 1
    fi
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

ROLLBACK_DIR="$(mktemp -d "$DATA_DIR/runtime/.source-deploy-rollback.XXXXXX")"
WEB_PLIST_CANDIDATE="$ROLLBACK_DIR/web.next.plist"
DISCORD_PLIST_CANDIDATE="$ROLLBACK_DIR/discord.next.plist"
if [[ -f "$WEB_PLIST" ]]; then
    ORIGINAL_WEB_PLIST_EXISTS=1
    cp "$WEB_PLIST" "$ROLLBACK_DIR/web.plist"
fi
if [[ -f "$DISCORD_PLIST" ]]; then
    ORIGINAL_DISCORD_PLIST_EXISTS=1
    cp "$DISCORD_PLIST" "$ROLLBACK_DIR/discord.plist"
fi
SOURCE_AGENT_SANDBOX_DIR="${HONE_AGENT_SANDBOX_DIR:-}"
if [[ -z "$SOURCE_AGENT_SANDBOX_DIR" && -f "$LEGACY_RUNTIME_PLIST" ]] \
    && command -v plutil >/dev/null 2>&1; then
    SOURCE_AGENT_SANDBOX_DIR="$(plutil -extract EnvironmentVariables.HONE_AGENT_SANDBOX_DIR raw -o - "$LEGACY_RUNTIME_PLIST" 2>/dev/null || true)"
fi
write_launch_agent_plist "$WEB_LABEL" "$RELEASE_DIR/hone-console-page" "$WEB_LOG" "$WEB_ERR" "$WEB_PLIST_CANDIDATE"
write_launch_agent_plist "$DISCORD_LABEL" "$RELEASE_DIR/hone-discord" "$DISCORD_LOG" "$DISCORD_ERR" "$DISCORD_PLIST_CANDIDATE"

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
if job_exists "$LEGACY_RUNTIME_LABEL"; then
    ORIGINAL_LEGACY_RUNTIME_EXISTS=1
    legacy_runtime_pid="$(job_pid "$LEGACY_RUNTIME_LABEL" || true)"
    if [[ -n "$legacy_runtime_pid" ]]; then
        ORIGINAL_LEGACY_RUNTIME_RUNNING=1
        while IFS= read -r child; do
            [[ -n "$child" ]] || continue
            child_binary="$(process_binary "$child")"
            if [[ "${child_binary##*/}" == "hone-discord" ]]; then
                ORIGINAL_LEGACY_DISCORD_RUNNING=1
            fi
        done < <(child_pids "$legacy_runtime_pid")
    fi
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
    auto) START_DISCORD=$((ORIGINAL_DISCORD_RUNNING || ORIGINAL_LEGACY_DISCORD_RUNNING)) ;;
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
stop_legacy_runtime_and_wait
transition wait_pid_and_lock
wait_locks_release || { echo "[deploy] process locks did not release" >&2; exit 1; }

transition start
log "starting web revision $REVISION"
install_launch_agent_plist "$WEB_PLIST_CANDIDATE" "$WEB_PLIST"
bootstrap_job "$WEB_PLIST"
transition startup
transition ready
wait_web_ready "$REVISION" || { echo "[deploy] web readiness timeout" >&2; exit 1; }

transition channel_login
if (( START_DISCORD )); then
    offset="$(wc -c < "$DISCORD_LOG" 2>/dev/null || printf 0)"
    log "starting Discord revision $REVISION"
    install_launch_agent_plist "$DISCORD_PLIST_CANDIDATE" "$DISCORD_PLIST"
    bootstrap_job "$DISCORD_PLIST"
    wait_discord_ready "$offset" || { echo "[deploy] Discord readiness timeout" >&2; exit 1; }
else
    rm -f "$DISCORD_PLIST"
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

if [[ -f "$LEGACY_RUNTIME_PLIST" ]]; then
    mv "$LEGACY_RUNTIME_PLIST" "$LEGACY_RUNTIME_DISABLED_PLIST"
    LEGACY_PLIST_DISABLED_DURING_DEPLOY=1
fi
ln -sfn "$RELEASE_DIR" "$RELEASE_ROOT/current"
DEPLOY_COMMITTED=1
ROLLBACK_ARMED=0
log "deployment complete revision=$REVISION web_pid=$running_web_pid discord=${START_DISCORD}"
