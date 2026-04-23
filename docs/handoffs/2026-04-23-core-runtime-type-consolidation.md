# Core Runtime 职责与类型收敛

- title: Core Runtime 职责与类型收敛
- status: done
- created_at: 2026-04-23
- updated_at: 2026-04-23
- owner: codex
- related_files:
  - `crates/hone-channels/src/agent_session.rs`
  - `crates/hone-channels/src/turn_builder.rs`
  - `crates/hone-channels/src/response_finalizer.rs`
  - `crates/hone-channels/src/run_event.rs`
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-core/src/channel_process.rs`
  - `packages/app/src/lib/types.ts`
  - `tests/fixtures/local_image_markers.json`
- related_docs:
  - `docs/archive/plans/core-runtime-type-consolidation.md`
  - `docs/archive/index.md`
  - `docs/repo-map.md`
- related_prs:
  - N/A

## Summary

本轮完成了核心运行链路里高优先级的职责与类型收敛，没有改动用户可见聊天行为、Web SSE wire event names 或 session storage version。

## What Changed

- `AgentSession` 现在把 prompt/skill turn 构建委托给 `turn_builder.rs`，把最终回复清洗、空回复兜底和本地图片稳定化委托给 `response_finalizer.rs`。
- 新增 `run_event.rs` 作为内部 canonical run event；runner event 直接复用，session event 仅再包装 `UserMessage / Segment / Done`。
- 新增 `AgentRunnerKind` / runner probe helper，`core.rs`、CLI 和 desktop runner 检测逻辑改用统一来源。
- 前端补齐 `HistoryAttachment` 与 `HistoryMsg.attachments`；core OS 扫描 DTO 改名为 `ObservedChannelProcess`。
- 新增 Rust/前端共享的本地图片 marker fixture；event engine 删除重复 `severity_rank`，改用 `Severity::rank()`。

## Verification

- `rtk cargo test -p hone-channels agent_session`
- `rtk cargo test -p hone-channels runners::tests`
- `rtk cargo test -p hone-event-engine subscription`
- `rtk cargo test -p hone-web-api routes::history`
- `rtk bun run test:web`
- `rtk bun --filter @hone-financial/app typecheck`
- `rtk cargo check --workspace --all-targets --exclude hone-desktop`
- `rtk cargo test --workspace --all-targets --exclude hone-desktop`
- `rtk bash tests/regression/run_ci.sh`

## Risks / Follow-ups

- 工作区里仍有本轮之外的 runtime 相关未提交改动，后续继续处理时要基于当前脏树叠加。
- company profile import 的 section-level merge 类型关系没有纳入本轮，后续若处理这块需要单独建计划。
- `cargo test --workspace --all-targets --exclude hone-desktop` 期间有一次 `ModuleNotFoundError: No module named 'matplotlib'` 的 stderr，但整个测试进程最终返回 0；如果后续把相关示例提升为硬门禁，需要先补稳定 Python 依赖策略。

## Next Entry Point

- `crates/hone-channels/src/agent_session.rs`
