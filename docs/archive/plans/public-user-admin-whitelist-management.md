# Public User Admin Whitelist Management

- title: Public user administrator and whitelist management
- status: done
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

- Backend role authority, audit log, atomic Beijing-day creation limit, public administrator routes, and shared responsive UI are implemented and deployed.
- The production PostgreSQL schema has been ensured and the unique active user for `13871396421` is verified with `is_admin=true`.
- A dry-run-first `hone-cli cloud web-admin` command is available for future grants and revocations.
- Commit `5eacfe98c0b2b3bdaac11fc23830c0ab91b14f3d` is pushed to `main`; Cloudflare Pages serves the matching administrator API/client markers.
- Affected Rust checks and tests pass: CLI 85, memory 132, core cloud-runtime 20, Web API 159 plus two credentialed ignores. The previously stale attachment-context assertion is aligned on the merged upstream baseline.
- Frontend typecheck, all tests, public production build, mobile/desktop visual QA, and the complete CI-safe regression suite pass; finance contracts are 44/44.
- Production Web and Feishu run from immutable `target/deploy-5eacfe98`; the 505-file manifest verifies, PostgreSQL/R2 remain healthy and authoritative, active chats are zero, and local/origin/public auth boundaries return expected JSON.

## Documentation Sync

- Update `docs/repo-map.md` for the new public-admin route/data flow.
- Update `docs/invariants.md` for PostgreSQL role authority, server-side authorization, atomic Beijing-day limit, audit, and self-disable rules.
- Record the cross-module architecture in `docs/decisions.md` and the role-tagging operation in `docs/runbooks/public-user-admin.md`.
- The completed plan is removed from `docs/current-plan.md`, archived here, and indexed from `docs/archive/index.md`.
- Deployment evidence and remaining operational cautions are preserved in `docs/handoffs/2026-07-31-public-user-admin-whitelist-management.md`.

## Risks / Open Questions

- Existing admin-console invite APIs use a separate administrator token and must keep their current behavior; the five-per-day rule applies specifically to user-facing public administrators.
- Live production mutation verification should use a controlled test member because a successful create consumes one of the administrator's five slots for that Beijing day. This deployment intentionally performed no real create/disable mutation.
- The existing Chrome profile had no authenticated HONE session. The deployment did not send an SMS or manufacture a user session merely for UI acceptance; production role readback, route mounting, bundle markers, authorization regressions, and anonymous fail-closed behavior were verified instead.
