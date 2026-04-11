# 单一聊天范围配置与群聊忙碌态控制

- title: 单一聊天范围配置与群聊忙碌态控制
- status: done
- created_at: 2026-03-27
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/decisions.md`
- related_prs:
  - N/A

## Summary

统一聊天范围配置模型，并给三渠道群聊显式触发补上忙碌态生命周期控制。

## What Changed

- Feishu / Telegram / Discord 将 `dm_only` 收敛为 `chat_scope=DM_ONLY|GROUPCHAT_ONLY|ALL`。
- 继续兼容旧 `dm_only` 配置。
- 三渠道群聊显式触发时，如果上一条仍在处理中，新消息会立即收到等待提示。
- 新问题文本会继续保留在群聊预触发窗口中。

## Verification

- `cargo check -p hone-core -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`
- `cargo test -p hone-core -p hone-channels`
- `cargo test -p hone-discord -p hone-feishu -p hone-telegram --no-run`

## Risks / Follow-ups

- 后续若继续收敛 chat scope 或群聊触发策略，需要同时验证兼容旧配置和群聊忙碌态体验。

## Next Entry Point

- `docs/archive/index.md`
