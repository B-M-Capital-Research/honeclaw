# 多渠道附件工程化卡点

- title: 多渠道附件工程化卡点
- status: done
- created_at: 2026-03-22
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/archive/plans/channel-attachment-gate.md`
- related_docs:
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

统一共享附件 ingest 的体积和图片合法性门禁，避免异常附件进入 prompt 或知识库。

## What Changed

- 共享附件 ingest 统一拦截超限附件与异常图片。
- 通用附件限制为 5MB，图片限制为 3MB。
- 图片会按最长边、总像素、长宽比做二次校验。
- 被拒附件不会进入 prompt 与 KB。
- 渠道 ack 会明确汇总拦截原因。

## Verification

- `cargo test -p hone-channels`
- `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`

## Risks / Follow-ups

- 后续若放宽限制或新增 MIME 类型，应一起更新共享 ingest 规则与渠道侧提示文案。

## Next Entry Point

- `docs/archive/plans/channel-attachment-gate.md`
