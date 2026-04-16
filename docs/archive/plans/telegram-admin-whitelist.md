# Telegram 管理员白名单支持

- title: Telegram 管理员白名单支持
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-core/src/config.rs`
  - `crates/hone-channels/src/core.rs`
  - `config.example.yaml`
  - `config.yaml`
  - `data/runtime/config_runtime.yaml`
- related_docs:
  - `docs/archive/index.md`
  - `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`

## Goal

Add first-class Telegram admin allowlist support so Telegram direct-chat identities can be recognized as administrators and use the existing shared admin flow.

## Scope

- Extended the admin config schema with a Telegram user ID allowlist.
- Wired Telegram into the shared `is_admin` / `is_admin_actor` path used by channel runtimes and admin intercepts.
- Added regression coverage for config deserialization and Telegram admin recognition.
- Updated local runtime config and canonical config so the current Telegram identity `8039067465` is allowlisted.

## Validation

- `cargo test -p hone-core`
- `cargo test -p hone-channels`

## Documentation Sync

- Archived this plan under `docs/archive/plans/` and added an index entry in `docs/archive/index.md`.
- Recorded the concrete outcome in `docs/handoffs/2026-04-16-telegram-admin-whitelist.md`.

## Risks / Open Questions

- Existing deployments that rely on `admins` schema stability now accept an additional Telegram field.
- Running binaries still need an intentional restart before the new Telegram admin path becomes effective.
