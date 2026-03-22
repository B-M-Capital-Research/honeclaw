# IM 渠道入站功能差异矩阵

日期：2026-03-17
状态：已完成

## 目标

- 深度梳理当前仓库中各 IM 渠道“消息进入 Hone 后”的能力边界与行为差异。
- 输出能直接回答“每个 IM 渠道进来的功能差异”问题的矩阵。

## 统一前提

- 四个 IM 渠道最终都收敛到 `AgentSession::run()`，共享会话恢复、额度控制、tool registry 与 runner 选择。
- 共享层当前默认恢复最近 12 条消息；同一 session 的整次 `run()` 有全局串行锁；用户对话默认受每日额度限制，定时任务路径跳过该额度。
- 群聊场景会在渠道侧显式禁用 cron 工具；格式约束通过 `channel_format_guidance()` 注入 prompt。

## 差异矩阵

| 维度 | iMessage | Telegram | Discord | 飞书 |
|---|---|---|---|---|
| 入站方式 | 本机轮询 `chat.db` | Bot long-polling `getUpdates` | Gateway event | 官方长连接事件，经 Go facade + Rust 业务层 |
| 事件/消息类型 | 仅文本；要求 `service='iMessage'` 且近 5 分钟新消息 | `text()` / `caption()` + 媒体附件（photo/document/audio/video/voice/animation）；支持 `media_group_id` 短窗合批；纯媒体消息不再直接忽略 | 文本 + 附件；另有 `/skill` slash command | `text` / `image` / `file` / `post`，并会下载附件 |
| 身份/准入 | `target_handle` 单点过滤；无 allowlist | `allow_from` + `dm_only` | `allow_from` + `dm_only` | 邮箱/手机号/open_id 白名单三层校验 + `dm_only` |
| 群聊触发 | 无群聊概念 | 群里必须 @ 机器人或回复机器人 | 当前群聊必须 direct mention；再进入 channel 级聚合回复 | 群里必须被 @；支持 `mentions` / `<at ...>` / post `at` 解析 |
| 群聊会话隔离 | 不适用 | `channel_scope=chat:<chat_id>` | `channel_scope=g:<guild>:c:<channel>` | 当前 actor 仅按 `open_id`；群消息不额外带 scope |
| 幂等 / 并发保护 | `handle+text` 120 秒短期去重 | 单实例锁防双进程 `getUpdates`；无消息级 dedup | 群聊靠单 channel worker 串行；DM 无渠道级 dedup | `message_id` 60 秒 dedup + per-session async mutex 串行化 |
| 附件入站 | 不支持 | 支持图片/文档/音频/视频/语音/动图入站，走共享附件 ingest/KB；暂不覆盖所有 Telegram 媒体类型 | 支持下载附件、解压压缩包、提取 PDF、入 KB | 支持图片/文件/post 附件、MIME 识别、PDF 提取、入 KB |
| 中间态反馈 | 先回 ACK，再流式/分段发回 iMessage | 占位消息 + 首条编辑 | 占位消息 + 首条编辑；群聊占位可带 @ | 占位卡片/消息；Gemini runner 下有 ticker/CardKit 流式更新 |
| 输出格式约束 | 纯文本优先 | HTML parse mode | Discord Markdown | 飞书卡片 Markdown，带预处理 |
| 定时任务创建 | 允许；另带本地 HTTP 投递入口 | 仅私聊允许 | 仅 DM / 非 guild slash 允许 | 仅 `p2p` 允许 |
| 渠道独有能力 | 本机 AppleScript 发消息、控制台事件推送 | `getUpdates` 冲突锁、HTML 编辑回退 | 群聊短窗口聚合、slash skill、question signal 识别 | 长连接 + facade、联系人解析、message/session metadata、卡片渲染链路 |

## 关键结论

- **iMessage 最轻量，也最“本机特权”**：它没有 webhook / bot 平台能力，完全依赖 macOS 本机 `chat.db` 轮询和 AppleScript 发送，因此只有文本、无附件、无群聊，但有最强的本机直连能力。
- **Telegram 仍然是较轻的 Bot 渠道实现**：它的入口比 Discord/飞书简单，但已经补齐媒体附件入站，并对相册消息做了短窗合批，群聊仍只做 @/reply 触发，重点在稳定处理与 HTML 输出回退。
- **Discord 当前是群聊编排最重的渠道**：除了 DM 问答，它还有 guild 群聊聚合窗口、占位符编辑、slash skill、附件入 KB；但代码里虽然保留了 `question_signal` / `trigger_mode` 配置，当前真正进入群聊回复链路前仍先要求 direct mention。
- **飞书当前是企业 IM 适配最重的渠道**：入站类型最多、准入规则最细、消息幂等/串行化最完整、附件/卡片/联系人元数据也最完整。

## 重要细节

- Discord 配置层暴露了 `mention_only | mention_or_question | all`，但 `handle_group_message()` 里在入队前就要求 direct mention，所以“问题信号触发群回复”目前更像预留能力，不是完全生效能力。
- 飞书是唯一在渠道层同时做了“消息级 dedup + session 级串行化”的 IM 渠道；Telegram 和 Discord 目前主要依赖共享层 `AgentSession` 的 run 锁，入口侧保护不如飞书细。
- Telegram / Discord / 飞书都在群聊场景禁用了 cron 工具；iMessage 没有群聊路径，因此不受这条限制。
- 飞书和 Discord 都把附件异步写入 KB，并在 PDF 成功提取后触发股票信息分析；Telegram 现在也会把媒体附件写入共享附件管线，并对相册做短窗合批，iMessage 仍没有对应入站能力。

## 关键文件

- `bins/hone-imessage/src/main.rs`
- `bins/hone-telegram/src/main.rs`
- `bins/hone-discord/src/handlers.rs`
- `bins/hone-discord/src/group_reply.rs`
- `bins/hone-feishu/src/main.rs`
- `crates/hone-channels/src/agent_session.rs`
- `crates/hone-channels/src/prompt.rs`

## 验证

- 未运行测试。
- 结论来自代码与现有 handoff 交叉核对。

## 后续建议

- 若目标是“渠道能力收敛”，继续补 Telegram 更广的媒体类型覆盖，以及 Discord/Telegram 入口侧去重/串行化保护。
- 若目标是“群聊触发一致性”，需要先决定 Discord 是否真的支持 `question_signal` 独立触发；当前配置与实际行为存在落差。
- 若目标是“统一 actor 隔离模型”，飞书群聊应评估是否补 `channel_scope`，否则同一 open_id 在不同群可能共享 session/额度上下文。
