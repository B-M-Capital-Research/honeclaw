- title: Hari Invest 默认投研 Skill 本地安装交接
- status: done
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files: `skills/hari-invest/`; `crates/hone-channels/src/prompt.rs`; `skills/README.md`
- related_docs: `docs/archive/plans/hari-invest-hone-default.md`; `docs/decisions.md#d-2026-08-10-01-load-hari-invest-before-every-investment-answer`; `/Users/wangxx/Documents/Playground/老王投资体系蒸馏/knowledge/当前状态.md`
- related_prs: none; local uncommitted change set

## Summary

蒸馏工作区生成的对外 Skill `hari-invest` 已安装到本地 HONE。公司、证券、ETF、行业、宏观、市场、基本面、护城河、估值、组合、仓位、投资复盘和评分类问答在形成最终回答前必须实际加载该 Skill；明显非投资问题不加载。内部 `laowang-investment-distiller` 未安装到 HONE，也不会暴露给普通用户。

## What Changed

- 新增 `skills/hari-invest/`，包含主 Skill、六条逻辑的默认索引、完整 references 与本地安装边界。
- 统一 prompt 新增 `DEFAULT_HARI_INVEST_POLICY`：原生 Codex 使用原生 Skill 加载机制，函数调用 runner 使用 `skill_tool(skill_name="hari-invest")`。
- 当前事实继续走 HONE 现有行情、财报、公告、网页与持仓工具；Skill 只提供判断框架、证伪纪律和输出结构。
- 命中演讲覆盖公司时允许同时加载 `company-thesis-ratings`，公司卡作为标的研究基线，Hari Invest 作为统一判断框架。
- 蒸馏工作区已回填 2026-08-10 本地安装授权；公开发布、公开传播和生产部署仍未授权。

## Verification

- `python3 .../quick_validate.py skills/hari-invest`：通过。
- `cargo fmt --all -- --check`：通过。
- `cargo test -p hone-channels prompt::tests -- --nocapture`：14/14 通过。
- `cargo test -p hone-tools discover_skills::tests -- --nocapture`：2/2 通过。
- `cargo test -p hone-tools skill_tool::tests -- --nocapture`：13/13 通过；matplotlib 不可用的既有可选图表 smoke 按测试设计跳过。
- 真实 `hone-mcp`：搜索“估值”同时发现 `company-thesis-ratings` 与 `hari-invest`；直接加载成功，主 prompt 含 `LOG-V0001..LOG-V0006`。
- 本地 runtime：8077 健康，`/api/skills` 返回 `hari-invest enabled=true`；Web 运行，Feishu、Discord、Telegram、iMessage 均 disabled；3000/3001 返回 HTTP 200，活跃对话为 0。

## Risks / Follow-ups

- 未发送真实投资问题，避免在没有用户指定问题和当前数据配置的情况下产生无意义的模型调用；统一 prompt 单元测试、真实 MCP 加载和运行时 Skill API 已覆盖强制加载链路的三个边界。
- 本机配置未提供事件引擎所需的独立 LLM API key，后端以既有降级模式启动；这与 Skill 加载无关。
- 若未来公开发布或生产部署，必须重新获得明确授权并复核推广字段、隐私和证据截止日期。

## Next Entry Point

下一次在本地 HONE 提问任一投资类问题即可使用新框架。更新蒸馏逻辑时，以 `/Users/wangxx/Documents/Playground/老王投资体系蒸馏/knowledge/07_已生成Skills/hari-invest/` 为维护源，再同步经过确认的最小发布包到 `skills/hari-invest/`。
