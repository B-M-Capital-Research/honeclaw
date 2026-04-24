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
  - docs/archive/plans/price-event-lane.md
  - docs/decisions.md

## Goal

修复价格事件日级去重导致的漏推：同一只股票先以低幅度进入摘要后，后续同日跨过高档必须能作为新事件即时推送。AAOI `+5.87% -> +6%/+8%/+10%` 是本轮主要回归场景。

## Result

- Price poller 新增三类 id：
  - `price_low:{symbol}:{date}`
  - `price_band:{symbol}:{date}:{up|down}:{band_bps}`
  - `price_close:{symbol}:{date}`
- 新增配置默认值：
  - `price_realert_step_pct: 2.0`
  - `price_intraday_min_gap_minutes: 30`
  - `price_symbol_direction_daily_cap: 2`
  - `price_close_direct_enabled: false`
- Router 对价格 band 绕开通用同 ticker cooldown，改用价格专属 gap/cap；正向和负向独立计数。
- Digest buffer 对价格事件按 actor/symbol/date/window 做 latest 更新，避免摘要堆叠同一只股票旧价格。
- 说明文档和 event-engine skill 里的本地命令包装前缀已删除，验证命令统一写原生命令。

## Verification

- `cargo test -p hone-event-engine price --lib`：23 passed, 1 ignored
- `cargo test -p hone-event-engine router --lib`：41 passed
- `cargo test -p hone-event-engine digest --lib`：38 passed
- `cargo test -p hone-core --lib`：59 passed
- `cargo fmt --all -- --check`：passed
- `cargo test -p hone-event-engine --lib`：229 passed, 13 ignored
- `cargo check --workspace --all-targets --exclude hone-desktop`：passed；`hone-cli` 有与本任务无关的 unused warnings
- `bash tests/regression/run_ci.sh`：passed
- `cargo test --workspace --all-targets --exclude hone-desktop`：passed；首次和 regression 并行跑时 `hone-channels` 单测 `run_zero_daily_conversation_limit_bypasses_quota` 出现 actor sandbox `os error 22` 抖动，单测重跑和非并行 workspace 重跑均通过

## Risks

- 旧 SQLite 里的 `price:{symbol}:{date}` 历史事件不迁移；新行为只影响后续事件。
- `price_close_direct_enabled=false` 会让收盘高波动默认进摘要；如果未来想凌晨即时推，需要显式打开。
- 本轮保留了目标价提醒的 payload/状态前置，但没有实现用户自定义目标价规则。
