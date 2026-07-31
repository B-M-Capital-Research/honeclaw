# Public Admin Whitelist Production Follow-up

- title: Public admin whitelist production list recovery
- status: in_progress
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - `crates/hone-core/src/cloud_runtime.rs`
  - `memory/src/web_auth.rs`
  - `crates/hone-web-api/src/routes/public_admin.rs`
- related_docs:
  - `docs/handoffs/2026-07-31-public-user-admin-whitelist-management.md`
  - `docs/decisions.md`
  - `docs/invariants.md`

## Goal

Restore the production administrator whitelist list without making the
management surface depend on full deserialization of every historical
authentication record or on the ancillary daily-count query.

## Scope

- Read a minimal, non-secret whitelist projection directly from PostgreSQL or
  SQLite.
- Bind Beijing calendar dates as text and cast them inside PostgreSQL for every
  administrator count/audit query, matching the Rust parameter type.
- Keep the list available when the daily creation count is unavailable, while
  conservatively disabling creation until the count recovers.
- Mark every administrator row, not only the current administrator, as
  non-disableable.
- Preserve the existing public response schema, client layout, and prompt
  answer format.

## Validation

- Run focused `hone-core`, `hone-memory`, and `hone-web-api` tests.
- Run compile checks and CI-safe regression gates.
- Build and verify an immutable production runtime from the pushed commit.
- Restart only the Web/Feishu lanes after active-chat drain; leave the tunnel
  supervisor unchanged.
- Verify cloud storage, public/origin health, authentication fail-closed
  behavior, and the authenticated administrator list through production logs.

## Documentation Sync

- Update the existing same-day handoff and the database-authoritative
  administration decision/invariant.
- On completion, remove this entry from `docs/current-plan.md`, archive this
  plan, and add the production follow-up to `docs/archive/index.md`.

## Risks / Open Questions

- A real authenticated production request is required for final positive-path
  confirmation; no synthetic SMS login or whitelist mutation will be created
  solely for acceptance.
- Daily-count failure must fail closed for creation but must not hide the list.
