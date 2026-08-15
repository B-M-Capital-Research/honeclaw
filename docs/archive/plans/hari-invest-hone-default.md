- title: Hari Invest 默认投研 Skill 接入
- status: archived
- created_at: 2026-08-10
- updated_at: 2026-08-10
- owner: Codex
- related_files: `skills/hari-invest/`; `crates/hone-channels/src/prompt.rs`; `skills/README.md`
- related_docs: `/Users/wangxx/Documents/Playground/老王投资体系蒸馏/knowledge/当前状态.md`; `docs/handoffs/2026-08-10-hari-invest-hone-default.md`; `docs/decisions.md#d-2026-08-10-01-load-hari-invest-before-every-investment-answer`

## Goal

把蒸馏工作区已经生成并通过静态评测的对外 Skill `hari-invest` 安装到本地 HONE，使投资类问答在形成最终判断前实际加载并遵循该框架，同时保留最新事实核验、隐私与非荐股边界。

## Scope

- 校验生成包的逻辑、来源、隐私和旧授权状态。
- 安装 HONE 原生 Skill 及其必要 references，不安装内部 `laowang-investment-distiller`。
- 在统一对话 prompt 中要求投资类问题加载 `hari-invest`，并与现有行情、财报、网页证据工具及公司评级 Skill 组合使用。
- 更新蒸馏工作区状态，记录本地安装授权，不扩大到公开发布或生产部署。

## Validation

- Skill validator：通过。
- `hone-channels` prompt tests：14/14 通过。
- `hone-tools` discover tests：2/2 通过；skill tool tests：13/13 通过。
- 真实 MCP `discover_skills` 能通过“估值”发现 `hari-invest`；`skill_tool` 成功加载并包含 `LOG-V0001` 至 `LOG-V0006`。
- 本地 runtime 重启成功；`/api/skills` 显示 `hari-invest enabled=true`，Web 为唯一运行渠道，3000/3001 前端可访问。

## Documentation Sync

- 已更新 `skills/README.md`、`docs/decisions.md`、蒸馏工作区 `knowledge/当前状态.md`、handoff 与 archive index。
- 本计划已退出活跃索引并归档。

## Risks / Open Questions

- 本次授权只覆盖本地 HONE 安装；不得把内部逐字稿或蒸馏器暴露给普通用户，也不代表允许公开仓库发布或生产部署。
- `hari-invest` 是判断框架，不是实时数据源；任何当前结论仍必须使用本轮行情、财报、公告或网页证据核验。
