# 单股投研门禁误伤定时任务修复

- title: 单股投研门禁误伤定时任务修复
- status: archived
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-channels/src/investment_response_guard.rs
- related_docs:
  - docs/handoffs/2026-07-16-investment-guard-scheduler-routing-fix.md
  - docs/decisions.md
  - docs/invariants.md

## Goal

修复定时任务包装字段 `repeat=daily/trading_day` 被单股投研门禁误识别为证券代码 `REPEAT`，导致批量定时任务在进入 runner 前失败的问题，并完整重启 runtime、验证健康状态。

## Scope

- 从最新真实会话和日志确认误识别输入与影响范围。
- 将交互式单股门禁限制为直接、唯一证券实体问题。
- 排除 scheduler/heartbeat 包装、多标的比较和通用财经缩写。
- 将实体搜索收紧为精确 symbol 匹配。
- 重启并完成同型消息真实回归和服务健康检查。

## Validation

- 投研分类器定向单测覆盖 scheduler、heartbeat、REPEAT、通用缩写、多标的、NEBIUS/NBIS 和精确 symbol 匹配。
- `cargo test -p hone-channels --lib`。
- `bash tests/regression/ci/test_finance_automation_contracts.sh`。
- `cargo build -p hone-cli`。
- 带 `repeat=daily` 和“财报分析”的真实隔离 Web 消息成功完成且没有 `market_data.preflight REPEAT`。
- 单实例 runtime、API、Postgres、S3、Web、Discord、Feishu 健康检查。

## Documentation Sync

- 补充 `D-2026-07-15-03` 的适用范围。
- 同步 `docs/invariants.md`、`docs/repo-map.md` 和交接记录。

## Risks / Open Questions

- 定时任务仍必须依靠 scheduler 自身的工具与输出契约完成事实核验；本修复只移除不属于它的交互式单股前置门禁。
- 新增财经缩写应优先补分类器回归，避免靠生产事故扩充排除表。
