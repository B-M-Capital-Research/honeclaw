# 群聊回复追加链路统一

- title: 群聊回复追加链路统一
- status: archived
- created_at: 2026-03-19
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/handoffs/2026-03-19-group-reply-append-chain.md`

## Goal

Unify placeholder and reply-chain behavior for group replies across Discord, Telegram, and Feishu.

## Scope

- Group placeholders now consistently stay as `@用户 + 正在思考中...`.
- Tool reasoning no longer overwrites the placeholder.
- The first final reply always re-mentions the triggering user.
- Multi-part replies are chained as replies to avoid interruption by unrelated messages.

## Validation

- `cargo check -p hone-discord -p hone-feishu -p hone-telegram`
- `cargo test -p hone-discord -p hone-telegram`

## Documentation Sync

- Historical closure is tracked in `docs/archive/index.md` and `docs/handoffs/2026-03-19-group-reply-append-chain.md`.

## Risks / Open Questions

- Archived. Resume from the handoff if another channel needs to adopt the same reply-chain rules.
