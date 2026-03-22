# 群聊回复追加链路统一

最后更新：2026-03-19
状态：已完成

## 目标

- 让支持编辑的 IM 渠道在群聊回复时统一采用“追加链路”。
- 群聊首条回复需要 @ 被回复用户。
- 多段回复时，后续消息必须 reply 前一条消息，避免被其他消息插入打断阅读。
- 最终落盘的首条回复内容应为 `@用户 + 完整回复`，工具调用过程消息不应保留在最终可见链路里。

## 涉及文件

- `bins/hone-discord/src/group_reply.rs`
- `bins/hone-discord/src/utils.rs`
- `bins/hone-discord/src/handlers.rs`
- `bins/hone-feishu/src/main.rs`
- `bins/hone-feishu/src/client.rs`
- `bins/hone-telegram/src/main.rs`
- `docs/current-plan.md`
- `docs/technical-spec.md`
- `docs/repo-map.md`
- `docs/handoffs/*.md`

## Todo

- [x] 梳理 Discord / Feishu / Telegram 的群聊回复发送接口，补齐“首条 mention + 后续 reply 前一条”的链式发送能力。
- [x] 调整群聊回复中的可见内容构造，确保首条消息带用户 mention，后续分段只保留正文并引用前一条消息。
- [x] 处理工具调用的可见性，避免工具调用过程内容落入最终可见回复链。
- [x] 验证：`cargo check -p hone-discord -p hone-feishu -p hone-telegram`、`cargo test -p hone-discord -p hone-telegram`
- [x] 文档同步：更新 `docs/current-plan.md`，并补 `docs/handoffs/2026-03-19-group-reply-append-chain.md`

## 当前进展

- Discord：
  - 群聊占位符保持为 `@用户 + 正在思考中...`
  - 群聊 tool reasoning 不再覆盖占位符
  - 最终回复首段统一补 `@用户`，后续分段 reply 前一条消息
- Telegram：
  - 群聊占位符保持为 `@用户 + 正在思考中...`
  - 群聊 tool reasoning 已隐藏
  - 多段消息通过 `ReplyParameters` 串成回复链
- Feishu：
  - 群聊发送目标从用户 `open_id` 明确拆为群 `chat_id`
  - 群聊占位符和最终回复统一补 `<at id="..."></at>`
  - 多卡片分段通过 `POST /im/v1/messages/:message_id/reply` 串成回复链

## 阻塞

- 无。

## 风险

- 尚未补三渠道的集成 / 人工回归，只完成了编译校验；真实群聊里的平台渲染细节仍建议补一轮 smoke test。
