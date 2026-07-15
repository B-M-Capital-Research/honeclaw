# 投研回复格式契约执行修复

- title: 投研回复格式契约执行修复
- status: archived
- created_at: 2026-07-15
- updated_at: 2026-07-15
- owner: Codex
- related_files:
  - soul.md
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/agent_session/
  - crates/hone-channels/src/runners/
- related_docs:
  - docs/handoffs/2026-07-15-response-contract-enforcement.md
  - docs/decisions.md
  - docs/invariants.md

## Goal

基于最新 NBIS/MBIS 真实会话定位“完整大 prompt 已恢复但最终回复仍草率”的实际断点，确保需要深度投研的用户问题稳定执行规定的分析顺序、数据核验、双边论证、估值、风险、证伪和动作结构。

## Result

prompt audit 证明断点是模型忽略软指令而不是 prompt 未注入。现已恢复完整 canonical prompt 与 actor-bound 安全 runner，并新增代码级本轮证券数据预检、九段最终回复校验、一次撤回重写和二次失败拒发。简单行情保持简短，深度单股与季度前瞻走完整契约。

## Verification

- Core、agent、channels 受影响测试通过。
- finance CI-safe contract 12/12 通过，`hone-cli` 构建通过。
- 原句 NBIS 隔离真实回归先记录 `market_data.preflight`，再输出 1-9 九段、数据时间、双估值与动作条件，最终 `run_finished success=true`。
- runtime 0.14.1 已重启，Postgres 与 S3 健康。

## Documentation Sync

- 长期决策：`D-2026-07-15-01`、`D-2026-07-15-02`、`D-2026-07-15-03`。
- 同步 `docs/invariants.md`、`docs/repo-map.md`、`skills/stock_research/SKILL.md`。
- 完整证据与风险见 `docs/handoffs/2026-07-15-response-contract-enforcement.md`。

## Risks

- 自然语言中每个非数值事实的证据蕴含仍需抽检；代码门禁当前保证数据查询、同标的关键字段、结构和显式假设口径。
- 数据提供方不可用时按设计拒绝数值性结论。
