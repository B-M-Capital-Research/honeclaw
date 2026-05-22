- title: Heartbeat max-iterations budget bump
- status: done
- created_at: 2026-05-23 03:06 CST
- updated_at: 2026-05-23 03:06 CST
- owner: Codex
- related_files:
  - crates/hone-channels/src/scheduler.rs
  - docs/bugs/scheduler_heartbeat_iteration_exhaustion_skips_alert.md
  - docs/bugs/README.md
- related_docs:
  - docs/archive/index.md
- related_prs:
  - N/A

## Summary

活跃 heartbeat `max_iterations_exceeded:10` 缺陷已按代码侧重新止血：heartbeat auxiliary function-calling 预算从 `10` 提到 `18`，并在 heartbeat prompt 增加“必须以最少工具调用收口”的约束，减少板块/多标的 heartbeat 为确认 noop 而反复穷举导致的预算触顶。

## What Changed

- `crates/hone-channels/src/scheduler.rs`
  - `HEARTBEAT_MAX_ITERATIONS` 从 `10` 提升到 `18`。
  - heartbeat prompt 新增工具预算约束，要求优先复用本轮已拿到的信息，并以最少工具调用收口。
- `docs/bugs/scheduler_heartbeat_iteration_exhaustion_skips_alert.md`
  - 状态更新为 `Fixed`，记录本轮修复、验证和重新打开条件。
- `docs/bugs/README.md`
  - 活跃缺陷计数回写为 `0`，并把该 bug 移入已修复列表。

## Verification

- `cargo test -p hone-channels heartbeat_prompt_requires_noop_json_for_contract_conflicts --lib -- --nocapture`
- `cargo test -p hone-channels heartbeat_runner_uses_capped_completion_budget --lib -- --nocapture`
- `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## Risks / Follow-ups

- 这是预算和 prompt 收口的公共止血，不是 provider 特判；如果真实窗口继续出现 `max_iterations_exceeded:18`，说明需要进一步缩减 heartbeat 工具面或补更细的阶段级诊断，而不应继续线性抬预算。
- 本轮没有重启 live 服务；运行态是否收敛需要后续巡检窗口继续观察。

## Next Entry Point

- `docs/bugs/scheduler_heartbeat_iteration_exhaustion_skips_alert.md`
