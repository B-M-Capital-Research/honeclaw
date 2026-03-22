# 群聊@触发与定时任务限制（2026-03-14）

## 变更摘要
- 群聊仅在被 @ 时响应：Discord 群聊强制 direct mention；飞书群聊新增 mention 检测；Telegram 保持 @/回复触发。
- 群聊禁用定时任务创建：AgentSession 增加 `with_cron_allowed`，群聊场景不注册 `cron_job` 工具。

## 关键改动
- Discord：群聊未 @ 直接忽略；群聊/群聊 slash 禁用 cron；群聊批处理会话禁用 cron。
  - `bins/hone-discord/src/main.rs`
- Telegram：群聊会话禁用 cron。
  - `bins/hone-telegram/src/main.rs`
- 飞书：解析 @ 触发（基于 mentions/at 标签/`<at ...>`），群聊未 @ 忽略；群聊会话禁用 cron。
  - `bins/hone-feishu/src/main.rs`
- 核心：`create_tool_registry` 支持 `allow_cron`，`AgentSession` 增加 `with_cron_allowed`。
  - `crates/hone-channels/src/core.rs`
  - `crates/hone-channels/src/agent_session.rs`

## 验证
- 未运行测试（逻辑改动，需手动在 Discord/Telegram/飞书群聊验证 @ 触发与 cron 禁用）

## 后续关注
- 飞书 mention 解析依赖 `mentions` / `post` 的 `at` 标签与 `<at ...>` 兜底；如遇漏判需基于真实事件样本调整解析逻辑。
