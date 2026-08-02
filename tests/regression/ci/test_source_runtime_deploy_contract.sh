#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DEPLOY_SCRIPT="$REPO_ROOT/scripts/deploy_source_runtime.sh"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

PROJECT="$TMP_ROOT/project"
FAKE_BIN="$TMP_ROOT/fake-bin"
STATE="$TMP_ROOT/state"
mkdir -p "$PROJECT/data/logs" "$PROJECT/data/runtime/locks" "$PROJECT/skills" \
    "$PROJECT/target/debug" "$PROJECT/old" "$FAKE_BIN" "$STATE"
printf 'agent:\n  runner: codex_acp\n' > "$PROJECT/config.yaml"
touch "$PROJECT/skills/.gitkeep"
printf 'data/\ntarget/\nold/\n' > "$PROJECT/.gitignore"
for binary in hone-console-page hone-discord hone-mcp; do
    printf '#!/bin/sh\nexit 0\n' > "$PROJECT/target/debug/$binary"
    chmod 0755 "$PROJECT/target/debug/$binary"
done
for binary in hone-console-page hone-discord; do
    printf '#!/bin/sh\nexit 0\n' > "$PROJECT/old/$binary"
    chmod 0755 "$PROJECT/old/$binary"
done
git -C "$PROJECT" init -q
git -C "$PROJECT" config user.name test
git -C "$PROJECT" config user.email test@example.invalid
git -C "$PROJECT" add .gitignore config.yaml skills/.gitkeep
git -C "$PROJECT" commit -qm baseline
REVISION="$(git -C "$PROJECT" rev-parse HEAD)"

cat > "$FAKE_BIN/launchctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
state="${FAKE_STATE:?}"
command="${1:?}"
shift
case "$command" in
    print)
        label="${1##*/}"
        [[ -f "$state/$label.loaded" ]] || exit 113
        printf 'state = %s\n' "$([[ -f "$state/$label.pid" ]] && printf running || printf exited)"
        if [[ -f "$state/$label.pid" ]]; then
            printf '\tpid = %s\n' "$(<"$state/$label.pid")"
        fi
        ;;
    remove)
        label="${1:?}"
        rm -f "$state/$label.loaded" "$state/$label.pid" "$state/$label.binary"
        printf 'remove %s\n' "$label" >> "$state/events"
        ;;
    submit)
        label="" stdout_log=""
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -l) label="$2"; shift 2 ;;
                -o) stdout_log="$2"; shift 2 ;;
                -e) shift 2 ;;
                --) shift; break ;;
                *) shift ;;
            esac
        done
        binary="${!#}"
        counter=100
        [[ ! -f "$state/counter" ]] || counter="$(<"$state/counter")"
        counter=$((counter + 1))
        printf '%s\n' "$counter" > "$state/counter"
        : > "$state/$label.loaded"
        printf '%s\n' "$binary" > "$state/$label.binary"
        printf 'submit %s %s\n' "$label" "$binary" >> "$state/events"
        if [[ "$binary" == *'/releases/source/'* ]] \
            && { [[ "$label" == *web* && "${FAKE_FAIL_NEW_WEB:-0}" == 1 ]] \
                || [[ "$label" == *discord* && "${FAKE_FAIL_NEW_DISCORD:-0}" == 1 ]]; }; then
            rm -f "$state/$label.pid"
        else
            printf '%s\n' "$counter" > "$state/$label.pid"
        fi
        if [[ "$label" == *discord* ]]; then
            if [[ "$binary" == *'/releases/source/'* && "${FAKE_FAIL_NEW_DISCORD:-0}" == 1 ]]; then
                :
            else
                printf 'Discord 已登录\n' >> "$stdout_log"
            fi
        fi
        ;;
    *) exit 2 ;;
esac
EOF

cat > "$FAKE_BIN/ps" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
pid="${2:?}"
printf 'ps %s\n' "$pid" >> "${FAKE_STATE:?}/events"
for file in "${FAKE_STATE:?}"/*.pid; do
    [[ -e "$file" ]] || continue
    if [[ "$(<"$file")" == "$pid" ]]; then
        printf '%s\n' "$pid"
        exit 0
    fi
done
exit 1
EOF

cat > "$FAKE_BIN/lsof" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'lsof %s\n' "$*" >> "${FAKE_STATE:?}/events"
for arg in "$@"; do
    [[ "$arg" != *.lock ]] || exit 1
done
pid=""
while [[ $# -gt 0 ]]; do
    if [[ "$1" == -p ]]; then pid="$2"; break; fi
    shift
done
[[ -n "$pid" ]] || exit 1
for file in "${FAKE_STATE:?}"/*.pid; do
    [[ -e "$file" ]] || continue
    if [[ "$(<"$file")" == "$pid" ]]; then
        label="${file##*/}"
        label="${label%.pid}"
        printf 'p%s\nn%s\n' "$pid" "$(<"${FAKE_STATE}/$label.binary")"
        exit 0
    fi
