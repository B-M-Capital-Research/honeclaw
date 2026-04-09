# 群聊预触发窗口统一改造

- title: 群聊预触发窗口统一改造
- status: done
- created_at: 2026-03-24
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/deliverables.md`
- related_prs:
  - N/A

## Summary

统一 Telegram / Discord / 飞书群聊的预触发缓存窗口，让显式触发前的上下文缓存行为一致。

## What Changed

- 三渠道统一为“未触发先静默缓存、显式 `@` / reply-to-bot 时再执行”的模型。
- 共享层新增按群 session 维护的预触发滑动窗口。
- 触发时会把最近 10 条、5 分钟内的群文本正式写入共享群 session。
- Discord 移除 question-signal 与短窗批处理路径。
- 群聊首条回复三渠道统一固定 mention 触发者。

## Verification

- `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`
- `cargo test -p hone-channels -p hone-core`
- `cargo test -p hone-discord -p hone-telegram --no-run`

## Risks / Follow-ups

- 以后如果新增渠道或调整群聊触发窗口，应优先复用共享层预触发模型，而不是回到渠道特化逻辑。

## Next Entry Point

- `docs/archive/index.md`
