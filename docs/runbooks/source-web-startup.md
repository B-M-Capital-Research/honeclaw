# Runbook: Source Web Startup

Last updated: 2026-08-03

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

### Local user UI without SMS

The public user UI keeps real SMS/email authentication by default. For a local source checkout where those providers are intentionally unconfigured, opt in to the local test account when starting the backend:

```bash
HONE_PUBLIC_DEV_LOGIN=true cargo run -p hone-cli -- start --build
```

When and only when deployment mode and cloud mode are both `local`, the login card shows **Enter local test account / 进入本地测试账号**. The backend creates or reuses a non-production test identity and issues the normal HttpOnly session cookie; the frontend does not fabricate authentication. The route returns `404` when the flag is absent, deployment mode is not local, or cloud mode is not local. Never add this variable to production service environments.

To exercise administrator-only local workflows such as decision-brain evidence review, opt in separately:

```bash
HONE_PUBLIC_DEV_LOGIN=true HONE_PUBLIC_DEV_ADMIN=true cargo run -p hone-cli -- start --build
```

`HONE_PUBLIC_DEV_ADMIN` is ignored unless the same local/local/dev-login gates pass. Local backend startup and each local test login synchronize the dedicated test account to the flag: restarting or logging in without the admin flag removes that test-only administrator role, including for an existing browser session. Never set either development flag in a production service environment.

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
2. builds Web, Discord, and MCP under the checkout-local `target/source-runtime/` with the root `source-runtime` Cargo profile (`debug=1`, `incremental=false`), compile-time Git SHA/build timestamp, and the closed provenance kind `direct_source_runtime`;
3. copies them into the immutable `data/releases/source/<git-sha>/` package and verifies its SHA-256/build-source/profile manifest;
4. waits for active chats to drain;
5. validates the configured Codex/OpenCode commands on an explicit persistent runtime `PATH`; this path excludes turn-local `.codex/tmp` and `.cache/codex-runtimes` entries even when the deployment is invoked from Codex Desktop;
6. rejects an unknown owner of port `8077`, then removes either the managed Web/Discord jobs or the legacy `com.honeclaw.source.runtime` supervisor; any captured child reparented to PID 1 is executable-reverified, sent TERM, and only then subject to bounded exact-PID KILL escalation before locks are checked;
7. atomically installs the revision-bound Web/Discord plists under `~/Library/LaunchAgents` and bootstraps those exact files, so the canary and next login use identical working-directory/environment semantics;
8. requires `/api/meta.build.git_sha` and `/api/meta.build.source=direct_source_runtime` plus ports `8077/8088`, then requires a fresh Discord login when Discord was previously running;
9. verifies the running executable paths, moves the legacy supervisor plist to the recoverable `.disabled-by-hone-source-deploy` name, and updates the `current` symlink only after success;
10. records the replaced `current` as `previous`, then prunes only older strictly recognized release directories after rollback is disarmed. Unknown names, symlinks, unexpected contents, and cleanup failures are retained.

One exit trap owns rollback. Any failure after the old runtime is stopped removes all partially started managed jobs, restores and bootstraps the previous managed plists, or restores and bootstraps the legacy supervisor plist, then verifies the services that were running before deployment. A loaded-but-crashed launchd job is still removed even when it no longer has a PID. The disabled legacy plist is retained as a rollback asset; do not load it alongside the managed Web plist because both would compete for the same backend ports.

Use `--allow-unpushed` only for an explicitly accepted local canary. `--skip-build` reuses exact binaries from `target/source-runtime/` and is for the isolated CI contract or a separately verified build, not the normal deployment path. Frontend Vite processes on `3000/3001` remain independent and are not restarted by this command.

## Build Storage Policy

Every worktree keeps its own writable Cargo `target/`. Do not point two worktrees at one shared target: concurrent builds can overwrite same-named outputs and invalidate the revision-to-binary proof used by direct deployment. The repository instead bounds the high-churn non-release lanes:

- `[profile.dev]`: line-level debug information and no incremental state for ordinary build/check/run work;
- `[profile.source-runtime]`: line-level debug information, no incremental state, output under `target/source-runtime/`;
- `[profile.test]`: line-level debug information, no incremental state, while the normal `cargo test ...` command remains unchanged.

The source release store keeps the verified `current` and `previous` revisions. Inspect both with:

```bash
readlink data/releases/source/current
readlink data/releases/source/previous
du -sh target data/releases/source
```

Old target trees are rebuildable caches, but clean only an exact checkout after verifying no running process executes from that target:

```bash
lsof +D /absolute/path/to/checkout/target
cargo clean --manifest-path /absolute/path/to/checkout/Cargo.toml
```

Do not delete a whole checkout to clean artifacts, and do not delete `data/releases/source/current` or `previous` manually while a managed runtime is active. A future cross-worktree content-addressed compiler cache may reduce repeated dependency compilation further; it is a separate optimization and must not change the per-worktree output boundary.

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
readlink data/releases/source/previous
```

`build.git_sha` must equal the requested revision, `build.source` must be `direct_source_runtime`, and `build.binary_sha256` must be nonempty. Startup logs must record the same bounded Git SHA/timestamp/profile/source/hash line. `acp_profiles` may be empty until a real adapter connection initializes; after a Codex/OpenCode turn it must report the detected adapter version, selected dialect, compatibility status, detection time, and runner build SHA without paths or credentials. The matching dialect-selection log must include the actual adapter version/status and, for Codex ACP, the companion Codex CLI version/status.

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
