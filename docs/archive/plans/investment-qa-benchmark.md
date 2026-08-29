- title: HONE 投资问答五题基准与 GPT-5.6 Sol 对照
- status: archived
- created_at: 2026-08-12
- updated_at: 2026-08-12
- owner: Codex
- related_files:
  - `tests/regression/manual/fixtures/investment_qa_benchmark_v1.json`
  - `tests/regression/manual/test_investment_qa_reference.sh`
- related_docs:
  - `docs/handoffs/2026-08-12-investment-qa-benchmark.md`

## Goal

建立一套可重复使用的五题投资问答基准，以相同问题测试 HONE 的数据准确性、结果导向、老王投资框架、公司研究 Skill 使用证据和总体合理性，并与隔离运行的 GPT-5.6 Sol 参考答案比较。

## Scope

- 覆盖最新财报、三方法估值、同产业二选一、护城河变化和组合行动五类高价值场景。
- 固化六维 100 分评分卡、硬失败条件和以后回归使用的通过门槛。
- 在本地 HONE 中逐题执行，并保存运行时工具审计证据。
- 通过隔离的 Codex CLI 生成 GPT-5.6 Sol 参考答案；参考模型不得读取 HONE 私有 Skill。
- 不在本任务中修复发现的运行时缺陷，也不执行发布。

## Validation

- `jq empty tests/regression/manual/fixtures/investment_qa_benchmark_v1.json`
- `bash -n tests/regression/manual/test_investment_qa_reference.sh`
- 五个 GPT-5.6 Sol 参考答案均非空。
- 五个 HONE 问题均在本地真实会话中完成，并核对审计日志中的 Skill / 数据工具执行证据。
- `git diff --check`

## Documentation Sync

- 结论与评分写入 `docs/handoffs/2026-08-12-investment-qa-benchmark.md`。
- 完成后直接归档本计划，并登记到 `docs/archive/index.md`；无需更新 `docs/current-plan.md`。
- 本任务没有更改架构、生产行为或长期决策，因此无需更新 `docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。

## Risks / Open Questions

- GPT-5.6 Sol 是独立对照，不是真相源；其数字同样必须接受一手来源核验。
- 参考运行依赖外部账号并产生模型费用，所以脚本默认跳过，只有显式设置开关才运行。
- 当前 HONE 的工具流参数归属异常会阻断数据与私有 Skill 调用；修复后必须使用同一套题重跑，不可沿用本次分数。
