# Public User Admin Whitelist Management

- title: Public user administrator and whitelist management
- status: in_progress
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - `crates/hone-core/src/cloud_runtime.rs`
  - `memory/src/web_auth.rs`
  - `crates/hone-web-api/src/routes/public_admin.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-web-api/src/types.rs`
  - `bins/hone-cli/src/cloud.rs`
  - `packages/app/src/components/public-admin-whitelist-panel.tsx`
  - `packages/app/src/pages/public-me.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/runbooks/public-user-admin.md`
  - `docs/handoffs/2026-07-31-public-user-admin-whitelist-management.md`

## Goal

Make PostgreSQL the authority for public-user administrator roles, mark domestic user `13871396421` as an administrator, and expose a responsive “我的 → 管理” module that lets authenticated administrators list, add, and disable domestic whitelist users.

## Scope

- Persist the administrator role separately from ordinary membership/invite data in PostgreSQL, with local SQLite parity only for development and tests. Provide a dry-run-first CLI for future role changes.
- Add cookie-session-authenticated public administrator APIs. Every request must re-check the server-side role; the frontend flag is display-only.
- Enforce a hard limit of five successful whitelist creations per administrator per Beijing natural day with an atomic server/database boundary. Duplicate or failed attempts do not consume quota.
- Record successful administrator mutations for audit. Prevent self-disable and prevent ordinary users from reading or mutating the whitelist.
- Add one responsive management surface under `/me`; the macOS and iOS shells inherit it because both load the same public Web application.
- Do not expose invite codes, API keys, password state, or unrelated internal quota details in the public administrator API.

## Validation

- Rust unit tests for role persistence, authorization, daily-limit behavior, duplicate handling, self-disable protection, and session revocation.
- Web API tests for the public projection and mutation marker, with storage-level authorization and mutation regressions covering ordinary-user denial, successful create/disable, daily-limit exhaustion, and session revocation.
- Frontend tests for administrator-only visibility, create-limit state, and disable interaction.
- Run focused Rust tests, `cargo check` for affected crates, frontend unit tests/typecheck, changed-file formatting, and CI-safe regression where practical.
- Verify the production PostgreSQL row for `13871396421` before and after marking it; never print connection secrets.

## Progress

- Backend role authority, audit log, atomic Beijing-day creation limit, public administrator routes, and shared responsive UI are implemented locally.
- The production PostgreSQL schema has been ensured and the unique active user for `13871396421` is verified with `is_admin=true`.
- A dry-run-first `hone-cli cloud web-admin` command is available for future grants and revocations.
- Affected Rust checks and tests pass. The full `hone-web-api` suite has one pre-existing unrelated attachment-context assertion failure; all 158 other tests pass and the focused public-admin tests pass.
- Frontend typecheck, all 312 tests, public production build, and mobile/desktop visual QA pass.
- No commit, push, release, application build, or production deployment is included in this task yet.

## Documentation Sync

- Update `docs/repo-map.md` for the new public-admin route/data flow.
- Update `docs/invariants.md` for PostgreSQL role authority, server-side authorization, atomic Beijing-day limit, audit, and self-disable rules.
- Record the cross-module architecture in `docs/decisions.md` and the role-tagging operation in `docs/runbooks/public-user-admin.md`.
- Keep this plan indexed in `docs/current-plan.md` while code or production verification remains active.
- On completion, create/update `docs/handoffs/2026-07-31-public-user-admin-whitelist-management.md`, archive this plan, and add an entry to `docs/archive/index.md`.

## Risks / Open Questions

- Existing admin-console invite APIs use a separate administrator token and must keep their current behavior; the five-per-day rule applies specifically to user-facing public administrators.
- The database role is active, but the current production binary/client will not expose or enforce the new management surface until this exact source change is committed, built, and deployed.
- Live production mutation verification should use a controlled test member because a successful create consumes one of the administrator's five slots for that Beijing day.
