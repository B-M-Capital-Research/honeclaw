# Public Admin Whitelist Production Follow-up

- title: Public admin whitelist production list recovery
- status: archived
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

- Production read-only storage probe: 162 domestic summaries, one or more
  administrators, legacy full-record list readable, and the corrected daily
  count returned zero without a serialization error.
- `cargo check -p hone-core -p hone-memory -p hone-web-api --tests`: passed.
- `cargo test -p hone-core cloud_runtime::tests`: 22 passed.
- `cargo test -p hone-memory`: 133 passed.
- `cargo test -p hone-web-api`: 160 passed, 2 credentialed tests ignored.
- `bash tests/regression/run_ci.sh`: passed, including finance contracts 44/44.
- Implementation commit `49ef8dd4e2d5298ad69f01b73d7a1b9be7fa5b87`
  passed the pre-push secret scan and is on `main`.
- Immutable runtime `target/deploy-49ef8dd4` has 505 hashed payload files;
  manifest SHA-256 is
  `300c55cc7413cf6e7732b2697d2150f2e812e1c698bac35b25ac58491ca7d68e`.
- Web and Feishu now run the exact immutable binaries after two zero-active-chat
  checks. The tunnel supervisor was not restarted.
- Final local `/api/meta` reports cloud mode, authoritative storage, healthy
  PostgreSQL/R2, zero local durable dependencies, and zero active chats.
  Local, origin, and public auth/admin probes return expected `401
  application/json`.

## Documentation Sync

- Update the existing same-day handoff and the database-authoritative
  administration decision/invariant.
- Removed the active entry, moved this plan to `docs/archive/plans/`, updated
  the existing handoff, decision/invariant, and archive index.

## Risks / Open Questions

- No synthetic SMS login or real whitelist mutation was created solely for
  acceptance. The administrator should refresh the already-authenticated
  management view for the final visual confirmation.
- `target/deploy-482c34d5` remains the immediate rollback package.
