# 文档计划与 handoff 清理

- title: 文档计划与 handoff 清理
- status: done
- created_at: 2026-03-17
- updated_at: 2026-04-09
- owner: shared
- related_files:
  - `docs/current-plan.md`
- related_docs:
  - `docs/adr/0001-repo-context-contract.md`
- related_prs:
  - N/A

## Summary

清理旧的计划与 handoff 布局，把任务索引重新收敛到活跃任务入口。

## What Changed

- 清空 `docs/current-plans/` 中已完成计划。
- 合并零碎 handoff。
- 把 `docs/current-plan.md` 恢复为活跃任务入口。

## Verification

- 历史记录未保留独立命令清单；此 handoff 作为结构性结果说明。

## Risks / Follow-ups

- 若未来再次让 `current-plan` 混入历史事项，索引会重新膨胀并失去入口价值。

## Next Entry Point

- `docs/adr/0001-repo-context-contract.md`
