#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DEPLOY_SCRIPT="$REPO_ROOT/scripts/deploy_source_runtime.sh"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

PROJECT="$TMP_ROOT/project"
FAKE_BIN="$TMP_ROOT/fake-bin"
STATE="$TMP_ROOT/state"
LAUNCH_AGENTS="$TMP_ROOT/launch-agents"
mkdir -p "$PROJECT/data/logs" "$PROJECT/data/runtime/locks" "$PROJECT/skills" \
    "$PROJECT/target/debug" "$PROJECT/old" "$FAKE_BIN" "$STATE" "$LAUNCH_AGENTS"
printf 'agent:\n  runner: codex_acp\n' > "$PROJECT/config.yaml"
touch "$PROJECT/skills/.gitkeep"
printf 'data/\ntarget/\nold/\n' > "$PROJECT/.gitignore"
for binary in hone-console-page hone-discord hone-mcp; do
    printf '#!/bin/sh\nexit 0\n' > "$PROJECT/target/debug/$binary"
    chmod 0755 "$PROJECT/target/debug/$binary"
done
for binary in hone-cli hone-console-page hone-discord; do
    printf '#!/bin/sh\nexit 0\n' > "$PROJECT/old/$binary"
    chmod 0755 "$PROJECT/old/$binary"
done
for command_name in codex codex-acp opencode; do
    printf '#!/bin/sh\nprintf "%%s fake-version\\n" "$(basename "$0")"\n' > "$FAKE_BIN/$command_name"
    chmod 0755 "$FAKE_BIN/$command_name"
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
        if [[ -f "$state/$label.children" ]]; then
            while IFS= read -r child_label; do
                if [[ "$label" != com.honeclaw.source.runtime || "${FAKE_ORPHAN_LEGACY_CHILDREN:-0}" != 1 ]]; then
                    rm -f "$state/$child_label.pid" "$state/$child_label.ppid" "$state/$child_label.binary"
                fi
            done < "$state/$label.children"
        fi
        rm -f "$state/$label.loaded" "$state/$label.pid" "$state/$label.binary"
        rm -f "$state/$label.children"
        printf 'remove %s\n' "$label" >> "$state/events"
        ;;
    bootstrap)
        shift
        plist="${1:?}"
        label="$(basename "$plist" .plist)"
        counter=200
        [[ ! -f "$state/counter" ]] || counter="$(<"$state/counter")"
        counter=$((counter + 1))
        printf '%s\n' "$counter" > "$state/counter"
        : > "$state/$label.loaded"
        printf '%s\n' "$counter" > "$state/$label.pid"
        if [[ "$label" == com.honeclaw.source.runtime ]]; then
            printf '%s\n' "${FAKE_PROJECT:?}/old/hone-cli" > "$state/$label.binary"
            printf '%s\n' legacy.web legacy.discord > "$state/$label.children"
            printf '%s\n' "$((counter + 1))" > "$state/legacy.web.pid"
            printf '%s\n' "$counter" > "$state/legacy.web.ppid"
            printf '%s\n' "${FAKE_PROJECT}/old/hone-console-page" > "$state/legacy.web.binary"
            printf '%s\n' "$((counter + 2))" > "$state/legacy.discord.pid"
            printf '%s\n' "$counter" > "$state/legacy.discord.ppid"
            printf '%s\n' "${FAKE_PROJECT}/old/hone-discord" > "$state/legacy.discord.binary"
        else
            binary="$(grep -o '<key>ProgramArguments</key><array><string>[^<]*' "$plist" | sed 's#.*<string>##')"
            stdout_log="$(grep -o '<key>StandardOutPath</key><string>[^<]*' "$plist" | sed 's#.*<string>##')"
            printf '%s\n' "$binary" > "$state/$label.binary"
            if [[ "$binary" == *'/releases/source/'* ]] \
                && { [[ "$label" == *web* && "${FAKE_FAIL_NEW_WEB:-0}" == 1 ]] \
                    || [[ "$label" == *discord* && "${FAKE_FAIL_NEW_DISCORD:-0}" == 1 ]]; }; then
                rm -f "$state/$label.pid"
            elif [[ "$label" == *discord* ]]; then
                printf 'Discord 已登录\n' >> "$stdout_log"
            fi
        fi
        printf 'bootstrap %s\n' "$label" >> "$state/events"
        ;;
    *) exit 2 ;;
