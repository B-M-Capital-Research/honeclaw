# 真群聊共享 Session 落地

- title: 真群聊共享 Session 落地
- status: done
- created_at: 2026-03-19
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/plans/group-shared-session.md`
- related_docs:
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

把群聊上下文从“按 actor 切分”改为“每个群一个共享 session”，并让 Web 控制台能按真实 `session_id` 浏览。

## What Changed

- 群聊会话归属从 actor 扩展为显式 `SessionIdentity`。
- Telegram / Feishu / Discord 群消息按“每个群一个 session”共享上下文。
- 群输入带发言人标识。
- 群 session 使用独立的恢复窗口与压缩阈值。
- Web 控制台按真实 `session_id` 浏览会话，并将群共享 session 标记为只读浏览。

## Verification

- `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`
- `cargo test -p hone-memory -p hone-channels`
- `cargo test -p hone-channels -p hone-memory -p hone-web-api --no-run`
- `bun run typecheck`

## Risks / Follow-ups

- 后续若再扩展共享群上下文，应先检查 `SessionIdentity`、恢复窗口和 Web 浏览模型是否仍一致。

## Next Entry Point

- `docs/archive/plans/group-shared-session.md`
