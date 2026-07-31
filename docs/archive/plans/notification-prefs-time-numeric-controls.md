# Notification Preference Time And Numeric Controls

- title: Notification Preference Time And Numeric Controls
- status: done
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - `crates/hone-core/src/quiet.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-tools/src/notification_prefs_tool.rs`
  - `crates/hone-tools/src/base.rs`
  - `crates/hone-tools/src/schedule_view.rs`
  - `crates/hone-web-api/src/routes/notification_prefs.rs`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/components/notification-preferences-model.ts`
  - `packages/app/src/components/notification-preferences-model.test.ts`
  - `docs/invariants.md`
  - `docs/decisions.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/handoffs/2026-07-31-notification-prefs-time-numeric-controls.md`

## Goal

让用户可以在普通 Agent 对话里可靠地调整通知的确定性时间与数值设置，包括具名摘要时段、价格阈值、大仓位阈值、静默时段以及单项恢复继承；提示词、模型和分类策略不进入本次可编辑范围。

## Scope

- 在 event-engine 偏好领域层建立 typed patch、三态继承语义和统一校验，Agent 工具与 Web API 共用
- 摘要时段支持稳定 `id`、显示名称和最少宏观条目数，同时兼容旧的纯时间字符串输入
- 支持统一、上涨、下跌价格阈值与大仓位阈值的设置和单项继承
- 保持 actor-scoped 存储与原有 REST 全量读写兼容
- 在通知概览中展示新增可调阈值
- 不开放 prompt、模型、分类器或其它非确定性策略字段

## Validation

- `cargo test -p hone-core --lib`
- `cargo test -p hone-event-engine --lib`
- `cargo test -p hone-tools --lib`
- `cargo test -p hone-web-api --lib`
- 聚焦验证结构化摘要时段、继承语义、数值边界、静默冲突和失败不落盘
- 对本次 Rust 文件执行格式检查，并运行 `cargo fmt --all -- --check` 记录任何无关既有失败
- 重建 `hone-cli` / `hone-discord`，受控重启后检查 `8077` / `8088`
- 本次不改新闻分类器或提示词，因此不运行 live LLM baseline

## Documentation Sync

- 更新 `docs/decisions.md`，记录确定性通知设置的领域补丁边界与提示词排除项
- 更新 `docs/invariants.md`，记录 Agent/API 共用校验和继承语义
- 完成后新增 handoff、更新 `docs/archive/index.md`，并把本计划移入 `docs/archive/plans/`
- 模块边界没有改变，因此无需更新 `docs/repo-map.md`

## Risks / Open Questions

- 必须区分“未修改”“恢复继承”和“显式空列表关闭摘要”，避免 `Option<Vec<_>>` 语义混淆
- 旧 Agent 调用仍可能发送 `["07:30", "21:00"]`，必须保持兼容
- 修改多个相关字段时必须先整体校验再原子保存，不能留下部分生效状态

## Result

- typed patch、统一校验、真实联合类型工具 schema、单项与复合原子 Agent action、Web API 复用和概览展示均已完成
- 分层回归与本地运行态重建/重启完成；详细证据见对应 handoff
- 模块边界未改变，`docs/repo-map.md` 无需更新
