# Notification Preference Time And Numeric Controls

- title: Notification Preference Time And Numeric Controls
- status: done
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - `crates/hone-core/src/quiet.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-tools/src/base.rs`
  - `crates/hone-tools/src/notification_prefs_tool.rs`
  - `crates/hone-tools/src/schedule_view.rs`
  - `crates/hone-web-api/src/routes/notification_prefs.rs`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/components/notification-preferences-model.ts`
- related_docs:
  - `docs/decisions.md#d-2026-07-31-01-keep-conversational-notification-controls-deterministic-and-domain-owned`
  - `docs/invariants.md#notification-delivery-constraints`
  - `docs/archive/plans/notification-prefs-time-numeric-controls.md`
- related_prs: none; committed directly to `main` in the 2026-07-31 notification repair change set

## Summary

普通渠道 Agent 现在可以可靠调整 actor-scoped 的确定性通知时间与数值：时区、具名摘要槽位、摘要宏观条目底线、勿扰时段、通用/上涨/下跌价格阈值和大仓位权重边界，并可逐项恢复继承。提示词、模型、分类策略和投资主线没有加入普通通知偏好工具。

## What Changed

- event-engine 新增 `NotificationDeliveryPatch` 与 `PreferenceUpdate::{Keep, Inherit, Set}`，所有补丁先在副本上整体校验，通过后才替换并保存
- `NotificationPrefs::validate` 统一校验 IANA 时区、`HH:MM`、槽位 id/时刻唯一性、标签边界、quiet overlap、kind tag 和数值范围；Agent 与 HTTP API 共用
- `set_digest_slots` 保持旧 `["07:30", "21:00"]` 兼容，同时支持 `{id,time,label,floor_macro}`；`null`/inherit、`[]`、非空列表分别表示继承、关闭、自定义
- 新增通用/上涨/下跌价格阈值和大仓位阈值的 set/inherit Agent action，并在 `get` / `get_overview` 中展示
- 新增 `update_delivery_controls` 复合 action，可把时区、摘要槽位、勿扰时段和四组数值作为一个最终状态原子校验/保存，避免合法迁移被旧值的中间冲突卡住
- Tool trait 新增可覆写的完整输入 schema；`notification_prefs.value` 正式声明 string/number/boolean/array/object/null 联合类型，不再依赖 Agent 违反字符串 schema
- Web API 类型补齐新增数值字段，但没有新增提示词或模型控制
- 本地时间是否落入 quiet window 的纯函数收口到 `hone-core::quiet`，领域校验和日程概览共用 `[from,to)` 语义

## Verification

- `cargo test -p hone-core --lib`: 132 passed
- `cargo test -p hone-event-engine --lib`: 532 passed, 13 ignored
- `cargo test -p hone-tools --lib`: 161 passed, 1 ignored
- `cargo test -p hone-tools --lib notification_prefs_tool::tests`: 35 passed
- `cargo test -p hone-web-api --lib routes::notification_prefs::tests`: 3 passed
- `bun run typecheck:web`: passed
- `bun run test:web`: 309 passed
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed with one unrelated existing `hone-web-api` dead-code warning
- Public Community Edge Worker `bun run typecheck && bun run test`: 45 passed
- `bash tests/regression/run_ci.sh`: 43/44 finance contracts passed; the only failure is the unchanged `origin/main` attachment-ingest phrase mismatch also covered by the unrelated failing full Web API assertion
- `git diff --check`: passed
- changed Rust files were formatted directly with `rustfmt`
- `cargo fmt --all -- --check` still reports only unrelated pre-existing formatting differences in `crates/hone-channels/src/agent_session/artifacts.rs` and `crates/hone-channels/src/core/bot_core.rs`
- full `cargo test -p hone-web-api --lib`: notification tests pass; aggregate result is 155 passed, 1 failed, 2 ignored because the unrelated unchanged test `public_chat_user_input_uses_shared_attachment_context` still expects an old attachment prompt phrase
- live LLM baseline was intentionally skipped because classifier code and prompts were not changed
- rebuilt `hone-cli`, `hone-console-page`, and `hone-discord`; final controlled restart produced CLI PID 25477, console PID 25480, Discord PID 25487, Discord login succeeded, and both 8077/8088 roots returned 200
- runtime readback confirmed both Discord actor preference files still use 07:30 `postmarket / 盘后要闻`, 21:00 `premarket / 盘前要闻`, and quiet hours 23:00–07:30

## Risks / Follow-ups

- No real conversational mutation was sent to a user actor during acceptance, to avoid changing live preferences merely for testing. Tool schema, domain, storage, overview, and API paths are covered by automated tests.
- The existing admin full-write API can still carry fields outside this conversational subset because it serializes the full `NotificationPrefs`; the ordinary Agent tool intentionally omits prompt/model/classifier/mainline mutation actions.
- This work is committed and pushed together with the user-approved SEC normalization and market-session digest fixes; no release or tag was created.

## Next Entry Point

Ask the Agent in any enabled channel for one safe setting change, then call `notification_prefs.get_overview` or inspect the actor row under `data/notif_prefs` to confirm the persisted value and inherited/null semantics.
