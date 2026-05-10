# Channel Delivery Config Borrowing

- title: Channel Delivery Config Borrowing
- status: done
- created_at: 2026-05-10
- updated_at: 2026-05-10
- owner: Codex
- related_files:
  - `config.example.yaml`
  - `crates/hone-core/src/config/channels.rs`
  - `bins/hone-cli/src/main.rs`
  - `bins/hone-cli/src/mutations.rs`
  - `bins/hone-cli/src/onboard.rs`
  - `bins/hone-cli/src/configure.rs`
  - `bins/hone-cli/src/reports.rs`
  - `bins/hone-desktop/src/sidecar/settings.rs`
  - `packages/app/src/lib/types.ts`
  - `packages/app/src/pages/settings.tsx`
  - `memory/src/cron_job/storage.rs`
  - `bins/hone-telegram/src/scheduler.rs`
  - `bins/hone-feishu/src/scheduler.rs`
  - `bins/hone-discord/src/scheduler.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-event-engine/src/router/dispatch.rs`
  - `crates/hone-event-engine/src/sinks/multi.rs`
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/handoffs/channel-delivery-config-borrowing-2026-05-10.md`
  - `docs/archive/index.md`

## Goal

Borrow the useful Hermes Agent configuration patterns without expanding Honeclaw's supported platforms. The work should make the existing iMessage, Feishu, Telegram, and Discord push-channel configuration more complete, discoverable, and restart-safe across CLI, Web/Desktop, and scheduled or proactive delivery, while preserving Honeclaw's origin-bound delivery model.

## Scope

- No platform expansion: do not add new channel adapters or broaden platform support beyond existing iMessage, Feishu, Telegram, and Discord.
- P0: Preserve and verify origin-bound delivery for existing channels.
  - A scheduled or proactive task created from a channel should persist its `channel` and `channel_target`.
  - Delivery should go back to that same originating channel target.
  - Do not introduce a platform-level default target as the primary model; it weakens the "created here, delivered here" expectation.
  - Runtime behavior for a missing or empty target should be deterministic and actionable rather than silently falling back to another destination.
- P0: Make existing allowlist and chat scope configuration visible in every supported management surface.
  - CLI onboarding and configure paths should cover allowlists and chat scope consistently.
  - Desktop settings should show the same allowlist and chat scope fields rather than only tokens.
  - Credential UX keeps the existing split: tokens may be visible on demand, secrets and API keys stay masked.
- P1: Add a channel-target directory for existing platforms only.
  - Resolve recently observed channel targets and scheduled-task targets into a discoverable list.
  - Keep target storage typed and local; avoid Hermes-style loose `extra` blobs as the primary contract.
  - Expose the directory through CLI first, then Web/Desktop selection UI.
- P1: Add channel/gateway health and restart guidance to configuration UX.
  - Show which sidecars need restart after a canonical config edit.
  - Prefer existing effective-config generation and sidecar lifecycle boundaries.

## Validation

- `cargo test -p hone-core`
- `cargo test -p hone-cli`
- Targeted scheduler tests for missing delivery targets if new runtime routing is added.
- Targeted desktop/web type or unit tests for settings shape changes.
- `cargo check --workspace --all-targets --exclude hone-desktop` before closing a behavior-changing milestone.
- `bun run test:web` when Web/Desktop settings code changes.

## Progress

### 2026-05-10

- Removed `home_channel` / `home_target` from this borrowing scope after confirming the product expectation: tasks created from a channel should deliver back to that same channel target.
- Added a regression test in `crates/hone-tools/src/cron_job_tool.rs` proving `cron_job` preserves the origin `channel_target` when a channel session creates a task.
- Expanded CLI channel configuration so `channels set`, `configure`, `onboard`, and `channels list` expose allowlist and `chat_scope` fields.
- Expanded Desktop/Web channel settings to read and write iMessage `target_handle`, Feishu allowlists / `chat_scope`, Telegram allowlist / `chat_scope`, and Discord allowlist / `chat_scope`.
- Updated `docs/invariants.md` and `docs/repo-map.md` for the origin-bound delivery invariant and the expanded desktop channel config surface.
- Verification run:
  - `cargo test -p hone-tools cron_job_tool_add_preserves_origin_channel_target`
  - `cargo test -p hone-cli build_channel_mutations_supports_allowlists`
  - `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop desktop_channel_settings`
  - `cargo test -p hone-cli`
  - `bun run typecheck:web`
  - `bun run test:web`
  - `cargo check --workspace --all-targets --exclude hone-desktop`
- Hardened scheduled delivery target handling:
  - `CronJobStorage::add_job` now rejects empty `channel_target` instead of falling back to `actor.user_id`.
  - `/api/cron/jobs` rejects missing `channel_target` for non-web actors; web keeps an explicit `actor.user_id` target fallback.
  - `TaskScheduler` records `target_missing` execution failures for legacy jobs with empty targets and does not dispatch them to unrelated destinations.
- Added a typed channel-target directory from cron definitions plus recent cron execution history, exposed through `hone-cli channels targets` and `hone-cli channels targets --json`.
- Added regression coverage for empty-target rejection, channel-target directory aggregation, scheduler `target_missing` diagnostics, and CLI parsing.
- Additional verification run:
  - `cargo test -p hone-memory channel_target`
  - `cargo test -p hone-scheduler scheduler_records_missing_channel_target_without_dispatching`
  - `cargo test -p hone-cli cli_parses_channels_targets_command`
  - `cargo test -p hone-memory`
  - `cargo test -p hone-scheduler`
  - `cargo test -p hone-cli`
  - `cargo test -p hone-web-api cron`
  - `bun run typecheck:web`
  - `cargo check --workspace --all-targets --exclude hone-desktop`

## Documentation Sync

- Keep this plan updated after each milestone so incomplete items remain visible.
- Update `docs/current-plan.md` while this task is active.
- Update `config.example.yaml` when existing channel fields become visible through CLI/Web.
- Update `docs/repo-map.md` if channel config data flow, sidecar settings, or delivery target routing changes.
- Update `docs/invariants.md` only if this work changes canonical-config, effective-config, or restart semantics.
- Add or update a handoff when a milestone completes, pauses, or leaves known follow-up risk.
- Archive this plan and update `docs/archive/index.md` when all P0/P1 acceptance items are closed or explicitly deferred.

## Risks / Open Questions

- Feishu target identity is more ambiguous than Telegram/Discord because the scheduler may receive user, email, mobile, open_id, or chat identifiers. The implementation must preserve existing validation and avoid silently sending to the wrong target.
- iMessage `target_handle` is an ingress filter / tracked handle, not a default outbound destination. Keep that distinction visible in CLI/Web copy.
- Existing event-engine digest and quiet-hour controls are stronger than Hermes' cron delivery model. Preserve those quality controls while adding better channel routing UX.
- Avoid copying Hermes' loose `extra` escape hatch as the main design. Honeclaw should keep typed config as the durable contract and use extension maps only where the repo already does so.

## Anti-Interruption Checklist

- [x] P0 origin-bound delivery is preserved for existing scheduled/proactive channel tasks.
- [x] P0 missing scheduled/proactive delivery targets fail or warn deterministically without fallback to an unrelated channel.
- [x] P0 CLI paths can inspect and update allowlist and chat scope without manual YAML edits.
- [x] P0 Web/Desktop settings can inspect and update the same core channel fields as CLI.
- [x] P1 channel-target directory has a typed source of truth and at least one CLI or API read path.
- [x] Restart/effective-config behavior is visible to users after channel config changes.
- [x] Config examples and relevant docs are updated in the same milestone as code.
- [x] Validation commands above have been run, or blockers are recorded here with concrete failure output.
