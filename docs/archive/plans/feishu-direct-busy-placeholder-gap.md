- title: Feishu Direct Busy Placeholder Gap
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: codex
- related_files:
  - bins/hone-feishu/src/handler.rs
  - docs/bugs/README.md
  - docs/bugs/feishu_direct_placeholder_without_agent_run.md
- related_docs:
  - docs/repo-map.md
  - docs/invariants.md
  - docs/runbooks/desktop-release-app-runtime.md

## Goal

为 Feishu 直聊链路补齐“同一 session 已有消息在处理中”时的明确 busy 策略，避免再次出现只发送 placeholder、但未真正进入 agent 处理主链路的假启动，并同步补充 bug 台账与新 release 构建。

## Scope

- 为 Feishu 直聊入口补充 session busy 检查与用户态提示。
- 保持群聊现有 pretrigger / busy 策略不退化。
- 为新缺陷补文档并同步更新 `docs/bugs/README.md`。
- 运行定向验证并重新构建 desktop release app。

## Validation

- 代码级验证 Feishu handler 的 direct busy 分支与 placeholder 发送顺序。
- 运行 Rust 定向测试。
- 执行 desktop release build，确认 `.app` / `.dmg` 产物生成成功。

## Documentation Sync

- 本任务涉及跨模块行为修复与 release 验证，满足动态计划准入标准，因此已落盘并完成归档。
- 修复与状态结论需要同步更新 `docs/bugs/README.md` 与对应 bug 文档。
- 本轮已补 handoff 并归档计划，同时更新 `docs/archive/index.md`。

## Risks / Open Questions

- 当前修复主要解决“placeholder 已发但实际未开始处理”的用户体验与并发入口策略，不直接根治深层 run 卡死根因。
- 若后续确认 `session.run()` 内部仍存在长时间卡住的独立原因，需要另建缺陷继续跟踪。
