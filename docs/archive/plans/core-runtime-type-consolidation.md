# Core Runtime 职责与类型收敛

- title: Core Runtime 职责与类型收敛
- status: done
- created_at: 2026-04-23
- updated_at: 2026-04-23
- owner: codex
- related_files:
  - `docs/current-plan.md`
  - `docs/archive/index.md`
  - `docs/handoffs/2026-04-23-core-runtime-type-consolidation.md`
  - `crates/hone-channels/src/run_event.rs`
  - `crates/hone-channels/src/turn_builder.rs`
  - `crates/hone-channels/src/response_finalizer.rs`
  - `crates/hone-channels/src/agent_session.rs`
  - `crates/hone-channels/src/runners/types.rs`
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-core/src/channel_process.rs`
  - `crates/hone-web-api/src/types.rs`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/repo-map.md`
  - `docs/current-plans/acp-runtime-refactor.md`

## Outcome

- `AgentSession` 保留 `run()` 公共入口，内部 prompt/skill turn 构建迁到 `turn_builder.rs`，输出清洗、空回复兜底与本地图片 marker 稳定化迁到 `response_finalizer.rs`。
- 新增 `run_event.rs` 作为 runner/session 内部 canonical run event；runner event 直接复用该类型，session event 只额外表达 `UserMessage / Segment / Done`。
- 新增 `AgentRunnerKind` 与 runner probe/self-managed-context helper；`core.rs`、CLI 与 desktop runner 检测逻辑改用统一来源。
- `HistoryMsg.attachments` / `HistoryAttachment` 已补到前端类型；core OS 进程扫描 DTO 已改名为 `ObservedChannelProcess`，避免和 Web API `ChannelProcessInfo` 同名。
- 本地图片 marker 已新增 Rust/前端共享 fixture；event engine 的重复 severity rank 已删除，统一使用 `Severity::rank()`。

## Goal

收敛核心运行链路里已经扩散的职责和重复类型，优先处理不改变用户可见行为、SSE 事件名或 session storage version 的高价值重构。

## Scope

- 拆分 `AgentSession` 内部的 prompt/skill turn 构建与 response finalization。
- 收敛 runner/session 事件父子关系，保留 Web SSE wire 事件名不变。
- 修正前端 `HistoryMsg` 与 Rust API DTO 的明显漂移。
- 将 runner 字符串逻辑收敛到统一 helper，并清理同名不同义 DTO。
- 补本地图片 marker 解析契约测试，先不引入 Rust-to-TS 类型生成。

## Validation

- done: `rtk cargo check -p hone-channels --all-targets`
- done: `rtk cargo check -p hone-core --all-targets`
- done: `rtk cargo check -p hone-web-api --all-targets`
- done: `rtk cargo check -p hone-event-engine --all-targets`
- done: `rtk bun --filter @hone-financial/app typecheck`
- done: `rtk cargo test -p hone-core agent_runner_kind_keeps_wire_values_and_probe_mapping -- --nocapture`
- done: `rtk cargo test -p hone-channels local_image_marker_contract_matches_shared_fixture -- --nocapture`
- done: `rtk cargo test -p hone-channels agent_session`
- done: `rtk cargo test -p hone-channels runners::tests`
- done: `rtk cargo test -p hone-event-engine subscription`
- done: `rtk cargo test -p hone-web-api routes::history`
- done: `rtk bun run test:web`
- done: `rtk cargo check --workspace --all-targets --exclude hone-desktop`
- done: `rtk cargo test --workspace --all-targets --exclude hone-desktop`
- done: `rtk bash tests/regression/run_ci.sh`

## Documentation Sync

- done: `docs/repo-map.md` 已补充 `run_event` / `turn_builder` / `response_finalizer` 边界。
- done: `docs/current-plan.md` 已移除活跃任务入口。
- done: 本计划页已归档到 `docs/archive/plans/core-runtime-type-consolidation.md`。
- done: `docs/archive/index.md` 与 handoff 已补入口。
- not needed: 本次没有改变长期测试规则、目录约定或用户可见运行契约，不更新 `docs/invariants.md`。

## Risks / Open Questions

- 当前工作区已有 runtime 相关未提交改动；实现必须叠加在现状上，不能回滚。
- company profile import 的 section-level merge 类型暂不纳入本轮合并范围。
- 事件类型收敛必须保持 Web SSE 事件名和前端状态机行为不变。
