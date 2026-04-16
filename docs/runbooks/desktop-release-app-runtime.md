# Runbook: Desktop Release App Runtime

Last updated: 2026-04-16

## Why This Exists

- We need one desktop startup mode that stays stable during daily use
- The requirement is:
  - the desktop app should keep running normally
  - normal code edits must not restart or disturb the running app
  - the app must keep using the repo-local runtime data and config under this checkout
- On macOS, the most reliable path is to run the packaged Tauri `.app` bundle, not `tauri dev`, and not the naked `target/release/hone-desktop` binary

## Recommended Mode

Use the release app bundle as the long-running desktop lane.

This mode gives us four guarantees:

1. The desktop host is not started by `tauri dev`, so Rust file watching is not active
2. Editing `.rs`, frontend source, or workflow files does not auto-restart the already-running desktop app
3. The app can still be pinned to this repo's `data/`, `skills/`, and runtime config through env overrides
4. The frontend assets are loaded in a Tauri-safe way through relative paths, so the app does not fall into the previous white-screen failure under `file://`

## When To Use This

- Use this mode when the desktop app should stay up for a long session
- Use this mode when backend config, runtime data, and channels should keep pointing at this checkout
- Use this mode when you want to avoid all code-watch interference

Do not use this mode when you specifically need:

- frontend HMR
- `tauri dev`
- fast host-side Rust iteration with automatic rebuilds

For those cases, use the development-oriented desktop workflows separately.

## Why `.app` Is Better Than The Naked Release Binary

On macOS, the raw `target/release/hone-desktop` binary is not the most reliable runtime shape for the desktop shell.

The `.app` bundle is preferred because:

- WebKit resource loading behaves correctly with the app bundle context
- Tauri resource resolution is closer to the real packaged runtime
- the previous white-screen issue was tied to desktop asset loading under release startup, and the bundled app path is the stable end-state

In short:

- acceptable for debugging: raw release binary
- recommended for stable daily desktop runtime: `.app/Contents/MacOS/hone-desktop`

## Important Paths

These env vars decide where the release app reads and writes runtime state.

- `HONE_DATA_DIR`
  - main repo-local data root
  - recommended value: `/Users/ecohnoch/Desktop/honeclaw/data`
- `HONE_DESKTOP_DATA_DIR`
  - desktop runtime data root
  - keep it equal to `HONE_DATA_DIR` so the desktop app uses the same repo-local data
- `HONE_CONFIG_PATH`
  - main runtime config file
  - recommended value: `/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml`
- `HONE_USER_CONFIG_PATH`
  - user config override path
  - in this setup we intentionally point it at the same runtime config file
- `HONE_DESKTOP_CONFIG_DIR`
  - desktop-only runtime config directory
  - recommended value: `/Users/ecohnoch/Desktop/honeclaw/data/runtime/desktop-config`
- `HONE_SKILLS_DIR`
  - skills directory for this checkout
  - recommended value: `/Users/ecohnoch/Desktop/honeclaw/skills`
- `CARGO_TARGET_DIR`
  - shared Rust build output root
  - recommended value: `/Users/ecohnoch/Library/Caches/honeclaw/target`

If these are not pinned explicitly, the app may fall back to other default locations, which is not what we want for the honeclaw-local workflow.

## Build The Release App

Run the build from the repo root:

```bash
cd /Users/ecohnoch/Desktop/honeclaw

env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target \
  bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json
```

Notes:

- `bins/hone-desktop/tauri.generated.conf.json` now uses `bun run build:web:desktop` before packaging
- `build:web:desktop` sets `HONE_APP_RELATIVE_BASE=1`
- that forces the frontend build to emit `./assets/...` instead of `/assets/...`
- this is required for correct release-mode loading inside the Tauri app shell
- if the live desktop/backend runtime is already using `/Users/ecohnoch/Library/Caches/honeclaw/target/...`, any backend or channel rebuild must use the same `CARGO_TARGET_DIR`
- otherwise you can end up rebuilding repo-local `target/` while the running `.app` and helper processes keep using stale cache-target binaries

