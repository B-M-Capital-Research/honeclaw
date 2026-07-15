# 投研回复格式契约执行修复交接

- title: 投研回复格式契约执行修复交接
- status: done
- created_at: 2026-07-15
- updated_at: 2026-07-15
- owner: Codex
- related_files:
  - soul.md
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/execution.rs
  - agents/function_calling/src/lib.rs
  - skills/stock_research/SKILL.md
- related_docs:
  - docs/archive/plans/response-contract-enforcement.md
  - docs/decisions.md
  - docs/invariants.md
- related_prs: none; implementation commit `c29de55c`

## Summary

最新真实用户会话 `Actor_web__direct__web-user-e05f5e5f74a3` 的问题是“我想了解Q3的时候nbis能不能起飞”。prompt audit 证明完整单股分析与强制输出顺序已经进入模型，但 strict function-calling runner 在一次迭代内没有调用工具，最终只给出简化 Bull/Bear 讨论并追问成本。断点因此不是 prompt 丢失，而是模型把软格式规则当成了可选项。

本轮把证券实体、行情与深度单股数据查询前移到代码预检，并在最终持久化前执行九段回复契约。草稿不合格会撤回并重写一次；重写后仍不合格则拒发。完整大 prompt 也从 pre-`71a4498e` 基线恢复为 canonical `soul.md`，runtime 副本每次物化都与其同步。

## What Changed

- 恢复 actor-bound function-calling 安全执行器；普通用户配置为 CLI/ACP 时自动走安全 runner，缺少 LLM 时 fail closed。
- 新增证券意图分类、同标的实体与正价格校验；深度单股强制查询 profile、financials、news，前瞻问题再查询 120 天财报日历。
- 深度单股答案强制九个编号章节、数据时间、事实/假设分层、至少两种估值方法及可触发的动作建议。
- 最终答案在发送和持久化前校验；支持 `StreamReset` 后一次定向重写，仍不完整时拒发。
- 简单“现在多少钱”仍保持简短，但同样不能绕过当轮实体和价格核验。
- `stock_research` skill、长期决策、不变量、repo map 与 CI-safe finance contract 同步更新。

## Verification

- `cargo test -p hone-core`：117 passed。
- `cargo test -p hone-agent`：7 passed。
- `cargo test -p hone-channels --lib`：482 passed。
- `bash tests/regression/ci/test_finance_automation_contracts.sh`：12/12 passed。
- `cargo build -p hone-cli`：passed。
- 真实隔离回归 `live-response-contract-probe-20260715b` 使用原句“我想了解Q3的时候NBIS能不能起飞”：日志先出现 `market_data.preflight symbol=NBIS deep_single_stock=true outlook=true`，随后 strict function-calling runner 完成 5,419 字回复，包含 1 到 9 全部章节、北京时间、已核验/假设标识、P/S、EV/EBITDA 与情景分析，最终 `run_finished success=true`。
- runtime 已重启为 screen `hone-runtime-market-data-guard`；`/api/meta` 返回 0.14.1，Postgres 与 S3 均健康。

## Risks / Follow-ups

- 当前代码能证明本轮证据已查询、同标的价格/财务非空并验证答案结构，但无法静态证明自然语言中的每个非数值描述都被证据逐字蕴含；后续抽检应继续关注模型是否把“公司获投资”夸张成个人投资等表述。
- 行情、财务和新闻仍依赖配置的数据提供方；提供方故障时会拒绝数值结论，这是预期的 fail-closed 行为。
- 分类器刻意聚焦明确 ticker/证券别名与深度意图，避免把所有简短行情问题膨胀成九段长答；新增语言或模糊公司别名时应补 classifier 回归样本。

## Next Entry Point

先看 `crates/hone-channels/src/investment_response_guard.rs` 的分类、证据预检与完整性校验，再看 `crates/hone-channels/src/agent_session/core.rs` 的撤回/重写边界。线上抽检以 `market_data.preflight`、`agent.run.retry investment_contract` 和最终 `run_finished` 三类日志为入口。
