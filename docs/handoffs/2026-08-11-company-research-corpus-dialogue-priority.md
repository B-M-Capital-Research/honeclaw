# 历史公司研究语料对话优先调用交接

- title: 历史公司研究语料对话优先调用
- status: done locally; not deployed
- created_at: 2026-08-11
- updated_at: 2026-08-11
- owner: Codex

## 完成结果

HONE 现有 51 份授权逐字稿和 4 份研究工作簿此前已经形成 52 张公司研究卡。本轮没有重复复制原文，而是增加确定性的中英文别名索引和按本轮问题投影机制。用户点名覆盖公司时，系统只加入对应压缩卡，并要求 Agent 在最终回答前同时实际加载 `company-thesis-ratings` 和 `hari-invest`。

公司卡优先用于商业模式、基本面结构、护城河、产业链位置、风险和证伪条件；Hari 负责机会区/持有区/风险区/数据不足的当前决策。股价、最新财报、指引、订单、新闻、行业状态和估值输入继续使用原有工具链，不从旧演讲抄数字。如果新的一手证据与旧卡冲突，新证据优先，回答需要说明原逻辑是加强、削弱还是失效。

## 关键文件

- `skills/company-thesis-ratings/references/company-cards.json`：52 张压缩研究卡。
- `skills/company-thesis-ratings/references/company-index.json`：ticker、英文名和中文常用名索引。
- `crates/hone-channels/src/prompt.rs`：匹配、有限投影、事实边界与双 Skill 强制规则。
- `crates/hone-channels/src/turn_builder.rs`：把本轮命中的研究基线放入私有系统上下文。
- `tests/regression/ci/test_company_research_dialogue_contract.sh`：长期对话契约门禁。

## 验证

- 两个 Skill 通过 Skill Creator validator。
- `hone-channels`：793 passed / 1 ignored。
- 公司语料和 Hari 两个 CI-safe contract 通过。
- 中文“微软”、多标的 `APP / BE`、普通英文 `app / be`、未覆盖 `NVDA` 和 52/52 索引一致性均有自动化测试。
- Console binary 构建完成；本地管理端 8077、用户 API 8088 与前端 3001 正常，两个 Skill 均为 enabled。

## 风险与下一步

- 当前本地 public actor 没有配置安全函数调用模型，真实公开问答仍会失败关闭；不要为此把公共用户切到 host-capable Codex ACP。
- 接入安全模型后，用 MSFT、SNDK、APP+BE、NVDA 做四组真实对话抽检，确认前两组引用历史基本面/护城河逻辑，实时数据来自新证据，NVDA 不冒充有历史研究卡。
- 新增演讲材料时，先更新压缩卡与别名索引，再保持两份 JSON 的 symbol 集合完全一致。
