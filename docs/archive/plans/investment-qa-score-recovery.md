- title: HONE 投资问答五题提分与 Luna 能力审计
- status: done
- created_at: 2026-08-12
- updated_at: 2026-08-12
- owner: Codex
- related_files:
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-llm/src/provider.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-channels/src/turn_builder.rs`
  - `crates/hone-tools/src/skill_runtime.rs`
  - `tests/regression/manual/fixtures/investment_qa_benchmark_v1.json`
- related_docs:
  - `docs/handoffs/2026-08-12-investment-qa-benchmark.md`
  - `docs/handoffs/2026-08-12-investment-qa-score-recovery.md`

## Goal

使用完全相同的五题基准修复 HONE 投资问答链路，让当前事实、老王投资框架、公司研究基线和行动结论稳定进入最终答案；目标为单题不低于 75、平均不低于 80、零硬失败。若 GPT Luna 在完整工具与 Skill 链路修复后仍有明确能力上限，至少保留可复现的 70 分以上结果并区分模型、数据与编排瓶颈。

## Scope

- 复盘首次 149/500 的答案、工具审计和五题硬失败。
- 验证并修复 OpenAI-compatible 并行/分片工具调用，尤其是缺 `index/id` 的 arguments 归属。
- 确保投资题所需框架 Skill 与公司研究 Skill 有运行时成功证据，失败时有界重试或明确失败，不能只靠提示词自述。
- 让已取得的数据和来源真正进入最终答案，并保持一手来源、新鲜度和事实边界。
- 保持原五题、原评分卡和 GPT-5.6 Sol 参考答案不变，不通过降低题目难度或伪造数据提分。

## Validation

- 相关 Rust 单元与回归测试，覆盖多工具交错分片、Skill 门禁和证据保留。
- `cargo test -p hone-llm`、`cargo test -p hone-agent`、`cargo test -p hone-channels` 及受影响的 Web API 测试。
- 同一用户、全新对话、同一五题真实运行，保存答案与工具/Skill 审计。
- 按 fixture 固定六维评分卡人工复核；重大事实以公司 IR、SEC、监管/交易所等一手来源为准。
- `bun run typecheck:web`（若触及前端）、`bash scripts/ci/check_fmt_changed.sh`、`git diff --check`。

## Documentation Sync

- 更新 `docs/handoffs/2026-08-12-investment-qa-benchmark.md`，追加复测阶段、分数和 Luna 归因。
- 如改变长期投资问答门禁，更新 `docs/invariants.md` 与 `docs/decisions.md`。
- 完成后新增/更新 `docs/handoffs/2026-08-12-investment-qa-score-recovery.md`，归档本计划并更新 `docs/archive/index.md` 与 `docs/current-plan.md`。

## Risks / Open Questions

- Luna 可能能正确调用工具但仍不能稳定综合长证据；必须用工具成功率与最终答案质量分开判断。
- 本地缺少 FMP 时，部分结构化财务/估值仍需 SEC、IR 与可打开网页的降级链路；不得用搜索摘要冒充已核实事实。
- 两个所需 Skill 的实际运行时名称或安装状态可能与 fixture 不一致；需要先核实注册表，不得通过仅修改评分口径绕过。

## Completion

- 2026-08-12 使用原五题完成复测：432/500，平均 86.4，最低 80，5/5 通过。
- `hari-invest` 与 `company-thesis-ratings` 由服务端强制预载并进入运行时审计；公司研究卡在当前用户问题旁重复投影，减少长上下文遗忘。
- Nasdaq 精确代码行情、SEC Company Facts 和最新 8-K/6-K/10-Q 财报附件形成无 FMP 的一手降级链路。
- OpenAI-compatible 流式连接在收到 HTTP 响应前的瞬时传输失败仅重试一次；已开始的流不重放。
- 普通词汇重合不再注入无关研究资料，防止无关 ticker 污染单公司问题。
