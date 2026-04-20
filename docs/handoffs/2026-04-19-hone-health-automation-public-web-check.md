# Hone Health Automation Public Web Check Upgrade

- title: Hone Health Automation Public Web Check Upgrade
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-20
- owner: Codex
- related_files:
  - `.codex/automations/hone-health-30m/automation.toml`
  - `~/.codex/automations/hone-health-30m/automation.toml`
- related_docs:
  - `docs/archive/index.md`
  - `docs/runbooks/desktop-release-app-runtime.md`
- related_prs: N/A

## Summary

Upgraded the live `hone-health-30m` automation so it no longer treats “`8088` is listening” as sufficient proof that the public web surface is healthy. The health patrol now also checks for `packages/app/dist-public/index.html`, validates that `http://127.0.0.1:8088/` returns real frontend HTML, and uses `bun run build:web:public` as the first stopgap when only the public assets are missing.

## What Changed

- Added a repository snapshot for `hone-health-30m` under `.codex/automations/`.
- Expanded the health criteria to include:
  - `packages/app/dist-public/index.html` exists
  - `http://127.0.0.1:8088/` returns `200`
  - the response body contains a real frontend entrypoint instead of the `Hone Web assets not found` failure
- Added a minimal recovery path ahead of full restart:
  - if the rest of the release runtime is healthy and only the public assets are missing, run `bun run build:web:public`
  - re-check `dist-public` and `8088`
  - only escalate to full `hone-release` restart if the stopgap does not restore service
- Kept the task boundary as runtime operations only; the automation still does not modify business code or rebuild the release bundle.

## Verification

- Read the existing live automation at `~/.codex/automations/hone-health-30m/automation.toml`.
- Confirmed the repository previously had no tracked snapshot for this automation under `.codex/automations/`.
- Used the current failure mode as the target scenario:
  - `packages/app/dist-public/index.html` missing caused `8088` to fail for real users
  - `8077` and the running `hone-desktop` process could still appear healthy
- Synced the repository snapshot and the live automation so both now require the public web asset and `8088` HTML checks.

## Risks / Follow-ups

- `bun run build:web:public` depends on the local frontend toolchain staying usable; if Bun or `node_modules` is broken, the automation will still need to escalate or stop with a clear blocker.
- This closes the “public web assets missing but process still up” blind spot, but it does not replace broader reverse-proxy, TLS, or public rate-limit monitoring.
- This task stayed out of `docs/current-plan.md` because it was a one-shot automation policy adjustment, not a multi-round tracked implementation stream.

## Next Entry Point

- `.codex/automations/hone-health-30m/automation.toml`
