- title: 持仓记忆补齐持有期限与策略信息
- status: done
- created_at: 2026-04-15
- updated_at: 2026-04-15
- owner: codex
- related_files:
  - memory/src/portfolio.rs
  - crates/hone-tools/src/portfolio_tool.rs
  - crates/hone-web-api/src/routes/portfolio.rs
  - crates/hone-web-api/src/types.rs
  - packages/app/src/lib/types.ts
  - packages/app/src/components/portfolio-detail.tsx
  - packages/app/src/context/portfolio.tsx
- related_docs:
  - AGENTS.md
  - docs/current-plan.md

## Goal

让持仓记忆除“公司 + 价格”之外，还能稳定记录：

- 负数价格或特殊成本场景
- 用户希望长持还是短持
- 用户声明的特殊策略或备注

## Scope

- 扩展持仓存储模型与向后兼容读写
- 扩展 portfolio tool / Web API 输入输出
- 按需更新前端类型与持仓详情展示/编辑
- 为新增行为补自动化测试

## Validation

- `cargo test -p hone-memory portfolio`
- `cargo test -p hone-tools portfolio_`
- `cargo test -p hone-web-api portfolio`
- `bun run typecheck:web`
- `bun run test:web`

## Documentation Sync

- 任务进行中记录在 `docs/current-plan.md`
- 本次未改变模块边界或长期规则，不额外更新 `docs/repo-map.md` / `docs/invariants.md`
- 已从活跃索引移出，并归档计划文件到 `docs/archive/plans/`

## Risks / Open Questions

- 已兼容已有 portfolio JSON；新增字段均为可选，不会破坏旧数据读取
- 当前允许负成本价，以兼容净收权利金、指派后负成本等场景；若后续要区分“负成本”和“异常数据”，需单独补业务校验规则
- 前端当前仍通过普通输入框提交策略与备注；若后续需要更强约束，可再补结构化策略枚举
