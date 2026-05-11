# Runbook: Desktop Dev Runtime Isolation

Last updated: 2026-05-10

## Why This Exists

- Desktop UI development and backend/channel runtime development have different restart needs.
- Backend and channel processes should stay stable while the desktop shell or frontend hot reloads.
- Source checkout runtime startup uses the local CLI build path; desktop development uses explicit Tauri commands.
- Keeping these lanes separate avoids duplicate channel listeners, stale sidecar processes, and unnecessary Rust rebuild churn.

For the validated macOS release-app operating procedure, see [desktop-release-app-runtime.md](./desktop-release-app-runtime.md).

## Lanes

### Stable Runtime Lane

Start backend and enabled channel listeners from the source checkout:

```bash
cargo run -p hone-cli -- start --build
```

This builds the local CLI/runtime binaries, generates `data/runtime/effective-config.yaml`, starts `hone-console-page`, starts enabled channels, and writes `data/runtime/current.pid`.

### Desktop Shell Lane

Use this when validating bundled desktop sidecar behavior:

```bash
bun run tauri:prep:dev -- --skip-dev-command
bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json
```

Use this when the backend is already running in the stable runtime lane and you only want the desktop shell connected to it:

```bash
bun run tauri:prep:dev -- --skip-dev-command --shell-only
bunx tauri dev --config bins/hone-desktop/tauri.generated.conf.json
```

### Web Frontend Lane

Use these in separate terminals when browser UI hot reload is needed. Prefer the CLI wrapper so the admin and user surfaces use the same ports and backend URL defaults as the installed path:

```bash
cargo run -p hone-cli -- web admin-ui --dev
cargo run -p hone-cli -- web user-ui --dev
```

Direct Bun scripts remain available for frontend-only work: `bun run dev:web` and `bun run dev:web:public`.

## Stop And Restart

- Stop foreground runtime or frontend processes with `Ctrl-C`.
- If a background runtime remains, inspect `data/runtime/current.pid` and confirm the process command line before terminating it.
- The admin `restart_hone` tool restarts through the source CLI path and writes logs to `data/logs/restart.log`.

## Target Directory Policy

- Use one explicit `CARGO_TARGET_DIR` for local dev when disk usage matters.
- Prefer a cache directory outside the repo, for example `~/Library/Caches/honeclaw/target` on macOS.
- Keep target-triple-specific sidecar builds for packaging or dedicated validation rather than daily UI work.

Example:

```bash
export CARGO_TARGET_DIR="$HOME/Library/Caches/honeclaw/target"
cargo run -p hone-cli -- start --build
```

## Incident Stopgap: Enable Feishu On A Test Machine

Use the stable runtime lane and disable Feishu scheduler only when live messages must not be starved by overdue cron jobs:

```bash
HONE_FEISHU_DISABLE_SCHEDULER=1 cargo run -p hone-cli -- start --build
```

Then start the public frontend if local user-side smoke tests need it:

```bash
cargo run -p hone-cli -- web user-ui --dev
```

After Feishu comes online, watch logs for:

- Runner bootstrap failures such as Codex / adapter version rejection.
- Heartbeat parsing failures such as unstructured JSON output.
- Channel send-path failures from Feishu API responses.

Re-enable scheduler delivery by restarting without `HONE_FEISHU_DISABLE_SCHEDULER`.
