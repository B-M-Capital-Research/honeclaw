- title: 流式 HTTP 529 有界重试
- status: done
- created_at: 2026-09-08
- updated_at: 2026-09-08
- owner: bug-2
- related_files: crates/hone-llm/src/openai_compatible.rs
- related_docs: docs/bugs/scheduler_heartbeat_minimax_http_transport_failure_no_retry.md

## Goal
修复流式 5xx 在单 key 下绕过现有重试预算的问题。P1 runtime 缺席仍受 PostgreSQL 不可达阻塞，本轮禁止重启，不作虚假闭环。

## Scope
仅响应头阶段的 5xx；复用同 key 重试预算和退避，不重放已开始消费的流。

## Validation
本地 HTTP mock 覆盖 529 恢复、耗尽、零预算和 4xx；hone-llm 单元测试与 all-targets 编译检查、改动格式及 diff 检查。任何测试失败即停止提交推送。

## Documentation Sync
更新 bug 文档、docs/bugs/README.md、docs/invariants.md；完成后写 docs/handoffs/stream-http-529-retry-2026-09-08.md，计划归档并同步 docs/archive/index.md 与 docs/current-plan.md。

## Risks / Open Questions
上游持续不可用仍需明确失败；运行态自然部署后复核，本轮不构建 app、不重启、不发布。

## Completion
全部代码、回归、编译和文档步骤完成；41 测试通过。无模块边界变化，无需修改 repo-map 或 decisions。计划归档，运行态验证留在 handoff。
