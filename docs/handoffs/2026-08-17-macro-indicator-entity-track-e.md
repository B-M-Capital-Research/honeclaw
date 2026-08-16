# Portfolio 路径 tentative 实体过滤（Track E）

- title: Portfolio 路径 tentative 实体过滤（Track E）
- status: `done`
- created_at: `2026-08-17`
- updated_at: `2026-08-17`
- owner: Codex
- related_files: `crates/hone-channels/src/investment_response_guard.rs`
- related_docs: `docs/current-plans/macro-indicator-entity-2026-08-17.md`
- related_prs: 无；本地 commit 待在可写 Git 元数据的宿主环境创建，不 push

## Summary

`normalized_portfolio_snapshot` 现在会在 `explicit_symbols` 形成前排除无法由真实
holdings / watchlist 快照确认的 tentative mention。这样 `PCE` 等弱语法候选不会收窄
Portfolio 行情集合；真实账本中的 tentative symbol 仍因账本事实而保留。

## What Changed

- 过滤位置早于 `requested_symbols` / `explicit_symbols`，因此候选全被过滤时会回落到既有
  `portfolio_symbols` 分支，并从真实快照派生行情实体。
- 匹配只复用 `portfolio_record_market_symbol` 与 `provider_symbols_equivalent`；没有新增黑名单、
  配置或兼容分支。
- 新增 4 条单元回归，分别覆盖混合假候选、持仓保留、关注项保留和
  `j_e447df29` 完整生产原文。
- 未改三处 `EntityMatch::Unresolved` 或 `plain_ticker_mentions`。

## Verification

- 4 条新增回归：通过。
- 5 条既有红线测试：逐项通过，未修改断言。
- 变异验证：移除过滤后测试 1、测试 4 均失败，退出码均为 `101`；恢复后通过。
- 完整命令
  `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  已运行，但执行沙箱拒绝本机端口与监听，结果在 `hone-channels` 停止于
  `passed=825 failed=172`；失败表现为 FMP stub `bind: Operation not permitted` 与 PostgreSQL
  连接失败，不能视为门禁通过。进入本轮前记录的宿主机基线为 `2591/0`。
- 当前沙箱对主 worktree 的 `.git/worktrees/honeclaw-e` 只有读权限，`git add` 无法创建
  `index.lock`；本交付尚未形成 commit，也没有 push。

## Risks / Follow-ups

- 需要在允许访问 `127.0.0.1:5433` 且允许本地 stub 监听的宿主环境重跑完整门禁；按原基线加
  4 条新测试，预期总数为 `2595/0`，但本次没有在该环境确认。
- `scheduled_secondary_subject_without_rebinding` 丢弃真实 ticker 是已知后续项，不属于 Track E。

## Next Entry Point

Claude 合并 Track E commit 后，在宿主机导入 `/Users/zhangxuanren/Workspace/honeclaw/.env`
中的 `HONE_POSTGRES_*`，重跑完整 workspace 门禁；随后继续按父计划独立评审 Track B / C。
