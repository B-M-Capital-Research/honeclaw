# Public Admin All-channel Usage

- title: Public Admin All-channel Usage
- status: in_progress
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/public_admin.rs
  - crates/hone-web-api/src/types.rs
  - packages/app/src/components/public-admin-usage-panel.tsx
  - packages/app/src/lib/types.ts
  - packages/app/src/pages/public-workspace.css
- related_docs:
  - docs/current-plan.md
  - docs/handoffs/2026-08-02-public-admin-usage-analytics.md
  - docs/runbooks/public-user-admin.md

## Goal

Expand the administrator usage report from Web-only direct actors and Web cron executions to all supported user channels, including Feishu, Telegram, Discord, and iMessage, while preserving automation filtering, privacy boundaries, Beijing-date grouping, and the existing 14-day summary/charts/table UX. Produce an immediate read-only production channel breakdown, then deploy and verify the persistent logic change.

## Scope

- Read GCE production data and compare recent Web/Feishu/Telegram/Discord/iMessage question, actor, scheduled-run, delivered-push, and failed-delivery counts.
- Generalize session identity extraction to concrete actors across all supported channels; group sessions count the actual actor user id, never the shared scope as a person.
- Generalize cron execution aggregation beyond `channel=web`, retaining channel/scope identity so unrelated users do not merge.
- Add an explicit channel dimension to API rows and the administrator table.
- Keep `codex*`, scheduler/heartbeat envelopes, job metadata, non-user messages, unsupported channels, and group scopes without a concrete actor user id excluded.
- Deploy the exact pushed revision through the existing GCE drain/rollback workflow and verify the authenticated production `/me` surface.

## Validation

- Focused Rust unit tests for all-channel inclusion, channel separation, automation exclusion, group/direct handling, and summary counts.
- `cargo test -p hone-web-api --lib` and `cargo check -p hone-web-api`.
- Focused Web tests, `bun run test:web`, TypeScript typecheck, and public production build.
- GCE read-only pre/post channel totals and installed-CLI/runtime health checks.
- Authenticated production Chrome verification of channel-labelled rows, charts, summary, and collapsible whitelist.

## Documentation Sync

- Update `docs/handoffs/2026-08-02-public-admin-usage-analytics.md` with the all-channel phase and exact production totals.
- Update reusable statistics scope in `docs/invariants.md` and/or `docs/decisions.md` if the cross-channel privacy boundary changes.
- On completion remove this item from `docs/current-plan.md`, move this plan to `docs/archive/plans/`, and update `docs/archive/index.md`.

## Risks / Open Questions

- Feishu/Discord/Telegram/iMessage user identifiers do not map to domestic phone labels; responses must use bounded channel-aware labels and never expose unrelated secrets or message content outside the administrator boundary.
- Group/channel-scope sessions may represent shared conversations rather than one person; only the concrete actor user id counts, while the shared scope is retained solely for session routing and never becomes the user identity.
- Full session enumeration remains bounded only by the existing 14-day message filter; increased channel volume may accelerate the need for a database-side time projection.
