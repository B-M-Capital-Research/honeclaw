# IM 渠道共享入口收口

- title: IM 渠道共享入口收口
- status: archived
- created_at: 2026-03-17
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/handoffs/2026-03-17-im-channel-core-refactor.md`
  - `docs/adr/0002-agent-runtime-acp-refactor.md`

## Goal

Converge IM channel ingress, outbound delivery, and attachment ingest on shared abstractions.

## Scope

- Shared `ingress` / `outbound` abstractions now cover dedup, session locking, actor scope, and outbound placeholder / streaming probes.
- Discord / Feishu attachment ingest and KB pipeline moved into `hone-channels`.
- Feishu and iMessage no longer branch on `gemini_cli`; they consume `AgentSession` streaming events.

## Validation

- `cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`
- `cargo test -p hone-channels`
- `cargo check --workspace --all-targets`

## Documentation Sync

- Historical closure is tracked in `docs/archive/index.md` and `docs/handoffs/2026-03-17-im-channel-core-refactor.md`.

## Risks / Open Questions

- Archived. Any future channel-core refactor should start from the handoff plus ADR 0002.
