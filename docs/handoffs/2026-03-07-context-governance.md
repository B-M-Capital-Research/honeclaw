# Handoff: Context Governance Refresh

日期：2026-03-07
状态：已完成

## 本次目标

- 解决并行线程下 `docs/current-plan.md` 的冲突问题
- 控制 `docs/handoffs/*.md` 的碎片化增长
- 把新的治理规则写入仓库协作提示词和运行手册

## 已完成

- 将 `docs/current-plan.md` 改为活跃任务索引页
- 新增 `docs/current-plans/`，用于承载单任务计划页
- 新增 `docs/current-plans/README.md` 说明命名与模板
- 新增过一份单任务计划页示例，作为新治理结构的落地验证
- 更新 `AGENTS.md`，明确并行任务计划模型与 handoff 节制留档规则
- 更新 `docs/invariants.md` 与 `docs/runbooks/task-delivery.md`，统一执行约束
- 更新 `docs/decisions.md`，固化“索引页 + 单任务文件”与“handoff 节制留档”两条长期决策
- 更新 `docs/repo-map.md`，把新的动态文档结构写入阅读路径

## 影响范围

- `AGENTS.md`
- `docs/invariants.md`
- `docs/runbooks/task-delivery.md`
- `docs/decisions.md`
- `docs/repo-map.md`
- `docs/current-plan.md`
- `docs/current-plans/*`

## 验证

- 已完成跨文档一致性校对：
  - 动态计划结构
  - handoff 触发条件
  - 并行任务执行路径
- 已完成全文 diff 自检
- 本次仅涉及文档与协作规则，未运行 `cargo` / `bun` 测试

## 后续注意

- 后续新任务应先创建或复用 `docs/current-plans/*.md`，再登记到索引页
- 小型纯执行任务默认不再机械新增 handoff
- 同一主题若继续推进，应优先更新原 handoff，而不是重复创建新文件
