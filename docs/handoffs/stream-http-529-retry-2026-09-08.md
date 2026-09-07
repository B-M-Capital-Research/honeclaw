- title: 流式 HTTP 529 有界重试
- status: done
- created_at: 2026-09-08
- updated_at: 2026-09-08
- owner: bug-2
- related_files: crates/hone-llm/src/openai_compatible.rs
- related_docs: ../archive/plans/stream-http-529-retry-2026-09-08.md, ../bugs/scheduler_heartbeat_minimax_http_transport_failure_no_retry.md, ../invariants.md
- related_prs: 无；本轮直接提交推送

## Summary
修复流式 529 未进入同 key 重试预算的问题，P2 标为 Fixed；当前唯一 P1 runtime 缺席仍为 New。

## What Changed
5xx 响应在读取流前复用现有传输预算与退避；本地 HTTP mock 验证恢复、耗尽、零预算及 4xx 边界。缺陷文档与导航同步；未改变模块边界，无需更新 repo-map/decisions。

## Verification
- `cargo test -p hone-llm stream_http_errors_use_bounded_same_request_retries --lib`：通过。
- `cargo test -p hone-llm --all-targets`：41 passed。
- `cargo check -p hone-llm --all-targets`：通过。
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-llm/src/openai_compatible.rs` 与 `git diff --check`：通过。
- 默认格式脚本最初未发现未提交文件，因此另对实际改动文件直接 rustfmt --check。无前端/Worker 修改，未运行无关测试；没有运行完整 workspace PostgreSQL 测试。

## Risks / Follow-ups
- 未重启、部署或重建 app；上游持续不可用仍可能耗尽预算，不承诺每次送达。非流式请求及 required-to-Auto 独立兼容请求保持原行为。
- P1 现有日志显示 PostgreSQL 连接失败启动 panic；源码要求 PostgreSQL 为唯一权威存储，不能以 SQLite 降级绕过。需运维核对实际运行配置的数据库可达性，随后验证 `/api/meta` 和 scheduler/task_runs 增量。
- 部署后在既有日志中核对 529 后恢复和 delivered 结果；本轮 mock 已覆盖实际 HTTP 序列，不能冒充 live 验证。

## Next Entry Point
`docs/bugs/runtime_process_absent_stops_channels_and_scheduler.md`（P1）；本次 P2 等待自然部署后复核。
