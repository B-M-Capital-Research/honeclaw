# Runbook: Desktop Dev Runtime Isolation

Last updated: 2026-03-28

## Why This Exists

- Current desktop development mainly uses `./launch.sh --desktop`
- In this mode, `launch.sh` starts `bunx tauri dev`
- `tauri dev` watches Rust source files and rebuilds / relaunches the desktop host automatically
- The desktop host in bundled mode also starts the embedded web backend plus enabled channel sidecars
- As a result, changing `.rs` files during development can disturb the running desktop runtime and, in the current implementation, may leave duplicate or stale processes behind
- The current flow also builds overlapping Rust artifacts into multiple target trees, which makes the repo-local `target/` directory grow quickly

## Goal

We want a development model with two explicit guarantees:

1. Editing source files must not affect the running backend or channel processes unless we intentionally perform a full restart
2. Rust build artifacts should stop exploding inside the repo-local `target/` directory

## Available Modes

- `./launch.sh --desktop`
  - Current development mode
  - Starts `tauri dev`, so Rust source changes can rebuild / relaunch the desktop host
  - Desktop bundled mode still owns the embedded backend and enabled channels
- `./launch.sh --release`
  - Release desktop mode
  - Builds and starts the release desktop binary directly, without `tauri dev`
  - Normal source edits do not affect the already-running desktop process unless we intentionally restart it
  - The launcher pins `HONE_DESKTOP_DATA_DIR` to the repo-local `data/` directory and `HONE_DESKTOP_CONFIG_DIR` to `data/runtime/desktop-config/` by default, so release-mode desktop startup reuses project runtime data instead of silently switching to a separate app config/data root

## Current Problems

### 1. Runtime is coupled to Rust file watching

- `./launch.sh --desktop` starts `tauri dev`, not a static desktop binary
- `tauri dev` watches Rust code and relaunches `hone-desktop` when `.rs` files change
- In bundled mode, `hone-desktop` also owns the embedded web server and channel sidecars, so a desktop relaunch is not only a UI concern

### 2. Process stop semantics are weaker in desktop mode than in `launch.sh`

- `launch.sh` waits for known processes to exit and escalates to force-kill when needed
- Desktop sidecar management currently kills child processes but does not wait for clean exit before continuing
- This creates a restart overlap window where a new instance may be started before the old one is fully gone
- Historical investigation already linked duplicate consumer processes to repeated message handling

### 3. Bundled web runtime is too tightly embedded for daily dev

- The desktop bundled mode mixes three responsibilities into one relaunch boundary:
  - desktop shell
  - web backend
  - channel listeners
- That is convenient for smoke testing, but it is the wrong default for long-running multi-process development

### 4. `target/` growth is not only from "many rebuilds"

- The repo builds many workspace crates and binaries
- Desktop startup currently triggers overlapping build paths:
  - `cargo build` from `launch.sh`
  - `cargo build --target <triple>` from `scripts/prepare_tauri_sidecar.sh`
  - `cargo run` from `tauri dev`
- This commonly creates at least two debug artifact trees:
  - `target/debug`
  - `target/<target-triple>/debug`
- Incremental compilation caches for multiple binaries and crates then accumulate under both trees

## Recommended Target Model

Use a split development model instead of one "all-in-one bundled hot reload" loop.

### A. Stable runtime lane

This lane is the source of truth for backend and channel processes during daily development.

- Start backend and channels explicitly, outside the desktop shell hot-reload loop
- Treat this runtime as long-lived and unchanged until we intentionally restart it
- Do not let desktop shell rebuilds own runtime lifecycle

Recommended shape:

- `hone-console-page` + channel listeners run as independent managed processes
- They are started once and stay up until a manual full restart
- Desktop connects to them in `remote` mode during normal development

### B. Desktop shell lane

This lane is only for the desktop UI host itself.

- The desktop shell should be launched separately from the runtime lane
- For daily work, prefer a static desktop binary or explicit `cargo run -p hone-desktop` without file-watch orchestration
- Frontend HMR may still be used, but Rust host rebuilds should happen only when we manually restart the shell

### C. Bundled mode lane

Keep bundled mode, but reduce its role.

- Use bundled mode for integration checks, packaging checks, and sidecar lifecycle validation
- Do not use bundled mode as the default long-running dev workflow

## Concrete Workflow Recommendation

### Daily development default

Use this as the primary workflow.

1. Start the stable runtime once
   - Start `hone-console-page` and the enabled channel binaries with the normal non-desktop launcher path
   - Keep them running until we explicitly decide to restart the whole runtime
2. Start the frontend dev server separately
   - Run Vite for web UI changes
3. Start the desktop shell in `remote` mode
   - Desktop connects to the existing backend URL instead of starting bundled runtime
4. Edit code freely
   - Frontend changes refresh the UI
   - Rust source changes do not restart backend or channel processes automatically
5. When Rust runtime changes need to take effect
   - Perform one intentional full restart of the stable runtime

This is the only workflow that fully satisfies the requirement that code edits must not disturb running services unless we choose a whole restart.

If we specifically want a desktop build that is insulated from source edits while still reusing project-local runtime data, prefer `./launch.sh --release`.

### Bundled validation workflow

