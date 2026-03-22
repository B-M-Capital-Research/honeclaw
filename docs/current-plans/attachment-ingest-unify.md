# IM 渠道共享入口收口

## 目标

- 将 Telegram / Discord / 飞书 / iMessage 的重复入口能力收敛到 `crates/hone-channels`。
- 覆盖统一入站 envelope、actor scope、消息去重、session 串行锁、附件 ingest/KB 管线、统一出站 placeholder/错误/分段发送。
- 保留渠道特有的传输协议、身份校验、mention 语义和富文本渲染。

## 进展

- Telegram 入口已开始接入媒体附件入站：图片、文档、音频、视频、语音、动图会先下载到本地，再走共享附件 ingest / KB 管线。
- Telegram 对 `media_group_id` 的相册消息做了短窗合批，避免多图相册被拆成多次各算一个附件。
- 共享 actor sandbox 默认根目录已从 `/tmp/hone-agent-sandboxes` 收敛到工作区内 `data/agent-sandboxes`；Telegram / 飞书的附件下载临时目录也统一走该工作区根，避免 `opencode_acp` 读取工作区外路径失败。

## 涉及文件

- `crates/hone-channels/src/ingress.rs`
- `crates/hone-channels/src/outbound.rs`
- `crates/hone-channels/src/attachments.rs`
- `crates/hone-channels/src/lib.rs`
- `bins/hone-imessage/src/main.rs`
- `bins/hone-telegram/src/main.rs`
- `bins/hone-discord/src/attachments.rs`
- `bins/hone-discord/src/handlers.rs`
- `bins/hone-discord/src/main.rs`
- `bins/hone-discord/src/types.rs`
- `bins/hone-discord/src/utils.rs`
- `bins/hone-feishu/src/main.rs`
- `docs/repo-map.md`
- `docs/current-plan.md`
- `docs/handoffs/`（完成后补对应收口 handoff）

## 验证

- `cargo check -p hone-channels -p hone-imessage -p hone-telegram -p hone-discord -p hone-feishu`
- `cargo test -p hone-channels`

## 文档同步

- 任务开始时更新 `docs/current-plan.md`
- 如模块边界变化，同步更新 `docs/repo-map.md`
- 完成后新增对应的 `docs/handoffs/*.md`
