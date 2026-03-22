# 2026-03-17 register-admin 运行时拦截

## 结果

- 新增工程侧硬拦截：消息文本命中 `'/register-admin AMM'` 或 `/register-admin AMM` 时，不再进入 agent
- 命中后把当前 `ActorIdentity` 写入 `HoneBotCore` 的运行时管理员 override 集合，并直接回复确认文案
- 管理员判定新增 `is_admin_actor`，优先支持按 actor 的运行时提权；`restart_hone` 工具注册与后续 session/scheduler 均可识别

## 影响范围

- `crates/hone-channels/src/core.rs`
  - 新增运行时管理员 override、拦截匹配与单测
- 渠道入口：
  - `bins/hone-imessage/src/main.rs`
  - `bins/hone-feishu/src/main.rs`
  - `bins/hone-telegram/src/main.rs`
  - `bins/hone-discord/src/handlers.rs`
  - `bins/hone-discord/src/group_reply.rs`
  - `bins/hone-discord/src/scheduler.rs`
  - `crates/hone-web-api/src/routes/chat.rs`
  - `crates/hone-web-api/src/routes/events.rs`

## 约束

- override 仅驻留内存，不修改 `config.yaml`，进程重启后失效
- override 按 `ActorIdentity(channel, user_id, channel_scope)` 生效，不会扩散到同用户的其它 scope
- Feishu 保留原有 email/mobile/open_id 静态管理员判断，并额外支持当前 actor 的运行时 override

## 验证

- `cargo test -p hone-channels`
- `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`

