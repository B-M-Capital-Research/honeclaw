# Runbook: Source Web Startup

Last updated: 2026-08-02

This runbook covers starting the full local source checkout Web stack with the local CLI build path.
Use it when you need the backend, enabled channel listeners, admin Vite frontend, and public Vite frontend running from the latest local code.

## What The Source Web Stack Starts

- `hone-console-page` on the admin backend port, default `http://127.0.0.1:8077`.
- `hone-console-page` on the public backend port, default `http://127.0.0.1:8088`.
- Enabled channel listeners: iMessage, Discord, Feishu/Lark, and Telegram.
- Admin Vite frontend, default `http://127.0.0.1:3000`.
- Public Vite frontend, default `http://127.0.0.1:3001`.

Disabled channels are expected to log a startup message and then skip themselves. Treat that as normal when the matching `*.enabled=false` in `config.yaml`.

## Freshen Code First

Check the branch and worktree before pulling:

```bash
git status --short --branch
git pull --ff-only
```

If there are local changes, inspect them before pulling or restarting. Do not discard user edits just to free the runtime lane.

## Stop Old Runtime Owners

An already-open desktop app can own the same backend ports. The common symptom is `hone-desktop` or `hone-console-page` listening on `8077` and `8088`.

If the runtime is in a foreground terminal, stop it with `Ctrl-C`. Otherwise inspect the supervisor pid:

```bash
cat data/runtime/current.pid
```

Then inspect ports:

```bash
lsof -nP -iTCP:8077 -sTCP:LISTEN
lsof -nP -iTCP:8088 -sTCP:LISTEN
lsof -nP -iTCP:3000 -sTCP:LISTEN
lsof -nP -iTCP:3001 -sTCP:LISTEN
```

If a packaged desktop app still owns `8077/8088`, close the app or terminate that specific PID after confirming it is the stale owner.

## Start The Full Web Stack

Start the backend and enabled channels from source:

```bash
cargo run -p hone-cli -- start --build
```

In separate terminals, start the frontends through the CLI wrapper:

```bash
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH cargo run -p hone-cli -- web admin-ui --dev
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH cargo run -p hone-cli -- web user-ui --dev
```

Why this shape matters:

- `hone-cli start --build` builds the Rust runtime binaries before starting services.
- The first cold build can take several minutes; later starts reuse the Cargo target dir.
- The CLI starts the backend first, waits for `/api/meta`, then starts enabled channel listeners.
- `hone-cli web admin-ui --dev` and `hone-cli web user-ui --dev` keep Vite frontends as separate foreground processes, so frontend crashes do not silently tear down the runtime backend.

Direct Bun scripts remain supported when you want to bypass the CLI wrapper:

```bash
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH bun run dev:web
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH bun run dev:web:public
```

## Deploy One Reviewed Source Revision

Use the revision-bound deployment state machine when replacing a long-running source Web/Discord runtime. This is different from an ordinary foreground development start:

```bash
bash scripts/deploy_source_runtime.sh \
  --project-root /absolute/path/to/reviewed/checkout \
  --config /absolute/path/to/runtime/config.yaml \
  --data-dir /absolute/path/to/runtime/data \
  --skills-dir /absolute/path/to/runtime/skills \
  --revision "$(git rev-parse HEAD)"
```

The checkout may be a clean feature worktree while config/data/skills remain in the normal runtime checkout. The command:

1. refuses a dirty, unexpected, or unpushed revision by default;
2. builds Web, Discord, and MCP with compile-time Git SHA/build timestamp;
3. copies them into the immutable `data/releases/source/<git-sha>/` package and verifies its SHA-256 manifest;
4. waits for active chats to drain;
5. validates the configured Codex/OpenCode commands on an explicit persistent runtime `PATH`; this path excludes turn-local `.codex/tmp` and `.cache/codex-runtimes` entries even when the deployment is invoked from Codex Desktop;
6. rejects an unknown owner of port `8077`, then removes either the managed Web/Discord jobs or the legacy `com.honeclaw.source.runtime` supervisor; any captured child reparented to PID 1 is executable-reverified, sent TERM, and only then subject to bounded exact-PID KILL escalation before locks are checked;
7. atomically installs the revision-bound Web/Discord plists under `~/Library/LaunchAgents` and bootstraps those exact files, so the canary and next login use identical working-directory/environment semantics;
8. requires `/api/meta.build.git_sha` plus ports `8077/8088`, then requires a fresh Discord login when Discord was previously running;
9. verifies the running executable paths, moves the legacy supervisor plist to the recoverable `.disabled-by-hone-source-deploy` name, and updates the `current` symlink only after success.

One exit trap owns rollback. Any failure after the old runtime is stopped removes all partially started managed jobs, restores and bootstraps the previous managed plists, or restores and bootstraps the legacy supervisor plist, then verifies the services that were running before deployment. A loaded-but-crashed launchd job is still removed even when it no longer has a PID. The disabled legacy plist is retained as a rollback asset; do not load it alongside the managed Web plist because both would compete for the same backend ports.

Use `--allow-unpushed` only for an explicitly accepted local canary. `--skip-build` is for the isolated CI contract or a separately verified exact build, not the normal deployment path. Frontend Vite processes on `3000/3001` remain independent and are not restarted by this command.

## macOS Rollup Native Addon Failure

Symptom:

```text
Error: Cannot find module @rollup/rollup-darwin-arm64
ERR_DLOPEN_FAILED
code signature ... not valid for use in process: mapping process and mapped file (non-platform) have different Team IDs
```

Root cause:

- The Codex desktop environment may put `Codex.app`'s bundled Node ahead of Homebrew Node in `PATH`.
- Vite/Rollup loads a native optional dependency from `node_modules`.
- macOS can reject that native addon when the host Node process has a different Team ID than the mapped native file.

Confirm which Node is being used:

```bash
which node
codesign -dv "$(which node)"
codesign -dv node_modules/.bun/@rollup+rollup-darwin-arm64@*/node_modules/@rollup/rollup-darwin-arm64/rollup.darwin-arm64.node
```

Preferred fix:

```bash
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH cargo run -p hone-cli -- web admin-ui --dev
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH cargo run -p hone-cli -- web user-ui --dev
```

Notes:

- Running `bun install` may be harmless, but it may report "no changes" and leave the code-signing problem unchanged.
- Re-signing the Rollup native addon alone may not fix the mismatch if the wrong host Node remains first in `PATH`.
- Prefer changing `PATH` for the startup command instead of deleting `node_modules` as a first response.
- Direct `bun run dev:web` / `bun run dev:web:public` is still valid when you want the shortest frontend-only command.

## Verify Startup

Expected probes:

```bash
curl -fsS http://127.0.0.1:8077/api/meta
curl -I http://127.0.0.1:3000/
curl -I http://127.0.0.1:3001/
lsof -nP -iTCP:8077 -sTCP:LISTEN
lsof -nP -iTCP:8088 -sTCP:LISTEN
cat data/runtime/current.pid
```

For a revision-bound deployment, also require:

```bash
curl -fsS http://127.0.0.1:8077/api/meta
readlink data/releases/source/current
```

`build.git_sha` must equal the requested revision. `acp_profiles` may be empty until a real adapter connection initializes; after a Codex/OpenCode turn it must report the detected adapter version, selected dialect, compatibility status, detection time, and runner build SHA without paths or credentials.

Expected URLs:

- Admin backend/API: `http://127.0.0.1:8077`.
- Public backend/API: `http://127.0.0.1:8088`.
- Admin frontend: `http://127.0.0.1:3000`.
- Public frontend: `http://127.0.0.1:3001`.

## Stop

Stop foreground processes with `Ctrl-C` in each terminal. If a background runtime remains, inspect and terminate the recorded supervisor pid after confirming it is the stale Hone process:

```bash
cat data/runtime/current.pid
ps -p "$(cat data/runtime/current.pid)" -o pid,command
```