The expected bundle path on macOS is:

```bash
/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app
```

Current repo helpers now align with this cache target:

- `launch.sh --release` builds and launches the packaged `.app` executable on macOS
- `scripts/build_desktop.sh` also defaults to `/Users/<user>/Library/Caches/honeclaw/target`
- do not mix these helpers with the legacy `hone-financial/target` cache path

## Start The Release App

Run the executable inside the `.app` bundle with the repo-local runtime env:

```bash
env \
  HONE_DESKTOP_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data \
  HONE_DESKTOP_CONFIG_DIR=/Users/ecohnoch/Desktop/honeclaw/data/runtime/desktop-config \
  HONE_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_USER_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data \
  HONE_SKILLS_DIR=/Users/ecohnoch/Desktop/honeclaw/skills \
  /Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone\ Financial.app/Contents/MacOS/hone-desktop
```

This is the recommended long-running command.

## Start The Backend Lane

When the desktop is configured to use the local backend on `127.0.0.1:8077`, the backend lane must be restarted with the same repo-local env and the same cache target directory assumptions as the desktop lane.

Preferred foreground diagnostic launch:

```bash
env \
  HONE_WEB_PORT=8077 \
  HONE_CONSOLE_URL=http://127.0.0.1:8077 \
  HONE_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_USER_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data \
  HONE_DESKTOP_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data \
  HONE_DESKTOP_CONFIG_DIR=/Users/ecohnoch/Desktop/honeclaw/data/runtime/desktop-config \
  HONE_SKILLS_DIR=/Users/ecohnoch/Desktop/honeclaw/skills \
  /Users/ecohnoch/Library/Caches/honeclaw/target/release/hone-console-page
```

Preferred detached launch after the foreground smoke test is known-good:

```bash
env \
  RUST_LOG=warn \
  HONE_WEB_PORT=8077 \
  HONE_CONSOLE_URL=http://127.0.0.1:8077 \
  HONE_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_USER_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data \
  HONE_DESKTOP_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data \
  HONE_DESKTOP_CONFIG_DIR=/Users/ecohnoch/Desktop/honeclaw/data/runtime/desktop-config \
  HONE_SKILLS_DIR=/Users/ecohnoch/Desktop/honeclaw/skills \
  nohup /Users/ecohnoch/Library/Caches/honeclaw/target/release/hone-console-page \
    >> /Users/ecohnoch/Desktop/honeclaw/data/runtime/logs/backend_release_restart.log 2>&1 < /dev/null &
```

Operational note:

- if a detached launch exits immediately without creating a fresh `hone-console-page.lock`, do not keep guessing
- switch to the foreground diagnostic launch first and capture the first startup output
- only go back to a detached launch once the foreground path is confirmed healthy

## Expected Runtime Behavior

After startup:

- the desktop window should render normally
- the app should keep using `/Users/ecohnoch/Desktop/honeclaw/data`
- the app should not restart when `.rs`, frontend source, or other repo files change
- there should be no `tauri dev`, `bun --watch`, or `cargo watch` process in the runtime path

This mode is intentionally static:

- if code changes should take effect, rebuild and restart explicitly
- nothing should hot-reload the host automatically

## Verification Checklist

### 1. Confirm the app process is running

```bash
ps -axo pid=,ppid=,command= | rg '[h]one-desktop'
```

Expected shape:

- the command path should point into `Hone Financial.app/Contents/MacOS/hone-desktop`

### 2. Confirm no watch-driven desktop process exists

```bash
ps -axo pid=,ppid=,command= | rg '[t]auri dev|[b]un --watch|[c]argo watch'
```

Expected result:

- no relevant process should be listed

### 3. Confirm the backend still responds if you are using repo-local runtime services

```bash
curl http://127.0.0.1:8077/api/meta
```

### 4. Confirm the workflow runner still responds if it is part of the local session

```bash
curl http://127.0.0.1:3213/api/workflows
```

### 5. Confirm enabled channels are actually online

When the desktop app is using a remote backend, do not rely only on one startup log line or on the window being visible.

