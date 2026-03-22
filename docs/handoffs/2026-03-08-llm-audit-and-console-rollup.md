# 2026-03-08 LLM Audit 与 Console 前端收口

## 归并范围

- LLM 审计底座
- LLM 审计页面与 JSON viewer
- 北京时间时间戳统一
- 前端构建修复与侧边栏状态折叠

## 已完成

- 新增 `hone-core` 审计记录 / sink 抽象，`memory` 落地 SQLite 审计存储，默认滚动保留最近 30 天。
- `hone-channels` 与 `agents/*` 接入 function calling、Gemini、Codex、session compression 的审计记录。
- `hone-console-page` 提供只读 `llm-audit` 查询接口，`packages/app` 新增审计列表页与详情页。
- 审计详情页改为递归 JSON viewer，支持逐层展开/折叠，默认展开第一层。
- LLM 审计、session 与总结消息的记录时间改为北京时间写入。
- 修复 `packages/app` 的 `vite build` 依赖解析问题，并把侧边栏渠道状态卡收敛为默认展示 4 个、可展开的形态。

## 验证

- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `cd packages/app && bun run build`

## 备注

- 本文档替代原先拆散的 `2026-03-08-llm-audit.md`、`llm_audit_page.md` 以及对应的零碎 plan 页。
