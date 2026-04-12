# 主系统 `/report` 指令桥接本地研报 Workflow

- title: 主系统 `/report` 指令桥接本地研报 Workflow
- status: in_progress
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: Codex
- related_files:
  - crates/hone-channels/src/core.rs
  - crates/hone-core/src/config/server.rs
  - crates/hone-web-api/src/routes/chat.rs
  - bins/hone-discord/src/handlers.rs
  - bins/hone-telegram/src/handler.rs
  - bins/hone-feishu/src/handler.rs
  - bins/hone-imessage/src/main.rs
  - config.example.yaml
- related_docs:
  - docs/current-plan.md
  - docs/repo-map.md

## Goal

- 在主系统里新增 `/report` 预拦截命令，支持 `/report 公司名` 启动本地 `company_report` workflow，支持 `/report 进度` 查询当前研报任务进度。

## Scope

- 复用 `/register-admin AMM` 所在的前置拦截链路，而不是进入 `AgentSession` 普通对话流程。
- 为主系统补充可配置的本地 workflow runner 地址。
- 统一覆盖 Discord、Telegram、Feishu、iMessage 与 Web Chat 入口。
- 默认把研报运行必需字段补齐：`genPost=完整跑完`、`research_topic=新闻`、`news=""`、`task_id=""`、`validateCode="bamangniubi"`。

## Validation

- `cargo test -p hone-channels`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- 用真实本地 workflow runner 验证：
  - `/report 公司名` 对应的启动请求能成功返回 `running`
  - `/report 进度` 能读到 percent 与活跃节点摘要

## Documentation Sync

- 更新 `docs/current-plan.md` 活跃任务索引。
- 更新 `docs/repo-map.md`，注明 `hone-channels::core` 承担跨渠道 `/report` 预拦截与 local workflow bridge。
- 任务完成后视结果补 `docs/handoffs/` 并从活跃计划归档。

## Risks / Open Questions

- 当前 `/report 进度` 采用全局最新 `company_report` run 视角，不做 actor 级隔离；若未来允许并发多任务，需要补 actor/run 关联层。
- 本地 workflow runner 目前无鉴权，默认依赖回环地址与本机私有部署假设。