Check the backend status API directly:

```bash
curl http://127.0.0.1:8077/api/channels
```

Expected shape:

- `web` should be `running`
- each enabled channel such as `discord`, `feishu`, and `telegram` should report `running`
- a disabled channel such as `imessage` may legitimately report `disabled`

### 6. Confirm the session list API is not silently empty

The desktop can look "empty" even when the underlying session data is present if the backend session listing path is failing.

Check directly:

```bash
curl http://127.0.0.1:8077/api/users
```

Expected result:

- the response should be a populated JSON array when session history exists
- `[]` is only valid when the runtime really has no sessions
- if the workspace has historical data under `data/sessions/` or `data/sessions.sqlite3` but `/api/users` still returns `[]`, treat that as a backend bug, not as proof that the data directory is wrong or empty

### 7. Confirm a known session can still return history

If `/api/users` is populated, also spot-check one real session:

```bash
curl 'http://127.0.0.1:8077/api/history?session_id=<real-session-id>'
```

Expected result:

- `messages` should contain prior conversation content for that session
- if `/api/users` works but `/api/history` fails for known sessions, treat that as a separate history route bug

## Restart Policy

Treat this mode as a static runtime.

- code edits do not apply live
- to pick up desktop-side code changes, rebuild and restart the app intentionally
- to change backend behavior, restart the backend lane intentionally

This is a feature, not a limitation. It is exactly what keeps the running app insulated from ongoing code edits.

## Known Caveats

### Remote backend probe noise

- the desktop log may briefly record a remote `/api/meta` probe failure during startup
- in the validated release app path, this did not prevent the window from rendering or the app from staying up
- treat this as startup noise unless it becomes persistent

### Persistent `Connection refused` on `127.0.0.1:8077`

- if the desktop keeps reporting `request send failed` for `http://127.0.0.1:8077/api/meta`, first verify whether anything is actually listening on `8077`
- check with:

```bash
lsof -nP -iTCP:8077 -sTCP:LISTEN
curl http://127.0.0.1:8077/api/meta
```

- if nothing is listening, the problem is the backend lane, not the desktop bundle
- in the 2026-04-15 recovery, the release desktop app and `backend.json` were already correct, but the backend supervisor path had failed to keep `hone-console-page` bound to `8077`
- a direct long-lived launch with explicit env restored the service immediately:

```bash
env \
  HONE_WEB_PORT=8077 \
  HONE_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_USER_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml \
  HONE_DATA_DIR=/Users/ecohnoch/Desktop/honeclaw/data \
  HONE_SKILLS_DIR=/Users/ecohnoch/Desktop/honeclaw/skills \
  /Users/ecohnoch/Library/Caches/honeclaw/target/debug/hone-console-page
```

- once that backend is healthy again, the desktop log should switch from repeated probe failures to `remote backend connected: http://127.0.0.1:8077`
- after backend recovery, also verify `curl http://127.0.0.1:8077/api/channels` so you know the enabled channels really came back, not just the web API
- after backend recovery, also verify `curl http://127.0.0.1:8077/api/users`; if that still returns `[]` while the data files obviously exist, the backend has a session listing regression rather than a path problem

### Stale startup lock after the old process is already gone

- both desktop and backend lanes can fail startup because a `data/runtime/locks/*.lock` file still points at a dead pid
- before deleting a lock, verify the recorded pid is truly absent with `ps -p <pid>`
- if the pid is gone, removing the stale lock is safe and is usually required before restart
- do not delete a lock blindly when the pid is still alive; that risks double-starting the same lane

Typical examples from 2026-04-16:

- `hone-desktop.lock` pointed at dead pid `12535`, which made every new desktop launch abort as "already occupied"
- `hone-console-page.lock` can show the same failure mode after a backend crash or interrupted restart

### Supervisor caveat for remote backend mode

- if a supervisor or launcher path does not reliably preserve `HONE_WEB_PORT=8077`, `hone-console-page` can silently fall back to a random port
- that produces a misleading state where the desktop app is open, but remote mode still fails because it keeps probing `127.0.0.1:8077`
- when diagnosing this class of failure, prefer a startup path where the runtime env is explicit and inspectable

