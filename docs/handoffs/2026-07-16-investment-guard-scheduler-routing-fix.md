# 单股投研门禁误伤定时任务修复交接

- title: 单股投研门禁误伤定时任务修复交接
- status: done
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-channels/src/investment_response_guard.rs
- related_docs:
  - docs/archive/plans/investment-guard-scheduler-routing-fix.md
  - docs/decisions.md
  - docs/invariants.md
- related_prs: none; this local change set

## Summary

最新两个 09:30 会话以及多个历史定时任务都返回“当前无法稳定核验证券实体 `REPEAT`”。真实输入以 `[定时任务触发]` 开头，并包含权威配置 `repeat=trading_day/daily`。交互式单股分类器扫描了整个系统包装文本，把赋值键 `repeat` 当作候选 ticker；正文中的“分析、财报、估值”等词又使其进入深度单股门禁，最终在 runner 启动前失败。

此前 `c776b808` 只排除了部分赋值上下文，仍未建立“定时任务不属于直接单股交互”的路由边界，因此生产上继续出现 REPEAT，并伴随 CNN、PPI、GPU、HBM、EBITDA 等报告缩写误识别。

## What Changed

- `[定时任务触发]` / `[心跳任务触发]` 及全角等价包装不进入交互式单股门禁；scheduler 继续使用自己的工具与输出契约。
- 常见财经、宏观、技术、调度缩写不再作为 ticker 候选。
- 中文文本中的小写拉丁候选仅在唯一时才可作为 ticker；多证券比较不再套单股九段门禁。
- `NEBIUS` 公司别名仍稳定映射为 `NBIS`。
- 证券实体搜索只接受用户 hint 的精确 symbol；删除“找不到精确项就使用搜索第一条”的危险回退。

## Verification

- 分类器定向测试：15 passed。
- 完整 `hone-channels` library 测试：491 passed；finance CI-safe contract：12/12 passed。
- `hone-cli` 构建通过。
- 真实隔离消息 `live-repeat-guard-probe-20260716` 同时包含 `[定时任务触发]`、`repeat=daily`、`财报分析`，返回“调度路由健康检查通过”，`run_finished success=true`；该会话日志没有 `market_data.preflight`，证明没有再进入 REPEAT 证券门禁。
- 所有旧 runtime/channel 进程显式停止并确认 8077 端口释放后，仅拉起一个新 runtime。`/api/meta` 返回 0.14.1，Postgres 与 S3 均为 connected；Web console、Discord、Feishu 子进程重新就绪。

## Risks / Follow-ups

- scheduler 自身仍需对具体证券和宏观数据调用工具；不能因为排除了交互式单股门禁就降低其既有事实核验标准。
- 分类器采用保守策略：多标的比较交给正常 runner 处理，不强制套单股九段结构。这避免只分析第一个 ticker，但多标的专属结构可作为未来独立契约实现。
- 重启期间曾因旧、新 runtime 的 `--build` 短暂重叠出现一次构建期 `No space left on device`；最终显式停止全部旧进程后，单独构建成功并以非并发构建方式启动。交付时根卷可用约 23 GiB、runtime 仅一实例；`target/` 约 52 GiB，后续若继续增长应清理可再生成的 incremental cache。

## Next Entry Point

从 `classify_investment_response_contract`、`is_scheduled_task_envelope`、`extract_security_hint` 和 `resolve_verified_symbol` 四处开始。线上排查同时查看输入包装、`market_data.preflight` 与 `agent.run` 的先后关系。
