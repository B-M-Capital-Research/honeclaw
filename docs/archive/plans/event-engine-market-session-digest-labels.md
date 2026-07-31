- title: Event Engine 盘前盘后摘要标签与静音合并修复
- status: done
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - data/notif_prefs/discord__direct__*.json (ignored local runtime state)
  - crates/hone-event-engine/src/unified_digest/scheduler.rs
  - docs/invariants.md
- related_docs:
  - docs/current-plan.md
  - docs/handoffs/2026-07-31-event-engine-market-session-digest-labels.md

## Goal

修复北京时间 21:00 的盘前事件被命名为“盘后要闻”的配置反置，并让静音结束时刻与命名 digest slot 重合时沿用该 slot 的用户可见标签，避免盘后事件只能以“晨间静音合集”送达。

## Scope

- 将当前两份 Discord actor 偏好收敛为 `07:30 postmarket / 盘后要闻`、`21:00 premarket / 盘前要闻`，并把静音结束调整到 `07:30`。
- 在 unified digest scheduler 中复用命名 slot 的标签渲染静音结束合集；没有同刻命名 slot 时保留原有“晨间静音合集”。
- 增加单元回归，覆盖同刻命名 slot、无标签 slot 和无同刻 slot。
- 不修改 ExtendedHoursPoller 的美东盘前/盘后窗口判定，不触碰前一轮 SEC JSON 修复。

## Validation

- `cargo test -p hone-event-engine quiet_flush_ --lib`：3 passed。
- `cargo test -p hone-event-engine --lib`：527 passed，13 ignored。
- 本次 scheduler 文件 rustfmt check 与 `git diff --check` 通过；全仓 fmt 仅剩两个未改动 hone-channels 文件的既有差异。
- 已用 `jq` 核对所有本地 `data/notif_prefs/*.json` 的 `digest_slots` / `quiet_hours`，没有剩余反置的 Discord 配置。
- 只做配置与单元回归，不触发真实测试推送；等待下一次自然 21:00 / 07:30 投递作为线上验收。

## Documentation Sync

- 更新 `docs/invariants.md`，记录 quiet flush 与同刻命名 slot 的标签契约。
- 完成后写 handoff，计划页归档到 `docs/archive/plans/`，从 `docs/current-plan.md` 移除并更新 `docs/archive/index.md`。

## Risks / Open Questions

- `data/notif_prefs/` 是忽略的本机运行态，修复会即时生效但不会进入 Git；代码回归负责防止静音结束时再次丢失命名标签。
- 新构建已受控重启，Discord 重新登录且 8077/8088 均返回 HTTP 200；最终用户可见验收仍依赖下一次自然推送。
