# 多渠道附件工程化卡点

- title: 多渠道附件工程化卡点
- status: archived
- created_at: 2026-03-22
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/handoffs/2026-03-22-channel-attachment-gate.md`

## Goal

Unify shared attachment ingest limits and rejection behavior across channels.

## Scope

- Shared attachment ingest now rejects oversized attachments and invalid images.
- General attachments are capped at 5MB; images at 3MB with additional dimension and ratio checks.
- Rejected attachments do not enter prompts or the KB, and channel acknowledgements summarize the rejection reason.

## Validation

- `cargo test -p hone-channels`
- `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`

## Documentation Sync

- Historical closure is tracked in `docs/archive/index.md` and `docs/handoffs/2026-03-22-channel-attachment-gate.md`.

## Risks / Open Questions

- Archived. Follow-up work should start from the handoff if new attachment policy changes are needed.
