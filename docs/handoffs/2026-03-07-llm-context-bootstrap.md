# Handoff: LLM Context Bootstrap

日期：2026-03-07
状态：已完成

## 本次目标

- 把“LLM-first 开发的上下文外化实践”落到仓库里，建立最小可维护文档集
-

## 已完成

- 扩展 `AGENTS.md`，加入上下文资产分类、真相源优先级、完成定义
- 新增 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`
- 新增 `docs/adr/0001-repo-context-contract.md`
- 新增 `docs/runbooks/task-delivery.md`
- 新增并初始化 `docs/current-plan.md`
- 在 `docs/technical-spec.md` 顶部显式标记其为历史文档

## 为什么这样做

- 当前仓库已有多语言、多入口、多渠道结构，靠会话历史很难稳定交接
- 仓库里存在过时文档，必须明确真相源顺序，避免新会话误读

## 验证

- 本次仅涉及文档和协作规则，无业务代码改动
- 未运行 `cargo` / `bun` 测试，因为没有影响编译或运行行为

## 后续建议

- 下一次功能开发前，把 `docs/current-plan.md` 切换到该任务的真实状态
- 若继续长期维护技术架构文档，优先重写 `docs/technical-spec.md`，不要继续混用旧版 Python 结构说明
