# IM 渠道共享入口收口

- title: IM 渠道共享入口收口
- status: done
- created_at: 2026-03-17
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/plans/attachment-ingest-unify.md`
- related_docs:
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
- related_prs:
  - N/A

## Summary

收口 IM 渠道共享 `ingress` / `outbound` 抽象，并把附件 ingest 与执行链路尽量下沉到共享层。

## What Changed

- 新增共享 `ingress` / `outbound` 抽象，统一 dedup、session 锁、actor scope、出站占位/分段/流式探针。
- Discord / 飞书附件 ingest 与 KB 管线下沉到 `hone-channels`。
- Feishu / iMessage 去掉基于 `gemini_cli` 的执行分支，改为统一消费 `AgentSession` 流式事件。

## Verification

- `cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`
- `cargo test -p hone-channels`
- `cargo check --workspace --all-targets`

## Risks / Follow-ups

- 未来若继续做 channel core 收敛，应从共享 `ingress` / `outbound` 抽象继续推，而不是重新在渠道层分叉。

## Next Entry Point

- `docs/archive/plans/attachment-ingest-unify.md`
