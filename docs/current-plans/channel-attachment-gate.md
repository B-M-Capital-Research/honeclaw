# 多渠道附件工程化卡点

## 目标

- 在共享附件 ingest 层新增统一准入校验，覆盖 Telegram / Discord / 飞书。
- 将超限附件和异常图片拦截在 prompt / KB 持久化之前，减少无效 tokens 和异常输入。
- 保持渠道入口为薄适配，不在各渠道复制限制逻辑。

## 涉及文件

- `crates/hone-channels/src/attachments.rs`
- `crates/hone-channels/Cargo.toml`
- `docs/repo-map.md`
- `docs/current-plan.md`
- `docs/handoffs/2026-03-22-channel-attachment-gate.md`

## 实施要点

- 通用附件 5MB 上限，图片 3MB 上限。
- 图片二次读取真实尺寸，拦截超长边、超像素、超长宽比。
- 被拒附件通过 `ReceivedAttachment.error` 返回，且不进入 prompt、不入 KB。
- 渠道 ack 文案明确展示被拦截附件和原因。

## 验证

- `cargo test -p hone-channels`
- `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`

## 文档同步

- 更新 `docs/repo-map.md`
- 更新 `docs/current-plan.md`
- 完成后补 `docs/handoffs/2026-03-22-channel-attachment-gate.md`
