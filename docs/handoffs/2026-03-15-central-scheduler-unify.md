# 调度器中心化与全渠道接入（2026-03-15）

## 变更摘要
- 将定时任务执行逻辑下沉到 `hone-channels` 新模块 `scheduler`，由各渠道统一调用。
- Discord 新增 scheduler 启动与调度事件处理，支持定时任务推送到 Discord 频道/私聊。
- 保持 channel 过滤逻辑，仅投递当前渠道任务。

## 关键改动
- `crates/hone-channels/src/scheduler.rs`：新增 `build_scheduled_prompt` 与 `run_scheduled_task` 共享实现。
- `bins/hone-console-page/src/routes/events.rs`：改用共享 scheduler 执行逻辑。
- `bins/hone-feishu/src/main.rs`：改用共享 scheduler 执行逻辑。
- `bins/hone-telegram/src/main.rs`：改用共享 scheduler 执行逻辑。
- `bins/hone-discord/src/main.rs` + `bins/hone-discord/src/scheduler.rs`：新增 Discord 调度器与投递流程。
- `bins/hone-discord/src/utils.rs`：新增 `parse_channel_id_from_target`，支持从 `dm:` / `guild:...:channel:` 解析 channel_id。

## 验证
- `cargo check -p hone-channels -p hone-console-page -p hone-feishu -p hone-telegram -p hone-discord`

## 注意事项
- Discord 群聊仍禁用 cron（保持既有限制）；仅私聊可创建定时任务。
- Discord 定时任务 target 解析依赖 `dm:` 或 `guild:...:channel:` 结构，若后续 target 格式变化需同步更新解析逻辑。