done
exit 1
EOF

cat > "$FAKE_BIN/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
url="${!#}"
case "$url" in
    *api/runtime/active-chat-runs) printf '{"count":0}\n' ;;
    *8077/api/meta)
        file="${FAKE_STATE:?}/com.honeclaw.source.web.binary"
        [[ -f "$file" ]] || exit 22
        binary="$(<"$file")"
        if [[ "$binary" == *'/releases/source/'* ]]; then
            [[ "${FAKE_FAIL_NEW_WEB:-0}" != 1 ]] || exit 22
            revision="$(basename "$(dirname "$binary")")"
            printf '{"build":{"git_sha":"%s"}}\n' "$revision"
        else
            printf '{"build":{"git_sha":"old"}}\n'
        fi
        ;;
    *8088/) printf 'ok\n' ;;
    *) exit 22 ;;
esac
EOF

chmod 0755 "$FAKE_BIN/launchctl" "$FAKE_BIN/ps" "$FAKE_BIN/lsof" "$FAKE_BIN/curl"

reset_old_state() {
    rm -f "$STATE"/*.loaded "$STATE"/*.pid "$STATE"/*.binary "$STATE/events" "$STATE/counter"
    : > "$STATE/com.honeclaw.source.web.loaded"
    printf '41\n' > "$STATE/com.honeclaw.source.web.pid"
    printf '%s\n' "$PROJECT/old/hone-console-page" > "$STATE/com.honeclaw.source.web.binary"
    : > "$STATE/com.honeclaw.source.discord.loaded"
    printf '42\n' > "$STATE/com.honeclaw.source.discord.pid"
    printf '%s\n' "$PROJECT/old/hone-discord" > "$STATE/com.honeclaw.source.discord.binary"
    : > "$PROJECT/data/logs/hone-console-page-source.log"
    : > "$PROJECT/data/logs/hone-console-page-source.err.log"
    : > "$PROJECT/data/logs/hone-discord-source.log"
    : > "$PROJECT/data/logs/hone-discord-source.err.log"
}

run_deploy() {
    env PATH="$FAKE_BIN:$PATH" FAKE_STATE="$STATE" HONE_DEPLOY_TEST_MODE=1 "$@" \
        "$DEPLOY_SCRIPT" --project-root "$PROJECT" --revision "$REVISION" \
        --skip-build --allow-unpushed --startup-timeout 2 --drain-timeout 2 \
        --poll-interval 0.1
}

run_deploy_strict() {
    env PATH="$FAKE_BIN:$PATH" FAKE_STATE="$STATE" HONE_DEPLOY_TEST_MODE=1 "$@" \
        "$DEPLOY_SCRIPT" --project-root "$PROJECT" --revision "$REVISION" \
        --skip-build --startup-timeout 2 --drain-timeout 2 --poll-interval 0.1
}

assert_old_state() {
    [[ "$(<"$STATE/com.honeclaw.source.web.binary")" == "$PROJECT/old/hone-console-page" ]]
    [[ "$(<"$STATE/com.honeclaw.source.discord.binary")" == "$PROJECT/old/hone-discord" ]]
}

reset_old_state
run_deploy env
grep -q "/releases/source/$REVISION/hone-console-page" "$STATE/com.honeclaw.source.web.binary"
grep -q "/releases/source/$REVISION/hone-discord" "$STATE/com.honeclaw.source.discord.binary"
grep -q "remove com.honeclaw.source.web" "$STATE/events"
grep -q "submit com.honeclaw.source.web" "$STATE/events"
grep -q "ps 41" "$STATE/events"
grep -q "hone-console-page.lock" "$STATE/events"

reset_old_state
if run_deploy_strict env; then
    echo "expected unpushed revision refusal" >&2
    exit 1
fi
[[ ! -f "$STATE/events" ]]

printf 'agent:\n  runner: opencode_acp\n' > "$PROJECT/config.yaml"
reset_old_state
if run_deploy env; then
    echo "expected dirty worktree refusal" >&2
    exit 1
fi
[[ ! -f "$STATE/events" ]]
git -C "$PROJECT" restore config.yaml

reset_old_state
if run_deploy env FAKE_FAIL_NEW_WEB=1; then
    echo "expected new web readiness failure" >&2
    exit 1
fi
assert_old_state
grep -q "remove com.honeclaw.source.web" "$STATE/events"

reset_old_state
if run_deploy env FAKE_FAIL_NEW_DISCORD=1; then
    echo "expected new Discord readiness failure" >&2
    exit 1
fi
assert_old_state
grep -q "remove com.honeclaw.source.discord" "$STATE/events"

echo "source runtime deploy contract: ok"
