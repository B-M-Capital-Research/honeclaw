# IM 渠道共享入口收口

日期：2026-03-17

## 结果

- 在 `crates/hone-channels` 新增共享 `ingress` / `outbound` 模块：
  - `IncomingEnvelope`
  - `ActorScopeResolver`
  - `MessageDeduplicator`
  - `SessionLockRegistry`
  - `attach_stream_activity_probe`
  - 统一 `run_session_with_outbound`
- 将 Discord / Telegram / 飞书 / iMessage 的消息去重、session 串行锁、actor scope、群聊触发判定逐步收口到共享层。
- 将 Discord / 飞书的附件 ingest、落盘、解压/PDF 提取、KB 入库与分析触发收敛到 `crates/hone-channels/src/attachments.rs`。
- 去掉 Feishu / iMessage 的 `runner == "gemini_cli"` 执行分支，改为统一消费 `AgentSessionEvent::StreamDelta`；是否走流式展示不再由渠道硬编码 runner 决定。
- Discord 群聊触发不再在入口层强制只认 direct mention，`question_signal` 已交由共享 `GroupTriggerMode` 判定。
- 飞书群聊 actor 已补 `chat:<chat_id>` scope，不再只按 `open_id` 聚合 session。

## 仍保留在渠道层

- 平台协议接入：iMessage `chat.db` / AppleScript、Telegram long polling、Discord gateway、飞书 SDK/facade
- 平台身份与白名单
- 平台 mention / reply 语义解析
- 平台消息渲染与富文本限制
- Discord slash command、飞书 CardKit 这类平台专有入口

## 验证

- `cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`
- `cargo test -p hone-channels`
- `cargo check --workspace --all-targets`

## 已知边界

- `hone-desktop` 仍会基于 runner 名称检查本地二进制可用性，这属于桌面配置/诊断，不属于渠道消息处理链路。
