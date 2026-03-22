# 2026-03-19 群聊回复追加链路统一

## 结论

- 已统一 Discord / Telegram / Feishu 的群聊回复可见语义：
  - 首条占位消息保留 `@用户 + 正在思考中...`
  - 中间的 tool reasoning 会在同一条占位消息内累加展示
  - 最终首条回复统一变成 `@用户 + 完整回复`
  - 如果被截断成多条消息/卡片，后续段会 reply 前一条，保持阅读链路连续
- 私聊沿用同样的“同一条占位消息内累加中间态，最终替换成正文”逻辑，只是不补 `@用户`

## 本次改动

- `crates/hone-channels/src/outbound.rs`
  - 为 Telegram / Discord 直连回复引入 reasoning 累加 transcript，在同一条占位消息里追加中间态
- `bins/hone-discord/src/utils.rs`
  - `DiscordOutboundAdapter` 新增 `reply_prefix` / `show_reasoning`
  - `send_or_edit_segments` 改为首段 edit/发送后，后续分段 `reference_message` 前一条消息
- `bins/hone-discord/src/group_reply.rs`
  - 群聊占位符固定为 `@目标用户 + 正在思考中...`
  - 群聊 listener 改为在同一条占位消息里累加 reasoning，而不是关闭可见性
  - 最终正文由系统统一补 mention，prompt 中提示模型不要重复输出平台 @ 语法
- `bins/hone-telegram/src/main.rs`
  - 群聊占位符补 Telegram mention
  - reasoning 更新改为在同一条占位消息里累加；重复内容不再反复编辑
  - 后续分段通过 `ReplyParameters::new(previous_message_id)` 串成回复链
- `crates/hone-channels/src/outbound.rs`
  - 共享出站层为 Telegram / Discord 直连回复引入 reasoning 累加 transcript
  - 最终回复仍走原有分段发送逻辑，只在占位消息更新阶段显示累加中的中间态
- `bins/hone-feishu/src/client.rs`
  - 增加 `send_chat_message(...)`
  - 增加 `reply_message(...)`
- `bins/hone-feishu/src/main.rs`
  - 群聊发送目标显式切到 `chat_id`
  - 群聊占位符 / 最终回复统一补 `<at id="..."></at>`
  - 私聊和群聊都改为在同一条占位卡片里累加 reasoning，中间态结束后再整条替换成正文
  - 多卡片分段通过 `reply_message(...)` 回复上一张卡片

## 验证

- `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`
- `cargo test -p hone-channels progress_transcript_appends_entries_to_placeholder -- --nocapture`
- `cargo test -p hone-channels progress_transcript_skips_duplicate_entries -- --nocapture`
- `cargo test -p hone-feishu feishu_progress_transcript_appends_entries -- --nocapture`
- `cargo test -p hone-feishu feishu_progress_transcript_skips_duplicate_entries -- --nocapture`

## 未完成 / 风险

- 还没有做三渠道真人群聊 smoke test，尤其是：
  - Feishu 群聊里 `<at id="..."></at>` 在卡片 markdown 中的最终渲染
  - Telegram HTML mention 与 reply chain 在真实群里的展示
  - Discord reply chain 在被截断多段时的实际通知行为
