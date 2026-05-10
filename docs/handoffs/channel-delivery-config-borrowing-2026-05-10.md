# Channel Delivery Config Borrowing Handoff

- title: Channel Delivery Config Borrowing
- status: done
- created_at: 2026-05-10
- updated_at: 2026-05-10
- owner: Codex
- related_files:
  - `bins/hone-cli/src/{main,mutations,reports,configure,onboard,i18n}.rs`
  - `bins/hone-desktop/src/sidecar.rs`
  - `bins/hone-desktop/src/sidecar/settings.rs`
  - `crates/hone-tools/src/cron_job_tool.rs`
  - `crates/hone-scheduler/src/lib.rs`
  - `crates/hone-web-api/src/routes/cron.rs`
  - `memory/src/cron_job/{mod,storage,types}.rs`
  - `memory/src/lib.rs`
  - `packages/app/src/lib/types.ts`
  - `packages/app/src/lib/admin-content/settings.ts`
  - `packages/app/src/pages/settings.tsx`
  - `packages/app/src/pages/settings-model.ts`
  - `config.example.yaml`
- related_docs:
  - `docs/archive/plans/channel-delivery-config-borrowing.md`
  - `docs/current-plan.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
  - `docs/archive/index.md`
- related_prs:

## Summary

This milestone rejects `home_channel` as the primary delivery model. Honeclaw should keep origin-bound delivery: a channel-created cron or proactive task stores the source `channel` and `channel_target`, then delivers back to that same target.

The borrowed Hermes-style improvement is instead configuration visibility: existing allowlists, `chat_scope`, and iMessage `target_handle` are now configurable through CLI and Desktop/Web instead of requiring YAML edits.

The final milestone also borrows the "discoverable channel directory" idea in a Hone-native way: scheduled-task targets are aggregated into a typed local directory and surfaced through CLI inspection, without introducing a platform-level default destination.

## What Changed

- Added CLI `channels set` flags for Feishu allowlists, Telegram / Discord `allow_from`, and `chat_scope`.
- Expanded `hone-cli configure`, `hone-cli onboard`, and `hone-cli channels list` to expose allowlist and `chat_scope` details.
- Expanded Desktop sidecar channel settings read/write shape for iMessage `target_handle`, Feishu allowlists / `chat_scope`, Telegram allowlist / `chat_scope`, and Discord allowlist / `chat_scope`.
- Added matching Web settings draft fields, labels, inputs, and tests.
- Added `cron_job_tool_add_preserves_origin_channel_target` to lock the origin-bound task target behavior.
- Rejected empty `channel_target` during cron-job creation, except for the explicit web actor fallback path.
- Added scheduler diagnostics for legacy missing-target jobs: they record a `target_missing` execution failure and skip dispatch instead of sending to an unrelated target.
- Added `ChannelTargetRecord` plus `CronJobStorage::list_channel_targets()`, aggregating cron definitions and recent execution history.
- Added `hone-cli channels targets [--json]` as the first read path for the typed channel-target directory.
- Recorded the origin-bound invariant in `docs/invariants.md` and the expanded sidecar settings flow in `docs/repo-map.md`.

## Verification

- `cargo test -p hone-tools cron_job_tool_add_preserves_origin_channel_target`
- `cargo test -p hone-cli build_channel_mutations_supports_allowlists`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop desktop_channel_settings`
- `cargo test -p hone-cli`
- `bun run typecheck:web`
- `bun run test:web`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `cargo test -p hone-memory channel_target`
- `cargo test -p hone-scheduler scheduler_records_missing_channel_target_without_dispatching`
- `cargo test -p hone-cli cli_parses_channels_targets_command`
- `cargo test -p hone-memory`
- `cargo test -p hone-scheduler`
- `cargo test -p hone-web-api cron`

## Risks / Follow-ups

- `hone-cli channels targets` is inspection-only. A Web/Desktop selector can build on the same typed record later, but it was intentionally not added in this milestone.
- The channel-target directory currently covers cron definitions and recent cron execution history, not arbitrary inbound chat history.
- Do not reintroduce `home_channel` unless a separate system-created task flow exists with no origin channel target; even then it should be a narrow fallback, not the default channel UX.

## Next Entry Point

The plan is archived at `docs/archive/plans/channel-delivery-config-borrowing.md`. If this area continues, the next concrete step is a Web/Desktop selector backed by `CronJobStorage::list_channel_targets()`, not a `home_channel` fallback.
