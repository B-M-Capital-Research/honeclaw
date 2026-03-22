# 2026-03-13 Prompt 与 IM 渠道上线收口

## 归并范围

- `soul.md` 外置与 prompt 对齐
- 渠道格式指引注入
- Telegram / Discord 回复体验补齐
- 群聊隐私约束

## 已完成

- `agent.system_prompt_path` 落地，`config.yaml` / `config.example.yaml` 改为引用 `soul.md`，运行时按配置文件目录解析相对路径。
- `soul.md` 补齐技能触发约束与低信息问候/能力咨询指引，系统 prompt 从硬编码正文转为外部文件维护。
- 输出格式指引从全局 prompt 中拆出，改为按渠道注入；Discord / Telegram / iMessage / 飞书各自补齐限制说明。
- Telegram 完成首轮接入：支持私聊/群聊消息处理、调度任务投递、占位消息后编辑首段回复。
- Telegram 消息发送从 MarkdownV2 切换到 HTML parse mode，并补充 `<pre>/<code>` 伪表格建议。
- Telegram 的格式提示进一步补齐了官方支持的 HTML 语法边界：`<b>/<i>/<u>/<s>/<code>/<pre>/<a>`、`<tg-spoiler>`、`<blockquote>`、`<tg-emoji>`、`<tg-time>`，并强调了转义与嵌套限制。
- Discord 补齐占位符 + 编辑首条回复链路。
- Discord / 飞书 / Telegram 的群聊 prompt 增加隐私约束，禁止在群聊索要持仓明细或继续追问个人敏感数据。

## 验证

- `cargo check -p hone-discord`
- `cargo check -p hone-feishu -p hone-imessage`
- `cargo check -p hone-telegram`

## 未验证项

- Telegram 富文本与占位消息体验主要依赖手工验证；当时未补自动化回归。

## 备注

- 本文档替代原先拆散的 Telegram / Discord / prompt / 群聊隐私相关 handoff 与 plan 页。
- `prompt-loading.md` 属于阶段性阅读结论，已被 `soul.md` 外置后的真实实现覆盖，不再单独保留。
