# Runbook: Source Web Startup

Last updated: 2026-05-10

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

In separate terminals, start the frontends:

```bash
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH bun run dev:web
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH bun run dev:web:public
```

Why this shape matters:

- `hone-cli start --build` builds the Rust runtime binaries before starting services.
- The first cold build can take several minutes; later starts reuse the Cargo target dir.
- The CLI starts the backend first, waits for `/api/meta`, then starts enabled channel listeners.
- Vite frontends stay as separate foreground processes, so frontend crashes do not silently tear down the runtime backend.

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
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH bun run dev:web
env PATH=/opt/homebrew/bin:$HOME/.bun/bin:$PATH bun run dev:web:public
```

Notes:

- Running `bun install` may be harmless, but it may report "no changes" and leave the code-signing problem unchanged.
- Re-signing the Rollup native addon alone may not fix the mismatch if the wrong host Node remains first in `PATH`.
- Prefer changing `PATH` for the startup command instead of deleting `node_modules` as a first response.

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
