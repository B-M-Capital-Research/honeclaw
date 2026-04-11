# 渠道运行态心跳替代 pid 判活

- title: 渠道运行态心跳替代 pid 判活
- status: done
- created_at: 2026-03-18
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/decisions.md`
- related_prs:
  - N/A

## Summary

把渠道存活判断从 pid 文件与 `kill -0` 切到带 `pid` 的 heartbeat 文件。

## What Changed

- 四个渠道二进制都会每 30 秒写一次 `runtime/*.heartbeat.json`。
- `/api/channels` 改为基于心跳新鲜度呈现运行状态。
- 旧的 `runtime/*.pid` + `kill -0` 判活逻辑不再作为主依据。

## Verification

- `cargo check -p hone-core -p hone-web-api -p hone-desktop -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage`
- `cargo test -p hone-core -p hone-web-api`

## Risks / Follow-ups

- 后续所有运行态展示、清理和锁冲突处理都应优先复用 heartbeat，而不是回到 pid 猜测。

## Next Entry Point

- `docs/archive/index.md`
