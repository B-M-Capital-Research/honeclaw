# Runbook: Desktop Release App Runtime

Last updated: 2026-04-14

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

The expected bundle path on macOS is:

```bash
/Users/ecohnoch/Library/Caches/honeclaw/target/release/bundle/macos/Hone Financial.app
```

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

### `launch.sh --release` vs direct `.app` launch

- `./launch.sh --release` is useful as a build-and-run helper
- but the most reliable macOS runtime shape is still the packaged `.app`
- if there is ever a discrepancy, prefer the `.app` bundle path documented above

## Recommended Team Habit

For a stable desktop session on this checkout:

1. Build the release app bundle
2. Start the `.app` executable with the pinned repo-local env vars
3. Leave it running during normal work
4. Rebuild and restart only when you intentionally want new desktop code to take effect

That is the current best-practice path for honeclaw desktop usage on macOS.