Use only when validating desktop-managed runtime behavior.

- Start desktop in bundled mode
- Verify desktop-side runtime start / stop behavior
- Exit after the specific validation is done
- Do not leave this mode running for long development sessions

## Required Implementation Direction

The workflow above implies the following code and script changes should be made in follow-up work.

### 1. Make remote-mode desktop development first-class

- Add a dedicated desktop dev entry that does not use `tauri dev` as the runtime owner
- The command should boot only what is needed for UI iteration
- Desktop should default to connecting to an already-running backend during development

Preferred direction:

- Introduce a "desktop shell only" dev command
- Introduce a separate "runtime only" dev command
- Make current `--desktop` behavior opt-in for bundled integration testing rather than the default recommendation

### 2. Remove automatic Rust watch from the long-running lane

- Do not use `tauri dev` as the always-on entrypoint for runtime work
- If `tauri dev` remains available, document it as a short-lived validation tool
- The default persistent dev command must run a static process tree

### 3. Strengthen process supervision in desktop mode

- Desktop-managed child stop must wait for process exit before spawning replacements
- If graceful stop times out, escalate in a controlled way
- Embedded web backend shutdown must have a real owned handle instead of fire-and-forget background tasks
- Runtime status must be derived from actual ownership, not only from optimistic startup logs

### 4. Collapse Rust artifact output into one dev target root

- Use one explicit `CARGO_TARGET_DIR` for all local dev entrypoints
- Avoid mixing default-target builds and extra `--target <triple>` builds unless cross-compilation is actually needed
- Ensure `launch.sh`, desktop preparation scripts, and direct cargo commands all use the same target root

Preferred direction:

- Pick one dev target root outside the repo worktree, for example under `~/Library/Caches` or another cache directory
- Keep repo-local `target/` out of the normal workflow
- If Tauri sidecar packaging still needs target-triple-specific artifacts, keep that as a packaging path, not the daily dev path

### 5. Stop rebuilding sidecar binaries on every desktop dev start

- Sidecar preparation should become explicit or incremental-aware
- Daily UI startup should not always rebuild every channel binary
- Channel binaries should be rebuilt only when those binaries actually changed or when we intentionally refresh the runtime lane

## Proposed Command Model

This section is the recommended end-state for scripts and operator habit.

- `./launch.sh`
  - Start stable backend + channel runtime only
- `./launch.sh --web`
  - Start stable runtime + Vite for browser console work
- `./launch.sh --desktop-remote`
  - Start desktop shell only, connected to the already-running backend
- `./launch.sh --desktop-bundled`
  - Start bundled desktop integration mode for short validation sessions
- `./launch.sh --release`
  - Start release desktop binary with project-local runtime data pinned through env overrides
- `./launch.sh restart-runtime`
  - Intentionally restart the stable backend + channel runtime

The important rule is:

- only `restart-runtime` should change the long-running backend / channel process set

## Target Size Control Policy

To keep local storage predictable, adopt all of the following together.

### Build output policy

- Use one shared `CARGO_TARGET_DIR` across all local dev commands
- Prefer host-native debug builds in daily development
- Reserve target-triple-specific sidecar builds for packaging or dedicated validation

### Cache policy

- Keep incremental compilation enabled for the single shared dev target root
- Periodically clear stale packaging-only artifacts, not the whole dev cache
- Avoid repo-local `target/` as the default storage location for heavy Rust artifacts

### Build trigger policy

- Do not rebuild all runtime binaries on every desktop shell start
- Do not rebuild sidecars just because the frontend dev server is started
- Separate "start" from "rebuild"

## Rollout Plan

### Phase 1. Workflow correction

- Stop recommending `./launch.sh --desktop` for long-running daily development
- Recommend split runtime + desktop remote mode instead
- Document bundled mode as validation-only

### Phase 2. Script split

- Add explicit commands for:
  - runtime only
  - desktop remote shell only
  - bundled validation
  - runtime restart
- Route all of them through one shared `CARGO_TARGET_DIR`

### Phase 3. Ownership hardening

- Refactor desktop bundled mode so embedded web server and channel sidecars have real shutdown ownership
- Wait for child exit before respawn
- Add startup guards that reject duplicate managed channel instances in bundled mode

### Phase 4. Artifact cleanup

- Move daily dev artifacts out of repo-local `target/`
- Remove redundant rebuilds from desktop startup
- Keep cross-target sidecar builds only where they are truly needed

## Success Criteria

The migration is successful only if all of the following are true.

- Editing any `.rs` file during daily development does not restart backend or channel services automatically
- Desktop shell restarts do not duplicate runtime processes
- Runtime restart happens only through an explicit restart action
- Repo-local `target/` is no longer the main sink for daily dev artifacts
- Daily startup no longer rebuilds every sidecar binary by default

## Notes For Follow-Up Tasks

- This document is a solution and rollout target, not a statement that the repository already behaves this way
- The current repository still has known gaps around desktop child shutdown and embedded backend ownership
- Any implementation task for this area should update:
  - `docs/repo-map.md` if the launcher / runtime boundaries change
  - `docs/decisions.md` if the workflow becomes the accepted long-term default
  - this runbook when the commands or rollout status change
