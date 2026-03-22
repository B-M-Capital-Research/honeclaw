# 多渠道附件工程化卡点

## 目标

- 为 Feishu / Discord / Telegram 共用的附件 ingest 增加统一工程化卡点，拦截超限附件和异常图片。

## 结果

- 共享附件链路新增通用附件大小、图片大小、图片真实尺寸与长宽比校验。
- 被拒附件不会进入 prompt，也不会进入 KB 持久化。
- 渠道 ack 文案会汇总被拦截的附件与原因，便于用户理解为什么没处理。

## 验证

- `cargo test -p hone-channels`
- `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`

## 风险与后续

- 当前图片尺寸探测依赖文件可被本地格式探测器识别；若后续引入更多图片格式，需同步补 feature 或做 graceful fallback。
- Web / KB 独立上传入口暂未接入相同限制，如未来也需要统一策略，应复用 `hone-channels` 的准入 helper。