esac
EOF

cat > "$FAKE_BIN/kill" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
signal="${1:?}"
pid="${2:?}"
printf 'kill %s %s\n' "$signal" "$pid" >> "${FAKE_STATE:?}/events"
for file in "${FAKE_STATE}"/*.pid; do
    [[ -e "$file" ]] || continue
    if [[ "$(<"$file")" == "$pid" ]]; then
        label="${file##*/}"
        label="${label%.pid}"
        rm -f "${FAKE_STATE}/$label.pid" "${FAKE_STATE}/$label.ppid" \
            "${FAKE_STATE}/$label.binary"
        exit 0
    fi
done
exit 1
EOF

cat > "$FAKE_BIN/ps" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == -axo ]]; then
    for file in "${FAKE_STATE:?}"/*.ppid; do
        [[ -e "$file" ]] || continue
        label="${file##*/}"
        label="${label%.ppid}"
        [[ -f "${FAKE_STATE}/$label.pid" ]] || continue
        printf '%s %s\n' "$(<"${FAKE_STATE}/$label.pid")" "$(<"$file")"
    done
    exit 0
fi
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
if [[ "$*" == *'-iTCP:8077'* ]]; then
    if [[ -n "${FAKE_UNMANAGED_PID:-}" ]]; then
        printf 'p%s\n' "$FAKE_UNMANAGED_PID"
        exit 0
    fi
    for label in com.honeclaw.source.web legacy.web; do
        if [[ -f "${FAKE_STATE}/$label.pid" ]]; then
            printf 'p%s\n' "$(<"${FAKE_STATE}/$label.pid")"
            exit 0
        fi
    done
    exit 1
fi
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
        [[ -f "$file" ]] || file="${FAKE_STATE:?}/legacy.web.binary"
        [[ -f "$file" ]] || exit 22
        binary="$(<"$file")"
        if [[ "$binary" == *'/releases/source/'* ]]; then
            [[ "${FAKE_FAIL_NEW_WEB:-0}" != 1 ]] || exit 22
            revision="$(basename "$(dirname "$binary")")"
            printf '{"build":{"git_sha":"%s","source":"direct_source_runtime"}}\n' "$revision"
        else
            printf '{"build":{"git_sha":"old"}}\n'
        fi
        ;;
    *8088/) printf 'ok\n' ;;
    *) exit 22 ;;
esac
EOF

chmod 0755 "$FAKE_BIN/launchctl" "$FAKE_BIN/kill" "$FAKE_BIN/ps" \
    "$FAKE_BIN/lsof" "$FAKE_BIN/curl"

write_fake_managed_plist() {
    local label="$1" binary="$2" stdout_log="$3" target="$4"
    printf '<plist><dict><key>Label</key><string>%s</string><key>ProgramArguments</key><array><string>%s</string></array><key>EnvironmentVariables</key><dict><key>PATH</key><string>%s:/usr/bin:/bin</string></dict><key>StandardOutPath</key><string>%s</string></dict></plist>\n' \
        "$label" "$binary" "$FAKE_BIN" "$stdout_log" > "$target"
}

