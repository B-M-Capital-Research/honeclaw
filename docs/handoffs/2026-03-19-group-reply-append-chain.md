# 群聊回复追加链路统一

- title: 群聊回复追加链路统一
- status: done
- created_at: 2026-03-19
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/plans/group-reply-append-chain.md`
- related_docs:
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

统一 Discord / Telegram / Feishu 的群聊占位符、首条提及和多段 reply 链行为。

## What Changed

- 群聊占位符统一保留为 `@用户 + 正在思考中...`。
- 群聊 tool reasoning 不再覆盖占位符。
- 最终首条回复统一补 `@用户`。
- 多段回复会串成 reply 链，避免被中间消息打断。

## Verification

- `cargo check -p hone-discord -p hone-feishu -p hone-telegram`
- `cargo test -p hone-discord -p hone-telegram`

## Risks / Follow-ups

- 若后续新增渠道或重构 reply adapter，应以这份 handoff 作为群聊回复体验的基线。

## Next Entry Point

- `docs/archive/plans/group-reply-append-chain.md`
