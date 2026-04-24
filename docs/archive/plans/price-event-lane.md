# Price Event Lane 增量改造

- title: Price Event Lane 增量改造
- status: done
- created_at: 2026-04-24
- updated_at: 2026-04-24
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/pollers/price.rs
  - crates/hone-event-engine/src/router.rs
  - crates/hone-event-engine/src/store.rs
  - crates/hone-event-engine/src/digest.rs
  - crates/hone-core/src/config/event_engine.rs
  - config.example.yaml
- related_docs:
  - docs/current-plan.md
  - docs/decisions.md

## Goal

把价格事件从 `price:{symbol}:{date}` 的日级去重改为可增量升级的价格 lane。核心验收是 AAOI 这类先以低幅度进入摘要、随后同日跨过 `+6%/+8%` 的走势必须能即时推送，同时保留频率控制，避免单股波动把队列打爆。

## Scope

- 价格 poller 生成分层事件 id：
  - 低幅摘要：`price_low:{symbol}:{date}`
  - 盘中跨档：`price_band:{symbol}:{date}:{up|down}:{band_bps}`
  - 收盘摘要：`price_close:{symbol}:{date}`
- 新增价格专属阈值/频率配置：
  - `price_realert_step_pct`
  - `price_intraday_min_gap_minutes`
  - `price_symbol_direction_daily_cap`
  - `price_close_direct_enabled`
- 摘要 buffer 对价格事件按 actor/symbol/date/window 采用 latest 更新，避免摘要里堆叠同一只股票的旧价格。
- 盘中正向和负向独立计数；涨后回落不推，除非反向跨过负向高档。
- 本轮不实现用户自定义目标价规则，但保留 payload/状态字段，避免后续重复拆模型。
- 删除本次涉及说明中的本地命令包装前缀描述，验证命令统一写原生命令。

## Validation

- done: `cargo test -p hone-event-engine price --lib`：23 passed, 1 ignored
- done: `cargo test -p hone-event-engine router --lib`：41 passed
- done: `cargo test -p hone-event-engine digest --lib`：38 passed
- done: `cargo test -p hone-core --lib`：59 passed
- done: `cargo fmt --all -- --check`
- done: `cargo test -p hone-event-engine --lib`：229 passed, 13 ignored
- done: `cargo check --workspace --all-targets --exclude hone-desktop`：passed，`hone-cli` 存在与本任务无关的未使用 import/function warnings
- done: `bash tests/regression/run_ci.sh`：passed
- done: `cargo test --workspace --all-targets --exclude hone-desktop`：passed；首次并行运行时 `hone-channels` 单测 `run_zero_daily_conversation_limit_bypasses_quota` 因 actor sandbox `os error 22` 抖动失败，单测重跑和非并行 workspace 重跑均通过

## Documentation Sync

- done: 开始时更新 `docs/current-plan.md` 与本计划页。
- done: 更新 `docs/decisions.md`，记录价格事件从日级事件改为 band/state lane。
- done: 完成后写入 `docs/handoffs/2026-04-24-price-event-lane.md`，计划页归档到 `docs/archive/plans/price-event-lane.md`，并更新 `docs/archive/index.md`。

## Risks / Open Questions

- 历史 `price:{symbol}:{date}` 事件仍会留在 SQLite；新 id 只影响后续事件，不迁移旧数据。
- 价格事件绕开通用同 ticker cooldown 后，必须依赖价格专属 gap/cap 保持频率合理。
- FMP quote endpoint 在不同解析节点上偶发超时；回放验证应缓存输入 fixture，避免测试依赖实时网络。