### `/api/users` returns `[]` even though `data/sessions.sqlite3` is populated

- this is a backend session-listing failure, not evidence that the repo-local `data/` path is wrong
- on 2026-04-16 the live database had dozens of sessions, but the UI looked empty because `/api/users` returned `[]`
- two failure modes mattered:
  - one unreadable sqlite row could poison the whole list operation if the backend aborted the entire listing on deserialize failure
  - some historical rows did not carry embedded `actor/session_identity`, but their `session_id` still contained enough information to recover the actor/session identity
- if this symptom returns:
  1. verify the data still exists with `sqlite3 data/sessions.sqlite3 'select count(*) from sessions;'`
  2. verify `data/sessions/` still contains historical JSON files
  3. call `curl http://127.0.0.1:8077/api/users`
  4. if the count is non-zero but the API still returns `[]`, restart the backend with the current fixed binary before touching the data directory

### Detached backend restart exits silently

- on 2026-04-16, one detached `nohup` restart path exited immediately without creating a new lock and without binding `8077`
- the same binary launched fine in the foreground with the same env and immediately restored `/api/users`
- if a detached restart fails without a clear log line:
  - do not assume the binary itself is broken
  - run the foreground diagnostic launch first
  - once it binds `8077`, confirm `/api/meta`, `/api/users`, `/api/history`, and `/api/channels`
  - only then convert it back to a detached or supervised lane

### Blank logs panel or `broken pipe` symptoms

- if the desktop window is visible but the logs panel is blank, or Codex runner starts surfacing `broken pipe`-style backend failures, do not assume the channel processes are the first problem
- first check these endpoints directly:

```bash
curl http://127.0.0.1:8077/api/meta
curl http://127.0.0.1:8077/api/logs
curl http://127.0.0.1:8077/api/channels
```

- in the 2026-04-15 incident, `/api/logs` was the failing path even though `/api/meta` and the desktop window could still come up
- the backend had an old panic path around multibyte plaintext log lines and malformed file content; after the fix, `/api/logs` should tolerate non-UTF-8 file bytes, multibyte plaintext, and a poisoned in-memory log buffer
- if these symptoms reappear after the fix is merged, the most likely operational cause is that you rebuilt repo-local `target/` but the live runtime is still serving binaries from `/Users/ecohnoch/Library/Caches/honeclaw/target/...`
- in that case, rebuild the backend/channel binaries with the same cache `CARGO_TARGET_DIR`, then restart the affected process

### Runtime config changed but process still is not `multi-agent`

- if `data/runtime/config_runtime.yaml` says `agent.runner: multi-agent` but the live channel/backend process still behaves like an older runner, verify that the process was restarted after a build that used the same target directory as the live runtime
- the 2026-04-15 recovery also exposed a code path where `HoneConfig::from_file()` had been reading only the base YAML without merging the runtime overlay
- after the fix, `from_file()` now loads the merged config, so runtime-only overrides should apply to channel/backend processes without requiring manual sync back into the base config file
- if the symptom ever returns, verify both the effective config file path and the actual binary path of the running process before changing more config

### `launch.sh --release` vs direct `.app` launch

- `./launch.sh --release` is useful as a build-and-run helper
- it now launches the same packaged `.app/Contents/MacOS/hone-desktop` path documented above
- `launch.sh` also records its own supervisor pid in `data/runtime/current.pid`, which is the contract used by `restart_hone` to stop the previous supervisor cleanly before relaunch
- if there is ever a discrepancy, prefer the `.app` bundle path documented above and verify `current.pid` still points to the active `launch.sh` supervisor

## Recommended Team Habit

For a stable desktop session on this checkout:

1. Build the release app bundle
2. Start the `.app` executable with the pinned repo-local env vars
3. Leave it running during normal work
4. Rebuild and restart only when you intentionally want new desktop code to take effect

That is the current best-practice path for honeclaw desktop usage on macOS.
