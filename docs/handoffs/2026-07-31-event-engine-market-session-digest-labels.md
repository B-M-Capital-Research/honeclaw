- title: Event Engine 盘前盘后摘要标签与静音合并修复
- status: done
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/unified_digest/scheduler.rs
  - docs/invariants.md
  - data/notif_prefs/discord__direct__*.json (ignored local runtime state)
- related_docs:
  - docs/archive/plans/event-engine-market-session-digest-labels.md
  - docs/archive/index.md
- related_prs: []

## Summary

2026-07-30 21:01 的 Discord 摘要使用 `postmarket / 盘后要闻` 槽位，但其中三条 ExtendedHours 事件均为 `window=pre`。北京时间 21:00 在美股夏令时对应美东 09:00，本应是盘前摘要。根因是两份 Discord actor 偏好把 `07:30/21:00` 的盘后/盘前名称反置；同时原 `23:00–07:00` quiet flush 会先清空缓冲，使 07:30 命名盘后摘要失去内容。

## What Changed

- 两份本地 Discord actor 偏好统一为：
  - `07:30`：`postmarket / 盘后要闻`
  - `21:00`：`premarket / 盘前要闻`
  - `quiet_hours`：`23:00–07:30`
- `UnifiedDigestScheduler` 在 quiet flush 与某个 actor digest slot 同刻时复用该 slot 的 label；没有同刻 slot 时继续使用“晨间静音合集”。
- 提取统一的 slot label fallback，普通 digest 与 quiet flush 不再各自实现不同规则。
- 新增三条单元回归，覆盖同刻命名 slot、同刻无标签 slot、没有同刻 slot。
- 更新通知投递不变量，明确命名盘前/盘后 slot 的用户可见语义不得在 quiet flush 中丢失。

## Verification

- `cargo test -p hone-event-engine quiet_flush_ --lib`：3 passed。
- `cargo test -p hone-event-engine --lib`：527 passed，13 ignored，0 failed。
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-event-engine/src/unified_digest/scheduler.rs`：通过。
- `git diff --check`：通过。
- `cargo build --bin hone-cli --bin hone-discord`：通过。
- 全量 `cargo fmt --all -- --check` 仍被未改动的 `crates/hone-channels/src/agent_session/artifacts.rs` 与 `crates/hone-channels/src/core/bot_core.rs` 既有格式差异阻塞；本次文件无格式差异。
- `jq` 扫描所有 `data/notif_prefs/*.json`：两份 Discord 偏好均为正确映射，没有剩余反置配置。
- 受控 SIGINT 停止旧 supervisor 后以新构建重启；管理端 `8077`、用户端 `8088` 均返回 HTTP 200，Discord 成功重新登录。
- 未运行 live LLM/news classifier baseline：本次是确定性 scheduler label 行为，不涉及分类器、prompt 或模型。

## Risks / Follow-ups

- `data/notif_prefs/` 是忽略的本机运行态，不进入 Git；当前运行时每次读取该目录，修复已经生效。
- 没有主动发送 Discord 测试消息。下一次自然 21:00 推送应显示“盘前要闻”，下一次自然 07:30 quiet flush 应显示“盘后要闻”；届时应从 `delivery_log` 做最终用户可见验收。

## Next Entry Point

在下一次自然槽位后查询 `data/events.sqlite3`：核对 `unified-digest:*@slot:premarket:*` 的 21:00 标题，以及 `quiet-flush:*@07:30:*` 的 07:30 标题和所含 ExtendedHours `window`。
