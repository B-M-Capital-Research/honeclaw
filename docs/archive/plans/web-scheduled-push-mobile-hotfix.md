# Web Scheduled Push Mobile Hotfix

- title: Web Scheduled Push Mobile Hotfix
- status: archived
- created_at: 2026-07-10
- updated_at: 2026-07-10
- owner: Codex
- related_files:
  - `crates/hone-core/src/cloud_runtime.rs`
  - `memory/src/cron_job/history.rs`
  - `crates/hone-web-api/src/routes/public_pushes.rs`
  - `packages/app/src/components/public-nav.tsx`
  - `packages/app/src/components/public-push-center.tsx`
- related_docs:
  - `docs/decisions.md`
  - `docs/repo-map.md`
  - `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`

## Result

- Fixed the production backend/frontend contract skew that rendered scheduled messages as empty mobile shells.
- Added an independent mobile push bell beside the hamburger menu.
- Added deterministic, actor-scoped, idempotent backfill for pre-upgrade scheduled messages.
- Imported 79 historical pushes for the affected actor without replacing existing sessions or marking messages read.

## Verification

- `cargo test -p hone-web-api --lib -- --nocapture`: 94 passed, 2 ignored.
- New memory bulk import/read-preservation and deterministic legacy-id tests passed.
- `bun run typecheck:web`, `bun run test:web` (203 passed), and `bun run build:web:public` passed.
- `cargo check --workspace --all-targets --exclude hone-desktop` and `bash tests/regression/run_ci.sh` passed.
- Actor-scoped HTTP backfill returned five recent cards with 79 unread total in 93ms; temporary auth data was removed.

## Risks

- The existing unrelated memory test `sqlite_runtime_backend_backfills_existing_json_even_when_shadow_write_disabled` remains red with `QueryReturnedNoRows` and should be handled in its own task.
- Cloudflare Pages deployment must be verified by asset hash and mobile layout after pushing `main`.
