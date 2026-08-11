# Company Research Corpus Dialogue Priority

- title: 历史公司研究语料在对话中的优先调用
- status: completed
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex
- related_files:
  - `skills/company-thesis-ratings/`
  - `skills/hari-invest/SKILL.md`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/turn_builder.rs`
  - `tests/regression/ci/test_company_research_dialogue_contract.sh`
- related_docs:
  - `docs/repo-map.md`
  - `docs/decisions.md#d-2026-08-11-11-compose-covered-company-dialogue-from-historical-thesis-cards-and-current-evidence`
  - `docs/handoffs/2026-08-11-company-research-corpus-dialogue-priority.md`

## Goal

让 HONE 在用户询问已覆盖公司时，优先使用此前授权上传的演讲与研究文件所形成的公司研究卡，回答基本面、商业模式、护城河、风险和证伪条件；价格、财报、新闻、产业状态和估值输入继续由当前工具证据链核验。

## Completed Scope

- 复用 51 份逐字稿、4 份工作簿所形成的 52 张压缩研究卡，没有把完整私有原文注入每轮上下文。
- 新增 52 家 ticker、英文名和中文常用名索引；每轮只投影当前明确命中的公司，最多八家。
- 命中公司卡时强制实际加载 `company-thesis-ratings` 与 `hari-invest`，前者提供公司特定历史基线，后者提供统一判断和输出纪律。
- 当前行情、财报、指引、订单、新闻、产业状态和估值输入仍由原工具链核验；冲突时当前一手证据优先。
- 短 ticker 采用显式大写门槛，避免 `app`、`be` 等普通单词误触发私有研究上下文。
- 保留逐字稿隐私边界，不允许向用户回显原文、内部文件名或 Skill 路径。

## Verification

- Skill Creator validation：`company-thesis-ratings` 与 `hari-invest` 均通过。
- `cargo test -p hone-channels --lib`：793 passed / 1 ignored。
- 新公司语料对话契约与 Hari 对话契约均通过。
- 覆盖中文别名、多短 ticker、普通单词误命中、未覆盖 NVDA、52 卡/52 索引一致性。
- Rust formatting、`git diff --check`、console binary build 与本地服务重启通过；`/api/skills` 显示两个 Skill 均已启用。

## Remaining Operational Step

普通 public actor 仍需要目标环境已有的安全函数调用模型或 `hone_cloud` 才能完成真实问答；本轮没有放宽 actor sandbox。首次接入模型后，应使用微软、闪迪、AppLovin/Bloom Energy 多标的和未覆盖 NVDA 四组问题做可见回答抽检。
