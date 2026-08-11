- title: Continuous Research Ten-day Brief
- status: archived
- created_at: 2026-08-11
- updated_at: 2026-08-11
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/key_event_chain.rs
  - packages/app/src/components/key-event-chain-dashboard.tsx
  - packages/app/src/components/key-event-chain-dashboard.css
  - packages/app/src/lib/types.ts
- related_docs:
  - docs/decisions.md#d-2026-08-11-06-treat-the-next-ten-days-as-a-verification-queue-not-a-prediction-calendar
  - docs/handoffs/2026-08-11-key-event-chain-and-serenity-source.md

## Goal

在关键事件链详情中实现“前十日复盘 + 后十日验证问题”，把点状来源事件变成可连续复盘的研究视图，同时不把模板问题、作者观点或聚合翻译伪装为未来事实。

## Completed Scope

- 过去十日仅保留当前快照内有原链的事件，输出主题复盘、方向状态和证据 ID。
- 未来十日输出带复查截止日的开放验证问题；无当前证据时标记 `waiting_for_source`。
- 相对上次快照按事件 ID 计算新增数量，首次运行只建立版本基线。
- 宽泛标题优先提取包含主题关键词的证据句，避免复盘卡偏题。
- 十日简报作为关键事件链二级视图，不新增首页入口；保存后可连同证据边界发送到对话。

## Validation

- Key-event chain Rust 9/9；Web API 258 passed / 2 ignored。
- Web 438/438；TypeScript；public production build。
- 本地 Web-only worker 生成真实 source-only 十日快照。
- 认证浏览器验收事件链/十日切换、桌面布局、原链、开放/等待状态和问答承接。

## Documentation Sync

- Updated `docs/repo-map.md`, `docs/decisions.md`, the existing same-theme handoff and `docs/archive/index.md`.

## Risks / Open Questions

- Serenity 的公开历史深度有限；无命中不等于主题无变化。
- 模型未配置时影响方向保持待验证。
- 复查截止日不是事件日期，未来接入一手事件日历时必须使用独立字段和来源契约。
