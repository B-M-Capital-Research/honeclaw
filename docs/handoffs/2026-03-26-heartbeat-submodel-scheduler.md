# 子模型配置与心跳任务调度

- title: 子模型配置与心跳任务调度
- status: done
- created_at: 2026-03-26
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/index.md`
- related_docs:
  - `docs/deliverables.md`
- related_prs:
  - N/A

## Summary

补齐 Desktop 子模型配置和 heartbeat 任务类型，使压缩与心跳调度能力具备独立入口。

## What Changed

- Desktop 设置页新增 OpenRouter 子模型配置。
- 会话压缩切到子模型。
- cron 新增 `heartbeat` 任务类型与标签。
- heartbeat 任务按 30 分钟轮询、未命中时不投递，并在任务中心与 cron API 中正常显示。

## Verification

- `cargo test -p hone-memory -p hone-scheduler -p hone-tools -p hone-core -p hone-web-api -p hone-channels`
- `cargo check -p hone-desktop`
- `npm run typecheck`

## Risks / Follow-ups

- 后续若继续扩展 heartbeat 或子模型调度能力，需要一起验证 UI、scheduler 与压缩链路。

## Next Entry Point

- `docs/archive/index.md`
