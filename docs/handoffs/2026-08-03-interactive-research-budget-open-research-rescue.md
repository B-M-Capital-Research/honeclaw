- title: Interactive research budget starvation and open-research rescue
- status: done (not yet deployed)
- created_at: 2026-08-03
- updated_at: 2026-08-03
- owner: Claude
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-channels/src/investment_response_guard.rs`, `soul.md`, `tests/regression/ci/test_finance_automation_contracts.sh`, `docs/invariants.md`, `docs/decisions.md`
- related_docs: `docs/decisions.md#d-2026-08-03-02-guarantee-one-substantive-research-round-before-any-bounded-finance-final`, `docs/handoffs/2026-08-03-sndk-listing-evidence-and-gce-rollout.md`, `docs/current-plans/ticker-resolution-architecture.md`
- related_prs: none

## Summary

用户报告网页端严重劣化：`美股科技股和半导体股票方面的CTA是多少` 回答“无法提供具体的数字”，`sndk财报前瞻` 回答“SNDK 未上市”，整体表现“跟没联网一样”。

第一例是真实回归，根因在 `820a7240`（2026-07-22，TTFT 优化）引入的有界研究预算：`finance_tool_rounds` 从第一次 DataFetch 起就开始计数，实体解析与实质研究共用同一个三轮预算。当本轮的“代码”其实不是证券时（`CTA` 是管理期货/趋势策略术语），扫描器把它当成显式代码播种，实体优先契约要求 exact-symbol 取证，provider 无覆盖，三轮预算在任何一次 `web_search` 之前就被耗尽，随后 `tools=[]` 的强制终稿只能给出“无法提供具体数字”。工具全部正常，回答却等同于离线模型。

第二例由 `39470783 fix(agent): require current evidence for listing denials` 独立修复：真实根因是首批工具里一个格式错误的 `earnings_outlook` 让整批取消、工具关闭，模型随后发布陈旧退市记忆。本轮不重复实现该守卫，只保留“零实质证据不得收口”的互补不变量。

第三个问题在排查中独立发现：`67b9a915`（当日 12:21）压缩 `soul.md` 时删除了 `每个公司或证券问题先调用 DataFetch \`search\``，而 acceptance 检查 `20.current-data-capability` 正在守护这句话；该提交在检查为红的情况下进入了 `main`。

## What Changed

- 实体轮与研究轮分账：identity-only 轮走独立的 `MAX_AGENT_OWNED_FINANCE_IDENTITY_ROUNDS=2`，三轮研究预算只覆盖实质取证。
- 预留开放检索预算：`AGENT_OWNED_FINANCE_RESERVED_OPEN_RESEARCH_CALLS=2` 个全局槽位仅 `web_search` 可用；预留位或单工具位被拒绝不再等同于预算耗尽。
- 硬不变量：预算到点但本轮除实体搜索外没有执行过任何业务调用时，不进入 `tools=[]` 终稿，而是给同一个 Agent 一轮开放检索（工具表移除 `data_fetch`，附专用指令）。被拦截批次的安全收口保持原路径。
- 种子置信度：仅靠 clause-subject 语法支撑的裸大写 token 输出为 tentative 候选而非显式代码；全部种子为 tentative 时，发现契约追加一段低置信说明，要求 Agent 判断真实主题、必要时放弃候选。
- `soul.md` 恢复被删除的强制 search 规则，新增“实体校验服务于用户原问题，不能取代它”的收口规则，并压缩与 Agent 指令重复的 `entity_route` 协议文本以留出预算余量（11,979 / 12,000）。
- 验收脚本 `29` 同步更新为新的测试名，并新增对新常量、新守卫函数与新回归测试的引用。

## Verification

- Passed: `hone-agent` 142 项、`hone-channels` 727 项、仓库级 `cargo test --workspace --lib`。
- Passed: 全部 44 项 finance acceptance contracts（修复前为 43，`20.current-data-capability` 为红）。
- Passed: `tests/regression/run_ci.sh`（exit 0）。
- Passed: 改动文件 `rustfmt --check`、`git diff --check`。
- 新回归 `unresolvable_identity_never_closes_a_finance_turn_without_open_research` 在把新常量置零后确实失败，证明其覆盖的是真实缺陷而非同义重述。

## Risks / Follow-ups

- 延迟：健康的单标的问题多出一轮工具轮（实体轮不再占研究预算）。总轮数仍硬上界为 2+3，调用上限未变。若首屏延迟成为问题，应调低研究轮而不是恢复实体轮与研究轮共账。
- `web_search` 未注册时不触发救援轮；此时仍走原 `tools=[]` 终稿。
- 上市矛盾检测按句匹配 + 历史语气豁免，可能漏掉跨句表述；纠正轮上限为 1，代价是一次额外生成，不会改变答案语义。
- `soul.md` 的 12,000 字符预算余量极小（当前 11,979）。本次事故的直接诱因就是在这个预算下压缩 prompt 时删掉了被守护的规则。建议后续把 `soul.md` 与 `prompt.rs` / Agent 指令三处重复的实体协议做一次真正的去重，而不是继续在同一预算内互相挤压。
- 本轮改动尚未部署。上一次 SNDK 修复的 GCE 回滚点与流程见 `docs/handoffs/2026-08-03-sndk-listing-evidence-and-gce-rollout.md`。

## Next Entry Point

部署后用两个原始问句做 canary：`美股科技股和半导体股票方面的CTA是多少` 应出现至少一次 `web_search` 并围绕 CTA 仓位/资金流作答，不得再输出“无法提供具体数字”；`sndk财报前瞻` 应保持当前上市结论。若要继续收敛，从 `soul.md` / `DEFAULT_FINANCE_DOMAIN_POLICY` / `POST_IDENTITY_EVIDENCE_SYSTEM_INSTRUCTION` 三处重复的实体协议去重开始。