reset_old_state() {
    rm -f "$STATE"/*.loaded "$STATE"/*.pid "$STATE"/*.ppid "$STATE"/*.binary \
        "$STATE"/*.children "$STATE/events" "$STATE/counter"
    rm -f "$LAUNCH_AGENTS"/*
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
    write_fake_managed_plist com.honeclaw.source.web "$PROJECT/old/hone-console-page" \
        "$PROJECT/data/logs/hone-console-page-source.log" \
        "$LAUNCH_AGENTS/com.honeclaw.source.web.plist"
    write_fake_managed_plist com.honeclaw.source.discord "$PROJECT/old/hone-discord" \
        "$PROJECT/data/logs/hone-discord-source.log" \
        "$LAUNCH_AGENTS/com.honeclaw.source.discord.plist"
}

reset_legacy_state() {
    reset_old_state
    rm -f "$STATE/com.honeclaw.source.web.loaded" "$STATE/com.honeclaw.source.web.pid" \
        "$STATE/com.honeclaw.source.web.binary" "$STATE/com.honeclaw.source.discord.loaded" \
        "$STATE/com.honeclaw.source.discord.pid" "$STATE/com.honeclaw.source.discord.binary"
    : > "$STATE/com.honeclaw.source.runtime.loaded"
    printf '40\n' > "$STATE/com.honeclaw.source.runtime.pid"
    printf '%s\n' "$PROJECT/old/hone-cli" > "$STATE/com.honeclaw.source.runtime.binary"
    printf '%s\n' legacy.web legacy.discord > "$STATE/com.honeclaw.source.runtime.children"
    printf '41\n' > "$STATE/legacy.web.pid"
    printf '40\n' > "$STATE/legacy.web.ppid"
    printf '%s\n' "$PROJECT/old/hone-console-page" > "$STATE/legacy.web.binary"
    printf '42\n' > "$STATE/legacy.discord.pid"
    printf '40\n' > "$STATE/legacy.discord.ppid"
    printf '%s\n' "$PROJECT/old/hone-discord" > "$STATE/legacy.discord.binary"
    printf '<plist>legacy</plist>\n' > "$LAUNCH_AGENTS/com.honeclaw.source.runtime.plist"
}

run_deploy() {
    env PATH="$FAKE_BIN:$PATH" FAKE_STATE="$STATE" FAKE_PROJECT="$PROJECT" \
        HONE_DEPLOY_TEST_MODE=1 HONE_DEPLOY_KILL_COMMAND="$FAKE_BIN/kill" \
        HONE_DEPLOY_LAUNCH_AGENT_DIR="$LAUNCH_AGENTS" \
        HONE_SOURCE_RUNTIME_PATH="$FAKE_BIN:/usr/bin:/bin" "$@" \
        "$DEPLOY_SCRIPT" --project-root "$PROJECT" --revision "$REVISION" \
        --skip-build --allow-unpushed --startup-timeout 2 --drain-timeout 2 \
        --poll-interval 0.1 --terminate-grace 1
}

run_deploy_strict() {
    env PATH="$FAKE_BIN:$PATH" FAKE_STATE="$STATE" FAKE_PROJECT="$PROJECT" \
        HONE_DEPLOY_TEST_MODE=1 HONE_DEPLOY_KILL_COMMAND="$FAKE_BIN/kill" \
        HONE_DEPLOY_LAUNCH_AGENT_DIR="$LAUNCH_AGENTS" \
        HONE_SOURCE_RUNTIME_PATH="$FAKE_BIN:/usr/bin:/bin" "$@" \
        "$DEPLOY_SCRIPT" --project-root "$PROJECT" --revision "$REVISION" \
        --skip-build --startup-timeout 2 --drain-timeout 2 --poll-interval 0.1 \
        --terminate-grace 1
}

assert_old_state() {
    [[ "$(<"$STATE/com.honeclaw.source.web.binary")" == "$PROJECT/old/hone-console-page" ]]
    [[ "$(<"$STATE/com.honeclaw.source.discord.binary")" == "$PROJECT/old/hone-discord" ]]
}

reset_old_state
run_deploy env
grep -q '"build_source":"direct_source_runtime"' "$PROJECT/data/releases/source/$REVISION/manifest.json"
grep -q "/releases/source/$REVISION/hone-console-page" "$STATE/com.honeclaw.source.web.binary"
grep -q "/releases/source/$REVISION/hone-discord" "$STATE/com.honeclaw.source.discord.binary"
grep -q "remove com.honeclaw.source.web" "$STATE/events"
grep -q "bootstrap com.honeclaw.source.web" "$STATE/events"
grep -q "ps 41" "$STATE/events"
grep -q "hone-console-page.lock" "$STATE/events"
grep -q "$REVISION/hone-console-page" "$LAUNCH_AGENTS/com.honeclaw.source.web.plist"
grep -q "$REVISION/hone-discord" "$LAUNCH_AGENTS/com.honeclaw.source.discord.plist"
! grep -q '/.codex/tmp/' "$LAUNCH_AGENTS/com.honeclaw.source.discord.plist"
plist_runtime_path="$(grep -o '<key>PATH</key><string>[^<]*' "$LAUNCH_AGENTS/com.honeclaw.source.discord.plist" | sed 's#.*<string>##')"
PATH="$plist_runtime_path" codex --version >/dev/null
if command -v plutil >/dev/null 2>&1; then
    plutil -lint "$LAUNCH_AGENTS/com.honeclaw.source.web.plist" >/dev/null
    plutil -lint "$LAUNCH_AGENTS/com.honeclaw.source.discord.plist" >/dev/null
fi

reset_legacy_state
run_deploy env
grep -q "remove com.honeclaw.source.runtime" "$STATE/events"
grep -q "/releases/source/$REVISION/hone-console-page" "$STATE/com.honeclaw.source.web.binary"
[[ ! -f "$LAUNCH_AGENTS/com.honeclaw.source.runtime.plist" ]]
[[ -f "$LAUNCH_AGENTS/com.honeclaw.source.runtime.plist.disabled-by-hone-source-deploy" ]]

reset_legacy_state
run_deploy env FAKE_ORPHAN_LEGACY_CHILDREN=1
grep -q 'kill -TERM 41' "$STATE/events"
grep -q 'kill -TERM 42' "$STATE/events"

reset_legacy_state
if run_deploy env FAKE_FAIL_NEW_WEB=1; then
    echo "expected legacy-to-managed Web readiness failure" >&2
    exit 1
fi
grep -q "bootstrap com.honeclaw.source.runtime" "$STATE/events"
[[ -f "$LAUNCH_AGENTS/com.honeclaw.source.runtime.plist" ]]
[[ ! -f "$LAUNCH_AGENTS/com.honeclaw.source.runtime.plist.disabled-by-hone-source-deploy" ]]

reset_old_state
rm -f "$STATE/com.honeclaw.source.web.loaded" "$STATE/com.honeclaw.source.web.pid" \
    "$STATE/com.honeclaw.source.web.binary" "$STATE/com.honeclaw.source.discord.loaded" \
    "$STATE/com.honeclaw.source.discord.pid" "$STATE/com.honeclaw.source.discord.binary"
if run_deploy env FAKE_UNMANAGED_PID=99; then
    echo "expected unmanaged port-owner refusal" >&2
    exit 1
fi
[[ ! -f "$STATE/events" ]] || ! grep -Eq '^(remove|bootstrap) ' "$STATE/events"

reset_old_state
ephemeral_path="$TMP_ROOT/user/.codex/tmp/turn/bin"
mkdir -p "$ephemeral_path"
if run_deploy env HONE_SOURCE_RUNTIME_PATH="$ephemeral_path:$FAKE_BIN:/usr/bin:/bin"; then
    echo "expected ephemeral persistent PATH refusal" >&2
    exit 1
fi
[[ ! -f "$STATE/events" ]] || ! grep -Eq '^(remove|bootstrap) ' "$STATE/events"
rm -rf "$TMP_ROOT/user/.codex"

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
grep -q "bootstrap com.honeclaw.source.web" "$STATE/events"

reset_old_state
if run_deploy env FAKE_FAIL_NEW_DISCORD=1; then
    echo "expected new Discord readiness failure" >&2
    exit 1
fi
assert_old_state
grep -q "bootstrap com.honeclaw.source.discord" "$STATE/events"

echo "source runtime deploy contract: ok"
