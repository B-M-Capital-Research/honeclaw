# 真群聊共享 Session 落地

- title: 真群聊共享 Session 落地
- status: archived
- created_at: 2026-03-19
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/handoffs/2026-03-19-group-shared-session.md`

## Goal

Move group conversations from per-actor session ownership to an explicit shared group session model.

## Scope

- Session ownership expanded from actor to explicit `SessionIdentity`.
- Telegram / Feishu / Discord group messages now share one session per group.
- Group prompts include speaker labels and use dedicated restore/compression thresholds.
- The Web console browses these sessions by real `session_id` and marks them read-only.

## Validation

- `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`
- `cargo test -p hone-memory -p hone-channels`
- `cargo test -p hone-channels -p hone-memory -p hone-web-api --no-run`
- `bun run typecheck`

## Documentation Sync

- Historical closure is tracked in `docs/archive/index.md` and `docs/handoffs/2026-03-19-group-shared-session.md`.

## Risks / Open Questions

- Archived. Future follow-up should start from the handoff and the session identity decisions it depends on.
