# 额度与定时任务可靠性修复

- title: 额度与定时任务可靠性修复
- status: done
- created_at: 2026-03-29
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/deliverables.md`
- related_prs:
  - N/A

## Summary

同时修复普通用户额度策略和 heartbeat / cron 在错过原窗口后的可靠性问题。

## What Changed

- 普通用户每日对话额度从 20 调整为 12。
- 非 heartbeat 定时任务支持“错过原始 5 分钟窗口后的同日单次补触发”。
- heartbeat 的 JSON 解析失败会被安全抑制，不再把控制输出发给用户。

## Verification

- `cargo test -p hone-memory`
- `cargo test -p hone-channels`

## Risks / Follow-ups

- 后续若继续调整额度或定时任务补偿策略，需要同步验证跨天、进程恢复、以及 heartbeat 例外逻辑。

## Next Entry Point

- `docs/archive/index.md`
